//! Authentication Service

use crate::models::{AuthResponse, LoginRequest, RegisterRequest, UserResponse};
use crate::services::config::Config;
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("User not found")]
    UserNotFound,
    #[error("Token generation failed")]
    TokenGenerationFailed,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,          // User ID
    pub email: String,
    pub role: String,
    pub role_id: String,
    pub exp: i64,
    pub iat: i64,
    pub iss: String,
    pub typ: String,
}

impl Claims {
    pub fn new(user_id: Uuid, email: String, role: String, role_id: Uuid, config: &Config) -> Self {
        let now = Utc::now();
        let exp = now + Duration::minutes(config.jwt_expiration_minutes);
        
        Self {
            sub: user_id.to_string(),
            email,
            role,
            role_id: role_id.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            iss: config.jwt_issuer.clone(),
            typ: "access".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshClaims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
    pub iss: String,
    pub typ: String,
}

pub struct AuthService {
    pool: PgPool,
    config: Config,
}

impl AuthService {
    pub fn new(pool: PgPool, config: Config) -> Self {
        Self { pool, config }
    }

    /// Hash a password using Argon2
    pub fn hash_password(&self, password: &str) -> Result<String, AuthError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2::Params::new(
                self.config.argon2_mem,
                self.config.argon2_time,
                self.config.argon2_parallelism,
                Some(32),
            ).map_err(|e| AuthError::TokenGenerationFailed)?,
        );
        
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| AuthError::TokenGenerationFailed)?
            .to_string();
        
        Ok(hash)
    }

    /// Verify a password against a hash
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, AuthError> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|_| AuthError::InvalidCredentials)?;
        
        let argon2 = Argon2::default();
        Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }

    /// Generate JWT access token
    pub fn generate_token(&self, claims: &Claims) -> Result<String, AuthError> {
        encode(
            &Header::new(Algorithm::HS512),
            claims,
            &EncodingKey::from_secret(self.config.jwt_secret.as_bytes()),
        )
        .map_err(|_| AuthError::TokenGenerationFailed)
    }

    /// Generate JWT refresh token
    pub fn generate_refresh_token(&self, user_id: Uuid) -> Result<String, AuthError> {
        let now = Utc::now();
        let exp = now + Duration::days(self.config.jwt_refresh_expiration_days);
        
        let claims = RefreshClaims {
            sub: user_id.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            iss: self.config.jwt_issuer.clone(),
            typ: "refresh".to_string(),
        };
        
        encode(
            &Header::new(Algorithm::HS512),
            &claims,
            &EncodingKey::from_secret(self.config.jwt_secret.as_bytes()),
        )
        .map_err(|_| AuthError::TokenGenerationFailed)
    }

    /// Validate JWT token
    pub fn validate_token(&self, token: &str) -> Result<TokenData<Claims>, AuthError> {
        let mut validation = Validation::new(Algorithm::HS512);
        validation.set_issuer(&[&self.config.jwt_issuer]);
        validation.validate_exp = true;
        
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.config.jwt_secret.as_bytes()),
            &validation,
        )
        .map_err(|_| AuthError::InvalidToken)
    }

    /// Validate refresh token
    pub fn validate_refresh_token(&self, token: &str) -> Result<TokenData<RefreshClaims>, AuthError> {
        let mut validation = Validation::new(Algorithm::HS512);
        validation.set_issuer(&[&self.config.jwt_issuer]);
        validation.validate_exp = true;
        
        decode::<RefreshClaims>(
            token,
            &DecodingKey::from_secret(self.config.jwt_secret.as_bytes()),
            &validation,
        )
        .map_err(|_| AuthError::InvalidToken)
    }

    /// Authenticate user with email and password
    pub async fn login(&self, request: LoginRequest) -> Result<AuthResponse, AuthError> {
        // Fetch user with role
        let (user_id, email, password_hash, role_name, role_id): (Uuid, String, String, String, Uuid) = 
            sqlx::query_as(
                r#"SELECT u.id, u.email, u.password_hash, r.name, u.role_id 
                   FROM users u 
                   JOIN roles r ON u.role_id = r.id 
                   WHERE u.email = $1 AND u.is_active = TRUE"#
            )
            .bind(&request.email)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| AuthError::InvalidCredentials)?;

        // Verify password
        if !self.verify_password(&request.password, &password_hash)? {
            return Err(AuthError::InvalidCredentials);
        }

        // Update last login
        sqlx::query("UPDATE users SET last_login = NOW() WHERE id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        // Generate tokens
        let claims = Claims::new(user_id, email.clone(), role_name.clone(), role_id, &self.config);
        let token = self.generate_token(&claims)?;
        let refresh_token = self.generate_refresh_token(user_id)?;

        Ok(AuthResponse {
            token,
            refresh_token,
            expires_in: self.config.jwt_expiration_minutes * 60,
            user: UserResponse {
                id: user_id,
                email,
                role: role_name,
                created_at: Utc::now(),
            },
        })
    }

    /// Register a new user
    pub async fn register(&self, request: RegisterRequest) -> Result<AuthResponse, AuthError> {
        // Check if user already exists
        let exists: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)"
        )
        .bind(&request.email)
        .fetch_one(&self.pool)
        .await?;

        if exists.0 {
            return Err(AuthError::UserAlreadyExists);
        }

        // Get default role (VIP) if not specified
        let role_id: Uuid = if let Some(role_name) = &request.role {
            sqlx::query_as::<_, (Uuid,)>("SELECT id FROM roles WHERE name = $1")
                .bind(role_name)
                .fetch_one(&self.pool)
                .await
                .map_err(|_| AuthError::UserNotFound)?.0
        } else {
            sqlx::query_as::<_, (Uuid,)>("SELECT id FROM roles WHERE name = 'VIP'")
                .fetch_one(&self.pool)
                .await
                .map_err(|_| AuthError::UserNotFound)?.0
        };

        // Hash password
        let password_hash = self.hash_password(&request.password)?;

        // Create user
        let user_id = Uuid::new_v4();
        
        sqlx::query(
            r#"INSERT INTO users (id, email, password_hash, role_id, created_at, updated_at) 
               VALUES ($1, $2, $3, $4, NOW(), NOW())"#
        )
        .bind(user_id)
        .bind(&request.email)
        .bind(password_hash)
        .bind(role_id)
        .execute(&self.pool)
        .await?;

        // Get role name
        let role_name: (String,) = sqlx::query_as("SELECT name FROM roles WHERE id = $1")
            .bind(role_id)
            .fetch_one(&self.pool)
            .await?;

        // Generate tokens
        let claims = Claims::new(user_id, request.email.clone(), role_name.0.clone(), role_id, &self.config);
        let token = self.generate_token(&claims)?;
        let refresh_token = self.generate_refresh_token(user_id)?;

        Ok(AuthResponse {
            token,
            refresh_token,
            expires_in: self.config.jwt_expiration_minutes * 60,
            user: UserResponse {
                id: user_id,
                email: request.email,
                role: role_name.0,
                created_at: Utc::now(),
            },
        })
    }

    /// Refresh access token using refresh token
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<AuthResponse, AuthError> {
        let token_data = self.validate_refresh_token(refresh_token)?;
        let user_id = Uuid::parse_str(&token_data.claims.sub)
            .map_err(|_| AuthError::InvalidToken)?;

        // Fetch user with role
        let (email, role_name, role_id): (String, String, Uuid) = sqlx::query_as(
            r#"SELECT u.email, r.name, u.role_id 
               FROM users u 
               JOIN roles r ON u.role_id = r.id 
               WHERE u.id = $1 AND u.is_active = TRUE"#
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| AuthError::UserNotFound)?;

        // Generate new access token
        let claims = Claims::new(user_id, email.clone(), role_name.clone(), role_id, &self.config);
        let new_token = self.generate_token(&claims)?;
        let new_refresh_token = self.generate_refresh_token(user_id)?;

        Ok(AuthResponse {
            token: new_token,
            refresh_token: new_refresh_token,
            expires_in: self.config.jwt_expiration_minutes * 60,
            user: UserResponse {
                id: user_id,
                email,
                role: role_name,
                created_at: Utc::now(),
            },
        })
    }

    /// Check if user has a specific permission
    pub async fn has_permission(&self, user_id: Uuid, action: &str, resource: &str) -> Result<bool, sqlx::Error> {
        let has_permission: (bool,) = sqlx::query_as(
            r#"SELECT EXISTS(
                SELECT 1 FROM users u
                JOIN role_permissions rp ON u.role_id = rp.role_id
                JOIN permissions p ON rp.permission_id = p.id
                WHERE u.id = $1 AND p.action = $2 AND p.resource = $3
            )"#
        )
        .bind(user_id)
        .bind(action)
        .bind(resource)
        .fetch_one(&self.pool)
        .await?;

        Ok(has_permission.0)
    }
}
