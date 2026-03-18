//! API Routes Module

pub mod health;
pub mod auth;
pub mod pages;
pub mod blocks;
pub mod collections;
pub mod admin;
pub mod system;
pub mod media;

use axum::Router;
use std::sync::Arc;
use crate::services::app_state::AppState;

/// Create the main API router combining all routes
pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .merge(health::router())
        .merge(auth::router())
        .merge(pages::router())
        .merge(blocks::router())
        .merge(collections::router())
        .merge(admin::router())
        .merge(system::router())
        .merge(media::router())
        .with_state(state)
}
