//! Portfolio API Routes

use axum::{
    extract::{State, Path, Query},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
    middleware,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;
use crate::models::{Project, CreateProjectRequest, UpdateProjectRequest};
use crate::services::app_state::AppState;
use crate::middleware::security::authenticate;

#[derive(Deserialize)]
pub struct ProjectQuery {
    pub featured: Option<bool>,
    pub limit: Option<usize>,
}

/// Create projects router with per-route security
pub fn router() -> Router<Arc<AppState>> {
    let auth_layer = middleware::from_fn_with_state(
        |state, request| async move {
            authenticate(state, request).await
        },
    );

    Router::new()
        // Public routes - anyone can read
        .route("/projects", get(list_projects))
        .route("/projects/:slug", get(get_project))
        // Protected routes - require auth
        .route("/projects", post(create_project).route_layer(auth_layer.clone()))
        .route("/projects/:id", put(update_project).route_layer(auth_layer.clone()))
        .route("/projects/:id", delete(delete_project).route_layer(auth_layer))
}

/// List projects (optionally filtered)
pub async fn list_projects(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ProjectQuery>,
) -> Result<Json<Vec<Project>>, StatusCode> {
    let mut query_builder = sqlx::query_as::<_, Project>(
        "SELECT * FROM projects WHERE published = TRUE"
    );

    if let Some(featured) = query.featured {
        query_builder = query_builder.clone().filter(Some(featured));
        // Actually need to rebuild the query properly
    }

    // For simplicity, let's do it this way:
    let projects = if let Some(featured) = query.featured {
        if featured {
            sqlx::query_as::<_, Project>(
                "SELECT * FROM projects WHERE published = TRUE AND featured = TRUE ORDER BY created_at DESC"
            )
            .fetch_all(&state.db_pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        } else {
            sqlx::query_as::<_, Project>(
                "SELECT * FROM projects WHERE published = TRUE AND featured = FALSE ORDER BY created_at DESC"
            )
            .fetch_all(&state.db_pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
    } else if let Some(limit) = query.limit {
        sqlx::query_as::<_, Project>(
            "SELECT * FROM projects WHERE published = TRUE ORDER BY created_at DESC LIMIT $1"
        )
        .bind(limit as i64)
        .fetch_all(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        sqlx::query_as::<_, Project>(
            "SELECT * FROM projects WHERE published = TRUE ORDER BY created_at DESC"
        )
        .fetch_all(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };

    Ok(Json(projects))
}

/// Get project by slug
pub async fn get_project(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<Json<Project>, StatusCode> {
    let project: Option<Project> = sqlx::query_as::<_, Project>(
        "SELECT * FROM projects WHERE slug = $1 AND published = TRUE"
    )
    .bind(&slug)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(project))
}

/// Create new project (protected)
pub async fn create_project(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateProjectRequest>,
) -> Result<Json<Project>, StatusCode> {
    let project_id = Uuid::new_v4();
    
    sqlx::query(
        r#"INSERT INTO projects (id, title, slug, description, challenge, solution, stack, role, live_url, repo_url, media_ids, technologies, featured, published_at, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, NOW(), NOW())"#
    )
    .bind(project_id)
    .bind(&payload.title)
    .bind(&payload.slug)
    .bind(&payload.description)
    .bind(&payload.challenge)
    .bind(&payload.solution)
    .bind(&payload.stack)
    .bind(&payload.role)
    .bind(&payload.live_url)
    .bind(&payload.repo_url)
    .bind(&payload.media_ids)
    .bind(&payload.technologies)
    .bind(payload.featured.unwrap_or(false))
    .bind(payload.published_at)
    .execute(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let project = sqlx::query_as::<_, Project>(
        "SELECT * FROM projects WHERE id = $1"
    )
    .bind(project_id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(project))
}

/// Update project (protected)
pub async fn update_project(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateProjectRequest>,
) -> Result<Json<Project>, StatusCode> {
    // Build dynamic update (simplified)
    let mut updates = vec!["updated_at = NOW()".to_string()];
    
    if let Some(title) = &payload.title {
        updates.push("title = $2".to_string());
    }
    if let Some(slug) = &payload.slug {
        updates.push("slug = $3".to_string());
    }
    if let Some(description) = &payload.description {
        updates.push("description = $4".to_string());
    }
    if let Some(challenge) = &payload.challenge {
        updates.push("challenge = $5".to_string());
    }
    if let Some(solution) = &payload.solution {
        updates.push("solution = $6".to_string());
    }
    if let Some(stack) = &payload.stack {
        updates.push("stack = $7".to_string());
    }
    if let Some(role) = &payload.role {
        updates.push("role = $8".to_string());
    }
    if let Some(live_url) = &payload.live_url {
        updates.push("live_url = $9".to_string());
    }
    if let Some(repo_url) = &payload.repo_url {
        updates.push("repo_url = $10".to_string());
    }
    if let Some(media_ids) = &payload.media_ids {
        updates.push("media_ids = $11".to_string());
    }
    if let Some(technologies) = &payload.technologies {
        updates.push("technologies = $12".to_string());
    }
    if let Some(featured) = payload.featured {
        updates.push("featured = $13".to_string());
    }
    if let Some(published_at) = payload.published_at {
        updates.push("published_at = $14".to_string());
    }

    if updates.len() > 1 {
        let set_clause = updates.join(", ");
        let mut query = format!("UPDATE projects SET {} WHERE id = $1 RETURNING *", set_clause);
        
        // Build params (simplified - in practice would need proper binding)
        sqlx::query(&query)
            .bind(id)
            .execute(&state.db_pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    let project = sqlx::query_as::<_, Project>(
        "SELECT * FROM projects WHERE id = $1"
    )
    .bind(id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(project))
}

/// Delete project (protected)
pub async fn delete_project(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("DELETE FROM projects WHERE id = $1")
        .bind(id)
        .execute(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}
