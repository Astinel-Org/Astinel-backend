# Project Directory Tree

```
sentinel/
│
├── Cargo.toml                          # Workspace root
├── rust-toolchain.toml                 # Stable Rust pinning
├── rustfmt.toml                        # Formatter config
├── .gitignore
├── .gitattributes
│
├── ARCHITECTURE.md                     # Architecture document (this phase)
├── DEPS.md                             # Dependency analysis (this phase)
├── DIRECTORY.md                        # This file
├── ROADMAP.md                          # Implementation roadmap
├── CONTRIBUTING.md                     # Contributor guide (Phase 1)
├── SECURITY.md                         # Security policy (Phase 1)
├── CHANGELOG.md                        # Release changelog
├── README.md                           # Project readme (Phase 1)
│
├── LICENSE                             # Apache 2.0 (Phase 1)
│
├── crates/
│   ├── sentinel-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── severity.rs             # Severity enum
│   │       ├── category.rs             # Category enum
│   │       ├── finding.rs              # Finding struct
│   │       ├── rule.rs                 # Rule trait
│   │       ├── ast.rs                  # Ast type
│   │       ├── registry.rs             # RuleRegistry
│   │       ├── score.rs                # Security score computation
│   │       └── error.rs                # Core error types
│   │
│   ├── sentinel-config/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── config.rs               # SentinelConfig struct
│   │       ├── loader.rs               # ConfigLoader
│   │       ├── template.rs             # Default config template
│   │       └── error.rs                # Config error types
│   │
│   ├── sentinel-parser/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── parser.rs               # Main parser entry point
│   │       ├── visitor.rs              # AstVisitor trait
│   │       ├── soroban.rs              # Soroban-specific pattern detection
│   │       ├── patterns/
│   │       │   ├── mod.rs
│   │       │   ├── storage.rs          # Storage operation detection
│   │       │   ├── auth.rs             # Authorization detection
│   │       │   ├── panic.rs            # Panic detection
│   │       │   ├── ttl.rs              # TTL operation detection
│   │       │   ├── arithmetic.rs       # Arithmetic detection
│   │       │   └── upgrade.rs          # Upgrade detection
│   │       └── error.rs                # Parser error types
│   │
│   ├── sentinel-rules/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── engine.rs               # Rule engine (execution, filtering)
│   │       └── rules/
│   │           ├── mod.rs
│   │           ├── missing_auth.rs     # Missing require_auth()
│   │           ├── unsafe_panic.rs     # Unsafe panic!
│   │           ├── large_storage.rs    # Large storage writes
│   │           ├── dead_code.rs        # Dead code
│   │           ├── unused_storage.rs   # Unused storage
│   │           ├── missing_ttl.rs      # Missing TTL extension
│   │           ├── auth_mistake.rs     # Authorization mistakes
│   │           ├── integer_overflow.rs # Integer overflow risks
│   │           ├── gas_optimization.rs # Gas optimization
│   │           └── contract_upgrade.rs # Contract upgrade risks
│   │
│   ├── sentinel-report/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── report.rs               # Report trait
│   │       ├── markdown.rs             # Markdown report
│   │       ├── json.rs                 # JSON report
│   │       ├── html.rs                 # HTML report
│   │       ├── score.rs                # Score formatting
│   │       └── templates/
│   │           └── report.html.jinja   # HTML template
│   │
│   ├── sentinel-utils/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── fs.rs                   # Filesystem utilities
│   │       ├── path.rs                 # Path resolution
│   │       └── logging.rs              # Logging initialization
│   │
│   ├── sentinel-ai/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs                  # Stub (future)
│   │
│   └── sentinel-cli/
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── commands/
│           │   ├── mod.rs
│           │   ├── init.rs             # sentinel init
│           │   ├── scan.rs             # sentinel scan
│           │   ├── doctor.rs           # sentinel doctor
│           │   ├── report.rs           # sentinel report
│           │   ├── rules_cmd.rs        # sentinel rules
│           │   └── verify.rs           # sentinel verify (stub)
│           └── output.rs               # Colored terminal output
│
├── docs/
│   ├── book.toml                       # mdBook config
│   └── src/
│       ├── SUMMARY.md
│       ├── introduction.md
│       ├── installation.md
│       ├── quickstart.md
│       ├── configuration.md
│       ├── rules/
│       │   ├── overview.md
│       │   ├── missing-require-auth.md
│       │   ├── unsafe-panic.md
│       │   ├── large-storage-write.md
│       │   ├── dead-code.md
│       │   ├── unused-storage.md
│       │   ├── missing-ttl.md
│       │   ├── auth-mistake.md
│       │   ├── integer-overflow.md
│       │   ├── gas-optimization.md
│       │   └── contract-upgrade.md
│       ├── reports.md
│       ├── ci-integration.md
│       ├── extending.md
│       └── contributing.md
│
├── examples/
│   ├── basic-contract/
│   │   └── src/
│   │       └── lib.rs
│   ├── vulnerable-contracts/
│   │   ├── missing_auth/
│   │   ├── unsafe_panic/
│   │   └── ...
│   └── secure-contract/
│       └── src/
│           └── lib.rs
│
├── tests/
│   ├── integration/
│   │   ├── scan_command.rs
│   │   ├── config_loading.rs
│   │   └── reporting.rs
│   └── fixtures/
│       ├── sentinel.toml
│       ├── contracts/
│       │   ├── missing_auth.rs
│       │   ├── unsafe_panic.rs
│       │   ├── all_vulnerabilities.rs
│       │   └── secure.rs
│       └── reports/
│           └── expected_report.md
│
├── benches/
│   ├── lib.rs
│   └── benchmarks/
│       ├── parser.rs
│       ├── rules.rs
│       └── full_scan.rs
│
├── .github/
│   ├── workflows/
│   │   ├── ci.yml                      # cargo fmt, clippy, test, doc
│   │   ├── release.yml                 # GitHub Release workflow
│   │   └── audit.yml                   # Dependency auditing
│   ├── ISSUE_TEMPLATE/
│   │   ├── bug_report.md
│   │   └── feature_request.md
│   └── PULL_REQUEST_TEMPLATE.md
│
└── target/                             # (gitignored)
```

## File Count Summary

| Directory | Files (est.) |
|-----------|-------------|
| Root config | 5 |
| Documentation | 6 |
| `crates/sentinel-core` | 8 source |
| `crates/sentinel-config` | 5 source |
| `crates/sentinel-parser` | 10 source |
| `crates/sentinel-rules` | 12 source |
| `crates/sentinel-report` | 7 source |
| `crates/sentinel-utils` | 4 source |
| `crates/sentinel-ai` | 1 source |
| `crates/sentinel-cli` | 9 source |
| `docs/src` | 17 |
| `examples` | ~6 |
| `tests` | ~10 |
| `benches` | ~4 |
| `.github` | ~5 |
| **Total** | **~110** |
