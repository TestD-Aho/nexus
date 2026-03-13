//! Health Check Endpoint

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde::Serialize;
use std::sync::Arc;
use crate::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub timestamp: String,
}

/// Health check endpoint
pub async fn health_check(
    State(state): State<Arc<AppState>>,
) -> Result<Json<HealthResponse>, StatusCode> {
    // Check database connection
    sqlx::query("SELECT 1")
        .execute(&state.db_pool)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

    Ok(Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    }))
}
