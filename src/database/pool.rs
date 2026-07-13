use std::time::Duration;

use sqlx::PgPool;

use crate::database::connection::DbConfig;

pub type DbPool = PgPool;

pub async fn create_pool(config: &DbConfig) -> Result<DbPool, sqlx::Error> {
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .connect_timeout(Duration::from_secs(config.connect_timeout_seconds))
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(config.idle_timeout_seconds))
        .connect(&config.database_url())
        .await
}
