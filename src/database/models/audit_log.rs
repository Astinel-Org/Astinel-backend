use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct AuditLog {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub actor_id: Option<Uuid>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub details: serde_json::Value,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl AuditLog {
    pub fn new(
        organization_id: Uuid,
        action: String,
        resource_type: String,
        details: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            organization_id,
            actor_id: None,
            action,
            resource_type,
            resource_id: None,
            details,
            ip_address: None,
            created_at: Utc::now(),
        }
    }
}
