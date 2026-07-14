use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct NotificationEvent {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub event_type: String,
    pub title: String,
    pub message: String,
    pub severity: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
}

impl NotificationEvent {
    pub fn new(
        organization_id: Uuid,
        event_type: String,
        title: String,
        message: String,
        severity: String,
        resource_type: Option<String>,
        resource_id: Option<Uuid>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            organization_id,
            event_type,
            title,
            message,
            severity,
            resource_type,
            resource_id,
            is_read: false,
            created_at: Utc::now(),
        }
    }
}
