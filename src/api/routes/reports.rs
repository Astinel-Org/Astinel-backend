use axum::{Router, routing::get, Json, extract::{State, Path}};
use std::sync::Arc;
use serde::Serialize;
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
    pub created_at: String,
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
        created_at: report.created_at.to_rfc3339(),
    }))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/v1/reports/{id}", get(get_report))
}
