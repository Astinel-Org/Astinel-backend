use crate::state::AppState;
use axum::{extract::State, routing::get, Json, Router};
use serde_json::{json, Value};
use std::sync::Arc;

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
