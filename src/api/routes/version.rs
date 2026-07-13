use axum::{Router, routing::get, Json};
use std::sync::Arc;
use serde_json::{json, Value};
use crate::state::AppState;

async fn version() -> Json<Value> {
    Json(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "build": option_env!("BUILD_HASH").unwrap_or("dev"),
    }))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/v1/version", get(version))
}
