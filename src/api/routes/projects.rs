use axum::{Router, routing::{get, post}, Json, extract::{State, Path}};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
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
    pub repository_url: Option<String>,
    pub language: Option<String>,
    pub default_branch: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub repository_url: Option<String>,
    pub language: Option<String>,
    pub default_branch: Option<String>,
}

#[derive(Serialize)]
pub struct ProjectResponse {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub repository_url: Option<String>,
    pub default_branch: String,
    pub language: String,
    pub created_at: String,
    pub updated_at: String,
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
        req.language.unwrap_or_else(|| "rust".to_string()),
    );

    let created = state.project_repository.create(&project).await?;

    Ok(ApiResponse::ok(ProjectResponse {
        id: created.id.to_string(),
        name: created.name,
        slug: created.slug,
        description: created.description,
        repository_url: created.repository_url,
        default_branch: created.default_branch,
        language: created.language,
        created_at: created.created_at.to_rfc3339(),
        updated_at: created.updated_at.to_rfc3339(),
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
        repository_url: p.repository_url,
        default_branch: p.default_branch,
        language: p.language,
        created_at: p.created_at.to_rfc3339(),
        updated_at: p.updated_at.to_rfc3339(),
    }).collect();
    Ok(Json(resp))
}

async fn get_project(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<ProjectResponse>>, ApiError> {
    if !auth.is_authenticated {
        return Err(ApiError::Auth(crate::auth::AuthError::PermissionDenied));
    }

    let project = state.project_repository
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Project not found".to_string()))?;

    Ok(ApiResponse::ok(ProjectResponse {
        id: project.id.to_string(),
        name: project.name,
        slug: project.slug,
        description: project.description,
        repository_url: project.repository_url,
        default_branch: project.default_branch,
        language: project.language,
        created_at: project.created_at.to_rfc3339(),
        updated_at: project.updated_at.to_rfc3339(),
    }))
}

async fn update_project(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateProjectRequest>,
) -> Result<Json<ApiResponse<ProjectResponse>>, ApiError> {
    if !auth.is_authenticated {
        return Err(ApiError::Auth(crate::auth::AuthError::PermissionDenied));
    }

    let mut project = state.project_repository
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Project not found".to_string()))?;

    if let Some(name) = req.name {
        project.name = name.clone();
        project.slug = name.to_lowercase().replace(' ', "-");
    }
    if let Some(desc) = req.description {
        project.description = Some(desc);
    }
    if let Some(url) = req.repository_url {
        project.repository_url = Some(url);
    }
    if let Some(lang) = req.language {
        project.language = lang;
    }
    if let Some(branch) = req.default_branch {
        project.default_branch = branch;
    }

    let updated = state.project_repository.update(&project).await?;

    Ok(ApiResponse::ok(ProjectResponse {
        id: updated.id.to_string(),
        name: updated.name,
        slug: updated.slug,
        description: updated.description,
        repository_url: updated.repository_url,
        default_branch: updated.default_branch,
        language: updated.language,
        created_at: updated.created_at.to_rfc3339(),
        updated_at: updated.updated_at.to_rfc3339(),
    }))
}

async fn delete_project(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    if !auth.is_authenticated {
        return Err(ApiError::Auth(crate::auth::AuthError::PermissionDenied));
    }

    let project = state.project_repository
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Project not found".to_string()))?;

    state.project_repository.delete(project.id).await?;

    Ok(ApiResponse::ok(()))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/projects", post(create_project).get(list_projects))
        .route("/v1/projects/{id}", get(get_project).put(update_project).delete(delete_project))
}
