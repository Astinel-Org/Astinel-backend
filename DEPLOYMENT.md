# Deployment

## Prerequisites

- Rust 1.85+
- PostgreSQL 16+
- Redis 7+
- OpenSSL (for HTTPS, via reverse proxy)

## Environment

All configuration is via environment variables. See [Configuration](#configuration) for the full list.

Copy `.env.example` to `.env` and adjust:

```bash
cp .env.example .env
```

## Database

Create the database:

```bash
createdb astinel
```

Migrations run automatically on server startup. To run them manually:

```bash
DATABASE_URL=postgresql://user:pass@localhost/astinel cargo run --bin astinel-server
```

## Running

### Development

```bash
cargo run --bin astinel-server
```

The server listens on `0.0.0.0:8080` by default.

### CLI Tool

```bash
# Scan a single file
cargo run --bin sentinel scan path/to/contract.rs

# Print version
cargo run --bin sentinel -- --version
```

### Production Build

```bash
cargo build --release
./target/release/astinel-server
```

## Docker

A Dockerfile is not provided at this time. Deployment requires manual setup of PostgreSQL, Redis, and the Rust toolchain.

## Reverse Proxy

HTTPS termination should be handled by a reverse proxy (nginx, Caddy, or similar). The application does not serve HTTPS directly.

Example nginx configuration:

```nginx
server {
    listen 443 ssl;
    server_name api.astinel.io;

    ssl_certificate /etc/ssl/certs/astinel.crt;
    ssl_certificate_key /etc/ssl/private/astinel.key;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

## Monitoring

Prometheus metrics available at `/metrics`. Configure your Prometheus server to scrape this endpoint:

```yaml
scrape_configs:
  - job_name: 'astinel'
    static_configs:
      - targets: ['localhost:8080']
```

## Health Checks

The `/v1/health` endpoint returns:

```json
{ "status": "ok", "version": "0.6.0", "service": "astinel-backend" }
```

## Known Considerations

- Webhook secrets are stored in plaintext in the database
- The OpenAPI spec at `/v1/openapi.json` is a static placeholder, not auto-generated
- Contract deployment requires the `SOROBAN_RPC_URL` and `ASTINEL_CONTRACTS_DIR` environment variables to be set
- GitHub App integration is optional and only initialized when `GITHUB_APP_ID` is set
