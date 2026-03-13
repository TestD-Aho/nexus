//! Configuration Service

use serde::Deserialize;
use std::env;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(String),
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    // Server
    pub server_host: String,
    pub server_port: u16,
    
    // Database
    pub database_url: String,
    
    // JWT
    pub jwt_secret: String,
    pub jwt_issuer: String,
    pub jwt_expiration_minutes: i64,
    pub jwt_refresh_expiration_days: i64,
    
    // Security
    pub argon2_time: u32,
    pub argon2_mem: u32,
    pub argon2_parallelism: u32,
    
    // Rate limiting
    pub rate_limit_requests: u32,
    pub rate_limit_window_secs: u64,
    
    // File uploads
    pub upload_dir: String,
    pub max_upload_size: usize,
    
    // Optional: Redis
    pub redis_url: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        Ok(Config {
            server_host: env::var("NEXUS_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: env::var("NEXUS_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidConfig("Invalid port".to_string()))?,
            
            database_url: env::var("DATABASE_URL")
                .map_err(|_| ConfigError::MissingEnvVar("DATABASE_URL".to_string()))?,
            
            jwt_secret: env::var("JWT_SECRET")
                .map_err(|_| ConfigError::MissingEnvVar("JWT_SECRET".to_string()))?,
            jwt_issuer: env::var("JWT_ISSUER").unwrap_or_else(|_| "nexus-cms".to_string()),
            jwt_expiration_minutes: env::var("JWT_EXPIRATION_MINUTES")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .unwrap_or(60),
            jwt_refresh_expiration_days: env::var("JWT_REFRESH_DAYS")
                .unwrap_or_else(|_| "7".to_string())
                .parse()
                .unwrap_or(7),
            
            argon2_time: env::var("ARGON2_TIME")
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .unwrap_or(3),
            argon2_mem: env::var("ARGON2_MEM")
                .unwrap_or_else(|_| "65536".to_string())
                .parse()
                .unwrap_or(65536),
            argon2_parallelism: env::var("ARGON2_PARALLELISM")
                .unwrap_or_else(|_| "4".to_string())
                .parse()
                .unwrap_or(4),
            
            rate_limit_requests: env::var("RATE_LIMIT_REQUESTS")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .unwrap_or(100),
            rate_limit_window_secs: env::var("RATE_LIMIT_WINDOW_SECS")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .unwrap_or(60),
            
            upload_dir: env::var("NEXUS_UPLOAD_DIR")
                .unwrap_or_else(|_| "./uploads".to_string()),
            max_upload_size: env::var("NEXUS_MAX_UPLOAD_SIZE")
                .unwrap_or_else(|_| "10485760".to_string())
                .parse()
                .unwrap_or(10 * 1024 * 1024), // 10MB default
            
            redis_url: env::var("REDIS_URL").ok(),
        })
    }
}
