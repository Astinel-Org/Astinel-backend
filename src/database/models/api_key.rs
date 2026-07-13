use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct ApiKey {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub key_hash: String,
    pub key_prefix: String,
    pub permissions: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

impl ApiKey {
    pub fn new(
        organization_id: Uuid,
        name: String,
        key_hash: String,
        key_prefix: String,
        permissions: serde_json::Value,
        expires_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            organization_id,
            name,
            key_hash,
            key_prefix,
            permissions,
            created_at: Utc::now(),
            expires_at,
            last_used_at: None,
            is_active: true,
        }
    }
}
