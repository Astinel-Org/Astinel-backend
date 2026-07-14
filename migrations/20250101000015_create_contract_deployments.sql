CREATE TABLE contract_deployments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    contract_name VARCHAR(64) NOT NULL,
    network VARCHAR(32) NOT NULL DEFAULT 'testnet',
    contract_id VARCHAR(64) NOT NULL,
    wasm_hash VARCHAR(64),
    deploy_tx_hash VARCHAR(64),
    version INTEGER NOT NULL DEFAULT 1,
    metadata JSONB NOT NULL DEFAULT '{}',
    status VARCHAR(32) NOT NULL DEFAULT 'deployed',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_contract_deployments_project ON contract_deployments(project_id, deleted_at);
CREATE INDEX idx_contract_deployments_network ON contract_deployments(network);
CREATE UNIQUE INDEX idx_contract_deployments_contract ON contract_deployments(contract_id, network) WHERE deleted_at IS NULL;
