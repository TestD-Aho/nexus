//! API Routes Module

pub mod health;
pub mod auth;
pub mod pages;
pub mod blocks;
pub mod collections;
pub mod admin;
pub mod system;
pub mod media;

use axum::{
    Router,
    middleware,
};
use std::sync::Arc;
use crate::services::app_state::AppState;
use crate::middleware::security::{authenticate, require_admin};

/// Create the main API router combining all routes
pub fn router(state: Arc<AppState>) -> Router {
    // Admin routes - require admin role
    let admin_routes = admin::router()
        .route_layer(middleware::from_fn_with_state(state.clone(), require_admin));

    // Protected routes - require authentication
    let protected_routes = Router::new()
        .merge(pages::router())
        .merge(blocks::router())
        .merge(collections::router())
        .merge(system::router())
        .route_layer(middleware::from_fn_with_state(state.clone(), authenticate));

    // Public routes - no auth required
    let public_routes = Router::new()
        .merge(health::router())
        .merge(auth::router())
        .merge(media::router());

    public_routes
        .merge(protected_routes)
        .merge(admin_routes)
        .with_state(state)
}
