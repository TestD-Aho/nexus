//! Rate Limiting Middleware

use crate::AppState;
use axum::{
    body::Body,
    extract::{Request, State},
    http::{header, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use governor::{
    clock::DefaultClock,
    middleware::NoOpMiddleware,
    state::{keyed::DefaultKeyedStateStore, InMemoryState},
    Quota, RateLimiter,
};
use std::sync::Arc;
use std::time::Duration;
use tower::limit::GlobalConcurrencyLimitLayer;
use tower::ServiceBuilder;

/// Rate limiter type
type RateLimiterT = RateLimiter<String, DefaultKeyedStateStore<String>, DefaultClock, NoOpMiddleware>;

/// Create rate limiter middleware
pub fn create_rate_limiter(max_requests: u32, window_secs: u64) -> RateLimiterT {
    RateLimiter::keyed(
        Quota::with_period(Duration::from_secs(window_secs))
            .unwrap()
            .allow_hits(max_requests),
    )
}

/// Rate limit middleware layer
pub async fn layer(
    State(state): State<Arc<AppState>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    // Skip rate limiting for health checks
    if request.uri().path() == "/health" {
        return next.run(request).await;
    }

    // Get client identifier (IP or user ID if authenticated)
    let client_id = get_client_id(&request);
    
    // Check rate limit (simple in-memory implementation)
    let rate_limit_state = state.rate_limit_state.read().await;
    
    let entry = rate_limit_state.entry(client_id.clone()).or_insert_with(|| {
        RateLimitEntry {
            requests: 0,
            window_start: std::time::Instant::now(),
        }
    });
    
    let window_duration = Duration::from_secs(state.config.rate_limit_window_secs);
    
    // Reset window if expired
    if entry.window_start.elapsed() > window_duration {
        entry.requests = 0;
        entry.window_start = std::time::Instant::now();
    }
    
    // Check limit
    if entry.requests >= state.config.rate_limit_requests {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            r#"{"error": "rate_limit_exceeded", "message": "Too many requests. Please try again later."}"#
        ).into_response();
    }
    
    entry.requests += 1;
    drop(rate_limit_state);
    
    let mut response = next.run(request).await;
    
    // Add rate limit headers
    response.headers_mut().insert(
        "X-RateLimit-Limit",
        HeaderValue::from(state.config.rate_limit_requests),
    );
    response.headers_mut().insert(
        "X-RateLimit-Remaining",
        HeaderValue::from(state.config.rate_limit_requests.saturating_sub(entry.requests)),
    );
    response.headers_mut().insert(
        "X-RateLimit-Reset",
        HeaderValue::from(entry.window_start.elapsed().as_secs() + window_duration.as_secs()),
    );
    
    response
}

/// Get client identifier from request
fn get_client_id(request: &Request<Body>) -> String {
    // Try to get user ID from auth header first
    if let Some(auth) = request.headers().get("authorization") {
        if let Ok(token) = auth.to_str() {
            if let Some(stripped) = token.strip_prefix("Bearer ") {
                // Use token hash as identifier (don't decode, just hash it)
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                stripped.hash(&mut hasher);
                return format!("token:{}", hasher.finish());
            }
        }
    }
    
    // Fall back to IP address
    if let Some(forwarded) = request.headers().get("x-forwarded-for") {
        if let Ok(ip) = forwarded.to_str() {
            return format!("ip:{}", ip.split(',').next().unwrap_or(ip));
        }
    }
    
    if let Some(real_ip) = request.headers().get("x-real-ip") {
        if let Ok(ip) = real_ip.to_str() {
            return format!("ip:{}", ip);
        }
    }
    
    "ip:unknown".to_string()
}

/// Rate limit entry for tracking
struct RateLimitEntry {
    requests: u32,
    window_start: std::time::Instant,
}
