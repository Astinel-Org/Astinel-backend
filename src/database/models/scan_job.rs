use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct ScanJob {
    pub id: Uuid,
    pub project_id: Uuid,
    pub branch: String,
    pub commit_sha: String,
    pub status: String,
    pub trigger: String,
    pub config: serde_json::Value,
    pub priority: i32,
    pub queued_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ScanJob {
    pub fn new(
        project_id: Uuid,
        branch: String,
        commit_sha: String,
        status: String,
        trigger: String,
        config: serde_json::Value,
        priority: i32,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            project_id,
            branch,
            commit_sha,
            status,
            trigger,
            config,
            priority,
            queued_at: None,
            started_at: None,
            completed_at: None,
            error_message: None,
            created_at: now,
            updated_at: now,
        }
    }
}
