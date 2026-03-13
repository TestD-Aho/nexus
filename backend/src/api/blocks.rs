//! Blocks API Routes

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
use crate::models::{Block, CreateBlockRequest, ReorderBlocksRequest, UpdateBlockRequest};
use crate::AppState;
use crate::middleware::security::authenticate;

/// Create blocks router
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/blocks", get(list_blocks))
        .route("/api/v1/blocks", post(create_block))
        .route("/api/v1/blocks/:id", get(get_block))
        .route("/api/v1/blocks/:id", put(update_block))
        .route("/api/v1/blocks/:id", delete(delete_block))
        .route("/api/v1/blocks/reorder", post(reorder_blocks))
}

/// List blocks (optionally filtered by page)
#[derive(Deserialize)]
pub struct BlockQuery {
    pub page_id: Option<Uuid>,
    pub status: Option<String>,
}

pub async fn list_blocks(
    State(state): State<Arc<AppState>>,
    Query(query): Query<BlockQuery>,
) -> Result<Json<Vec<Block>>, StatusCode> {
    let blocks = if let Some(page_id) = query.page_id {
        sqlx::query_as::<_, Block>(
            "SELECT * FROM blocks WHERE page_id = $1 ORDER BY order_index"
        )
        .bind(page_id)
        .fetch_all(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else if let Some(status) = query.status {
        sqlx::query_as::<_, Block>(
            "SELECT * FROM blocks WHERE status = $1 ORDER BY order_index"
        )
        .bind(status)
        .fetch_all(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        sqlx::query_as::<_, Block>(
            "SELECT * FROM blocks ORDER BY order_index"
        )
        .fetch_all(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };

    Ok(Json(blocks))
}

/// Get single block
pub async fn get_block(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Block>, StatusCode> {
    sqlx::query_as::<_, Block>(
        "SELECT * FROM blocks WHERE id = $1"
    )
    .bind(id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)
    .map(Json)
}

/// Create new block
pub async fn create_block(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateBlockRequest>,
) -> Result<Json<Block>, StatusCode> {
    // Require authentication
    let _claims = authenticate(State(state.clone()), axum::extract::Request::new(
        axum::body::Body::empty()
    )).await.map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Check permission
    let has_permission = sqlx::query_as::<_, (bool,)>(
        r#"SELECT EXISTS(
            SELECT 1 FROM role_permissions rp
            JOIN permissions p ON rp.permission_id = p.id
            WHERE rp.role_id = $1 AND p.action = 'create' AND p.resource = 'blocks'
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

    let block_id = Uuid::new_v4();
    
    // Get next order index for the page
    let max_order: (Option<i32>,) = sqlx::query_as(
        "SELECT MAX(order_index) FROM blocks WHERE page_id = $1"
    )
    .bind(payload.page_id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let next_order = max_order.0.unwrap_or(-1) + 1;

    sqlx::query(
        r#"INSERT INTO blocks (
            id, page_id, block_type, order_index, status, title,
            content, styling, access_control,
            schedule_start, schedule_end,
            created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW(), NOW())
        RETURNING *"#
    )
    .bind(block_id)
    .bind(payload.page_id)
    .bind(&payload.block_type)
    .bind(next_order)
    .bind("draft")
    .bind(&payload.title)
    .bind(&payload.content)
    .bind(payload.styling.unwrap_or(serde_json::json!({})))
    .bind(payload.access_control.unwrap_or(serde_json::json!({"require_auth": false, "allowed_roles": ["*"]})))
    .bind(&payload.schedule_start)
    .bind(&payload.schedule_end)
    .execute(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let block = sqlx::query_as::<_, Block>(
        "SELECT * FROM blocks WHERE id = $1"
    )
    .bind(block_id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(block))
}

/// Update block
pub async fn update_block(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateBlockRequest>,
) -> Result<Json<Block>, StatusCode> {
    // Require authentication
    let _claims = authenticate(State(state.clone()), axum::extract::Request::new(
        axum::body::Body::empty()
    )).await.map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Check permission
    let has_permission = sqlx::query_as::<_, (bool,)>(
        r#"SELECT EXISTS(
            SELECT 1 FROM role_permissions rp
            JOIN permissions p ON rp.permission_id = p.id
            WHERE rp.role_id = $1 AND p.action = 'update' AND p.resource = 'blocks'
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

    sqlx::query(
        r#"UPDATE blocks SET
            block_type = COALESCE($2, block_type),
            title = COALESCE($3, title),
            content = COALESCE($4, content),
            styling = COALESCE($5, styling),
            access_control = COALESCE($6, access_control),
            status = COALESCE($7, status),
            schedule_start = COALESCE($8, schedule_start),
            schedule_end = COALESCE($9, schedule_end),
            updated_at = NOW()
           WHERE id = $1
           RETURNING *"#
    )
    .bind(id)
    .bind(&payload.block_type)
    .bind(&payload.title)
    .bind(&payload.content)
    .bind(&payload.styling)
    .bind(&payload.access_control)
    .bind(&payload.status)
    .bind(&payload.schedule_start)
    .bind(&payload.schedule_end)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)
    .map(Json)
}

/// Delete block
pub async fn delete_block(
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
            WHERE rp.role_id = $1 AND p.action = 'delete' AND p.resource = 'blocks'
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

    sqlx::query("DELETE FROM blocks WHERE id = $1")
        .bind(id)
        .execute(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

/// Reorder blocks
pub async fn reorder_blocks(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ReorderBlocksRequest>,
) -> Result<Json<Vec<Block>>, StatusCode> {
    // Require authentication
    let _claims = authenticate(State(state.clone()), axum::extract::Request::new(
        axum::body::Body::empty()
    )).await.map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Check permission
    let has_permission = sqlx::query_as::<_, (bool,)>(
        r#"SELECT EXISTS(
            SELECT 1 FROM role_permissions rp
            JOIN permissions p ON rp.permission_id = p.id
            WHERE rp.role_id = $1 AND p.action = 'reorder' AND p.resource = 'blocks'
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

    // Update order for each block in a transaction
    for item in &payload.blocks {
        sqlx::query(
            "UPDATE blocks SET order_index = $2, updated_at = NOW() WHERE id = $1"
        )
        .bind(item.id)
        .bind(item.order_index)
        .execute(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    // Return all reordered blocks
    let blocks = sqlx::query_as::<_, Block>(
        "SELECT * FROM blocks WHERE id = ANY($1) ORDER BY order_index"
    )
    .bind(payload.blocks.iter().map(|b| b.id).collect::<Vec<_>>())
    .fetch_all(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(blocks))
}
