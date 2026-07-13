use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Project {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub repository_url: Option<String>,
    pub local_path: Option<String>,
    pub default_branch: String,
    pub language: String,
    pub settings: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Project {
    pub fn new(organization_id: Uuid, name: String, slug: String, language: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            organization_id,
            name,
            slug,
            description: None,
            repository_url: None,
            local_path: None,
            default_branch: "main".to_string(),
            language,
            settings: serde_json::Value::Object(Default::default()),
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }
}
