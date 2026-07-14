use async_trait::async_trait;
use uuid::Uuid;

use crate::database::models::ContractDeployment;
use crate::database::pool::DbPool;

#[async_trait]
pub trait ContractDeploymentRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<ContractDeployment>, sqlx::Error>;
    async fn find_by_project(
        &self,
        project_id: Uuid,
        network: &str,
    ) -> Result<Vec<ContractDeployment>, sqlx::Error>;
    async fn find_by_contract_name(
        &self,
        project_id: Uuid,
        contract_name: &str,
        network: &str,
    ) -> Result<Option<ContractDeployment>, sqlx::Error>;
    async fn create(
        &self,
        deployment: &ContractDeployment,
    ) -> Result<ContractDeployment, sqlx::Error>;
    async fn update_version(
        &self,
        id: Uuid,
        new_version: i32,
        new_wasm_hash: &str,
    ) -> Result<ContractDeployment, sqlx::Error>;
    async fn delete(&self, id: Uuid) -> Result<(), sqlx::Error>;
}

pub struct ContractDeploymentRepositoryImpl {
    pool: DbPool,
}

impl ContractDeploymentRepositoryImpl {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ContractDeploymentRepository for ContractDeploymentRepositoryImpl {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<ContractDeployment>, sqlx::Error> {
        sqlx::query_as::<_, ContractDeployment>(
            "SELECT * FROM contract_deployments WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_by_project(
        &self,
        project_id: Uuid,
        network: &str,
    ) -> Result<Vec<ContractDeployment>, sqlx::Error> {
        sqlx::query_as::<_, ContractDeployment>(
            "SELECT * FROM contract_deployments WHERE project_id = $1 AND network = $2 AND deleted_at IS NULL ORDER BY created_at DESC",
        )
        .bind(project_id)
        .bind(network)
        .fetch_all(&self.pool)
        .await
    }

    async fn find_by_contract_name(
        &self,
        project_id: Uuid,
        contract_name: &str,
        network: &str,
    ) -> Result<Option<ContractDeployment>, sqlx::Error> {
        sqlx::query_as::<_, ContractDeployment>(
            "SELECT * FROM contract_deployments WHERE project_id = $1 AND contract_name = $2 AND network = $3 AND deleted_at IS NULL ORDER BY version DESC LIMIT 1",
        )
        .bind(project_id)
        .bind(contract_name)
        .bind(network)
        .fetch_optional(&self.pool)
        .await
    }

    async fn create(
        &self,
        deployment: &ContractDeployment,
    ) -> Result<ContractDeployment, sqlx::Error> {
        sqlx::query_as::<_, ContractDeployment>(
            "INSERT INTO contract_deployments (id, project_id, contract_name, network, contract_id, wasm_hash, deploy_tx_hash, version, metadata, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) RETURNING *",
        )
        .bind(deployment.id)
        .bind(deployment.project_id)
        .bind(&deployment.contract_name)
        .bind(&deployment.network)
        .bind(&deployment.contract_id)
        .bind(&deployment.wasm_hash)
        .bind(&deployment.deploy_tx_hash)
        .bind(deployment.version)
        .bind(&deployment.metadata)
        .bind(&deployment.status)
        .bind(deployment.created_at)
        .bind(deployment.updated_at)
        .fetch_one(&self.pool)
        .await
    }

    async fn update_version(
        &self,
        id: Uuid,
        new_version: i32,
        new_wasm_hash: &str,
    ) -> Result<ContractDeployment, sqlx::Error> {
        sqlx::query_as::<_, ContractDeployment>(
            "UPDATE contract_deployments SET version = $1, wasm_hash = $2, updated_at = NOW() WHERE id = $3 AND deleted_at IS NULL RETURNING *",
        )
        .bind(new_version)
        .bind(new_wasm_hash)
        .bind(id)
        .fetch_one(&self.pool)
        .await
    }

    async fn delete(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE contract_deployments SET deleted_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
