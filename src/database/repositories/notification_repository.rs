use async_trait::async_trait;
use uuid::Uuid;

use crate::database::models::NotificationEvent;
use crate::database::pool::DbPool;

#[async_trait]
pub trait NotificationRepository: Send + Sync {
    async fn create(&self, event: &NotificationEvent) -> Result<NotificationEvent, sqlx::Error>;
    async fn list_by_organization(&self, org_id: Uuid, limit: i64) -> Result<Vec<NotificationEvent>, sqlx::Error>;
    async fn count_unread(&self, org_id: Uuid) -> Result<i64, sqlx::Error>;
    async fn mark_read(&self, id: Uuid) -> Result<(), sqlx::Error>;
    async fn mark_all_read(&self, org_id: Uuid) -> Result<(), sqlx::Error>;
}

pub struct NotificationRepositoryImpl {
    pool: DbPool,
}

impl NotificationRepositoryImpl {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl NotificationRepository for NotificationRepositoryImpl {
    async fn create(&self, event: &NotificationEvent) -> Result<NotificationEvent, sqlx::Error> {
        sqlx::query_as::<_, NotificationEvent>(
            "INSERT INTO notification_events (id, organization_id, event_type, title, message, severity, resource_type, resource_id, is_read, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING *",
        )
        .bind(event.id)
        .bind(event.organization_id)
        .bind(&event.event_type)
        .bind(&event.title)
        .bind(&event.message)
        .bind(&event.severity)
        .bind(&event.resource_type)
        .bind(event.resource_id)
        .bind(event.is_read)
        .bind(event.created_at)
        .fetch_one(&self.pool)
        .await
    }

    async fn list_by_organization(&self, org_id: Uuid, limit: i64) -> Result<Vec<NotificationEvent>, sqlx::Error> {
        sqlx::query_as::<_, NotificationEvent>(
            "SELECT * FROM notification_events WHERE organization_id = $1 ORDER BY created_at DESC LIMIT $2",
        )
        .bind(org_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    async fn count_unread(&self, org_id: Uuid) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar::<_, Option<i64>>(
            "SELECT COUNT(*) FROM notification_events WHERE organization_id = $1 AND is_read = false",
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map(|c| c.unwrap_or(0))
    }

    async fn mark_read(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE notification_events SET is_read = true WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn mark_all_read(&self, org_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE notification_events SET is_read = true WHERE organization_id = $1 AND is_read = false")
            .bind(org_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
