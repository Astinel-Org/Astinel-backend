# Astinel

Security scanning platform for Stellar and Soroban smart contracts.

[![CI](https://github.com/Astinel-Org/Astinel-backend/actions/workflows/ci.yml/badge.svg)](https://github.com/Astinel-Org/Astinel-backend/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/astinel-backend)](https://crates.io/crates/astinel-backend)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE)

## Overview

Astinel analyzes Soroban smart contract source code for security vulnerabilities, gas inefficiencies, and best-practice violations. It provides a REST API for triggering scans, managing findings, generating reports, and deploying contracts to Stellar networks. An included CLI tool (`sentinel`) enables local scanning without the server.

The platform targets developers migrating to Stellar, security auditors, and DevSecOps teams managing Soroban deployments.

## Features

### Static Analysis
- Custom parser built on `syn` + `proc-macro2` that produces a Soroban-specific AST
- 10 built-in rules across security, performance, gas, and best-practice categories
- Configurable rule filtering and suppression (`// sentinel-ignore`)
- 0--100 scoring system based on finding severity and count

### Reporting
- 5 output formats: Pretty (terminal), Compact, JSON (versioned), Markdown, SARIF 2.1.0

### Authentication & Authorization
- JWT access + refresh token pair with separate signing secrets
- Stellar wallet challenge-response authentication via ed25519-dalek
- API key authentication with hash + prefix pattern
- RBAC with 4 roles (Owner, Admin, Developer, Viewer) and 12 granular permissions

### CI/CD Integration
- GitHub App: JWT-based auth, installation access tokens, check run posting
- Webhook payload verification via HMAC-SHA256

### AI Analysis
- Pluggable `AiProvider` trait with Ollama LLM integration
- Fix suggestion, free-form analysis, and health check endpoints

### Contract Deployment
- HTTP-based Soroban RPC client (no Soroban SDK dependency)
- 5 contract templates: access-control, payable, TTL management, upgradeable, stellar-asset
- WASM upload, contract creation, and deployment record in database

### Monitoring
- Prometheus metrics at `/metrics`
- Redis-backed scan job queue (LPUSH/BRPOP) with priority
- Sliding window rate limiting
- Event-based notification system with read/unread tracking

## Architecture

```mermaid
graph TD
    CLI["sentinel CLI"]
    API["HTTP Clients"]
    GH["GitHub App"]
    Router["Axum Router"]
    Auth["Auth + RBAC"]
    Scanner["Scan Engine"]
    AI["AI Provider (Ollama)"]
    DP["Contract Deployer"]
    PG[("PostgreSQL")]
    R[("Redis")]
    SOR[("Soroban RPC")]

    CLI --> Scanner
    API --> Router --> Auth
    GH --> Auth
    Auth --> Scanner & AI & DP
    Scanner --> PG & R
    AI --> OLLAMA[("Ollama")]
    DP --> SOR
    DP --> PG
```

## Technology Stack

| Layer | Technology | Rationale |
|---|---|---|
| Backend framework | Axum 0.8 | Async, type-safe routing, tower middleware |
| Database | PostgreSQL 16 | ACID compliance, JSONB, full-text search |
| Cache / Queue | Redis 7 | LPUSH/BRPOP queue, sliding-window rate limiting |
| Parser | syn 2.0 + proc-macro2 | Mature Rust parsing, full token access |
| Auth | jsonwebtoken + argon2 + ed25519-dalek | Industry standard JWT, memory-hard hashing |
| AI | reqwest (HTTP) | No SDK dependency, any OpenAI-compatible API |
| Metrics | metrics + prometheus | Lightweight pull-based exposition |
| GitHub | octocrab 0.41 | Full async GitHub API coverage |

## Quick Start

```bash
# Prerequisites: Rust 1.85+, PostgreSQL 16, Redis 7

git clone https://github.com/Astinel-Org/Astinel-backend.git
cd Astinel-backend
cargo build --release

# Set up database
createdb astinel

# Run server (migrations auto-run on startup)
DATABASE_URL=postgresql://postgres:postgres@localhost/astinel \
  REDIS_URL=redis://localhost:6379 \
  cargo run --bin astinel-server

# Or scan locally via CLI
cargo run --bin sentinel scan path/to/contract.rs
```

## Configuration

See [.env.example](.env.example) for all 29 environment variables with defaults and descriptions.

Key variables:

| Variable | Default | Required For |
|---|---|---|
| `DATABASE_URL` | composite | Core operation |
| `REDIS_URL` | `redis://127.0.0.1:6379` | Core operation |
| `JWT_ACCESS_SECRET` | dev default | Auth (change in production) |
| `GITHUB_APP_ID` | — | GitHub integration |
| `OLLAMA_URL` | `http://localhost:11434` | AI features |
| `SOROBAN_RPC_URL` | Soroban testnet | Contract deployment |

## Project Structure

```
src/
├── ai/                  # AI provider trait and Ollama integration
├── api/                 # Axum router, middleware, errors, metrics
│   └── routes/          # 13 route modules
├── auth/                # JWT, RBAC, wallet auth, API keys
├── cache/               # Redis sessions, rate limiting, dedup
├── cli/                 # sentinel CLI binary (clap)
├── config/              # TOML configuration loader
├── contracts/           # Soroban RPC client, deployer, types
├── core/                # Domain model (AST, findings, rules, severity)
├── database/            # PostgreSQL: pool, migrations, models, repositories
├── jobs/                # Redis queue: scan job scheduling, worker
├── scanner/             # Static analysis: parser, rules, reports
├── services/            # Business logic layer
├── state/               # AppState shared across handlers
└── telemetry/           # Tracing and logging
```

## Testing

```bash
cargo test                    # Unit tests
cargo clippy -- -D warnings   # Linting (zero warnings required)
cargo fmt --check             # Formatting
cargo deny check              # Security advisory audit
```

## Related Repositories

| Repository | Description |
|---|---|
| [Astinel-frontend](https://github.com/Astinel-Org/Astinel-frontend) | Next.js web application |
| [Astinel-contracts](https://github.com/Astinel-Org/Astinel-contracts) | Soroban smart contract suite |

## License

Licensed under MIT or Apache 2.0 at your option.
