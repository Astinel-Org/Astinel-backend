use crate::api::errors::ApiError;
use crate::api::response::ApiResponse;
use crate::state::AppState;
use axum::{extract::State, routing::post, Json, Router};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct InstallRequest {
    pub installation_id: i64,
    pub setup_action: Option<String>,
}

async fn github_install(
    State(state): State<Arc<AppState>>,
    Json(req): Json<InstallRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    let org_id = uuid::Uuid::new_v4();

    let existing = sqlx::query_scalar::<_, i64>(
        "SELECT installation_id FROM github_installations WHERE installation_id = $1",
    )
    .bind(req.installation_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    if existing.is_some() {
        return Err(ApiError::Conflict(
            "Installation already registered".to_string(),
        ));
    }

    sqlx::query(
        "INSERT INTO github_installations (id, organization_id, installation_id, account_login, account_type, repository_selection) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(org_id)
    .bind(org_id)
    .bind(req.installation_id)
    .bind("pending")
    .bind("Organization")
    .bind("selected")
    .execute(&state.pool)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(ApiResponse::ok(serde_json::json!({
        "status": "registered",
        "installation_id": req.installation_id,
    })))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/v1/webhooks/github", post(github_install))
}
