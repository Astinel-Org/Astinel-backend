use axum::{Router, routing::{get, post}, Json, extract::{State, Path}};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::state::AppState;
use crate::auth::AuthContext;
use crate::api::response::ApiResponse;
use crate::api::errors::ApiError;

#[derive(Deserialize)]
pub struct TriggerScanRequest {
    pub project_id: String,
    pub branch: Option<String>,
}

#[derive(Serialize)]
pub struct ScanResponse {
    pub id: String,
    pub status: String,
    pub project_id: String,
    pub created_at: String,
}

async fn trigger_scan(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Json(req): Json<TriggerScanRequest>,
) -> Result<Json<ApiResponse<ScanResponse>>, ApiError> {
    if !auth.is_authenticated {
        return Err(ApiError::Auth(crate::auth::AuthError::PermissionDenied));
    }

    let project_id = Uuid::parse_str(&req.project_id)
        .map_err(|_| ApiError::BadRequest("Invalid project_id".to_string()))?;

    let job = state.scan_service
        .enqueue_scan(project_id, req.branch.unwrap_or_else(|| "main".to_string()))
        .await?;

    Ok(ApiResponse::ok(ScanResponse {
        id: job.id.to_string(),
        status: job.status,
        project_id: job.project_id.to_string(),
        created_at: job.created_at.to_rfc3339(),
    }))
}

async fn list_scans(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
) -> Result<Json<Vec<ScanResponse>>, ApiError> {
    if !auth.is_authenticated {
        return Err(ApiError::Auth(crate::auth::AuthError::PermissionDenied));
    }
    let org_id = auth.org_id.unwrap_or_default();
    let scans = state.scan_repository
        .list_jobs_for_project(org_id)
        .await?;
    let resp = scans.into_iter().map(|j| ScanResponse {
        id: j.id.to_string(),
        status: j.status,
        project_id: j.project_id.to_string(),
        created_at: j.created_at.to_rfc3339(),
    }).collect();
    Ok(Json(resp))
}

async fn get_scan(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<ScanResponse>>, ApiError> {
    if !auth.is_authenticated {
        return Err(ApiError::Auth(crate::auth::AuthError::PermissionDenied));
    }
    let job = state.scan_repository
        .find_job_by_id(id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Scan not found".to_string()))?;

    Ok(ApiResponse::ok(ScanResponse {
        id: job.id.to_string(),
        status: job.status,
        project_id: job.project_id.to_string(),
        created_at: job.created_at.to_rfc3339(),
    }))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/scans", post(trigger_scan).get(list_scans))
        .route("/v1/scans/{id}", get(get_scan))
}
