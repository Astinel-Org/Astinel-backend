use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct GithubInstallation {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub installation_id: i64,
    pub account_login: String,
    pub account_type: String,
    pub avatar_url: Option<String>,
    pub permissions: serde_json::Value,
    pub repository_selection: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
