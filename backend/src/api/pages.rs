//! Pages API Routes

use axum::{
    extract::{State, Path, Query},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use axum::extract::State;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;
use crate::models::{CreatePageRequest, Page, UpdatePageRequest};
use crate::AppState;
use crate::middleware::security::{authenticate, require_permission};

#[derive(Deserialize)]
pub struct PageQuery {
    pub published: Option<bool>,
}

/// Create pages router
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/pages", get(list_pages))
        .route("/api/v1/pages", post(create_page))
        .route("/api/v1/pages/:slug", get(get_page))
        .route("/api/v1/pages/:id", put(update_page))
        .route("/api/v1/pages/:id", delete(delete_page))
}

/// List all pages
pub async fn list_pages(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PageQuery>,
) -> Result<Json<Vec<Page>>, StatusCode> {
    let pages = if query.published.unwrap_or(false) {
        sqlx::query_as::<_, Page>(
            "SELECT * FROM pages WHERE is_published = TRUE ORDER BY title"
        )
        .fetch_all(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        sqlx::query_as::<_, Page>(
            "SELECT * FROM pages ORDER BY title"
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
        "SELECT * FROM pages WHERE slug = $1"
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
            "SELECT * FROM pages WHERE id = $1"
        )
        .bind(uuid)
        .fetch_optional(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?
    };

    // Check if user can view this page
    if !page.is_published {
        // Try to authenticate for unpublished pages
        // For now, just return not found for unpublished pages
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
    // Require authentication
    let _claims = authenticate(State(state.clone()), axum::extract::Request::new(
        axum::body::Body::empty()
    )).await.map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Check permission
    let has_permission = sqlx::query_as::<_, (bool,)>(
        r#"SELECT EXISTS(
            SELECT 1 FROM role_permissions rp
            JOIN permissions p ON rp.permission_id = p.id
            WHERE rp.role_id = $1 AND p.action = 'create' AND p.resource = 'pages'
        )"#
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .map(|r| r.0)
    .unwrap_or(false);

    if !has_permission {
        return Err(StatusCode::FORBIDDEN);
    }

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
        "SELECT * FROM pages WHERE id = $1"
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
    // Require authentication
    let _claims = authenticate(State(state.clone()), axum::extract::Request::new(
        axum::body::Body::empty()
    )).await.map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Check permission
    let has_permission = sqlx::query_as::<_, (bool,)>(
        r#"SELECT EXISTS(
            SELECT 1 FROM role_permissions rp
            JOIN permissions p ON rp.permission_id = p.id
            WHERE rp.role_id = $1 AND p.action = 'update' AND p.resource = 'pages'
        )"#
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .map(|r| r.0)
    .unwrap_or(false);

    if !has_permission {
        return Err(StatusCode::FORBIDDEN);
    }

    // Build dynamic update query
    let mut updates = vec!["updated_at = NOW()".to_string()];
    let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres>> + Send + Sync> = vec![];
    let mut param_count = 0;

    if let Some(slug) = &payload.slug {
        param_count += 1;
        updates.push(format!("slug = ${}", param_count));
        params.push(Box::new(slug.clone()));
    }
    if let Some(title) = &payload.title {
        param_count += 1;
        updates.push(format!("title = ${}", param_count));
        params.push(Box::new(title.clone()));
    }
    if let Some(desc) = &payload.description {
        param_count += 1;
        updates.push(format!("description = ${}", param_count));
        params.push(Box::new(desc.clone()));
    }
    if let Some(published) = payload.is_published {
        param_count += 1;
        updates.push(format!("is_published = ${}", param_count));
        params.push(Box::new(published));
        if published {
            updates.push("published_at = NOW()".to_string());
        }
    }
    if let Some(home) = payload.is_home {
        param_count += 1;
        updates.push(format!("is_home = ${}", param_count));
        params.push(Box::new(home));
    }
    if let Some(meta_title) = &payload.meta_title {
        param_count += 1;
        updates.push(format!("meta_title = ${}", param_count));
        params.push(Box::new(meta_title.clone()));
    }
    if let Some(meta_desc) = &payload.meta_description {
        param_count += 1;
        updates.push(format!("meta_description = ${}", param_count));
        params.push(Box::new(meta_desc.clone()));
    }

    if updates.len() > 1 {
        param_count += 1;
        let query = format!(
            "UPDATE pages SET {} WHERE id = ${} RETURNING *",
            updates.join(", "),
            param_count
        );
        
        params.push(Box::new(id));
        
        // Execute with dynamic params (simplified - in production use query_as with proper typing)
        sqlx::query(&query)
            .execute(&state.db_pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    let page = sqlx::query_as::<_, Page>(
        "SELECT * FROM pages WHERE id = $1"
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
    // Require authentication
    let _claims = authenticate(State(state.clone()), axum::extract::Request::new(
        axum::body::Body::empty()
    )).await.map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Check permission
    let has_permission = sqlx::query_as::<_, (bool,)>(
        r#"SELECT EXISTS(
            SELECT 1 FROM role_permissions rp
            JOIN permissions p ON rp.permission_id = p.id
            WHERE rp.role_id = $1 AND p.action = 'delete' AND p.resource = 'pages'
        )"#
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .map(|r| r.0)
    .unwrap_or(false);

    if !has_permission {
        return Err(StatusCode::FORBIDDEN);
    }

    sqlx::query("DELETE FROM pages WHERE id = $1")
        .bind(id)
        .execute(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}
