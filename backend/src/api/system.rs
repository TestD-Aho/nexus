//! System API Routes (Feature Flags, Maintenance Mode)

use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::Json,
    routing::{get, put},
    Router,
    middleware,
};
use sqlx::Row;
use std::sync::Arc;
use uuid::Uuid;
use crate::models::FeatureFlag;
use crate::services::app_state::AppState;
use crate::middleware::security::require_admin;

/// Create system router with admin-only access for mutations
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        // Public routes
        .route("/system/feature-flags", get(list_feature_flags))
        .route("/system/settings", get(get_settings))
        // Protected routes - admin only
        .route("/system/feature-flags/:key", put(update_feature_flag))
        .route("/system/settings", put(update_settings))
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

/// Get system settings
pub async fn get_settings(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let is_maintenance = state.is_maintenance_mode().await;
    
    let settings: Option<(bool, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT maintenance_mode, maintenance_message, cv_url FROM system_settings WHERE id = 1"
    )
    .fetch_optional(&state.db_pool)
    .await
    .ok()
    .flatten();

    let (db_maintenance, message, cv_url) = settings.unwrap_or((false, None, None));

    Ok(Json(serde_json::json!({
        "maintenance_mode": is_maintenance,
        "message": message,
        "cv_url": cv_url
    })))
}

/// Update system settings
pub async fn update_settings(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut updates = vec!["updated_at = NOW()".to_string()];
    
    if let Some(enabled) = payload.get("maintenance_mode").and_then(|v| v.as_bool()) {
        updates.push(format!("maintenance_mode = {}", enabled));
    }
    
    if let Some(message) = payload.get("message").and_then(|v| v.as_str()) {
        updates.push("maintenance_message = $1".to_string());
    } else if payload.get("message").is_some() {
        updates.push("maintenance_message = NULL".to_string());
    }
    
    if let Some(cv_url) = payload.get("cv_url").and_then(|v| v.as_str()) {
        updates.push("cv_url = $2".to_string());
    } else if payload.get("cv_url").is_some() {
        updates.push("cv_url = NULL".to_string());
    }

    let set_clause = updates.join(", ");
    let query = format!("UPDATE system_settings SET {} WHERE id = 1", set_clause);
    
    let message = payload.get("message").and_then(|v| v.as_str());
    let cv_url = payload.get("cv_url").and_then(|v| v.as_str());

    sqlx::query(&query)
        .bind(message)
        .bind(cv_url)
        .execute(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(enabled) = payload.get("maintenance_mode").and_then(|v| v.as_bool()) {
        state.set_maintenance_mode(enabled).await;
    }

    get_settings(State(state)).await
}
