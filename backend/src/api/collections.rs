//! Collections API Routes (Custom Post Types)

use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
    middleware,
};
use std::sync::Arc;
use uuid::Uuid;
use crate::models::{Collection, CollectionItem, CreateCollectionItemRequest, CreateCollectionRequest, UpdateCollectionItemRequest};
use crate::services::app_state::AppState;
use crate::middleware::security::authenticate;

/// Create collections router with per-route security
pub fn router() -> Router<Arc<AppState>> {
    // let auth_layer = middleware::from_fn_with_state(
        // |state, request| async move {
            // authenticate(state, request).await
        },
    );

    Router::new()
        // Public routes - anyone can read
        .route("/collections", get(list_collections))
        .route("/collections/:name", get(get_collection))
        // Protected routes - require auth
        .route("/collections", post(create_collection).route_layer(// auth_layer.clone()))
        .route("/collections/:name/items", post(create_item_in_collection).route_layer(// auth_layer.clone()))
        .route("/collections/:name/items/:id", put(update_item_in_collection).route_layer(// auth_layer.clone()))
        .route("/collections/:name/items/:id", delete(delete_item_in_collection).route_layer(// auth_layer))
}

/// List all collections (public)
pub async fn list_collections(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Collection>>, StatusCode> {
    sqlx::query_as::<_, Collection>(
        "SELECT * FROM collections ORDER BY name"
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    .map(Json)
}

/// Create collection (protected)
pub async fn create_collection(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateCollectionRequest>,
) -> Result<Json<Collection>, StatusCode> {
    let id = Uuid::new_v4();
    
    sqlx::query(
        "INSERT INTO collections (id, name, slug, description, schema, created_at) VALUES ($1, $2, $3, $4, $5, NOW())"
    )
    .bind(id)
    .bind(&payload.name)
    .bind(&payload.slug)
    .bind(&payload.description)
    .bind(&payload.schema)
    .execute(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let collection = sqlx::query_as::<_, Collection>(
        "SELECT * FROM collections WHERE id = $1"
    )
    .bind(id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(collection))
}

/// Get collection by name and its items (public)
pub async fn get_collection(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Get collection
    let collection: Collection = sqlx::query_as(
        "SELECT * FROM collections WHERE slug = $1 OR name = $1"
    )
    .bind(&name)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    // Get items
    let items = sqlx::query_as::<_, CollectionItem>(
        "SELECT * FROM collection_items WHERE collection_id = $1 ORDER BY created_at DESC"
    )
    .bind(collection.id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "collection": collection,
        "items": items
    })))
}

/// Create item in collection (protected)
pub async fn create_item_in_collection(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(payload): Json<CreateCollectionItemRequest>,
) -> Result<Json<CollectionItem>, StatusCode> {
    // Get collection
    let collection: Collection = sqlx::query_as(
        "SELECT * FROM collections WHERE slug = $1 OR name = $1"
    )
    .bind(&name)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let item_id = Uuid::new_v4();
    
    sqlx::query(
        r#"INSERT INTO collection_items (id, collection_id, data, created_at, updated_at)
           VALUES ($1, $2, $3, NOW(), NOW())"#
    )
    .bind(item_id)
    .bind(collection.id)
    .bind(&payload.data)
    .execute(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let item = sqlx::query_as::<_, CollectionItem>(
        "SELECT * FROM collection_items WHERE id = $1"
    )
    .bind(item_id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(item))
}

/// Update item in collection (protected)
pub async fn update_item_in_collection(
    State(state): State<Arc<AppState>>,
    Path((_name, id)): Path<(String, Uuid)>,
    Json(payload): Json<UpdateCollectionItemRequest>,
) -> Result<Json<CollectionItem>, StatusCode> {
    sqlx::query(
        "UPDATE collection_items SET data = $2, updated_at = NOW() WHERE id = $1 RETURNING *"
    )
    .bind(id)
    .bind(&payload.data)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)
    .map(Json)
}

/// Delete item from collection (protected)
pub async fn delete_item_in_collection(
    State(state): State<Arc<AppState>>,
    Path((_name, id)): Path<(String, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("DELETE FROM collection_items WHERE id = $1")
        .bind(id)
        .execute(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}
