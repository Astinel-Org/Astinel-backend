use async_trait::async_trait;
use uuid::Uuid;

use crate::database::models::ApiKey;
use crate::database::pool::DbPool;

#[async_trait]
pub trait ApiKeyRepository: Send + Sync {
    async fn find_by_hash(&self, hash: &str) -> Result<Option<ApiKey>, sqlx::Error>;
    async fn create(&self, key: &ApiKey) -> Result<ApiKey, sqlx::Error>;
    async fn list_by_organization(&self, org_id: Uuid) -> Result<Vec<ApiKey>, sqlx::Error>;
    async fn delete(&self, id: Uuid) -> Result<(), sqlx::Error>;
    async fn update_last_used(&self, id: Uuid) -> Result<(), sqlx::Error>;
}

pub struct ApiKeyRepositoryImpl {
    pool: DbPool,
}

impl ApiKeyRepositoryImpl {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ApiKeyRepository for ApiKeyRepositoryImpl {
    async fn find_by_hash(&self, hash: &str) -> Result<Option<ApiKey>, sqlx::Error> {
        sqlx::query_as::<_, ApiKey>(
            "SELECT * FROM api_keys WHERE key_hash = $1 AND is_active = true AND (expires_at IS NULL OR expires_at > NOW())",
        )
        .bind(hash)
        .fetch_optional(&self.pool)
        .await
    }

    async fn create(&self, key: &ApiKey) -> Result<ApiKey, sqlx::Error> {
        sqlx::query_as::<_, ApiKey>(
            "INSERT INTO api_keys (id, organization_id, name, key_hash, key_prefix, permissions, expires_at, last_used_at, is_active, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING *",
        )
        .bind(key.id)
        .bind(key.organization_id)
        .bind(&key.name)
        .bind(&key.key_hash)
        .bind(&key.key_prefix)
        .bind(&key.permissions)
        .bind(key.expires_at)
        .bind(key.last_used_at)
        .bind(key.is_active)
        .bind(key.created_at)
        .fetch_one(&self.pool)
        .await
    }

    async fn list_by_organization(&self, org_id: Uuid) -> Result<Vec<ApiKey>, sqlx::Error> {
        sqlx::query_as::<_, ApiKey>(
            "SELECT * FROM api_keys WHERE organization_id = $1 AND is_active = true ORDER BY created_at DESC",
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn delete(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE api_keys SET is_active = false WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn update_last_used(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE api_keys SET last_used_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
