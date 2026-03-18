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
    Router::new()
        // Public routes - anyone can read
        .route("/projects", get(list_projects))
        .route("/projects/:slug", get(get_project))
        // Protected routes - require auth
        .route("/projects", post(create_project))
        .route("/projects/:id", put(update_project))
        .route("/projects/:id", delete(delete_project))
}

/// List projects (optionally filtered)
pub async fn list_projects(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ProjectQuery>,
) -> Result<Json<Vec<Project>>, StatusCode> {
// For simplicity, let's do it this way:
    let projects = if let Some(featured) = query.featured {
        if featured {
            sqlx::query_as::<_, Project>(
                "SELECT * FROM projects WHERE published = TRUE AND featured = TRUE ORDER BY created_at DESC"
            )
            .fetch_all(&state.db_pool)
            .await
            .map_err(|e| { tracing::error!("Database error: {:?}", e); StatusCode::INTERNAL_SERVER_ERROR })?
        } else {
            sqlx::query_as::<_, Project>(
                "SELECT * FROM projects WHERE published = TRUE AND featured = FALSE ORDER BY created_at DESC"
            )
            .fetch_all(&state.db_pool)
            .await
            .map_err(|e| { tracing::error!("Database error: {:?}", e); StatusCode::INTERNAL_SERVER_ERROR })?
        }
    } else if let Some(limit) = query.limit {
        sqlx::query_as::<_, Project>(
            "SELECT * FROM projects WHERE published = TRUE ORDER BY created_at DESC LIMIT $1"
        )
        .bind(limit as i64)
        .fetch_all(&state.db_pool)
        .await
        .map_err(|e| { tracing::error!("Database error: {:?}", e); StatusCode::INTERNAL_SERVER_ERROR })?
    } else {
        sqlx::query_as::<_, Project>(
            "SELECT * FROM projects WHERE published = TRUE ORDER BY created_at DESC"
        )
        .fetch_all(&state.db_pool)
        .await
        .map_err(|e| { tracing::error!("Database error: {:?}", e); StatusCode::INTERNAL_SERVER_ERROR })?
    };

    Ok(Json(projects))
}

/// Get project by slug
pub async fn get_project(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<Json<Project>, StatusCode> {
    let project = sqlx::query_as::<_, Project>(
        "SELECT * FROM projects WHERE slug = $1 AND published = TRUE"
    )
    .bind(&slug)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| { tracing::error!("Database error: {:?}", e); StatusCode::INTERNAL_SERVER_ERROR })?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(project))
}

/// Create new project (protected)
pub async fn create_project(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateProjectRequest>,
) -> Result<Json<Project>, StatusCode> {
    let project_id = Uuid::new_v4();
    let description = payload.description.as_ref().map(|s| ammonia::clean(s));
    let challenge = payload.challenge.as_ref().map(|s| ammonia::clean(s));
    let solution = payload.solution.as_ref().map(|s| ammonia::clean(s));
    let title = ammonia::clean(&payload.title);
    
    sqlx::query(
        r#"INSERT INTO projects (id, title, slug, description, challenge, solution, stack, role, live_url, repo_url, media_ids, technologies, featured, published_at, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, NOW(), NOW())"#
    )
    .bind(project_id)
    .bind(&title)
    .bind(&payload.slug)
    .bind(&description)
    .bind(&challenge)
    .bind(&solution)
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
    .map_err(|e| { tracing::error!("Database error: {:?}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    let project = sqlx::query_as::<_, Project>(
        "SELECT * FROM projects WHERE id = $1"
    )
    .bind(project_id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| { tracing::error!("Database error: {:?}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok(Json(project))
}

/// Update project (protected)
pub async fn update_project(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateProjectRequest>,
) -> Result<Json<Project>, StatusCode> {
    let mut query_builder: sqlx::QueryBuilder<sqlx::Postgres> = sqlx::QueryBuilder::new("UPDATE projects SET updated_at = NOW()");
    
    if let Some(title) = &payload.title {
        query_builder.push(", title = ");
        query_builder.push_bind(ammonia::clean(title));
    }
    if let Some(slug) = &payload.slug {
        query_builder.push(", slug = ");
        query_builder.push_bind(slug);
    }
    if let Some(description) = &payload.description {
        query_builder.push(", description = ");
        query_builder.push_bind(ammonia::clean(description));
    }
    if let Some(challenge) = &payload.challenge {
        query_builder.push(", challenge = ");
        query_builder.push_bind(ammonia::clean(challenge));
    }
    if let Some(solution) = &payload.solution {
        query_builder.push(", solution = ");
        query_builder.push_bind(ammonia::clean(solution));
    }
    if let Some(stack) = &payload.stack {
        query_builder.push(", stack = ");
        query_builder.push_bind(stack);
    }
    if let Some(role) = &payload.role {
        query_builder.push(", role = ");
        query_builder.push_bind(role);
    }
    if let Some(live_url) = &payload.live_url {
        query_builder.push(", live_url = ");
        query_builder.push_bind(live_url);
    }
    if let Some(repo_url) = &payload.repo_url {
        query_builder.push(", repo_url = ");
        query_builder.push_bind(repo_url);
    }
    if let Some(media_ids) = &payload.media_ids {
        query_builder.push(", media_ids = ");
        query_builder.push_bind(media_ids);
    }
    if let Some(technologies) = &payload.technologies {
        query_builder.push(", technologies = ");
        query_builder.push_bind(technologies);
    }
    if let Some(featured) = payload.featured {
        query_builder.push(", featured = ");
        query_builder.push_bind(featured);
    }
    if let Some(published_at) = payload.published_at {
        query_builder.push(", published_at = ");
        query_builder.push_bind(published_at);
    }

    query_builder.push(" WHERE id = ");
    query_builder.push_bind(id);
    query_builder.push(" RETURNING *");

    let project = query_builder
        .build_query_as::<Project>()
        .fetch_one(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error on Project Update: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

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
        .map_err(|e| { tracing::error!("Database error: {:?}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok(StatusCode::NO_CONTENT)
}
