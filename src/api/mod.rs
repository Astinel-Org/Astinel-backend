pub mod routes;
pub mod middleware;
pub mod errors;
pub mod response;
pub mod extractors;
pub mod metrics;

use axum::Router;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use crate::state::AppState;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .merge(routes::health::routes())
        .merge(routes::version::routes())
        .merge(routes::auth::routes())
        .merge(routes::projects::routes())
        .merge(routes::scans::routes())
        .merge(routes::reports::routes())
        .merge(routes::findings::routes())
        .merge(routes::wallet::routes())
        .merge(routes::webhooks::routes())
        .merge(routes::dashboard::routes())
        .merge(routes::notifications::routes())
        .merge(routes::openapi::routes())
        .merge(metrics::routes())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .layer(axum::middleware::from_fn_with_state(state.clone(), self::middleware::auth_middleware))
        .with_state(state)
}
