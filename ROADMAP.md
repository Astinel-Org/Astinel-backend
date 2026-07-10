# Sentinel Roadmap

## Phase 0: Architecture & Planning (Current)

- [x] Repository initialized
- [x] Rust toolchain verified (1.96.0)
- [ ] Architecture documented
- [ ] Crate dependency graph finalized
- [ ] First commit (scaffolding)

## Phase 1: Workspace Scaffolding

**Goal:** Establish the Cargo workspace, all crate stubs, and CI pipeline.

- Root `Cargo.toml` with workspace `[workspace]`
- 8 crate stubs under `crates/` (each with `Cargo.toml`, `src/lib.rs`)
- `docs/`, `examples/`, `tests/`, `benches/`, `.github/` directories
- `LICENSE` (Apache-2.0), `README.md`, `CONTRIBUTING.md`, `SECURITY.md`
- `.github/workflows/ci.yml` — `cargo fmt`, `clippy`, `test`, `doc`
- `rust-toolchain.toml` pinning stable
- `rustfmt.toml` for consistent formatting
- `.gitignore`
- Initial `git commit`

*Key decisions:*
- Workspace members declared explicitly (not glob) for deterministic builds
- Every crate has `edition = 2021`, `publish = false` initially
- Dependency versions pinned in workspace `Cargo.toml` for consistency

## Phase 2: sentinel-core

**Goal:** Foundation types shared by every crate.

- `Severity` enum (`Critical`, `High`, `Medium`, `Low`, `Info`)
- `Category` enum (`Security`, `Performance`, `Gas`, `BestPractice`, `Upgrade`)
- `Finding` struct (rule_id, severity, category, file, line, col, message, recommendation, fix_example)
- `Rule` trait (id, name, description, severity, check)
- `Ast` type (opaque container for parsed contract representation)
- `RuleRegistry` — holds all registered rules
- `ScanResult` — holds findings + computed security score
- `Score` computation logic (0–100 based on weighted findings)

*Key decisions:*
- Use `thiserror` for library error types
- All types implement `Serialize`/`Deserialize` for report output
- `Rule` trait uses `&dyn Ast` — rules never own the AST
- `Category` is exhaustive; `#[non_exhaustive]` for forward compat
- `Severity` derives `PartialOrd` for sorting

## Phase 3: sentinel-config

**Goal:** Parse `sentinel.toml` with clear error messages.

- `SentinelConfig` struct (rules, severity_threshold, format, ignore_paths)
- Deserialize from TOML
- `ConfigLoader::load(path)` returns `Result<SentinelConfig>`
- Merge with CLI overrides
- Default config when no file present
- `sentinel init` generates a template config

*Key decisions:*
- `serde` + `toml` for parsing
- Config paths relative to project root
- `rules` field supports `"default"`, `"all"`, or `["rule-id-1", ...]`
- `severity` is a minimum threshold; findings below it are filtered

## Phase 4: sentinel-parser

**Goal:** Parse Soroban contract source into a traversable AST.

- Parse Rust source using `syn` crate
- Extract Soroban-specific items:
  - `#[contractimpl]` / `#[contract]` attributes
  - Function signatures and bodies
  - `require_auth!()` / `require_auth` calls
  - Storage calls (`Env::storage()` → `get`/`set`/`has`/`del`)
  - `fn __check_auth()` implementations
  - `fn upgrade()` / `fn migrate()` patterns
  - `panic!()` / `panic_with_error!()` / `unwrap()` / `expect()` calls
  - TTL operations (`extend_ttl`)
  - Integer arithmetic (overflow-prone patterns)
- `Ast` struct holds the parsed representation
- `AstVisitor` trait for walking the AST
- For non-Rust or invalid input, return structured errors

*Key decisions:*
- Use `syn` (not a custom parser) — Soroban contracts are Rust
- AST is a curated representation, not a full Rust AST
- Visitor pattern enables rules to only look at what they need
- Parse only `.rs` files; skip `target/`, `node_modules/`, etc.
- File-level granularity: each file parsed independently

## Phase 5: sentinel-rules

**Goal:** The rule engine with initial rules — the heart of Sentinel.

- `RuleRegistry::builtins()` returns all shipped rules
- Each rule as a separate module in `src/rules/`
- Rule execution:
  - Filter rules by config (severity threshold, enabled list)
  - For each rule, call `check(ast)` → `Vec<Finding>`
  - Collect and deduplicate findings
  - Compute security score

**Initial rules (10):**

| # | Rule ID | Description | Severity |
|---|---------|-------------|----------|
| 1 | `missing-require-auth` | Public functions missing authorization | Critical |
| 2 | `unsafe-panic` | `panic!()` / `unwrap()` in contract code | High |
| 3 | `large-storage-write` | Storage writes without size consideration | Medium |
| 4 | `dead-code` | Unused functions / dead code | Low |
| 5 | `unused-storage` | Storage writes never read | Medium |
| 6 | `missing-ttl` | Persistent storage without TTL extension | High |
| 7 | `auth-mistake` | Incorrect authorization patterns | Critical |
| 8 | `integer-overflow` | Unchecked arithmetic overflow | High |
| 9 | `gas-optimization` | Suboptimal gas patterns | Low |
| 10 | `contract-upgrade` | Unsafe upgrade patterns | High |

*Key decisions:*
- Each rule is a separate file; no rule exceeds ~200 lines
- Rules are stateless — pure functions from `&Ast` to `Vec<Finding>`
- Rules can register AST node types they care about (performance)
- Finding positions use 1-indexed line/column

## Phase 6: sentinel-cli

**Goal:** A polished CLI that feels like `cargo` / `clippy`.

- `sentinel init` — generate `sentinel.toml`
- `sentinel scan [path]` — scan project, print findings
- `sentinel doctor` — check Rust/Soroban environment
- `sentinel report` — generate report files
- `sentinel rules` — list available rules with descriptions
- `sentinel verify` — produce signed security manifest (future)
- Colored output with `--json` flag for machine-readable
- Exit code 0 = no findings above threshold, 1 = findings found, 2 = error

*Key decisions:*
- Use `clap` v4 with derive API
- Subcommand-centric design
- Output to stderr for diagnostics, stdout for machine output
- `--quiet` / `--verbose` flags
- `--format` flag overrides config

## Phase 7: sentinel-report

**Goal:** Beautiful, useful reports in multiple formats.

- `Report` trait (generate)
- `MarkdownReport` — formatted with severity badges, code snippets
- `JsonReport` — structured JSON for CI/tooling
- `HtmlReport` — standalone HTML with styling

*Key decisions:*
- Reports include: summary table, security score, per-finding details
- HTML is self-contained (no external dependencies)
- JSON follows schema that can be consumed by GitHub Actions, SARIF converter
- Reports write to `sentinel-report/` directory by default

## Phase 8: Integration Testing

**Goal:** Full confidence in correctness.

- Unit tests for every public function
- Integration tests with example Soroban contracts
- Snapshot testing for report output
- Property-based testing for config parsing
- Benchmark suite with criterion
- Test Soroban contracts in `tests/fixtures/`

## Phase 9: Documentation

**Goal:** Professional, comprehensive documentation.

- `README.md`: what, why, quick start, architecture diagram
- `CONTRIBUTING.md`: how to add rules, code style, PR process
- `SECURITY.md`: vulnerability disclosure policy
- mdBook-based docs site: `docs/`
- `cargo doc` with examples on all public items
- Rule authoring guide
- CI badge (docs build)

## Phase 10: Performance Optimization

**Goal:** Fast enough for any codebase.

- Parallel rule execution with `rayon`
- Incremental parsing (only changed files)
- Caching parsed ASTs
- Lazy rule loading
- Benchmark-driven optimization

## Future

- `sentinel-ai` — AI explanation layer (uses contracts with AI providers)
- VS Code extension (LSP-based)
- GitHub Action
- `sentinel fix` — auto-fix suggestions
- Rule marketplace
- SARIF output
- Security score badge for READMEs

---

*This roadmap is a living document. Phases may be reordered as priorities evolve.*
