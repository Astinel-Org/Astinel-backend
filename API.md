# API Reference

Base URL: `http://localhost:8080`

All endpoints return JSON. Responses follow the shape:

```
{ "success": true, "data": <payload> }
```

Error responses:

```
{ "error": { "code": "...", "message": "..." } }
```

## Authentication

| Method | Auth | Header |
|---|---|---|
| Public | None | — |
| Auth required | JWT Bearer | `Authorization: Bearer <token>` |
| Auth required | API Key | `X-API-Key: <key>` |

## Health

| Method | Path | Auth | Description |
|---|---|---|---|
| GET | `/v1/health` | None | Service health check |
| GET | `/v1/version` | None | Version info |
| GET | `/v1/openapi.json` | None | OpenAPI 3.1 spec (static stub) |
| GET | `/metrics` | None | Prometheus metrics |

## Auth

| Method | Path | Request | Response |
|---|---|---|---|
| POST | `/v1/auth/register` | `{ email, password, display_name? }` | `{ access_token, refresh_token, user }` |
| POST | `/v1/auth/login` | `{ email, password }` | `{ access_token, refresh_token, user }` |
| POST | `/v1/auth/refresh` | `{ refresh_token }` | `{ access_token, refresh_token }` |
| POST | `/v1/auth/logout` | `{ refresh_token }` | `{ success: true }` |
| POST | `/v1/auth/wallet/challenge` | `{ public_key }` | `{ challenge, expires_at }` |
| POST | `/v1/auth/wallet/login` | `{ public_key, signature, challenge }` | `{ access_token, refresh_token }` |

## Projects

All require authentication.

| Method | Path | Request | Response |
|---|---|---|---|
| GET | `/v1/projects` | `?organization_id=<uuid>` | `Project[]` |
| POST | `/v1/projects` | `{ name, slug?, description?, repository_url?, default_branch?, language? }` | `Project` |
| GET | `/v1/projects/:id` | — | `Project` |
| PUT | `/v1/projects/:id` | `{ name?, description?, ... }` | `Project` |
| DELETE | `/v1/projects/:id` | — | `{ success: true }` |

Project shape: `{ id, name, slug, description, repository_url, default_branch, language, local_path, settings, created_at, updated_at }`

## Scans

All require authentication.

| Method | Path | Request | Response |
|---|---|---|---|
| POST | `/v1/scans/trigger` | `{ project_id, branch?, commit_sha?, config? }` | `Scan` (status: queued) |
| GET | `/v1/scans/:id` | — | `Scan` |
| GET | `/v1/scans/:id/status` | — | `{ status, progress }` (cached in Redis) |
| GET | `/v1/scans/:id/results` | — | `ScanResult` + `Finding[]` |

Scan shape: `{ id, project_id, branch, status, trigger, priority, progress, score, queued_at, started_at, completed_at }`

ScanResult shape: `{ id, scan_job_id, status, score, total_files, total_rules, total_findings, critical, high, medium, low, info, duration_ms }`

## Findings

All require authentication.

| Method | Path | Query Params | Response |
|---|---|---|---|
| GET | `/v1/findings` | `scan_id?, severity?, category?, file_path?, page?, per_page?` | `Finding[]` |
| PATCH | `/v1/findings/:id` | — | `{ success: true }` |

PATCH body: `{ is_suppressed: boolean }`

Finding shape: `{ id, scan_result_id, rule_id, severity, category, file_path, line, column, message, recommendation, fix_example, is_suppressed, created_at }`

## Reports

All require authentication.

| Method | Path | Query Params | Response |
|---|---|---|---|
| GET | `/v1/reports` | `scan_result_id?, project_id?` | `Report[]` |
| GET | `/v1/reports/:id` | — | `Report` |

Report shape: `{ id, scan_result_id, format, content, file_path, file_size, created_at }`

## Dashboard

All require authentication.

| Method | Path | Response |
|---|---|---|
| GET | `/v1/dashboard` | `{ total_projects, total_scans, total_findings, critical_findings, high_findings, medium_findings, average_score, recent_scans: [...], findings_by_severity: [...] }` |

## Notifications

All require authentication.

| Method | Path | Response |
|---|---|---|
| GET | `/v1/notifications` | `Notification[]` (max 200, default 50) |
| GET | `/v1/notifications/unread-count` | `{ count }` |
| POST | `/v1/notifications/:id/read` | `{ success: true }` |
| POST | `/v1/notifications/read-all` | `{ success: true }` |

Notification shape: `{ id, organization_id, event_type, title, message, severity, resource_type, resource_id, is_read, created_at }`

## Webhooks

| Method | Path | Description |
|---|---|---|
| POST | `/v1/webhooks/github` | Register GitHub App installation |

## AI

All require authentication.

| Method | Path | Request | Response |
|---|---|---|---|
| POST | `/v1/ai/fix-suggestion` | `{ finding_id?, code?, message? }` | `{ suggestion }` |
| POST | `/v1/ai/analyze` | `{ code, prompt? }` | `{ analysis }` |
| GET | `/v1/ai/health` | — | `{ available: bool }` |

## Contracts

| Method | Path | Auth | Description |
|---|---|---|---|
| GET | `/v1/contracts` | None | List 5 supported contract templates |
| GET | `/v1/contracts/health` | None | Soroban RPC health check |
| GET | `/v1/projects/:id/contracts` | Required | List deployments (`?network=testnet`) |
| POST | `/v1/projects/:id/contracts/deploy` | Required | Deploy a contract |

Deploy request: `{ contract_name, network? }`

## Error Codes

| Status | Code | Meaning |
|---|---|---|
| 400 | BadRequest | Invalid input or validation failure |
| 401 | Unauthorized | Missing or expired credentials |
| 401 | InvalidToken | Malformed or revoked token |
| 403 | PermissionDenied | RBAC check failed |
| 404 | NotFound | Resource does not exist |
| 409 | Conflict | Duplicate resource |
| 422 | Unprocessable | Semantic validation failure |
| 500 | InternalError | Unexpected server error |

## Pagination

List endpoints accept optional `page` and `per_page` query parameters. Default `per_page` is 50, maximum is 200. Results include pagination metadata in the response where applicable.
