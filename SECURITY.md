# Security

## Authentication

### JWT Tokens

The platform uses a dual-token JWT system:

| Token | Lifetime | Storage | Purpose |
|---|---|---|---|
| Access token | 15 minutes | Client (memory/localStorage) | API authorization |
| Refresh token | 7 days | Client + Redis (jti hash) | Token rotation |

Both tokens are signed with HS256 using separate secrets (`JWT_ACCESS_SECRET`, `JWT_REFRESH_SECRET`). Refresh tokens can be revoked by deleting the corresponding jti entry from Redis.

### Stellar Wallet Authentication

Challenge-response authentication using ed25519-dalek:

1. Client requests a challenge nonce via `POST /v1/auth/wallet/challenge`
2. Client signs the challenge with their Stellar wallet private key
3. Server verifies the signature against the provided public key
4. On success, server issues a standard JWT token pair

### API Key Authentication

API keys use a hash + prefix pattern:
- The full key is returned once at creation and cannot be retrieved
- A prefix (first 8 characters) is stored for identification
- The key is hashed with argon2 before storage
- Clients authenticate via `X-API-Key` header

## Authorization

### RBAC Model

4 roles with escalating permissions:

| Role | Inherits | Permissions |
|---|---|---|
| Owner | Admin | Administer, ManageMembers, ManageSettings |
| Admin | Developer | ManageApiKeys, ManageWebhooks |
| Developer | Viewer | CreateProject, UpdateProject, DeleteProject, TriggerScan |
| Viewer | — | ViewScan, ViewReport, ViewFindings |

Permission checks are enforced at the service layer after JWT/API key verification.

## Secrets Management

- JWT signing secrets default to development values — must be changed in production
- Database credentials are passed via environment variables
- GitHub App private keys are loaded from environment variables (PEM format)
- Deployer secret keys for Soroban are passed via environment variables
- Webhook secrets are stored in plaintext in the `webhooks` table (known limitation)

## Input Validation

- All API inputs are validated through serde deserialization with strict types
- SQL injection is prevented by sqlx's parameterized queries
- Scan inputs are parsed through the `syn` parser before analysis
- Finding PATCH operations validate the `is_suppressed` field type

## Rate Limiting

Sliding window rate limiting via Redis:
- Configurable window and max request count per key
- Applied per authentication context (user, API key, or IP)
- Rate limit headers included in responses

## Webhook Verification

GitHub webhook payloads are verified using HMAC-SHA256:
- The raw request body is hashed with the shared webhook secret
- The result is compared against the `X-Hub-Signature-256` header
- Only verified payloads are processed

## Known Security Considerations

- **Token storage**: The frontend stores JWT tokens in localStorage, making them accessible to JavaScript (XSS-vulnerable). HTTP-only cookies are not used.
- **No CSP**: The frontend does not configure Content Security Policy headers.
- **No auto-redirect on 401**: The frontend does not automatically redirect to login on token expiry.
- **Refresh token**: The frontend does not implement automatic refresh token rotation.
- **No HTTPS**: TLS termination is expected to be handled by a reverse proxy.
- **Webhook secret storage**: Webhook secrets are stored without encryption at rest.
