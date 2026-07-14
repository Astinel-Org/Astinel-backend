# Changelog

## v0.6.0 (2026-07-14)

- Add Soroban contract integration bridge
- ContractDeployment migration, model, and repository
- SorobanRpcClient (HTTP-based, no SDK dependency)
- ContractDeployer with WASM upload and deployment
- ContractService for deployment orchestration
- API routes: list contracts, health, list deployments, deploy
- GitHub Actions CI workflow (build, test, lint, fmt, deny)

## v0.5.0 (2026-07-13)

- AI Security Analyst (Phase 5)
- AiProvider trait with OllamaProvider and DisabledProvider
- AI endpoints: fix-suggestion, analyze, health
- Configurable Ollama URL and model via env vars

## v0.4.0 (2026-07-12)

- Dashboard API with org stats and severity breakdown
- Scan lifecycle endpoints (cancel, retry, progress, result)
- Projects full CRUD (create, read, update, delete)
- Findings detail and PATCH suppress/resolve
- Reports listing with format-based download
- Notification events system (store, list, read)
- Prometheus metrics integration (/metrics)
- OpenAPI 3.1 spec endpoint (static stub)
- Request metrics in auth middleware

## v0.3.0 (2026-07-11)

- GitHub App integration (Phase 3)
- octocrab 0.41 for GitHub API access
- RS256 App JWT generation (10-min expiry)
- Installation access token exchange
- Webhook endpoint for registration
- Check run posting with scan status
- github_installations migration and model

## v0.2.0 (2026-07-10)

- Redis integration (Phase 2)
- RedisPool with connection management
- SessionStore for refresh tokens
- RateLimiter with sliding window
- WebhookDedup for idempotent processing
- ScanStatusCache for in-progress progress
- JobQueue rewritten to use Redis LPUSH/BRPOP

## v0.1.0 (2026-07-09)

- Initial release
- CLI scanning pipeline (`sentinel` binary)
- Custom Soroban parser (syn + proc-macro2)
- 10 built-in security rules
- 5 report formats (Pretty, Compact, JSON, Markdown, SARIF)
- Suppression engine (line and file level)
- Basic REST API server (auth, projects, scans, findings)
- PostgreSQL database with migrations
- 4 RBAC roles with 12 permissions
- JWT authentication with access/refresh tokens
