//! Authentication API Routes

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{post, get},
    Router,
};
use axum::extract::Path;
use std::sync::Arc;
use crate::models::{AuthResponse, LoginRequest, RegisterRequest};
use crate::AppState;
use crate::services::auth::AuthService;

/// Create auth router
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/auth/login", post(login))
        .route("/api/v1/auth/register", post(register))
        .route("/api/v1/auth/refresh", post(refresh))
        .route("/api/v1/auth/me", get(me))
}

/// Login endpoint
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    let auth_service = AuthService::new(state.db_pool.clone(), state.config.clone());
    
    auth_service
        .login(payload)
        .await
        .map(Json)
        .map_err(|e| {
            tracing::warn!("Login failed: {}", e);
            StatusCode::UNAUTHORIZED
        })
}

/// Register endpoint
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    let auth_service = AuthService::new(state.db_pool.clone(), state.config.clone());
    
    auth_service
        .register(payload)
        .await
        .map(Json)
        .map_err(|e| {
            tracing::warn!("Registration failed: {}", e);
            StatusCode::BAD_REQUEST
        })
}

/// Refresh token endpoint
pub async fn refresh(
    State(state): State<Arc<AppState>>,
    Json(refresh_token): Json<String>,
) -> Result<Json<AuthResponse>, StatusCode> {
    let auth_service = AuthService::new(state.db_pool.clone(), state.config.clone());
    
    auth_service
        .refresh_token(&refresh_token)
        .await
        .map(Json)
        .map_err(|e| {
            tracing::warn!("Token refresh failed: {}", e);
            StatusCode::UNAUTHORIZED
        })
}

/// Get current user info
pub async fn me(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // This would require auth middleware to extract claims
    // Simplified for now
    Ok(Json(serde_json::json!({
        "message": "Use JWT token in Authorization header"
    })))
}
