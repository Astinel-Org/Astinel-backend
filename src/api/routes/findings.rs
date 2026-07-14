use crate::api::errors::ApiError;
use crate::api::response::ApiResponse;
use crate::auth::AuthContext;
use crate::database::repositories::finding_repository::FindingRepository;
use crate::database::repositories::scan_repository::ScanRepository;
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct FindingsQuery {
    pub scan_id: Option<Uuid>,
    pub severity: Option<String>,
    pub category: Option<String>,
    pub file_path: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}

#[derive(Serialize)]
pub struct FindingResponse {
    pub id: String,
    pub scan_result_id: String,
    pub rule_id: String,
    pub severity: String,
    pub category: String,
    pub file_path: String,
    pub line: i32,
    pub column: i32,
    pub message: String,
    pub recommendation: String,
    pub fix_example: Option<String>,
    pub is_suppressed: bool,
    pub created_at: String,
}

#[derive(Deserialize)]
pub struct PatchFindingRequest {
    pub is_suppressed: Option<bool>,
}

async fn list_findings(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Query(query): Query<FindingsQuery>,
) -> Result<Json<Vec<FindingResponse>>, ApiError> {
    if !auth.is_authenticated {
        return Err(ApiError::Auth(crate::auth::AuthError::PermissionDenied));
    }
    let findings = if let Some(scan_id) = query.scan_id {
        let result = state.scan_repository.find_result_by_job(scan_id).await?;
        match result {
            Some(r) => state.finding_repository.list_by_scan_result(r.id).await?,
            None => vec![],
        }
    } else {
        vec![]
    };

    let resp = findings
        .into_iter()
        .map(|f| FindingResponse {
            id: f.id.to_string(),
            scan_result_id: f.scan_result_id.to_string(),
            rule_id: f.rule_id,
            severity: f.severity,
            category: f.category,
            file_path: f.file_path,
            line: f.line,
            column: f.column,
            message: f.message,
            recommendation: f.recommendation,
            fix_example: f.fix_example,
            is_suppressed: f.is_suppressed,
            created_at: f.created_at.to_rfc3339(),
        })
        .collect();
    Ok(Json(resp))
}

async fn get_finding(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<FindingResponse>>, ApiError> {
    if !auth.is_authenticated {
        return Err(ApiError::Auth(crate::auth::AuthError::PermissionDenied));
    }

    let finding = state
        .finding_repository
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Finding not found".to_string()))?;

    Ok(ApiResponse::ok(FindingResponse {
        id: finding.id.to_string(),
        scan_result_id: finding.scan_result_id.to_string(),
        rule_id: finding.rule_id,
        severity: finding.severity,
        category: finding.category,
        file_path: finding.file_path,
        line: finding.line,
        column: finding.column,
        message: finding.message,
        recommendation: finding.recommendation,
        fix_example: finding.fix_example,
        is_suppressed: finding.is_suppressed,
        created_at: finding.created_at.to_rfc3339(),
    }))
}

async fn patch_finding(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Path(id): Path<Uuid>,
    Json(req): Json<PatchFindingRequest>,
) -> Result<Json<ApiResponse<FindingResponse>>, ApiError> {
    if !auth.is_authenticated {
        return Err(ApiError::Auth(crate::auth::AuthError::PermissionDenied));
    }

    let mut finding = state
        .finding_repository
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Finding not found".to_string()))?;

    if let Some(suppressed) = req.is_suppressed {
        finding.is_suppressed = suppressed;
    }

    let updated = state.finding_repository.update(&finding).await?;

    Ok(ApiResponse::ok(FindingResponse {
        id: updated.id.to_string(),
        scan_result_id: updated.scan_result_id.to_string(),
        rule_id: updated.rule_id,
        severity: updated.severity,
        category: updated.category,
        file_path: updated.file_path,
        line: updated.line,
        column: updated.column,
        message: updated.message,
        recommendation: updated.recommendation,
        fix_example: updated.fix_example,
        is_suppressed: updated.is_suppressed,
        created_at: updated.created_at.to_rfc3339(),
    }))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/findings", get(list_findings))
        .route("/v1/findings/{id}", get(get_finding).patch(patch_finding))
}
