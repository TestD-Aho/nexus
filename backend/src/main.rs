//! Nexus Core - Main entry point

use axum::{
    Router,
    routing::{get, post, put},
    extract::{State, Path, Json},
    http::StatusCode,
    response::{Json as AxumJson, IntoResponse},
    middleware,
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, Row, Pool, Postgres, Executor};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::Utc;
use jsonwebtoken::{encode, decode, EncodingKey, DecodingKey, Algorithm, Validation, Header};
use argon2::{PasswordHasher, password_hash::SaltString, Argon2};

// ============ CONFIGURATION ============

#[derive(Clone)]
struct AppState {
    pool: Pool<Postgres>,
    jwt_secret: String,
    maintenance_mode: Arc<RwLock<bool>>,
}

#[derive(Debug, Deserialize)]
struct Config {
    database_url: String,
    jwt_secret: String,
    host: String,
    port: u16,
}

impl Config {
    fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://postgres:postgres@localhost/nexus".to_string()),
            jwt_secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| "nexus-secret-key-change-in-production".to_string()),
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("PORT").unwrap_or_else(|_| "3000".to_string()).parse().unwrap_or(3000),
        }
    }
}

// ============ MODELS ============

#[derive(Debug, Serialize, Clone)]
struct User {
    id: Uuid,
    email: String,
    role: String,
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct AuthResponse {
    token: String,
    user: User,
}

#[derive(Debug, Serialize)]
struct Page {
    id: Uuid,
    slug: String,
    title: String,
    is_published: bool,
}

#[derive(Debug, Deserialize)]
struct CreatePageRequest {
    slug: String,
    title: String,
    description: Option<String>,
    is_published: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct Block {
    page_id: Uuid,
    block_type: String,
    content: serde_json::Value,
    order_index: i32,
}

#[derive(Debug, Deserialize)]
struct CreateBlockRequest {
    page_id: Uuid,
    block_type: String,
    content: serde_json::Value,
    title: Option<String>,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

// ============ AUTH ============

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    email: String,
    role: String,
    exp: i64,
    iat: i64,
}

fn create_token(email: &str, role: &str, user_id: Uuid, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        role: role.to_string(),
        exp: (now + chrono::Duration::hours(24)).timestamp(),
        iat: now.timestamp(),
    };
    
    encode(&Header::new(Algorithm::HS512), &claims, &EncodingKey::from_secret(secret.as_bytes()))
}

fn verify_token(token: &str, secret: &str) -> Result<Claims, ()> {
    let mut validation = Validation::new(Algorithm::HS512);
    validation.validate_exp = true;
    
    decode(token, &DecodingKey::from_secret(secret.as_bytes()), &validation)
        .map(|t| t.claims)
        .map_err(|_| ())
}

fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut rand::rngs::OsRng);
    let argon2 = Argon2::default();
    argon2.hash_password(password.as_bytes(), &salt).unwrap().to_string()
}

fn verify_password(password: &str, hash: &str) -> bool {
    use argon2::PasswordVerifier;
    argon2::PasswordHash::new(hash)
        .and_then(|parsed| Argon2::default().verify_password(password.as_bytes(), &parsed))
        .is_ok()
}

// ============ DATABASE ============

async fn init_db(database_url: &str) -> Result<Pool<Postgres>, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(30))
        .connect(database_url)
        .await?;
    
    // Create tables
    pool.execute(r#"
        CREATE TABLE IF NOT EXISTS roles (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR(100) NOT NULL UNIQUE,
            is_system BOOLEAN DEFAULT FALSE
        )
    "#).await?;
    
    pool.execute(r#"
        CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            email VARCHAR(255) NOT NULL UNIQUE,
            password_hash VARCHAR(255) NOT NULL,
            role_id UUID REFERENCES roles(id),
            created_at TIMESTAMP DEFAULT NOW()
        )
    "#).await?;
    
    pool.execute(r#"
        CREATE TABLE IF NOT EXISTS pages (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            slug VARCHAR(255) NOT NULL UNIQUE,
            title VARCHAR(255) NOT NULL,
            description TEXT,
            is_published BOOLEAN DEFAULT FALSE,
            created_at TIMESTAMP DEFAULT NOW()
        )
    "#).await?;
    
    pool.execute(r#"
        CREATE TABLE IF NOT EXISTS blocks (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            page_id UUID REFERENCES pages(id) ON DELETE CASCADE,
            block_type VARCHAR(50) NOT NULL,
            title VARCHAR(255),
            content JSONB DEFAULT '{}',
            order_index INTEGER DEFAULT 0,
            created_at TIMESTAMP DEFAULT NOW()
        )
    "#).await?;
    
    pool.execute(r#"
        CREATE TABLE IF NOT EXISTS system_settings (
            id INTEGER PRIMARY KEY DEFAULT 1,
            maintenance_mode BOOLEAN DEFAULT FALSE
        )
    "#).await?;
    
    // Insert default data
    pool.execute(r#"
        INSERT INTO roles (id, name, is_system) VALUES 
            (gen_random_uuid(), 'Super-Architecte', TRUE),
            (gen_random_uuid(), 'Gestionnaire', TRUE),
            (gen_random_uuid(), 'VIP', TRUE),
            (gen_random_uuid(), 'Visiteur', TRUE)
        ON CONFLICT DO NOTHING
    "#).await?;
    
    pool.execute("INSERT INTO system_settings (id, maintenance_mode) VALUES (1, FALSE) ON CONFLICT DO NOTHING").await?;
    
    // Create default admin if not exists
    let admin_exists: (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM users WHERE email = 'admin@nexus.local')")
        .fetch_one(&pool).await?;
    
    if !admin_exists.0 {
        let role_id: (Uuid,) = sqlx::query_as("SELECT id FROM roles WHERE name = 'Super-Architecte'")
            .fetch_one(&pool).await?;
        let password_hash = hash_password("admin123");
        
        sqlx::query("INSERT INTO users (email, password_hash, role_id) VALUES ('admin@nexus.local', $1, $2)")
            .bind(&password_hash)
            .bind(role_id.0)
            .execute(&pool).await?;
    }
    
    Ok(pool)
}

// ============ HANDLERS ============

async fn health_check(State(state): State<Arc<AppState>>) -> Result<AxumJson<HealthResponse>, StatusCode> {
    sqlx::query("SELECT 1").execute(&state.pool).await.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok(AxumJson(HealthResponse { status: "healthy".to_string(), version: "0.1.0".to_string() }))
}

async fn login(State(state): State<Arc<AppState>>, Json(payload): Json<LoginRequest>) -> Result<AxumJson<AuthResponse>, StatusCode> {
    let row = sqlx::query("SELECT u.id, u.email, u.password_hash, r.name as role FROM users u JOIN roles r ON u.role_id = r.id WHERE u.email = $1")
        .bind(&payload.email)
        .fetch_optional(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    let user_id: Uuid = row.get("id");
    let email: String = row.get("email");
    let password_hash: String = row.get("password_hash");
    let role: String = row.get("role");
    
    if !verify_password(&payload.password, &password_hash) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    
    let token = create_token(&email, &role, user_id, &state.jwt_secret)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(AxumJson(AuthResponse { token, user: User { id: user_id, email, role } }))
}

async fn list_pages(State(state): State<Arc<AppState>>) -> Result<AxumJson<Vec<Page>>, StatusCode> {
    let rows = sqlx::query("SELECT id, slug, title, is_published FROM pages ORDER BY title")
        .fetch_all(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let pages: Vec<Page> = rows.iter().map(|row| Page {
        id: row.get("id"),
        slug: row.get("slug"),
        title: row.get("title"),
        is_published: row.get("is_published"),
    }).collect();
    
    Ok(AxumJson(pages))
}

async fn get_page(State(state): State<Arc<AppState>>, Path(slug): Path<String>) -> Result<AxumJson<serde_json::Value>, StatusCode> {
    let page_row = sqlx::query("SELECT id, slug, title, description, is_published FROM pages WHERE slug = $1")
        .bind(&slug)
        .fetch_optional(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let page_id: Uuid = page_row.get("id");
    let page_slug: String = page_row.get("slug");
    let title: String = page_row.get("title");
    let description: Option<String> = page_row.get("description");
    let is_published: bool = page_row.get("is_published");
    
    if !is_published {
        return Err(StatusCode::NOT_FOUND);
    }
    
    let blocks = sqlx::query("SELECT id, block_type, title, content, order_index FROM blocks WHERE page_id = $1 ORDER BY order_index")
        .bind(page_id)
        .fetch_all(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let blocks_json: Vec<serde_json::Value> = blocks.iter().map(|row| {
        serde_json::json!({
            "id": row.get::<Uuid, _>("id"),
            "type": row.get::<String, _>("block_type"),
            "title": row.get::<Option<String>, _>("title"),
            "content": row.get::<serde_json::Value, _>("content"),
            "order": row.get::<i32, _>("order_index"),
        })
    }).collect();
    
    Ok(AxumJson(serde_json::json!({
        "page": { "slug": page_slug, "title": title, "description": description },
        "blocks": blocks_json
    })))
}

async fn create_page(State(state): State<Arc<AppState>>, Json(payload): Json<CreatePageRequest>) -> Result<AxumJson<Page>, StatusCode> {
    let id = Uuid::new_v4();
    
    sqlx::query("INSERT INTO pages (id, slug, title, description, is_published) VALUES ($1, $2, $3, $4, $5)")
        .bind(id)
        .bind(&payload.slug)
        .bind(&payload.title)
        .bind(&payload.description)
        .bind(payload.is_published.unwrap_or(false))
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(AxumJson(Page { id, slug: payload.slug, title: payload.title, is_published: payload.is_published.unwrap_or(false) }))
}

async fn create_block(State(state): State<Arc<AppState>>, Json(payload): Json<CreateBlockRequest>) -> Result<AxumJson<serde_json::Value>, StatusCode> {
    let id = Uuid::new_v4();
    
    let max_order: (i32,) = sqlx::query_as("SELECT COALESCE(MAX(order_index), -1) FROM blocks WHERE page_id = $1")
        .bind(payload.page_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let order = max_order.0 + 1;
    
    sqlx::query("INSERT INTO blocks (id, page_id, block_type, title, content, order_index) VALUES ($1, $2, $3, $4, $5, $6)")
        .bind(id)
        .bind(payload.page_id)
        .bind(&payload.block_type)
        .bind(&payload.title)
        .bind(&payload.content)
        .bind(order)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(AxumJson(serde_json::json!({ "id": id, "order_index": order })))
}

async fn set_maintenance(State(state): State<Arc<AppState>>, Json(payload): Json<serde_json::Value>) -> Result<AxumJson<serde_json::Value>, StatusCode> {
    let enabled = payload.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
    
    sqlx::query("UPDATE system_settings SET maintenance_mode = $1 WHERE id = 1")
        .bind(enabled)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    *state.maintenance_mode.write().await = enabled;
    
    Ok(AxumJson(serde_json::json!({ "maintenance_mode": enabled })))
}

async fn get_maintenance(State(state): State<Arc<AppState>>) -> Result<AxumJson<serde_json::Value>, StatusCode> {
    let mode = *state.maintenance_mode.read().await;
    Ok(AxumJson(serde_json::json!({ "maintenance_mode": mode })))
}

// ============ MIDDLEWARE ============

async fn auth_middleware(State(state): State<Arc<AppState>>, request: axum::extract::Request, next: axum::middleware::Next) -> axum::response::Response {
    // Check maintenance mode first
    if *state.maintenance_mode.read().await {
        if request.uri().path() != "/health" && !request.uri().path().starts_with("/api/v1/auth") {
            return (StatusCode::SERVICE_UNAVAILABLE, r#"{"error":"maintenance"}"#).into_response();
        }
    }
    
    next.run(request).await
}

// ============ MAIN ============

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    let config = Config::from_env();
    
    tracing::info!("Connecting to database...");
    let pool = init_db(&config.database_url).await?;
    tracing::info!("Database initialized");
    
    let state = Arc::new(AppState {
        pool,
        jwt_secret: config.jwt_secret,
        maintenance_mode: Arc::new(RwLock::new(false)),
    });
    
    use tower_http::cors::{CorsLayer, Any};
    
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/auth/login", post(login))
        .route("/api/v1/pages", get(list_pages))
        .route("/api/v1/pages", post(create_page))
        .route("/api/v1/pages/:slug", get(get_page))
        .route("/api/v1/blocks", post(create_block))
        .route("/api/v1/system/maintenance", get(get_maintenance))
        .route("/api/v1/system/maintenance", put(set_maintenance))
        .layer(cors)
        .layer(axum::middleware::from_fn_with_state(state.clone(), auth_middleware))
        .with_state(state);
    
    let addr = format!("{}:{}", config.host, config.port);
    tracing::info!("🚀 Nexus starting on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
