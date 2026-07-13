use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Report {
    pub id: Uuid,
    pub scan_result_id: Uuid,
    pub format: String,
    pub content: String,
    pub file_path: Option<String>,
    pub file_size: i64,
    pub created_at: DateTime<Utc>,
}

impl Report {
    pub fn new(scan_result_id: Uuid, format: String, content: String, file_size: i64) -> Self {
        Self {
            id: Uuid::new_v4(),
            scan_result_id,
            format,
            content,
            file_path: None,
            file_size,
            created_at: Utc::now(),
        }
    }
}
