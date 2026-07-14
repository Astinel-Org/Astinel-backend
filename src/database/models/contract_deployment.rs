use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct ContractDeployment {
    pub id: Uuid,
    pub project_id: Uuid,
    pub contract_name: String,
    pub network: String,
    pub contract_id: String,
    pub wasm_hash: Option<String>,
    pub deploy_tx_hash: Option<String>,
    pub version: i32,
    pub metadata: serde_json::Value,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl ContractDeployment {
    pub fn new(
        project_id: Uuid,
        contract_name: String,
        network: String,
        contract_id: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            project_id,
            contract_name,
            network,
            contract_id,
            wasm_hash: None,
            deploy_tx_hash: None,
            version: 1,
            metadata: serde_json::json!({}),
            status: "deployed".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        }
    }
}
