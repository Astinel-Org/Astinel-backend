use crate::auth::{JwtService, NonceStore};
use crate::database::pool::DbPool;
use crate::database::repositories::*;
use crate::jobs::queue::JobQueue;
use crate::services::ScanService;

pub struct AppState {
    pub pool: DbPool,
    pub jwt_service: JwtService,
    pub nonce_store: NonceStore,
    pub queue: JobQueue,
    pub scan_service: ScanService,
    pub user_repository: UserRepositoryImpl,
    pub organization_repository: OrganizationRepositoryImpl,
    pub project_repository: ProjectRepositoryImpl,
    pub scan_repository: ScanRepositoryImpl,
    pub finding_repository: FindingRepositoryImpl,
    pub report_repository: ReportRepositoryImpl,
    pub api_key_repository: ApiKeyRepositoryImpl,
}

impl AppState {
    pub async fn new(pool: DbPool) -> Self {
        let (queue, _rx) = JobQueue::new();
        let jwt_service = JwtService::from_env();

        Self {
            scan_service: ScanService::new(pool.clone(), queue.clone(), ProjectRepositoryImpl::new(pool.clone())),
            queue,
            nonce_store: NonceStore::new(),
            jwt_service,
            user_repository: UserRepositoryImpl::new(pool.clone()),
            organization_repository: OrganizationRepositoryImpl::new(pool.clone()),
            project_repository: ProjectRepositoryImpl::new(pool.clone()),
            scan_repository: ScanRepositoryImpl::new(pool.clone()),
            finding_repository: FindingRepositoryImpl::new(pool.clone()),
            report_repository: ReportRepositoryImpl::new(pool.clone()),
            api_key_repository: ApiKeyRepositoryImpl::new(pool.clone()),
            pool,
        }
    }
}
