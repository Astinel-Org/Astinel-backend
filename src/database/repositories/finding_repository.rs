use async_trait::async_trait;
use uuid::Uuid;

use crate::database::models::Finding;
use crate::database::pool::DbPool;

#[async_trait]
pub trait FindingRepository: Send + Sync {
    async fn create(&self, finding: &Finding) -> Result<Finding, sqlx::Error>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Finding>, sqlx::Error>;
    async fn list_by_scan_result(
        &self,
        scan_result_id: Uuid,
    ) -> Result<Vec<Finding>, sqlx::Error>;
    async fn list_by_project(&self, project_id: Uuid) -> Result<Vec<Finding>, sqlx::Error>;
    async fn count_by_severity(
        &self,
        scan_result_id: Uuid,
    ) -> Result<Vec<(String, i64)>, sqlx::Error>;
    async fn update(&self, finding: &Finding) -> Result<Finding, sqlx::Error>;
    async fn list_with_filters(
        &self,
        scan_result_id: Uuid,
        severity: Option<&str>,
        category: Option<&str>,
        file_path: Option<&str>,
    ) -> Result<Vec<Finding>, sqlx::Error>;
}

pub struct FindingRepositoryImpl {
    pool: DbPool,
}

impl FindingRepositoryImpl {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FindingRepository for FindingRepositoryImpl {
    async fn create(&self, finding: &Finding) -> Result<Finding, sqlx::Error> {
        sqlx::query_as::<_, Finding>(
            "INSERT INTO findings (id, scan_result_id, rule_id, severity, category, file_path, line, column, message, recommendation, fix_example, is_suppressed, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) RETURNING *",
        )
        .bind(finding.id)
        .bind(finding.scan_result_id)
        .bind(&finding.rule_id)
        .bind(&finding.severity)
        .bind(&finding.category)
        .bind(&finding.file_path)
        .bind(finding.line)
        .bind(finding.column)
        .bind(&finding.message)
        .bind(&finding.recommendation)
        .bind(&finding.fix_example)
        .bind(finding.is_suppressed)
        .bind(finding.created_at)
        .fetch_one(&self.pool)
        .await
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Finding>, sqlx::Error> {
        sqlx::query_as::<_, Finding>("SELECT * FROM findings WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    async fn list_by_scan_result(
        &self,
        scan_result_id: Uuid,
    ) -> Result<Vec<Finding>, sqlx::Error> {
        sqlx::query_as::<_, Finding>(
            "SELECT * FROM findings WHERE scan_result_id = $1 ORDER BY severity DESC, file_path ASC",
        )
        .bind(scan_result_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn list_by_project(&self, project_id: Uuid) -> Result<Vec<Finding>, sqlx::Error> {
        sqlx::query_as::<_, Finding>(
            "SELECT f.* FROM findings f JOIN scan_results sr ON f.scan_result_id = sr.id JOIN scan_jobs sj ON sr.scan_job_id = sj.id WHERE sj.project_id = $1 ORDER BY f.created_at DESC",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn count_by_severity(
        &self,
        scan_result_id: Uuid,
    ) -> Result<Vec<(String, i64)>, sqlx::Error> {
        sqlx::query_as::<_, (String, i64)>(
            "SELECT severity, COUNT(*) as count FROM findings WHERE scan_result_id = $1 GROUP BY severity ORDER BY severity",
        )
        .bind(scan_result_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn update(&self, finding: &Finding) -> Result<Finding, sqlx::Error> {
        sqlx::query_as::<_, Finding>(
            "UPDATE findings SET is_suppressed = $1 WHERE id = $2 RETURNING *",
        )
        .bind(finding.is_suppressed)
        .bind(finding.id)
        .fetch_one(&self.pool)
        .await
    }

    #[allow(unused_assignments)]
    async fn list_with_filters(
        &self,
        scan_result_id: Uuid,
        severity: Option<&str>,
        category: Option<&str>,
        file_path: Option<&str>,
    ) -> Result<Vec<Finding>, sqlx::Error> {
        let mut query = String::from(
            "SELECT * FROM findings WHERE scan_result_id = $1",
        );
        let mut param_index = 2;

        if severity.is_some() {
            query.push_str(&format!(" AND severity = ${}", param_index));
            param_index += 1;
        }
        if category.is_some() {
            query.push_str(&format!(" AND category = ${}", param_index));
            param_index += 1;
        }
        if file_path.is_some() {
            query.push_str(&format!(" AND file_path LIKE ${}", param_index));
            param_index += 1;
        }

        query.push_str(" ORDER BY severity DESC, file_path ASC");

        let mut q = sqlx::query_as::<_, Finding>(&query).bind(scan_result_id);
        if let Some(s) = severity {
            q = q.bind(s);
        }
        if let Some(c) = category {
            q = q.bind(c);
        }
        if let Some(p) = file_path {
            q = q.bind(p);
        }

        q.fetch_all(&self.pool).await
    }
}
