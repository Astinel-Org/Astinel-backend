use axum::{Router, routing::get, Json, extract::{State, Query}};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::state::AppState;
use crate::auth::AuthContext;
use crate::api::errors::ApiError;
use crate::database::repositories::finding_repository::FindingRepository;

#[derive(Deserialize)]
pub struct FindingsQuery {
    pub scan_id: Option<String>,
    pub severity: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}

#[derive(Serialize)]
pub struct FindingResponse {
    pub id: String,
    pub rule_id: String,
    pub severity: String,
    pub category: String,
    pub file_path: String,
    pub line: i32,
    pub message: String,
}

async fn list_findings(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Query(_query): Query<FindingsQuery>,
) -> Result<Json<Vec<FindingResponse>>, ApiError> {
    if !auth.is_authenticated {
        return Err(ApiError::Auth(crate::auth::AuthError::PermissionDenied));
    }
    let org_id = auth.org_id.unwrap_or_default();
    let findings = state.finding_repository
        .list_by_scan_result(org_id)
        .await?;
    let resp = findings.into_iter().map(|f| FindingResponse {
        id: f.id.to_string(),
        rule_id: f.rule_id,
        severity: f.severity,
        category: f.category,
        file_path: f.file_path,
        line: f.line,
        message: f.message,
    }).collect();
    Ok(Json(resp))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/v1/findings", get(list_findings))
}
