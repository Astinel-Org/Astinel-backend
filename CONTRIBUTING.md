# Contributing to Astinel Backend

## Development Setup

```bash
git clone https://github.com/Astinel-Org/Astinel-backend.git
cd Astinel-backend
cargo build
cargo test
```

Requires Rust 1.85+, PostgreSQL 16, and Redis 7 for the full test suite.

## CI Expectations

Every pull request must pass:

```bash
cargo check          # Compilation
cargo test           # All unit tests
cargo clippy -- -D warnings  # Linting
cargo fmt --check    # Formatting
cargo deny check     # Security audit
```

## Commit Conventions

Use [conventional commits](https://www.conventionalcommits.org/):

```
feat: add Soroban contract health check endpoint
fix: handle null pointer dereference in parser
docs: update API reference for contract routes
test: add integration tests for auth middleware
refactor: extract RBAC check into reusable function
```

## Pull Request Process

1. Create a feature branch from `develop`
2. Write tests for new functionality
3. Ensure all CI checks pass
4. Submit PR against `develop`
5. At least one maintainer review required

## Code Style

- Follow `rustfmt` conventions (enforced in CI)
- `clippy` must pass with `-D warnings` (no warnings allowed)
- Prefer `thiserror` for error types over `anyhow` in library code
- Use `anyhow` for binary/CLI error handling
- Document public API items with doc comments
- Keep modules focused: one responsibility per module
