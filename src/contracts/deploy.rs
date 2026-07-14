use crate::contracts::rpc::SorobanRpcClient;
use crate::contracts::types::SUPPORTED_CONTRACTS;

pub struct DeployConfig {
    pub rpc_url: String,
    pub network_passphrase: String,
    pub source_account: String,
    pub secret_key: String,
}

pub struct ContractDeployer {
    rpc: SorobanRpcClient,
    config: DeployConfig,
}

#[derive(Debug, serde::Serialize)]
pub struct DeployResult {
    pub contract_name: String,
    pub contract_id: String,
    pub wasm_hash: String,
    pub tx_hash: String,
}

impl ContractDeployer {
    pub fn new(config: DeployConfig) -> Self {
        Self {
            rpc: SorobanRpcClient::new(config.rpc_url.clone()),
            config,
        }
    }

    pub fn from_env() -> Option<Self> {
        let rpc_url = std::env::var("SOROBAN_RPC_URL").ok()?;
        let network_passphrase = std::env::var("SOROBAN_NETWORK_PASSPHRASE")
            .unwrap_or_else(|_| "Test SDF Network ; September 2015".to_string());
        let source_account = std::env::var("SOROBAN_SOURCE_ACCOUNT").ok()?;
        let secret_key = std::env::var("SOROBAN_SECRET_KEY").ok()?;

        Some(Self::new(DeployConfig {
            rpc_url,
            network_passphrase,
            source_account,
            secret_key,
        }))
    }

    pub fn supported_contracts(&self) -> &'static [crate::contracts::types::ContractInfo] {
        SUPPORTED_CONTRACTS
    }

    pub fn resolve_wasm_path(contract_name: &str) -> Option<String> {
        let contracts_dir = std::env::var("ASTINEL_CONTRACTS_DIR")
            .unwrap_or_else(|_| "../Astinel-contracts".to_string());

        for info in SUPPORTED_CONTRACTS {
            if info.name == contract_name {
                return Some(format!(
                    "{}/target/wasm32-unknown-unknown/debug/{}",
                    contracts_dir, info.wasm_file
                ));
            }
        }
        None
    }

    pub async fn deploy(
        &self,
        contract_name: &str,
    ) -> Result<DeployResult, String> {
        let wasm_path = Self::resolve_wasm_path(contract_name)
            .ok_or_else(|| format!("unknown contract: {contract_name}"))?;

        let wasm_bytes = tokio::fs::read(&wasm_path)
            .await
            .map_err(|e| format!("failed to read wasm {wasm_path}: {e}"))?;

        let hash = blake2_hash(&wasm_bytes);

        let _rpc_health = self.rpc.get_health().await?;

        let _simulated = self
            .rpc
            .simulate_transaction(&format!("upload_wasm_{}", contract_name))
            .await?;

        let tx_hash = self
            .rpc
            .send_transaction(&format!("upload_wasm_tx_{}", contract_name))
            .await?;

        let contract_id = generate_contract_id(&self.config.source_account, &hash);

        Ok(DeployResult {
            contract_name: contract_name.to_string(),
            contract_id,
            wasm_hash: hash,
            tx_hash,
        })
    }

    pub async fn health(&self) -> Result<String, String> {
        self.rpc.get_health().await
    }
}

fn blake2_hash(bytes: &[u8]) -> String {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn generate_contract_id(source: &str, wasm_hash: &str) -> String {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(source.as_bytes());
    hasher.update(wasm_hash.as_bytes());
    let hash = hasher.finalize();
    hex::encode(hash)
}
