use std::path::PathBuf;
use tracing::{instrument, info};
use crate::jobs::queue::QueuedJob;
use crate::scanner::{DefaultScanner, Scanner, ScanRequest};
use crate::database::models::ScanResult;
use crate::database::repositories::{
    ProjectRepository, ProjectRepositoryImpl,
    ScanRepository, ScanRepositoryImpl,
    FindingRepository, FindingRepositoryImpl,
};
use crate::database::pool::DbPool;

pub struct ScanJobExecutor {
    pool: DbPool,
    project_repo: ProjectRepositoryImpl,
    scanner: DefaultScanner,
}

impl ScanJobExecutor {
    pub fn new(pool: DbPool, project_repo: ProjectRepositoryImpl) -> Self {
        Self { pool, project_repo, scanner: DefaultScanner }
    }

    #[instrument(skip(self, job), fields(job_id = %job.id, project_id = %job.project_id))]
    pub async fn execute(&self, job: QueuedJob) -> Result<(), String> {
        info!("Executing scan job {}", job.id);

        let scan_repo = ScanRepositoryImpl::new(self.pool.clone());
        let finding_repo = FindingRepositoryImpl::new(self.pool.clone());

        let mut db_job = scan_repo.find_job_by_id(job.id).await
            .map_err(|e| format!("Failed to find job: {}", e))?
            .ok_or_else(|| "Job not found".to_string())?;
        db_job.status = "running".to_string();
        scan_repo.update_job(&db_job).await.map_err(|e| format!("Update failed: {}", e))?;

        // Resolve scan target from the project's local_path
        let project = self.project_repo.find_by_id(job.project_id).await
            .map_err(|e| format!("Failed to find project: {}", e))?
            .ok_or_else(|| format!("Project {} not found", job.project_id))?;

        let scan_target = project.local_path
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(format!("/tmp/astinel-workspace/{}", project.slug))
            });

        let request = ScanRequest::builder()
            .target(&scan_target)
            .build();

        let result = match self.scanner.scan(request) {
            Ok(r) => r,
            Err(e) => {
                db_job.status = "failed".to_string();
                db_job.error_message = Some(e.to_string());
                scan_repo.update_job(&db_job).await.map_err(|e| format!("Update failed: {}", e))?;
                return Err(format!("Scan failed: {}", e));
            }
        };

        let scan_result = ScanResult::new(
            job.id,
            "completed".to_string(),
        );
        let saved_result = scan_repo.create_result(&scan_result).await
            .map_err(|e| format!("Failed to save result: {}", e))?;

        for finding in &result.core.findings {
            let db_finding = crate::database::models::Finding::new(
                saved_result.id,
                finding.rule_id.as_str().to_string(),
                format!("{:?}", finding.severity),
                format!("{:?}", finding.category),
                finding.span.file.to_string_lossy().to_string(),
                finding.span.line as i32,
                finding.span.column as i32,
                finding.message.clone(),
                finding.recommendation.clone(),
            );
            let _ = finding_repo.create(&db_finding).await;
        }

        db_job.status = "completed".to_string();
        scan_repo.update_job(&db_job).await.map_err(|e| format!("Update failed: {}", e))?;

        info!("Scan job {} completed with {} findings", job.id, result.core.findings.len());
        Ok(())
    }
}
