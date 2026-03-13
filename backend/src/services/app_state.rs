//! Application State Management

use crate::services::config::Config;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock as TokioRwLock;

/// Rate limit entry
pub struct RateLimitEntry {
    pub requests: u32,
    pub window_start: Instant,
}

/// Thread-safe application state
#[derive(Clone)]
pub struct AppState {
    pub db_pool: sqlx::PgPool,
    pub config: Config,
    
    // In-memory cache for feature flags and system settings
    feature_flags: Arc<TokioRwLock<HashMap<String, bool>>>,
    maintenance_mode: Arc<TokioRwLock<bool>>,
    
    // Rate limiting state
    pub rate_limit_state: Arc<TokioRwLock<HashMap<String, RateLimitEntry>>>,
}

impl AppState {
    pub fn new(db_pool: sqlx::PgPool, config: Config) -> Self {
        Self {
            db_pool,
            config,
            feature_flags: Arc::new(TokioRwLock::new(HashMap::new())),
            maintenance_mode: Arc::new(TokioRwLock::new(false)),
            rate_limit_state: Arc::new(TokioRwLock::new(HashMap::new())),
        }
    }

    /// Check if a feature flag is enabled
    pub async fn is_feature_enabled(&self, key: &str) -> bool {
        // First check memory cache
        {
            let flags = self.feature_flags.read().await;
            if let Some(enabled) = flags.get(key) {
                return *enabled;
            }
        }

        // Fallback to database (cached value will be set on first read)
        false
    }

    /// Set a feature flag
    pub async fn set_feature_flag(&self, key: &str, enabled: bool) {
        let mut flags = self.feature_flags.write().await;
        flags.insert(key.to_string(), enabled);
    }

    /// Check if maintenance mode is active
    pub async fn is_maintenance_mode(&self) -> bool {
        *self.maintenance_mode.read().await
    }

    /// Set maintenance mode
    pub async fn set_maintenance_mode(&self, enabled: bool) {
        let mut mode = self.maintenance_mode.write().await;
        *mode = enabled;
    }

    /// Load feature flags from database into memory
    pub async fn load_feature_flags(&self) -> Result<(), sqlx::Error> {
        let rows = sqlx::query_as::<_, (String, bool)>(
            "SELECT key, enabled FROM feature_flags"
        )
        .fetch_all(&self.db_pool)
        .await?;

        let mut flags = self.feature_flags.write().await;
        for (key, enabled) in rows {
            flags.insert(key, enabled);
        }

        Ok(())
    }

    /// Load system settings from database into memory
    pub async fn load_system_settings(&self) -> Result<(), sqlx::Error> {
        let row = sqlx::query_as::<_, (bool,)>(
            "SELECT maintenance_mode FROM system_settings WHERE id = 1"
        )
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some((maintenance_mode,)) = row {
            let mut mode = self.maintenance_mode.write().await;
            *mode = maintenance_mode;
        }

        Ok(())
    }
}
