use axum::{Router, routing::get, Json, extract::{State, Path, Query}};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::AppState;
use crate::auth::AuthContext;
use crate::api::response::ApiResponse;
use crate::api::errors::ApiError;

#[derive(Deserialize)]
pub struct ListContractsParams {
    pub network: Option<String>,
}

#[derive(Deserialize)]
pub struct DeployRequest {
    pub contract_name: String,
    pub network: Option<String>,
}

#[derive(Serialize)]
pub struct ContractInfoResponse {
    pub name: String,
    pub description: String,
    pub wasm_file: String,
}

async fn list_supported(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<ContractInfoResponse>>>, ApiError> {
    let contracts = state.contract_service.supported_contracts();
    let resp: Vec<ContractInfoResponse> = contracts
        .iter()
        .map(|c| ContractInfoResponse {
            name: c.name.to_string(),
            description: c.description.to_string(),
            wasm_file: c.wasm_file.to_string(),
        })
        .collect();
    Ok(ApiResponse::ok(resp))
}

async fn list_deployments(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Path(project_id): Path<Uuid>,
    Query(params): Query<ListContractsParams>,
) -> Result<Json<ApiResponse<Vec<crate::database::models::ContractDeployment>>>, ApiError> {
    if !auth.is_authenticated {
        return Err(ApiError::Auth(crate::auth::AuthError::PermissionDenied));
    }
    let network = params.network.as_deref().unwrap_or("testnet");
    let deployments = state
        .contract_service
        .list_contracts(project_id, network)
        .await?;
    Ok(ApiResponse::ok(deployments))
}

async fn deploy_contract(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Path(project_id): Path<Uuid>,
    Json(req): Json<DeployRequest>,
) -> Result<Json<ApiResponse<crate::database::models::ContractDeployment>>, ApiError> {
    if !auth.is_authenticated {
        return Err(ApiError::Auth(crate::auth::AuthError::PermissionDenied));
    }
    let network = req.network.as_deref().unwrap_or("testnet");
    let deployment = state
        .contract_service
        .deploy_contract(project_id, &req.contract_name, network)
        .await
        .map_err(|e| ApiError::BadRequest(e))?;
    Ok(ApiResponse::ok(deployment))
}

async fn contract_health(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    let health = state
        .contract_service
        .health()
        .await
        .map_err(|e| ApiError::BadRequest(e))?;
    Ok(ApiResponse::ok(serde_json::json!({
        "configured": health.is_some(),
        "network_health": health,
    })))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/contracts", get(list_supported))
        .route("/v1/contracts/health", get(contract_health))
        .route("/v1/projects/{project_id}/contracts", get(list_deployments).post(deploy_contract))
}
