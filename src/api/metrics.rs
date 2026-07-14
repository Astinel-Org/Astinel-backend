use axum::{Router, routing::get, extract::State, response::IntoResponse};
use std::sync::Arc;
use crate::state::AppState;

async fn get_metrics(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    state.metrics_handle.render()
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/metrics", get(get_metrics))
}
