//! Authentication API Routes

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{post, get},
    Router,
    body::Body,
    extract::Request,
};
use std::sync::Arc;
use crate::models::{AuthResponse, LoginRequest, RegisterRequest};
use crate::services::app_state::AppState;
use crate::services::auth::AuthService;
use crate::middleware::security::authenticate;

/// Create auth router
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/auth/login", post(login))
        .route("/auth/register", post(register))
        .route("/auth/refresh", post(refresh))
        .route("/auth/me", get(me))
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

/// Get current user info - requires authentication
pub async fn me(
    State(state): State<Arc<AppState>>,
    request: Request,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Extract and validate the JWT token
    let claims = authenticate(State(state), request).await.map_err(|e| {
        tracing::warn!("Auth failed: {}", e);
        StatusCode::UNAUTHORIZED
    })?;
    
    Ok(Json(serde_json::json!({
        "id": claims.sub,
        "email": claims.email,
        "role": claims.role,
        "role_id": claims.role_id
    })))
}
