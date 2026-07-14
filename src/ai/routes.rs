use axum::{Router, routing::{get, post}, Json, extract::State};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::state::AppState;
use crate::auth::AuthContext;
use crate::api::response::ApiResponse;
use crate::api::errors::ApiError;

#[derive(Deserialize)]
pub struct FixSuggestionRequest {
    pub finding_id: String,
    pub rule_id: String,
    pub message: String,
    pub file_path: String,
    pub code_snippet: String,
}

#[derive(Serialize)]
pub struct FixSuggestionResponse {
    pub suggestion: String,
}

#[derive(Deserialize)]
pub struct AnalyzeRequest {
    pub query: String,
    pub scan_summary: Option<String>,
}

#[derive(Serialize)]
pub struct AnalyzeResponse {
    pub analysis: String,
}

#[derive(Serialize)]
pub struct AiHealthResponse {
    pub available: bool,
    pub provider: String,
}

async fn fix_suggestion(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Json(req): Json<FixSuggestionRequest>,
) -> Result<Json<ApiResponse<FixSuggestionResponse>>, ApiError> {
    if !auth.is_authenticated {
        return Err(ApiError::Auth(crate::auth::AuthError::PermissionDenied));
    }

    let context = format!("[{}] {} at {}", req.rule_id, req.message, req.file_path);
    let suggestion = state.ai_provider
        .generate_fix_suggestion(&context, &req.code_snippet)
        .await
        .map_err(|e| ApiError::Internal(format!("AI service error: {}", e)))?;

    Ok(ApiResponse::ok(FixSuggestionResponse { suggestion }))
}

async fn analyze(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Json(req): Json<AnalyzeRequest>,
) -> Result<Json<ApiResponse<AnalyzeResponse>>, ApiError> {
    if !auth.is_authenticated {
        return Err(ApiError::Auth(crate::auth::AuthError::PermissionDenied));
    }

    let summary = req.scan_summary.unwrap_or_else(|| "No scan summary provided".to_string());
    let analysis = state.ai_provider
        .analyze_security(&req.query, &summary)
        .await
        .map_err(|e| ApiError::Internal(format!("AI service error: {}", e)))?;

    Ok(ApiResponse::ok(AnalyzeResponse { analysis }))
}

async fn health(
    State(state): State<Arc<AppState>>,
) -> Json<AiHealthResponse> {
    let available = state.ai_provider.health().await.unwrap_or(false);
    let provider = if std::env::var("OLLAMA_URL").is_ok() {
        "ollama"
    } else {
        "disabled"
    }.to_string();
    Json(AiHealthResponse { available, provider })
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/ai/fix-suggestion", post(fix_suggestion))
        .route("/v1/ai/analyze", post(analyze))
        .route("/v1/ai/health", get(health))
}
