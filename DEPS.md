# Dependencies Analysis

## External Crates

### Required (all phases)

| Crate | Version | Used By | Purpose |
|-------|---------|---------|---------|
| `clap` | 4.5 | sentinel-cli | Argument parsing with derive macros |
| `serde` | 1.0 | core, config, report, cli | Serialization framework |
| `serde_json` | 1.0 | config, report | JSON output, config format |
| `toml` | 0.8 | config | TOML configuration parsing |
| `anyhow` | 1.0 | cli | Top-level error handling in binary |
| `thiserror` | 1.0 | core, parser, config | Library error types |
| `tracing` | 0.1 | all crates | Structured logging |
| `tracing-subscriber` | 0.3 | cli | Log output formatting |

### Parser (Phase 4+)

| Crate | Version | Used By | Purpose |
|-------|---------|---------|---------|
| `syn` | 2.0 | sentinel-parser | Full Rust source parsing |
| `quote` | 1.0 | sentinel-parser | Token stream manipulation |
| `proc-macro2` | 1.0 | sentinel-parser | Token representation |

### Report (Phase 7)

| Crate | Version | Used By | Purpose |
|-------|---------|---------|---------|
| `pulldown-cmark` | 0.11 | sentinel-report | Markdown generation |
| `minijinja` | 2.0 | sentinel-report | HTML templating |
| `syntect` | 5.2 | sentinel-report | Syntax highlighting in HTML reports |

### Testing & Benchmarking

| Crate | Version | Used By | Purpose |
|-------|---------|---------|---------|
| `criterion` | 0.5 | benches | Benchmarking |
| `insta` | 1.39 | sentinel-report | Snapshot testing for reports |
| `pretty_assertions` | 1.4 | all (dev) | Readable test diffs |
| `test-log` | 0.2 | all (dev) | Trace logging in tests |

### CI / Developer Experience

| Tool | Purpose |
|------|---------|
| `cargo-hack` | Check feature combinations in CI |
| `cargo-deny` | License and dependency auditing |
| `cargo-outdated` | Dependency freshness checks |
| `typos` | Spell-check source code |

## Internal Dependency Graph (Detailed)

```
sentinel-core
  ├── serde (with derive)
  └── thiserror

sentinel-utils
  └── tracing

sentinel-config
  ├── sentinel-core
  ├── serde
  ├── toml
  └── thiserror

sentinel-parser
  ├── sentinel-core
  ├── syn (with full + extra-traits)
  ├── proc-macro2
  ├── quote
  └── thiserror

sentinel-rules
  ├── sentinel-core
  └── sentinel-parser

sentinel-report
  ├── sentinel-core
  ├── serde_json
  ├── pulldown-cmark
  └── minijinja

sentinel-ai
  └── sentinel-core

sentinel-cli
  ├── sentinel-core
  ├── sentinel-config
  ├── sentinel-parser
  ├── sentinel-rules
  ├── sentinel-report
  ├── sentinel-utils
  ├── clap (with derive + env)
  ├── anyhow
  ├── tracing
  └── tracing-subscriber
```

## Version Strategy

1. Workspace-level `[workspace.dependencies]` pins all versions
2. Each crate's `Cargo.toml` references workspace deps
3. MSRV is latest stable (1.96.0)
4. Dependencies are updated on a regular schedule via `cargo-outdated`
5. `cargo-deny` ensures no duplicate versions in the dependency tree

## Why These Choices

- **`syn` over custom parser:** Soroban contracts are Rust. `syn` is battle-tested, handles the full Rust grammar, and is maintained by the Rust team. A custom parser would be fragile and incomplete.

- **`minijinja` over `tera`:** Lighter, faster compile times, better security model (auto-escapes by default).

- **`pulldown-cmark` over manual string building:** Standards-compliant Markdown, easy to extend, well-maintained.

- **`tracing` over `log`:** Structured logging enables future span-based profiling, better for debugging rule performance.

- **`clap` derive over builder:** Less boilerplate, enforces declarative CLI design, good error messages for free.
