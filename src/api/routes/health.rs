use axum::{Router, routing::get, Json, extract::State};
use std::sync::Arc;
use serde_json::{json, Value};
use crate::state::AppState;

async fn health_check(State(_state): State<Arc<AppState>>) -> Json<Value> {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "service": "astinel-backend",
    }))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/v1/health", get(health_check))
}
