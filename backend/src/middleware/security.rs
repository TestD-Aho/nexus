//! Security Middleware - Headers, CORS, Auth, Maintenance Mode

use crate::models::Claims;
use crate::services::auth::AuthService;
use crate::AppState;
use axum::{
    body::Body,
    extract::{Request, State},
    http::{header, HeaderValue, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::sync::Arc;

/// Security middleware layer
pub async fn layer(
    State(state): State<Arc<AppState>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    // Add security headers
    let mut response = next.run(request).await;
    
    // Add security headers
    response.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    response.headers_mut().insert(
        header::X_FRAME_OPTIONS,
        HeaderValue::from_static("DENY"),
    );
    response.headers_mut().insert(
        header::X_XSS_PROTECTION,
        HeaderValue::from_static("1; mode=block"),
    );
    response.headers_mut().insert(
        header::STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );
    response.headers_mut().insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    response.headers_mut().insert(
        header::PERMISSIONS_POLICY,
        HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
    );

    response
}

/// Extract JWT token from Authorization header
pub fn extract_token(authorization: &Option<HeaderValue>) -> Option<String> {
    authorization
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}

/// Authenticate request - extracts and validates JWT
pub async fn authenticate(
    State(state): State<Arc<AppState>>,
    request: Request<Body>,
) -> Result<Claims, StatusCode> {
    let auth_header = request.headers().get("authorization");
    
    let token = extract_token(&auth_header)
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    let mut validation = Validation::new(jsonwebtoken::Algorithm::HS512);
    validation.set_issuer(&[&state.config.jwt_issuer]);
    validation.validate_exp = true;
    
    decode::<Claims>(
        &token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &validation,
    )
    .map(|t| t.claims)
    .map_err(|_| StatusCode::UNAUTHORIZED)
}

/// Check if user has required permission
pub async fn require_permission(
    State(state): State<Arc<AppState>>,
    request: Request<Body>,
    action: &str,
    resource: &str,
) -> Result<Claims, StatusCode> {
    let claims = authenticate(State(state.clone()), request).await?;
    
    // Check if user's role has the required permission
    let has_permission = sqlx::query_as::<_, (bool,)>(
        r#"SELECT EXISTS(
            SELECT 1 FROM role_permissions rp
            JOIN permissions p ON rp.permission_id = p.id
            WHERE rp.role_id = $1 AND p.action = $2 AND p.resource = $3
        )"#
    )
    .bind(&claims.role_id)
    .bind(action)
    .bind(resource)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .map(|r| r.0)
    .unwrap_or(false);

    if !has_permission {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(claims)
}

/// Require admin role (Super-Architecte)
pub async fn require_admin(
    State(state): State<Arc<AppState>>,
    request: Request<Body>,
) -> Result<Claims, StatusCode> {
    let claims = authenticate(State(state.clone()), request).await?;
    
    if claims.role != "Super-Architecte" {
        return Err(StatusCode::FORBIDDEN);
    }
    
    Ok(claims)
}

/// Check maintenance mode
pub async fn check_maintenance(
    State(state): State<Arc<AppState>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    // Skip maintenance check for health endpoint
    if request.uri().path() == "/health" || request.uri().path() == "/api/v1/auth/login" {
        return next.run(request).await;
    }
    
    // Check if maintenance mode is active
    if state.is_maintenance_mode().await {
        // Check for admin bypass token
        let auth_header = request.headers().get("authorization");
        if let Some(token) = extract_token(&auth_header) {
            // Try to validate as admin
            if let Ok(claims) = validate_admin_token(&state, &token).await {
                if claims.role == "Super-Architecte" {
                    return next.run(request).await;
                }
            }
        }
        
        // Return maintenance response
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            r#"{"error": "maintenance_mode", "message": "System under maintenance"}"#
        ).into_response();
    }
    
    next.run(request).await
}

async fn validate_admin_token(state: &Arc<AppState>, token: &str) -> Result<Claims, StatusCode> {
    let mut validation = Validation::new(jsonwebtoken::Algorithm::HS512);
    validation.set_issuer(&[&state.config.jwt_issuer]);
    
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &validation,
    )
    .map(|t| t.claims)
    .map_err(|_| StatusCode::UNAUTHORIZED)
}
