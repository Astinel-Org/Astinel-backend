use axum::{Router, routing::{get, post}, Json, extract::{State, Path}};
use std::sync::Arc;
use serde::Serialize;
use uuid::Uuid;
use crate::state::AppState;
use crate::auth::AuthContext;
use crate::api::response::ApiResponse;
use crate::api::errors::ApiError;
use crate::database::repositories::scan_repository::ScanRepository;

#[derive(Serialize)]
pub struct ScanResponse {
    pub id: String,
    pub project_id: String,
    pub branch: String,
    pub status: String,
    pub trigger: String,
    pub priority: i32,
    pub score: Option<i32>,
    pub progress: u8,
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[derive(Serialize)]
pub struct ProgressResponse {
    pub scan_id: String,
    pub status: String,
    pub progress: u8,
}

async fn trigger_scan(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<ScanResponse>>, ApiError> {
    if !auth.is_authenticated {
        return Err(ApiError::Auth(crate::auth::AuthError::PermissionDenied));
    }

    let project_id_str = req.get("project_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::BadRequest("Missing project_id".to_string()))?;
    let project_id = Uuid::parse_str(project_id_str)
        .map_err(|_| ApiError::BadRequest("Invalid project_id".to_string()))?;
    let branch = req.get("branch")
        .and_then(|v| v.as_str())
        .unwrap_or("main")
        .to_string();

    let job = state.scan_service
        .enqueue_scan(project_id, branch)
        .await?;

    Ok(ApiResponse::ok(ScanResponse {
        id: job.id.to_string(),
        project_id: job.project_id.to_string(),
        branch: job.branch,
        status: job.status,
        trigger: job.trigger,
        priority: job.priority,
        score: None,
        progress: 0,
        created_at: job.created_at.to_rfc3339(),
        completed_at: None,
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
    let mut resp = Vec::new();
    for j in scans {
        let result = state.scan_repository.find_result_by_job(j.id).await.ok().flatten();
        let progress = state.scan_status_cache.get_progress(&j.id.to_string()).await
            .map(|(p, _)| p)
            .unwrap_or(0);
        resp.push(ScanResponse {
            id: j.id.to_string(),
            project_id: j.project_id.to_string(),
            branch: j.branch,
            status: j.status,
            trigger: j.trigger,
            priority: j.priority,
            score: result.as_ref().map(|r| r.score),
            progress,
            created_at: j.created_at.to_rfc3339(),
            completed_at: j.completed_at.map(|t| t.to_rfc3339()),
        });
    }
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

    let result = state.scan_repository.find_result_by_job(job.id).await.ok().flatten();
    let progress = state.scan_status_cache.get_progress(&id.to_string()).await
        .map(|(p, _)| p)
        .unwrap_or(0);

    Ok(ApiResponse::ok(ScanResponse {
        id: job.id.to_string(),
        project_id: job.project_id.to_string(),
        branch: job.branch,
        status: job.status,
        trigger: job.trigger,
        priority: job.priority,
        score: result.as_ref().map(|r| r.score),
        progress,
        created_at: job.created_at.to_rfc3339(),
        completed_at: job.completed_at.map(|t| t.to_rfc3339()),
    }))
}

async fn cancel_scan(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    if !auth.is_authenticated {
        return Err(ApiError::Auth(crate::auth::AuthError::PermissionDenied));
    }

    let job = state.scan_repository
        .find_job_by_id(id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Scan not found".to_string()))?;

    if job.status == "completed" || job.status == "failed" || job.status == "cancelled" {
        return Err(ApiError::BadRequest("Scan already finished".to_string()));
    }

    let mut updated = job.clone();
    updated.status = "cancelled".to_string();
    updated.completed_at = Some(chrono::Utc::now());
    state.scan_repository.update_job(&updated).await?;

    state.scan_status_cache.mark_cancelled(&id.to_string()).await.ok();

    Ok(ApiResponse::ok(()))
}

async fn retry_scan(
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

    let new_job = state.scan_service
        .enqueue_scan(job.project_id, job.branch)
        .await?;

    Ok(ApiResponse::ok(ScanResponse {
        id: new_job.id.to_string(),
        project_id: new_job.project_id.to_string(),
        branch: new_job.branch,
        status: new_job.status,
        trigger: new_job.trigger,
        priority: new_job.priority,
        score: None,
        progress: 0,
        created_at: new_job.created_at.to_rfc3339(),
        completed_at: None,
    }))
}

async fn get_scan_progress(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<ProgressResponse>>, ApiError> {
    if !auth.is_authenticated {
        return Err(ApiError::Auth(crate::auth::AuthError::PermissionDenied));
    }

    let job = state.scan_repository
        .find_job_by_id(id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Scan not found".to_string()))?;

    let (progress, _phase) = state.scan_status_cache.get_progress(&id.to_string()).await
        .unwrap_or((0, "unknown".to_string()));

    Ok(ApiResponse::ok(ProgressResponse {
        scan_id: id.to_string(),
        status: job.status,
        progress,
    }))
}

async fn get_scan_result(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    if !auth.is_authenticated {
        return Err(ApiError::Auth(crate::auth::AuthError::PermissionDenied));
    }

    let result = state.scan_repository
        .find_result_by_job(id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Scan result not found".to_string()))?;

    Ok(ApiResponse::ok(serde_json::json!({
        "id": result.id.to_string(),
        "scan_job_id": result.scan_job_id.to_string(),
        "status": result.status,
        "score": result.score,
        "total_files": result.total_files,
        "total_rules": result.total_rules,
        "total_findings": result.total_findings,
        "suppressed_findings": result.suppressed_findings,
        "critical": result.critical,
        "high": result.high,
        "medium": result.medium,
        "low": result.low,
        "info": result.info,
        "duration_ms": result.duration_ms,
        "created_at": result.created_at.to_rfc3339(),
    })))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/scans", post(trigger_scan).get(list_scans))
        .route("/v1/scans/{id}", get(get_scan))
        .route("/v1/scans/{id}/result", get(get_scan_result))
        .route("/v1/scans/{id}/progress", get(get_scan_progress))
        .route("/v1/scans/{id}/cancel", post(cancel_scan))
        .route("/v1/scans/{id}/retry", post(retry_scan))
}
