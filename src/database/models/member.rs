use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct OrganizationMember {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub invited_by: Option<Uuid>,
    pub joined_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl OrganizationMember {
    pub fn new(organization_id: Uuid, user_id: Uuid, role: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            organization_id,
            user_id,
            role,
            invited_by: None,
            joined_at: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }
}
