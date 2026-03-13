//! Pages API Routes

use axum::{
    extract::{State, Path, Query},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;
use crate::models::{CreatePageRequest, Page, UpdatePageRequest};
use crate::services::app_state::AppState;

#[derive(Deserialize)]
pub struct PageQuery {
    pub published: Option<bool>,
}

/// Create pages router
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/pages", get(list_pages))
        .route("/pages", post(create_page))
        .route("/pages/:slug", get(get_page))
        .route("/pages/:id", put(update_page))
        .route("/pages/:id", delete(delete_page))
}

/// List all pages
pub async fn list_pages(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PageQuery>,
) -> Result<Json<Vec<Page>>, StatusCode> {
    let pages = if query.published.unwrap_or(false) {
        sqlx::query_as::<_, Page>(
            "SELECT id, slug, title, description, is_published, is_home, meta_title, meta_description, created_at, updated_at, published_at FROM pages WHERE is_published = TRUE ORDER BY title"
        )
        .fetch_all(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        sqlx::query_as::<_, Page>(
            "SELECT id, slug, title, description, is_published, is_home, meta_title, meta_description, created_at, updated_at, published_at FROM pages ORDER BY title"
        )
        .fetch_all(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };

    Ok(Json(pages))
}

/// Get page by slug
pub async fn get_page(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // First try to find by slug
    let page: Option<Page> = sqlx::query_as::<_, Page>(
        "SELECT id, slug, title, description, is_published, is_home, meta_title, meta_description, created_at, updated_at, published_at FROM pages WHERE slug = $1"
    )
    .bind(&slug)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // If not found by slug, try by ID
    let page = if let Some(p) = page {
        p
    } else {
        let uuid = Uuid::parse_str(&slug).map_err(|_| StatusCode::NOT_FOUND)?;
        sqlx::query_as::<_, Page>(
            "SELECT id, slug, title, description, is_published, is_home, meta_title, meta_description, created_at, updated_at, published_at FROM pages WHERE id = $1"
        )
        .bind(uuid)
        .fetch_optional(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?
    };

    // Check if user can view this page
    if !page.is_published {
        return Err(StatusCode::NOT_FOUND);
    }

    // Get blocks for this page (only published ones, respecting schedule)
    let now = chrono::Utc::now();
    let blocks = sqlx::query_as::<_, serde_json::Value>(
        r#"SELECT id, block_type, order_index, title, content, styling, access_control
           FROM blocks 
           WHERE page_id = $1 
           AND status = 'published'
           AND (schedule_start IS NULL OR schedule_start <= $2)
           AND (schedule_end IS NULL OR schedule_end >= $2)
           ORDER BY order_index"#
    )
    .bind(page.id)
    .bind(now)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "page": page,
        "blocks": blocks
    })))
}

/// Create new page
pub async fn create_page(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreatePageRequest>,
) -> Result<Json<Page>, StatusCode> {
    let page_id = Uuid::new_v4();
    
    sqlx::query(
        r#"INSERT INTO pages (id, slug, title, description, is_published, is_home, meta_title, meta_description, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())"#
    )
    .bind(page_id)
    .bind(&payload.slug)
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(payload.is_published.unwrap_or(false))
    .bind(payload.is_home.unwrap_or(false))
    .bind(&payload.meta_title)
    .bind(&payload.meta_description)
    .execute(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let page = sqlx::query_as::<_, Page>(
        "SELECT id, slug, title, description, is_published, is_home, meta_title, meta_description, created_at, updated_at, published_at FROM pages WHERE id = $1"
    )
    .bind(page_id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(page))
}

/// Update page
pub async fn update_page(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdatePageRequest>,
) -> Result<Json<Page>, StatusCode> {
    // Build dynamic update query (simplified)
    if let Some(slug) = &payload.slug {
        sqlx::query("UPDATE pages SET slug = $1, updated_at = NOW() WHERE id = $2")
            .bind(slug)
            .bind(id)
            .execute(&state.db_pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    if let Some(title) = &payload.title {
        sqlx::query("UPDATE pages SET title = $1, updated_at = NOW() WHERE id = $2")
            .bind(title)
            .bind(id)
            .execute(&state.db_pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    if let Some(published) = payload.is_published {
        sqlx::query("UPDATE pages SET is_published = $1, updated_at = NOW(), published_at = CASE WHEN $1 = TRUE THEN NOW() ELSE published_at END WHERE id = $2")
            .bind(published)
            .bind(id)
            .execute(&state.db_pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    let page = sqlx::query_as::<_, Page>(
        "SELECT id, slug, title, description, is_published, is_home, meta_title, meta_description, created_at, updated_at, published_at FROM pages WHERE id = $1"
    )
    .bind(id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(page))
}

/// Delete page
pub async fn delete_page(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("DELETE FROM pages WHERE id = $1")
        .bind(id)
        .execute(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}
