use axum::{Router, routing::post, Json, extract::State};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::state::AppState;
use crate::auth::AuthContext;
use crate::api::response::ApiResponse;
use crate::api::errors::ApiError;
use crate::database::models::Project;
use crate::database::repositories::project_repository::ProjectRepository;

#[derive(Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize)]
pub struct ProjectResponse {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub created_at: String,
}

async fn create_project(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Json(req): Json<CreateProjectRequest>,
) -> Result<Json<ApiResponse<ProjectResponse>>, ApiError> {
    if !auth.is_authenticated {
        return Err(ApiError::Auth(crate::auth::AuthError::PermissionDenied));
    }

    let org_id = auth.org_id.ok_or_else(|| ApiError::BadRequest("No organization context".to_string()))?;

    let project = Project::new(
        org_id,
        req.name.clone(),
        req.name.to_lowercase().replace(' ', "-"),
        "rust".to_string(),
    );

    let created = state.project_repository.create(&project).await?;

    Ok(ApiResponse::ok(ProjectResponse {
        id: created.id.to_string(),
        name: created.name,
        slug: created.slug,
        description: created.description,
        created_at: created.created_at.to_rfc3339(),
    }))
}

async fn list_projects(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
) -> Result<Json<Vec<ProjectResponse>>, ApiError> {
    if !auth.is_authenticated {
        return Err(ApiError::Auth(crate::auth::AuthError::PermissionDenied));
    }
    let org_id = auth.org_id.unwrap_or_default();
    let projects = state.project_repository.find_by_organization(org_id).await?;
    let resp: Vec<ProjectResponse> = projects.into_iter().map(|p| ProjectResponse {
        id: p.id.to_string(),
        name: p.name,
        slug: p.slug,
        description: p.description,
        created_at: p.created_at.to_rfc3339(),
    }).collect();
    Ok(Json(resp))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/projects", post(create_project).get(list_projects))
}
