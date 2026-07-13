use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Webhook {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub url: String,
    pub secret: String,
    pub events: serde_json::Value,
    pub is_active: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Webhook {
    pub fn new(
        organization_id: Uuid,
        name: String,
        url: String,
        secret: String,
        events: serde_json::Value,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            organization_id,
            name,
            url,
            secret,
            events,
            is_active: true,
            last_triggered_at: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }
}
