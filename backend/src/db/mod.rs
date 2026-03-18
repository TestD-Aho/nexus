//! Database module - Connection pool and migrations

use sqlx::{postgres::{PgPoolOptions, PgRow}, Row, Executor, Pool, Postgres};
use std::time::Duration;

/// Create PostgreSQL connection pool
pub async fn create_pool(database_url: &str) -> Result<Pool<Postgres>, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(database_url)
        .await
}

/// Run database migrations
pub async fn run_migrations(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    sqlx::migrate!("./migrations").run(pool).await?;
    tracing::info!("✅ Migrations completed");
    Ok(())
}
