//! Nexus Core - Main entry point
//! Uses the modular architecture from src/

use nexus_core::{
    services::{config::Config, app_state::AppState},
    db::{create_pool, run_migrations},
    api,
};
use axum::Router;
use std::sync::Arc;
use tower_http::cors::{CorsLayer, Any};
use tower_http::services::ServeDir;
use tokio::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    tracing::info!("🚀 Starting Nexus CMS...");

    // Load configuration
    let config = Config::load()
        .map_err(|e| format!("Failed to load config: {}", e))?;
    
    tracing::info!("📋 Configuration loaded");

    // Create database pool
    let pool = create_pool(&config.database_url).await
        .map_err(|e| format!("Failed to connect to database: {}", e))?;
    tracing::info!("✅ Database connected");

    // Run migrations
    run_migrations(&pool).await
        .map_err(|e| format!("Failed to run migrations: {}", e))?;
    tracing::info!("✅ Migrations completed");

    // Create app state
    let state = Arc::new(AppState::new(pool, config.clone()));
    
    // Load feature flags and system settings into memory cache
    state.load_feature_flags().await?;
    state.load_system_settings().await?;
    tracing::info!("✅ System state loaded");

    // Ensure upload directory exists
    fs::create_dir_all(&config.upload_dir).await?;
    tracing::info!("📁 Upload directory: {}", config.upload_dir);

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build main router
    let mut app = Router::new()
        .nest("/api/v1", api::router(state.clone()))
        .layer(cors);

    // Serve static files from upload directory (if not in production)
    #[cfg(not(feature = "production"))]
    {
        app = app.nest_service("/uploads", ServeDir::new(&config.upload_dir));
    }

    // Health check at root
    use axum::routing::get;
    app = app.route("/health", get(health_check));

    // Start server
    let addr = format!("{}:{}", config.server_host, config.server_port);
    tracing::info!("🌐 Server starting on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
        "service": "nexus-core"
    }))
}
