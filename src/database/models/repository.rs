use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Repository {
    pub id: Uuid,
    pub project_id: Uuid,
    pub provider: String,
    pub external_id: String,
    pub owner: String,
    pub name: String,
    pub full_name: String,
    pub clone_url: String,
    pub default_branch: String,
    pub is_private: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Repository {
    pub fn new(
        project_id: Uuid,
        provider: String,
        external_id: String,
        owner: String,
        name: String,
        full_name: String,
        clone_url: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            project_id,
            provider,
            external_id,
            owner,
            name,
            full_name,
            clone_url,
            default_branch: "main".to_string(),
            is_private: false,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }
}
