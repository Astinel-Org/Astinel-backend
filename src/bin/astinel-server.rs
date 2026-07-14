use std::sync::Arc;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let redis = astinel_backend::cache::redis::RedisPool::new(&redis_url).await?;

    let db_config = astinel_backend::database::DbConfig::from_env();
    let pool = astinel_backend::database::create_pool(&db_config).await?;

    astinel_backend::database::run_migrations(&pool).await?;

    let state = Arc::new(astinel_backend::state::AppState::new(pool, redis).await);

    let app = astinel_backend::api::create_router(state.clone());

    let addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    info!("Starting Astinel server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
