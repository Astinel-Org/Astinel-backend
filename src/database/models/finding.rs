use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Finding {
    pub id: Uuid,
    pub scan_result_id: Uuid,
    pub rule_id: String,
    pub severity: String,
    pub category: String,
    pub file_path: String,
    pub line: i32,
    pub column: i32,
    pub message: String,
    pub recommendation: String,
    pub fix_example: Option<String>,
    pub is_suppressed: bool,
    pub created_at: DateTime<Utc>,
}

impl Finding {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        scan_result_id: Uuid,
        rule_id: String,
        severity: String,
        category: String,
        file_path: String,
        line: i32,
        column: i32,
        message: String,
        recommendation: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            scan_result_id,
            rule_id,
            severity,
            category,
            file_path,
            line,
            column,
            message,
            recommendation,
            fix_example: None,
            is_suppressed: false,
            created_at: Utc::now(),
        }
    }
}
