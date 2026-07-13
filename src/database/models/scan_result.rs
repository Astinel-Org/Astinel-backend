use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct ScanResult {
    pub id: Uuid,
    pub scan_job_id: Uuid,
    pub status: String,
    pub total_files: i32,
    pub total_rules: i32,
    pub total_findings: i32,
    pub suppressed_findings: i32,
    pub critical: i32,
    pub high: i32,
    pub medium: i32,
    pub low: i32,
    pub info: i32,
    pub score: i32,
    pub duration_ms: i64,
    pub raw_output: Option<String>,
    pub report_hash: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl ScanResult {
    pub fn new(scan_job_id: Uuid, status: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            scan_job_id,
            status,
            total_files: 0,
            total_rules: 0,
            total_findings: 0,
            suppressed_findings: 0,
            critical: 0,
            high: 0,
            medium: 0,
            low: 0,
            info: 0,
            score: 0,
            duration_ms: 0,
            raw_output: None,
            report_hash: None,
            created_at: Utc::now(),
        }
    }
}
