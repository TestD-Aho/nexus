//! Media API Routes

use axum::{
    extract::{State, Path, Multipart},
    http::StatusCode,
    response::Json,
    routing::{get, post, delete},
    Router,
};
use std::sync::Arc;
use uuid::Uuid;
use crate::models::Media;
use crate::services::app_state::AppState;

/// Create media router
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/media", get(list_media))
        .route("/media/upload", post(upload_media))
        .route("/media/:id", delete(delete_media))
}

/// List all media
pub async fn list_media(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Media>>, StatusCode> {
    sqlx::query_as::<_, Media>(
        "SELECT * FROM media ORDER BY created_at DESC LIMIT 100"
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    .map(Json)
}

/// Upload media
pub async fn upload_media(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<Media>, StatusCode> {
    let field = multipart.next_field().await
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .ok_or(StatusCode::BAD_REQUEST)?;

    let filename = field.file_name()
        .ok_or(StatusCode::BAD_REQUEST)?
        .to_string();
    
    let data = field.bytes().await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    let mime_type = mime_guess::from_path(&filename)
        .first_or_octet_stream()
        .to_string();

    let media_id = Uuid::new_v4();
    let extension = std::path::Path::new(&filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin");
    
    let stored_filename = format!("{}.{}", media_id, extension);
    let url = format!("/uploads/{}", stored_filename);

    // Save file to disk
    let upload_path = std::path::Path::new(&state.config.upload_dir);
    tokio::fs::create_dir_all(upload_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let file_path = upload_path.join(&stored_filename);
    tokio::fs::write(&file_path, &data)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Save to database
    sqlx::query(
        r#"INSERT INTO media (id, filename, original_filename, mime_type, size, url, created_at)
           VALUES ($1, $2, $3, $4, $5, $6, NOW())"#
    )
    .bind(media_id)
    .bind(&stored_filename)
    .bind(&filename)
    .bind(&mime_type)
    .bind(data.len() as i64)
    .bind(&url)
    .execute(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let media = sqlx::query_as::<_, Media>(
        "SELECT * FROM media WHERE id = $1"
    )
    .bind(media_id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(media))
}

/// Delete media
pub async fn delete_media(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    // Get filename
    let filename: Option<String> = sqlx::query(
        "SELECT filename FROM media WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .and_then(|row| row.get(0));

    // Delete file
    if let Some(fname) = filename {
        let file_path = std::path::Path::new(&state.config.upload_dir).join(&fname);
        let _ = tokio::fs::remove_file(file_path).await;
    }

    // Delete from database
    sqlx::query("DELETE FROM media WHERE id = $1")
        .bind(id)
        .execute(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}
