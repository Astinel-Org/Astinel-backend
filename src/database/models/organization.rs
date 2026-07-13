use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Organization {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub owner_user_id: Uuid,
    pub billing_plan: String,
    pub is_active: bool,
    pub settings: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Organization {
    pub fn new(
        name: String,
        slug: String,
        owner_user_id: Uuid,
        billing_plan: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            slug,
            description: None,
            owner_user_id,
            billing_plan,
            is_active: true,
            settings: serde_json::Value::Object(Default::default()),
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }
}
