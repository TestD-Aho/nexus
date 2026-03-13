//! Nexus Core Library
//! A modular headless CMS built with Rust/Axum

pub mod api;
pub mod db;
pub mod middleware;
pub mod models;
pub mod services;
pub mod utils;

// Re-export commonly used types
pub use services::{config::Config, app_state::AppState};
pub use db::{create_pool, run_migrations};
