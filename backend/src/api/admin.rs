//! Admin API Routes

use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::Json,
    routing::{get, post, put},
    Router,
};
use std::sync::Arc;
use uuid::Uuid;
use crate::models::{AdminStats, CreateRoleRequest, Permission, Role, User};
use crate::AppState;
use crate::middleware::security::require_admin;

/// Create admin router
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/admin/users", get(list_users))
        .route("/api/v1/admin/users/:id", put(update_user))
        .route("/api/v1/admin/roles", get(list_roles))
        .route("/api/v1/admin/roles", post(create_role))
        .route("/api/v1/admin/permissions", get(list_permissions))
        .route("/api/v1/admin/stats", get(get_stats))
}

/// List all users (admin only)
pub async fn list_users(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
    let users = sqlx::query(
        r#"SELECT u.id, u.email, u.role_id, u.is_active, u.created_at, u.last_login, r.name as role_name
           FROM users u
           JOIN roles r ON u.role_id = r.id
           ORDER BY u.created_at DESC"#
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result: Vec<serde_json::Value> = users
        .iter()
        .map(|row| {
            serde_json::json!({
                "id": row.get::<Uuid, _>("id"),
                "email": row.get::<String, _>("email"),
                "role_id": row.get::<Uuid, _>("role_id"),
                "role_name": row.get::<String, _>("role_name"),
                "is_active": row.get::<bool, _>("is_active"),
                "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
                "last_login": row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("last_login"),
            })
        })
        .collect();

    Ok(Json(result))
}

/// Update user (admin only)
pub async fn update_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Update user fields
    if let Some(role_id) = payload.get("role_id").and_then(|v| v.as_str()) {
        if let Ok(uuid) = Uuid::parse_str(role_id) {
            sqlx::query("UPDATE users SET role_id = $2, updated_at = NOW() WHERE id = $1")
                .bind(id)
                .bind(uuid)
                .execute(&state.db_pool)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }
    }

    if let Some(is_active) = payload.get("is_active").and_then(|v| v.as_bool()) {
        sqlx::query("UPDATE users SET is_active = $2, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .bind(is_active)
            .execute(&state.db_pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    // Fetch updated user
    let user = sqlx::query(
        r#"SELECT u.id, u.email, u.role_id, u.is_active, u.created_at, r.name as role_name
           FROM users u
           JOIN roles r ON u.role_id = r.id
           WHERE u.id = $1"#
    )
    .bind(id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(serde_json::json!({
        "id": user.get::<Uuid, _>("id"),
        "email": user.get::<String, _>("email"),
        "role_id": user.get::<Uuid, _>("role_id"),
        "role_name": user.get::<String, _>("role_name"),
        "is_active": user.get::<bool, _>("is_active"),
    })))
}

/// List all roles
pub async fn list_roles(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Role>>, StatusCode> {
    sqlx::query_as::<_, Role>(
        "SELECT * FROM roles ORDER BY is_system DESC, name"
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    .map(Json)
}

/// Create new role
pub async fn create_role(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateRoleRequest>,
) -> Result<Json<Role>, StatusCode> {
    let role_id = Uuid::new_v4();
    
    sqlx::query(
        "INSERT INTO roles (id, name, description, is_system, created_at) VALUES ($1, $2, $3, FALSE, NOW())"
    )
    .bind(role_id)
    .bind(&payload.name)
    .bind(&payload.description)
    .execute(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Add permissions
    for perm_id in payload.permissions {
        sqlx::query(
            "INSERT INTO role_permissions (role_id, permission_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
        )
        .bind(role_id)
        .bind(perm_id)
        .execute(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    let role = sqlx::query_as::<_, Role>(
        "SELECT * FROM roles WHERE id = $1"
    )
    .bind(role_id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(role))
}

/// List all permissions
pub async fn list_permissions(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Permission>>, StatusCode> {
    sqlx::query_as::<_, Permission>(
        "SELECT * FROM permissions ORDER BY resource, action"
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    .map(Json)
}

/// Get admin stats
pub async fn get_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<AdminStats>, StatusCode> {
    let total_users: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total_pages: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM pages")
        .fetch_one(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total_blocks: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM blocks")
        .fetch_one(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total_collections: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM collections")
        .fetch_one(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total_media: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM media")
        .fetch_one(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AdminStats {
        total_users: total_users.0,
        total_pages: total_pages.0,
        total_blocks: total_blocks.0,
        total_collections: total_collections.0,
        total_media: total_media.0,
        active_sessions: 0, // Would need Redis for this
    }))
}
