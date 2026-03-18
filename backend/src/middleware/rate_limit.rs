//! Rate Limiting Middleware

use crate::services::app_state::{AppState, RateLimitEntry};
use axum::{
    body::Body,
    extract::{Request, State},
    http::{header, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use std::time::Duration;

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
    
    let window_duration = Duration::from_secs(state.config.rate_limit_window_secs);
    let max_requests = state.config.rate_limit_requests;
    
    // Check rate limit (simple in-memory implementation)
    let (requests, window_start, is_limited) = {
        let mut rate_limit_state = state.rate_limit_state.write().await;
        
        let entry = rate_limit_state.entry(client_id.clone()).or_insert_with(|| {
            RateLimitEntry {
                requests: 0,
                window_start: std::time::Instant::now(),
            }
        });
        
        // Reset window if expired
        if entry.window_start.elapsed() > window_duration {
            entry.requests = 0;
            entry.window_start = std::time::Instant::now();
        }
        
        // Check limit
        let is_limited = entry.requests >= max_requests;
        entry.requests += 1;
        
        (entry.requests, entry.window_start, is_limited)
    };
    
    // Check limit after releasing the lock
    if is_limited {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            r#"{"error": "rate_limit_exceeded", "message": "Too many requests. Please try again later."}"#
        ).into_response();
    }
    
    let mut response = next.run(request).await;
    
    // Add rate limit headers
    response.headers_mut().insert(
        "X-RateLimit-Limit",
        HeaderValue::from(max_requests),
    );
    response.headers_mut().insert(
        "X-RateLimit-Remaining",
        HeaderValue::from(max_requests.saturating_sub(requests)),
    );
    response.headers_mut().insert(
        "X-RateLimit-Reset",
        HeaderValue::from(window_start.elapsed().as_secs() + window_duration.as_secs()),
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
