use uuid::Uuid;

use crate::contracts::{ContractDeployer, SUPPORTED_CONTRACTS};
use crate::database::models::ContractDeployment;
use crate::database::pool::DbPool;
use crate::database::repositories::contract_deployment_repository::{
    ContractDeploymentRepository, ContractDeploymentRepositoryImpl,
};

pub struct ContractService {
    pool: DbPool,
    deployer: Option<ContractDeployer>,
}

impl ContractService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            deployer: ContractDeployer::from_env(),
            pool,
        }
    }

    pub fn deployer(&self) -> Option<&ContractDeployer> {
        self.deployer.as_ref()
    }

    pub fn supported_contracts(&self) -> &'static [crate::contracts::types::ContractInfo] {
        SUPPORTED_CONTRACTS
    }

    pub async fn list_contracts(
        &self,
        project_id: Uuid,
        network: &str,
    ) -> Result<Vec<ContractDeployment>, sqlx::Error> {
        let repo = ContractDeploymentRepositoryImpl::new(self.pool.clone());
        repo.find_by_project(project_id, network).await
    }

    pub async fn deploy_contract(
        &self,
        project_id: Uuid,
        contract_name: &str,
        network: &str,
    ) -> Result<ContractDeployment, String> {
        let deployer = self
            .deployer
            .as_ref()
            .ok_or_else(|| "Soroban deployer not configured (set SOROBAN_RPC_URL etc.)".to_string())?;

        let result = deployer.deploy(contract_name).await?;

        let repo = ContractDeploymentRepositoryImpl::new(self.pool.clone());
        let deployment = ContractDeployment::new(
            project_id,
            contract_name.to_string(),
            network.to_string(),
            result.contract_id,
        );

        repo.create(&deployment)
            .await
            .map_err(|e| format!("database error: {e}"))
    }

    pub async fn get_contract(
        &self,
        project_id: Uuid,
        contract_name: &str,
        network: &str,
    ) -> Result<Option<ContractDeployment>, sqlx::Error> {
        let repo = ContractDeploymentRepositoryImpl::new(self.pool.clone());
        repo.find_by_contract_name(project_id, contract_name, network)
            .await
    }

    pub async fn health(&self) -> Result<Option<String>, String> {
        match self.deployer.as_ref() {
            Some(d) => d.health().await.map(Some),
            None => Ok(None),
        }
    }
}
