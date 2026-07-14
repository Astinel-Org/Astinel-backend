use axum::{Router, routing::get, Json, extract::{State, Path, Query}};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::state::AppState;
use crate::auth::AuthContext;
use crate::api::response::ApiResponse;
use crate::api::errors::ApiError;
use crate::database::repositories::report_repository::ReportRepository;

#[derive(Serialize)]
pub struct ReportResponse {
    pub id: String,
    pub scan_result_id: String,
    pub format: String,
    pub content: String,
    pub file_path: Option<String>,
    pub file_size: i64,
    pub created_at: String,
}

#[derive(Deserialize)]
pub struct ReportsQuery {
    pub scan_result_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
}

async fn get_report(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<ReportResponse>>, ApiError> {
    if !auth.is_authenticated {
        return Err(ApiError::Auth(crate::auth::AuthError::PermissionDenied));
    }
    let report = state.report_repository
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Report not found".to_string()))?;

    Ok(ApiResponse::ok(ReportResponse {
        id: report.id.to_string(),
        scan_result_id: report.scan_result_id.to_string(),
        format: report.format,
        content: report.content,
        file_path: report.file_path,
        file_size: report.file_size,
        created_at: report.created_at.to_rfc3339(),
    }))
}

async fn list_reports(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Query(query): Query<ReportsQuery>,
) -> Result<Json<Vec<ReportResponse>>, ApiError> {
    if !auth.is_authenticated {
        return Err(ApiError::Auth(crate::auth::AuthError::PermissionDenied));
    }

    let reports = if let Some(scan_result_id) = query.scan_result_id {
        state.report_repository.find_by_scan_result(scan_result_id).await?
    } else if let Some(project_id) = query.project_id {
        state.report_repository.find_by_project(project_id).await?
    } else {
        return Err(ApiError::BadRequest("Provide scan_result_id or project_id".to_string()));
    };

    let resp = reports.into_iter().map(|r| ReportResponse {
        id: r.id.to_string(),
        scan_result_id: r.scan_result_id.to_string(),
        format: r.format,
        content: r.content,
        file_path: r.file_path,
        file_size: r.file_size,
        created_at: r.created_at.to_rfc3339(),
    }).collect();
    Ok(Json(resp))
}

async fn download_report(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Path((format, scan_result_id)): Path<(String, Uuid)>,
) -> Result<Json<ApiResponse<ReportResponse>>, ApiError> {
    if !auth.is_authenticated {
        return Err(ApiError::Auth(crate::auth::AuthError::PermissionDenied));
    }

    let reports = state.report_repository.find_by_scan_result(scan_result_id).await?;
    let report = reports.into_iter()
        .find(|r| r.format == format)
        .ok_or_else(|| ApiError::NotFound("Report not found for this format".to_string()))?;

    Ok(ApiResponse::ok(ReportResponse {
        id: report.id.to_string(),
        scan_result_id: report.scan_result_id.to_string(),
        format: report.format,
        content: report.content,
        file_path: report.file_path,
        file_size: report.file_size,
        created_at: report.created_at.to_rfc3339(),
    }))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/reports", get(list_reports))
        .route("/v1/reports/{id}", get(get_report))
        .route("/v1/reports/{format}/{scan_result_id}", get(download_report))
}
