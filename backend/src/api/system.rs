//! System API Routes (Feature Flags, Maintenance Mode)

use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::Json,
    routing::{get, put},
    Router,
    middleware,
};
use std::sync::Arc;
use uuid::Uuid;
use crate::models::FeatureFlag;
use crate::services::app_state::AppState;
use crate::middleware::security::require_admin;

/// Create system router with admin-only access for mutations
pub fn router() -> Router<Arc<AppState>> {
    let admin_layer = middleware::from_fn_with_state(
        |state, request| async move {
            require_admin(state, request).await
        },
    );

    Router::new()
        // Public routes
        .route("/system/feature-flags", get(list_feature_flags))
        .route("/system/maintenance", get(get_maintenance_mode))
        // Protected routes - admin only
        .route("/system/feature-flags/:key", put(update_feature_flag).route_layer(admin_layer.clone()))
        .route("/system/maintenance", put(set_maintenance_mode).route_layer(admin_layer))
}

/// List all feature flags
pub async fn list_feature_flags(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<FeatureFlag>>, StatusCode> {
    sqlx::query_as::<_, FeatureFlag>(
        "SELECT * FROM feature_flags ORDER BY key"
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    .map(Json)
}

/// Update feature flag
pub async fn update_feature_flag(
    State(state): State<Arc<AppState>>,
    Path(key): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<FeatureFlag>, StatusCode> {
    let enabled = payload
        .get("enabled")
        .and_then(|v| v.as_bool())
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Update database
    sqlx::query(
        "UPDATE feature_flags SET enabled = $2, updated_at = NOW() WHERE key = $1 RETURNING *"
    )
    .bind(&key)
    .bind(enabled)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    // Update in-memory cache
    state.set_feature_flag(&key, enabled).await;

    let flag = sqlx::query_as::<_, FeatureFlag>(
        "SELECT * FROM feature_flags WHERE key = $1"
    )
    .bind(&key)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(flag))
}

/// Get maintenance mode status
pub async fn get_maintenance_mode(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let is_maintenance = state.is_maintenance_mode().await;
    
    let message: Option<String> = sqlx::query(
        "SELECT maintenance_message FROM system_settings WHERE id = 1"
    )
    .fetch_optional(&state.db_pool)
    .await
    .ok()
    .and_then(|row| row.get(0));

    Ok(Json(serde_json::json!({
        "maintenance_mode": is_maintenance,
        "message": message
    })))
}

/// Set maintenance mode
pub async fn set_maintenance_mode(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let enabled = payload
        .get("enabled")
        .and_then(|v| v.as_bool())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let message = payload
        .get("message")
        .and_then(|v| v.as_str());

    // Update database
    sqlx::query(
        "UPDATE system_settings SET maintenance_mode = $1, maintenance_message = $2, updated_at = NOW() WHERE id = 1"
    )
    .bind(enabled)
    .bind(message)
    .execute(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Update in-memory cache
    state.set_maintenance_mode(enabled).await;

    tracing::info!("Maintenance mode set to: {}", enabled);

    Ok(Json(serde_json::json!({
        "maintenance_mode": enabled,
        "message": message
    })))
}
