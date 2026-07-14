use crate::state::AppState;
use axum::{routing::get, Json, Router};
use serde_json::{json, Value};
use std::sync::Arc;

async fn version() -> Json<Value> {
    Json(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "build": option_env!("BUILD_HASH").unwrap_or("dev"),
    }))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/v1/version", get(version))
}
