use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone)]
pub struct SorobanRpcClient {
    http_client: reqwest::Client,
    rpc_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcResponse {
    pub jsonrpc: String,
    pub id: u64,
    pub result: Option<Value>,
    pub error: Option<RpcError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetContractDataResult {
    pub xdr: String,
}

impl SorobanRpcClient {
    pub fn new(rpc_url: String) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            rpc_url,
        }
    }

    pub fn from_env() -> Self {
        let rpc_url = std::env::var("SOROBAN_RPC_URL")
            .unwrap_or_else(|_| "https://soroban-testnet.stellar.org".to_string());
        Self::new(rpc_url)
    }

    async fn call(&self, method: &str, params: Value) -> Result<Value, String> {
        let req = RpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: method.to_string(),
            params,
        };

        let resp = self
            .http_client
            .post(&self.rpc_url)
            .json(&req)
            .send()
            .await
            .map_err(|e| format!("RPC request failed: {e}"))?;

        let rpc_resp: RpcResponse = resp
            .json()
            .await
            .map_err(|e| format!("RPC response parse failed: {e}"))?;

        if let Some(err) = rpc_resp.error {
            return Err(format!("RPC error {}: {}", err.code, err.message));
        }

        rpc_resp.result.ok_or_else(|| "RPC returned empty result".to_string())
    }

    pub async fn get_health(&self) -> Result<String, String> {
        let result = self.call("getHealth", json!({})).await?;
        Ok(result.to_string())
    }

    pub async fn get_ledger_sequence(&self) -> Result<u32, String> {
        let result = self.call("getLatestLedger", json!({})).await?;
        result
            .get("sequence")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .ok_or_else(|| "missing sequence".to_string())
    }

    pub async fn get_contract_data(
        &self,
        contract_id: &str,
        key_xdr: &str,
    ) -> Result<GetContractDataResult, String> {
        let result = self
            .call(
                "getContractData",
                json!({
                    "contractId": contract_id,
                    "key": key_xdr,
                }),
            )
            .await?;
        serde_json::from_value(result).map_err(|e| format!("parse failed: {e}"))
    }

    pub async fn simulate_transaction(&self, tx_xdr: &str) -> Result<Value, String> {
        self.call(
            "simulateTransaction",
            json!({
                "transaction": tx_xdr,
            }),
        )
        .await
    }

    pub async fn send_transaction(&self, tx_xdr: &str) -> Result<String, String> {
        let result = self
            .call(
                "sendTransaction",
                json!({
                    "transaction": tx_xdr,
                }),
            )
            .await?;
        result
            .get("hash")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| "missing tx hash".to_string())
    }
}
