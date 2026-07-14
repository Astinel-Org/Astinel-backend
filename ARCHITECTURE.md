# Architecture

## System Context

```mermaid
graph TD
    User(("Developer / Auditor"))
    GHApp(("GitHub.com"))
    SC(("Stellar Network<br/>(Soroban RPC)"))

    subgraph Astinel Platform
        Server["Astinel Server"]
        CLI["sentinel CLI"]
        WebApp["Astinel Frontend<br/>(Next.js)"]
    end

    User -->|"HTTP :8080"| Server
    User -->|"terminal"| CLI
    User -->|"browser :3000"| WebApp
    WebApp -->|"HTTP :8080"| Server
    Server -->|"GitHub API"| GHApp
    Server -->|"JSON-RPC"| SC
```

## Container Architecture

```mermaid
graph TD
    LB["Reverse Proxy<br/>(optional)"]

    subgraph Core
        API["Axum HTTP Server"]
        CLI["CLI Binary"]
        Jobs["Scan Job Worker"]
    end

    subgraph Storage
        PG[("PostgreSQL")]
        R[("Redis")]
    end

    subgraph Integrations
        GH["GitHub Service"]
        AI["AI Provider"]
        DP["Contract Deployer"]
    end

    LB --> API
    API --> Auth["Auth Middleware"]
    Auth --> Routes["Route Handlers"]
    Routes --> Services["Business Services"]
    Services --> PG
    Services --> R
    Services --> GH
    Services --> AI
    Services --> DP
    Jobs --> R
    Jobs --> PG
    CLI --> PG
    CLI --> R
```

## Scan Pipeline

```mermaid
sequenceDiagram
    participant Client
    participant API as Axum API
    participant Queue as Redis Queue
    participant Worker as Scan Worker
    participant Parser as Parser
    participant Rules as Rule Engine
    participant DB as PostgreSQL

    Client->>API: POST /v1/scans/trigger
    API->>API: Validate auth + RBAC
    API->>DB: Create scan_job record
    API->>Queue: LPUSH scan job
    API-->>Client: 200 { id, status: "queued" }

    Worker->>Queue: BRPOP scan job
    Worker->>DB: Update status → "running"
    Worker->>Parser: Parse source files
    Parser-->>Worker: AST

    Worker->>Rules: Run 10 rules against AST
    Rules-->>Worker: Findings list
    Worker->>Worker: Calculate score (0-100)

    Worker->>DB: Insert findings
    Worker->>DB: Insert scan_result
    Worker->>DB: Generate reports (5 formats)
    Worker->>DB: Update scan_job status → "completed"

    Client->>API: GET /v1/scans/{id}/status
    API->>Redis: GET cached status
    API-->>Client: { status, progress }
```

## Auth Flow

```mermaid
sequenceDiagram
    participant User
    participant API
    participant DB
    participant Redis

    User->>API: POST /v1/auth/login
    API->>DB: Verify credentials
    API->>API: Generate access JWT (15m) + refresh JWT (7d)
    API->>Redis: Store refresh token (jti → TTL)
    API-->>User: { access_token, refresh_token }

    User->>API: GET /v1/projects (Authorization: Bearer access_token)
    API->>API: Verify JWT signature + expiry
    API->>API: Extract AuthContext (user_id, role, permissions)
    API->>API: Check RBAC permission
    API->>DB: Query with user_id filter
    API-->>User: 200 { data: [...] }

    User->>API: POST /v1/auth/refresh
    API->>API: Verify refresh JWT
    API->>Redis: Check jti exists (not revoked)
    API->>API: Issue new token pair
    API-->>User: { access_token, refresh_token }
```

## RBAC Model

4 roles, 12 permissions:

| Role | Inherits | Key Permissions |
|---|---|---|
| Owner | Admin | ManageMembers, Administer, ManageSettings |
| Admin | Developer | ManageApiKeys, ManageWebhooks |
| Developer | Viewer | CreateProject, UpdateProject, TriggerScan |
| Viewer | — | ViewScan, ViewReport, ViewFindings |

## Contract Deployment Flow

```mermaid
sequenceDiagram
    participant Client
    participant API
    participant Service as ContractService
    participant Deployer as ContractDeployer
    participant RPC as SorobanRpcClient
    participant DB

    Client->>API: POST /v1/projects/{id}/contracts/deploy
    API->>API: Validate auth + project ownership
    API->>Service: deploy_contract(project_id, name, network)
    Service->>Service: Lookup contract template info
    Service->>Deployer: deploy(&contract_info)
    Deployer->>Deployer: Read .wasm file from disk
    Deployer->>Deployer: Compute SHA-256 hash
    Deployer->>RPC: upload_wasm(wasm_bytes)
    RPC-->>Deployer: wasm_id
    Deployer->>RPC: create_contract(wasm_id, source, salt)
    RPC-->>Deployer: contract_id
    Deployer-->>Service: ContractDeployment { contract_id, wasm_hash }
    Service->>DB: Insert contract_deployments record
    Service-->>API: Deployment result
    API-->>Client: 201 { data: { contract_id, ... } }
```

## Built-in Rules

| Rule ID | Severity | Category | Detection |
|---|---|---|---|
| missing-require-auth | Critical | Security | Functions missing `caller.require_auth()` |
| unsafe-panic | High | Security | Bare `panic!()` or `unwrap()` in contract code |
| auth-mistake | High | Security | Incorrect `require_auth` target address |
| integer-overflow | High | Security | Unchecked arithmetic operations |
| large-storage-write | Medium | Performance | Persistent storage writes exceeding threshold |
| missing-ttl | Medium | Gas | `extend_ttl` not called after persistent writes |
| contract-upgrade | Medium | Upgrade | Direct `update_current_contract_wasm` calls |
| dead-code | Low | BestPractice | Unused functions and variables |
| unused-storage | Low | BestPractice | Storage writes without corresponding reads |
| gas-optimization | Info | Gas | Suboptimal patterns (e.g., `Vec` instead of `Map`) |

## Report Formats

| Format | MIME | Use Case |
|---|---|---|
| Pretty | text/plain | Terminal output (colorized, grouped by severity) |
| Compact | text/plain | One finding per line, pipe-delimited |
| JSON | application/json | Programmatic consumption (versioned schema) |
| Markdown | text/markdown | GitHub/GitLab issue or PR comment |
| SARIF | application/sarif+json | GitHub Code Scanning integration |

## Redis Operations

| Component | Key Pattern | Purpose |
|---|---|---|
| SessionStore | `session:refresh:{jti}` | Refresh token validity + TTL |
| RateLimiter | `ratelimit:{key}` | Sliding window counter |
| WebhookDedup | `webhook:dedup:{event_id}` | Idempotent webhook processing |
| ScanStatusCache | `scan:status:{scan_id}` | In-progress scan progress |
| JobQueue | `queue:scans` | Scan job FIFO (LPUSH/BRPOP) |

## Security Model

- Passwords hashed with Argon2id (memory-hard)
- JWT signed with HS256, separate secrets for access and refresh tokens
- Refresh tokens stored in Redis with TTL for revocation
- API keys stored as bcrypt-like hash with prefix for identification
- Stellar wallet auth uses ed25519-dalek challenge-response
- Webhook payloads verified via HMAC-SHA256
- RBAC enforced at the service layer after JWT verification
- Rate limiting via Redis sliding window (configurable per-endpoint)
