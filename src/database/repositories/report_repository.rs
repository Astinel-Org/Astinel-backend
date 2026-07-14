use async_trait::async_trait;
use uuid::Uuid;

use crate::database::models::Report;
use crate::database::pool::DbPool;

#[async_trait]
pub trait ReportRepository: Send + Sync {
    async fn create(&self, report: &Report) -> Result<Report, sqlx::Error>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Report>, sqlx::Error>;
    async fn find_by_scan_result(&self, scan_result_id: Uuid) -> Result<Vec<Report>, sqlx::Error>;
    async fn find_by_project(&self, project_id: Uuid) -> Result<Vec<Report>, sqlx::Error>;
}

pub struct ReportRepositoryImpl {
    pool: DbPool,
}

impl ReportRepositoryImpl {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ReportRepository for ReportRepositoryImpl {
    async fn create(&self, report: &Report) -> Result<Report, sqlx::Error> {
        sqlx::query_as::<_, Report>(
            "INSERT INTO reports (id, scan_result_id, format, content, file_path, file_size, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *",
        )
        .bind(report.id)
        .bind(report.scan_result_id)
        .bind(&report.format)
        .bind(&report.content)
        .bind(&report.file_path)
        .bind(report.file_size)
        .bind(report.created_at)
        .fetch_one(&self.pool)
        .await
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Report>, sqlx::Error> {
        sqlx::query_as::<_, Report>("SELECT * FROM reports WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    async fn find_by_scan_result(&self, scan_result_id: Uuid) -> Result<Vec<Report>, sqlx::Error> {
        sqlx::query_as::<_, Report>(
            "SELECT * FROM reports WHERE scan_result_id = $1 ORDER BY created_at DESC",
        )
        .bind(scan_result_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn find_by_project(&self, project_id: Uuid) -> Result<Vec<Report>, sqlx::Error> {
        sqlx::query_as::<_, Report>(
            "SELECT r.* FROM reports r JOIN scan_results sr ON r.scan_result_id = sr.id JOIN scan_jobs sj ON sr.scan_job_id = sj.id WHERE sj.project_id = $1 ORDER BY r.created_at DESC",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await
    }
}
