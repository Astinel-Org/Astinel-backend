use crate::state::AppState;
use axum::{extract::State, response::IntoResponse, routing::get, Router};
use std::sync::Arc;

async fn get_metrics(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    state.metrics_handle.render()
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/metrics", get(get_metrics))
}
