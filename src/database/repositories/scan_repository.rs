use async_trait::async_trait;
use uuid::Uuid;

use crate::database::models::{ScanJob, ScanResult};
use crate::database::pool::DbPool;

#[async_trait]
pub trait ScanRepository: Send + Sync {
    // Scan jobs
    async fn create_job(&self, job: &ScanJob) -> Result<ScanJob, sqlx::Error>;
    async fn find_job_by_id(&self, id: Uuid) -> Result<Option<ScanJob>, sqlx::Error>;
    async fn update_job(&self, job: &ScanJob) -> Result<ScanJob, sqlx::Error>;
    async fn list_jobs_for_project(
        &self,
        project_id: Uuid,
    ) -> Result<Vec<ScanJob>, sqlx::Error>;

    // Scan results
    async fn create_result(&self, result: &ScanResult) -> Result<ScanResult, sqlx::Error>;
    async fn find_result_by_id(&self, id: Uuid) -> Result<Option<ScanResult>, sqlx::Error>;
    async fn find_result_by_job(
        &self,
        scan_job_id: Uuid,
    ) -> Result<Option<ScanResult>, sqlx::Error>;
}

pub struct ScanRepositoryImpl {
    pool: DbPool,
}

impl ScanRepositoryImpl {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ScanRepository for ScanRepositoryImpl {
    async fn create_job(&self, job: &ScanJob) -> Result<ScanJob, sqlx::Error> {
        sqlx::query_as::<_, ScanJob>(
            "INSERT INTO scan_jobs (id, project_id, branch, commit_sha, status, trigger, config, priority, queued_at, started_at, completed_at, error_message, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14) RETURNING *",
        )
        .bind(job.id)
        .bind(job.project_id)
        .bind(&job.branch)
        .bind(&job.commit_sha)
        .bind(&job.status)
        .bind(&job.trigger)
        .bind(&job.config)
        .bind(job.priority)
        .bind(job.queued_at)
        .bind(job.started_at)
        .bind(job.completed_at)
        .bind(&job.error_message)
        .bind(job.created_at)
        .bind(job.updated_at)
        .fetch_one(&self.pool)
        .await
    }

    async fn find_job_by_id(&self, id: Uuid) -> Result<Option<ScanJob>, sqlx::Error> {
        sqlx::query_as::<_, ScanJob>("SELECT * FROM scan_jobs WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    async fn update_job(&self, job: &ScanJob) -> Result<ScanJob, sqlx::Error> {
        sqlx::query_as::<_, ScanJob>(
            "UPDATE scan_jobs SET status = $1, trigger = $2, config = $3, priority = $4, queued_at = $5, started_at = $6, completed_at = $7, error_message = $8, updated_at = NOW() WHERE id = $9 RETURNING *",
        )
        .bind(&job.status)
        .bind(&job.trigger)
        .bind(&job.config)
        .bind(job.priority)
        .bind(job.queued_at)
        .bind(job.started_at)
        .bind(job.completed_at)
        .bind(&job.error_message)
        .bind(job.id)
        .fetch_one(&self.pool)
        .await
    }

    async fn list_jobs_for_project(
        &self,
        project_id: Uuid,
    ) -> Result<Vec<ScanJob>, sqlx::Error> {
        sqlx::query_as::<_, ScanJob>(
            "SELECT * FROM scan_jobs WHERE project_id = $1 ORDER BY created_at DESC",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn create_result(&self, result: &ScanResult) -> Result<ScanResult, sqlx::Error> {
        sqlx::query_as::<_, ScanResult>(
            "INSERT INTO scan_results (id, scan_job_id, status, total_files, total_rules, total_findings, suppressed_findings, critical, high, medium, low, info, score, duration_ms, raw_output, report_hash, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17) RETURNING *",
        )
        .bind(result.id)
        .bind(result.scan_job_id)
        .bind(&result.status)
        .bind(result.total_files)
        .bind(result.total_rules)
        .bind(result.total_findings)
        .bind(result.suppressed_findings)
        .bind(result.critical)
        .bind(result.high)
        .bind(result.medium)
        .bind(result.low)
        .bind(result.info)
        .bind(result.score)
        .bind(result.duration_ms)
        .bind(&result.raw_output)
        .bind(&result.report_hash)
        .bind(result.created_at)
        .fetch_one(&self.pool)
        .await
    }

    async fn find_result_by_id(&self, id: Uuid) -> Result<Option<ScanResult>, sqlx::Error> {
        sqlx::query_as::<_, ScanResult>("SELECT * FROM scan_results WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    async fn find_result_by_job(
        &self,
        scan_job_id: Uuid,
    ) -> Result<Option<ScanResult>, sqlx::Error> {
        sqlx::query_as::<_, ScanResult>(
            "SELECT * FROM scan_results WHERE scan_job_id = $1 ORDER BY created_at DESC LIMIT 1",
        )
        .bind(scan_job_id)
        .fetch_optional(&self.pool)
        .await
    }
}
