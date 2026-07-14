use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub wasm_file: &'static str,
}

pub static SUPPORTED_CONTRACTS: &[ContractInfo] = &[
    ContractInfo {
        name: "astinel-access-control",
        description: "Access control with admin, operators, and pausing",
        wasm_file: "astinel_access_control.wasm",
    },
    ContractInfo {
        name: "astinel-payable",
        description: "ERC-20-like token with allowance and transfer_from",
        wasm_file: "astinel_payable.wasm",
    },
    ContractInfo {
        name: "astinel-ttl-management",
        description: "Persistent storage with configurable time-to-live",
        wasm_file: "astinel_ttl_management.wasm",
    },
    ContractInfo {
        name: "astinel-upgradeable",
        description: "Contract upgrade with wasm hash and versioning",
        wasm_file: "astinel_upgradeable.wasm",
    },
    ContractInfo {
        name: "astinel-stellar-asset",
        description: "Stellar Asset Frontend (SAFE) with mint, burn, transfer, approve",
        wasm_file: "astinel_stellar_asset.wasm",
    },
];
