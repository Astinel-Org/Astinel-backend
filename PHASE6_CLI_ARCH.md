# Phase 6 — sentinel-cli Architecture

## 1. CLI Philosophy

Every tool in the security analysis space makes tradeoffs between power and usability. Sentinel's CLI is designed for long-term maintainability, determinism, and developer trust.

### Design Principles

**Human-first UX**
Developers spend most of their time reading CLI output. Findings must be scannable: severity first, actionable message second, location third. Color, alignment, and summarization are not cosmetic; they are ergonomic necessities. A developer under deadline pressure should be able to spot the critical finding in under two seconds.

**Script-friendly UX**
CI/CD pipelines programmatically consume output. The `--json` and `--sarif` flags are first-class citizens, not afterthoughts. Every field in the JSON output is stable, documented, and versioned. Stderr is reserved for progress/diagnostics. Stdout contains only the requested output format. Piped output auto-disables color and spinners.

**Stable machine-readable output**
The JSON schema is part of the public API. It follows semver: breaking changes to the schema require a major version bump. A `--output-version` flag allows consumers pinning to an older schema to migrate at their own pace. The schema is documented in the repository as a standalone JSON Schema file.

**Deterministic execution**
Given the same input files, configuration, and version, sentinel produces the identical output — including the order of findings. This property is essential for:
- `git bisect` workflows where every commit must produce reproducible results
- CI cache invalidation
- Trust in the analysis tool

Determinism is preserved by sorting findings by severity → file → line → column → rule ID. No randomness, no timestamps in output, no thread-scheduling-dependent ordering.

**Predictable exit codes**
Scripts should never need to parse stdout/stderr to determine the result. Exit codes are the source of truth:
- `0`: clean
- `1`: findings detected
- `2–8`: operational failures (no findings to report)

Every exit code has exactly one meaning. They are documented constants, not magic numbers.

**Minimal surprises**
- No hidden files are created without consent.
- No network requests without explicit flags.
- Config is discovered via explicit convention (sentinel.toml in project root), not magic path traversal.
- Flags override config files which override defaults — no ambiguous precedence.

**Cross-platform consistency**
- Paths are normalized using `std::path` throughout.
- Colors use `supports-color` / `termcolor` detection standard, respecting `NO_COLOR`, `CLICOLOR`, `TERM=dumb`, and `--color`.
- Newlines are native (`\n` on Unix, `\r\n` on Windows for file output).
- All paths in output use `/` (POSIX) regardless of platform.

---

## 2. Crate Architecture

```
sentinel-cli/
├── src/
│   ├── main.rs
│   ├── app.rs
│   ├── commands.rs
│   ├── scan.rs
│   ├── check.rs
│   ├── config.rs
│   ├── output.rs
│   ├── progress.rs
│   ├── terminal.rs
│   ├── exit.rs
│   ├── diagnostics.rs
│   ├── paths.rs
│   └── errors.rs
├── tests/
│   ├── integration/
│   │   ├── scan_basic.rs
│   │   ├── scan_json.rs
│   │   ├── scan_sarif.rs
│   │   ├── config_loading.rs
│   │   ├── exit_codes.rs
│   │   └── error_handling.rs
│   ├── fixtures/
│   │   ├── valid-contract/
│   │   ├── vulnerable-contract/
│   │   ├── empty-project/
│   │   ├── broken-syntax/
│   │   └── config-files/
│   └── snapshots/
│       ├── pretty-output.txt
│       ├── json-output.json
│       └── compact-output.txt
└── Cargo.toml
```

### Module responsibilities

**`main.rs`**
Single responsibility: install panic hooks, parse CLI arguments, delegate to `app::run()`, exit with the returned code. Must be kept as a thin veneer — no logic beyond initialization and teardown. This makes the crate testable by calling `app::run()` directly with simulated arguments.

**`app.rs`**
Top-level application lifecycle. Responsible for:
- Initializing tracing/logging via `sentinel-utils`
- Loading configuration (user, project, CLI overlay)
- Dispatching to the appropriate command handler
- Converting result into exit code
- Ensuring resources are cleaned up

Owns the top-level error display. Does NOT parse CLI arguments (that is the runner's job). Does NOT execute scans.

**`commands.rs`**
Command registry. A single function `dispatch(command: Command, config: CliConfig) -> Result<ExitCode>` that matches on the parsed subcommand and delegates to the appropriate handler module. Keeps `app.rs` from becoming a match-statement monster.

**`scan.rs`**
The flagship command. Orchestrates the full execution pipeline:
1. Resolve the scan path(s)
2. Discover the project structure (workspace or single file)
3. Parse source files via `sentinel-parser`
4. Configure and run `sentinel-rules::RuleEngine`
5. Pass findings to the output formatter
6. Return `ExitCode`

Owns no state except what is passed in. Pure orchestration. Complex enough to warrant its own file, but not complex enough for sub-modules.

**`check.rs`**
A streamlined variant of `scan`. Targets a single file, skips workspace discovery, skips Cargo.toml analysis. Designed for editor integration and pre-commit hooks where latency is critical. Shares the same output pipeline as `scan`.

**`config.rs`**
Configuration loading and merging. Three tiers (highest to lowest precedence):
1. CLI flags (direct command-line arguments)
2. Project config (`sentinel.toml` in project root, or path from `--config`)
3. User config (`~/.config/sentinel/config.toml`)
4. Defaults (hardcoded sensible defaults)

Returns a unified `RunConfig` struct that the rest of the pipeline consumes without knowing the config source. `sentinel.toml` is documented with an annotated example schema published in the repository.

**`output.rs`**
Output abstraction. A trait `OutputFormatter` with implementations:
- `PrettyFormatter` — colored, grouped, human-readable
- `CompactFormatter` — one line per finding, no colors
- `JsonFormatter` — structured JSON, versioned schema
- `SarifFormatter` — SARIF 2.1.0 compliant

The formatter receives findings and writes to a `Writer` (either stdout or a file). Format selection is driven by the `--json`, `--sarif`, `--compact` flags.

**`progress.rs`**
Progress reporting. Abstracts away spinners, progress bars, and status messages. Three strategies:
- `AutoProgress` — detects terminal, renders spinners when interactive, silent otherwise
- `SilentProgress` — no output (for `--quiet` / CI)
- `VerboseProgress` — structured log lines (for `--verbose` / debugging)

Spinners are suppressed when stdout is piped, when `--quiet` is set, or when `--json`/`--sarif` is active.

**`terminal.rs`**
Terminal capability detection. Determines:
- Whether stdout is a terminal (for color/spinner decisions)
- Color support level (no color, ANSI 16, 256, truecolor)
- Terminal width (for line wrapping)
- Unicode support (for choosing between `✓`/`✔` and `[OK]`)
- Respects `NO_COLOR`, `CLICOLOR`, `TERM=dumb`, `FORCE_COLOR`

Does not render anything itself. Provides capability flags to `output.rs` and `progress.rs`.

**`exit.rs`**
Exit code definitions. A minimal enum:
```rust
ExitCode {
    Success,
    FindingsDetected,
    InvalidArguments,
    InvalidConfiguration,
    ParseFailure,
    InternalError,
    PermissionDenied,
    ProjectNotFound,
    UnsupportedProject,
}
```

Each variant maps to a specific i32 constant. The enum implements `Into<i32>` and is the single source of truth for exit behavior. No magic numbers anywhere.

**`diagnostics.rs`**
User-facing diagnostic messages. Handles:
- Warning about deprecated flags
- Encouraging feedback (e.g., "No issues found")
- Error messages with suggestions
- "Did you mean?" for mistyped rule IDs

These are structured messages, not raw strings. Each has a message template, optional suggestion, and optional help URL.

**`paths.rs`**
Path resolution utilities. Provides:
- `canonicalize(path)` — resolve symlinks, normalize
- `discover_project_root(path)` — walk up to find `Cargo.toml` or `sentinel.toml`
- `discover_workspace(path)` — find workspace members from `Cargo.toml`
- `find_source_files(path)` — recursively collect `.rs` files
- `is_hidden(path)` — skip `.git`, `target`, etc.

All paths in output are rendered relative to the project root for readability and reproducibility.

**`errors.rs`**
CLI-layer error types. Wraps errors from:
- `sentinel-core` (CoreError)
- `sentinel-parser` (ParserError)
- `sentinel-rules` (no error type currently — pure diagnostic output)
- I/O operations
- Config parsing
- JSON serialization

Each error variant knows its corresponding exit code, whether it is user-actionable, and whether it should display a full backtrace or a concise message.

---

## 3. Command Hierarchy

### `sentinel scan [path]`

**Purpose** — The primary command. Scans a Soroban smart contract project or individual file for security issues, code quality problems, and gas inefficiencies.

**Arguments**
- `path` — optional. File or directory to scan. Defaults to `.` (current directory).

**Flags**
- `--json` — output findings as JSON
- `--sarif` — output findings as SARIF
- `--compact` — compact one-line-per-finding output
- `--severity <levels>` — filter: `critical,high,medium,low,info`
- `--category <cats>` — filter: `security,gas,performance,best-practice,upgrade`
- `--rule <ids>` — only run specific rules (comma-separated)
- `--exclude <ids>` — skip specific rules
- `--config <path>` — path to config file
- `--workspace` — scan all workspace members
- `--threads <N>` — parallel threads (default: available cores)
- `--quiet` — no output except findings
- `--verbose` — detailed progress and debug info
- `--color <when>` — auto/always/never
- `--fail-on <severity>` — exit code 1 if any finding ≥ this severity
- `--score` — display security score summary
- `--timings` — show per-rule timing breakdown
- `--explain` — attach explanation to each finding (requires sentinel-ai)
- `--output <file>` — write output to file instead of stdout

**Future extensibility** — New flags are added as optional extras. The `--output-version` flag can pin JSON schema version. `--diff` can compare against a baseline. The `path` argument can accept multiple paths in future versions.

### `sentinel check <file>`

**Purpose** — Fast single-file scan. Skips workspace discovery, config loading, and Cargo.toml analysis. Minimal startup overhead (<50ms). Designed for editor save hooks and pre-commit.

**Arguments**
- `file` — required. Path to a single `.rs` file.

**Flags** — subset of `scan`: `--json`, `--sarif`, `--compact`, `--severity`, `--quiet`, `--color`, `--fail-on`

**Behavior** — Parses the single file, runs all applicable rules, outputs findings. No config file read (uses defaults + CLI flags only). Returns within the editor's latency budget.

### `sentinel report`

**Purpose** — View, compare, or manage previous scan reports. Reports are stored as JSON files (default: `.sentinel/reports/`).

**Subcommands**
- `report latest` — show the most recent scan result
- `report list` — list all saved reports
- `report compare <id1> <id2>` — diff two reports
- `report export <id> --format json|sarif` — export a report

**Future** — `report baseline` to track findings over time.

### `sentinel config`

**Purpose** — View and manage configuration.

**Subcommands**
- `config show` — display effective configuration (merged from all sources)
- `config init` — create default `sentinel.toml` in project root
- `config validate` — check a config file for errors
- `config path` — show where config files are looked for

### `sentinel doctor`

**Purpose** — System diagnostics. Check that sentinel is properly installed and configured.

**Checks**
- Sentinel version
- Rust toolchain version
- Project structure (Cargo.toml, soroban deps)
- Config file validity
- Permissions on scan targets
- Terminal capabilities

**Output** — Pass/fail for each check with actionable guidance for failures.

### `sentinel rules`

**Purpose** — List all available rules with metadata.

**Arguments** — optional `<rule-id>` to show detailed info for one rule.

**Flags**
- `--json` — machine-readable listing
- `--severity <filter>` — filter by severity
- `--category <filter>` — filter by category

**Output** — For each rule: ID, name, severity, category, short description, documentation URL.

### `sentinel explain <rule-id>`

**Purpose** — Get a detailed, natural-language explanation of a rule, including examples of vulnerable and fixed code. Uses `sentinel-ai` when available, falls back to built-in documentation.

**Arguments** — `rule-id` is required.

**Flags**
- `--ai` — force AI-powered explanation
- `--local` — use built-in documentation only

**Output** — Markdown rendered to terminal. Rule description, why it matters, vulnerable example, fixed example, reference links.

### `sentinel version`

**Purpose** — Print version information.

**Output** — Version number, commit hash, build date, Rust version used to compile. Follows `cargo` convention. Machine-readable with `--json`.

### `sentinel init`

**Purpose** — Bootstrap a new project with Sentinel configuration.

**Behavior** — Creates a `sentinel.toml` in the current directory. Optionally sets up `.gitignore` entries, CI configuration, and pre-commit hooks. Interactive prompts for key decisions. Non-interactive mode with `--defaults`.

---

## 4. scan Command — Detailed Design

### Argument parsing

The `path` argument is optional and defaults to `"."`. It is resolved as follows:
1. If `path` is a file, scan that single file.
2. If `path` is a directory, discover the project and scan all relevant files.
3. If `path` does not exist, return exit code 7 (ProjectNotFound).

Path resolution is in `paths.rs` and returns a canonical, absolute path for deterministic behavior regardless of the current working directory.

### Configuration precedence

1. CLI flags (highest)
2. Project config (`sentinel.toml` in project root)
3. User config (`~/.config/sentinel/config.toml`)
4. Hardcoded defaults

Merging rules:
- CLI flags override config file values
- Config file values override defaults
- Lists merge by convention: CLI `--rule` replaces config `rules.enable`, CLI `--exclude` replaces config `rules.disable`
- Severity threshold is the most restrictive across sources

### Validation

Before any parsing begins:
1. Config file is validated (exit 3 if invalid)
2. Path is validated (exit 7 if not found)
3. Flags are validated (exit 2 for contradictory flags like `--quiet --verbose`)
4. If `--workspace` is set, a Cargo workspace must exist

### Error reporting

Errors during the scan (parse failures in individual files, permission issues) are collected, not fatal. The scan continues and reports partial results. The exit code reflects whether any findings were detected, but stderr lists the skipped files and reasons.

### Performance

- Path discovery: O(n) where n is the number of files in the project tree (excluding hidden/target dirs)
- Parsing: O(n) on total lines of code. Individual files are parsed independently and could be parallelized.
- Rule execution: Rules iterate the AST. Expensive rules (cross-function analysis) are clearly documented.
- Output formatting: O(f) where f is the number of findings. JSON serialization dominates.

### Future compatibility

The `scan` command is the public face of Sentinel. Its flags and behavior are versioned. Deprecated flags are warned about via `diagnostics.rs` for at least two minor versions before removal. The `--output-version` flag ensures JSON consumers can pin to a known schema.

---

## 5. Execution Pipeline

```
┌─────────────┐
│  CLI Args   │  parsed by clap into CliConfig
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  app::run() │  initializes logging, loads config, dispatches
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  scan::run()│  orchestrates the scan pipeline
└──────┬──────┘
       │
       ▼
┌─────────────────┐
│  Config loading  │  project_config + user_config + cli_overlay → RunConfig
└──────┬──────────┘
       │
       ▼
┌─────────────────┐
│ Path discovery  │  find Cargo.toml, workspace members, source files
└──────┬──────────┘
       │
       ▼
┌─────────────────┐
│  Parsing        │  sentinel-parser::parse_project on each file
└──────┬──────────┘
       │
       ▼
┌─────────────────┐
│  AST            │  ParsedProject (curated AST, decoupled from syn)
└──────┬──────────┘
       │
       ▼
┌─────────────────┐
│  Rule Engine    │  sentinel-rules::RuleEngine::run(&ast)
└──────┬──────────┘
       │
       ▼
┌─────────────────┐
│  Findings       │  Vec<Finding>, filtered and sorted
└──────┬──────────┘
       │
       ▼
┌─────────────────┐
│  Suppression    │  sentinel-rules::SuppressionEngine
└──────┬──────────┘
       │
       ▼
┌─────────────────┐
│  Reporter       │  OutputFormatter::write(&findings, &summary)
└──────┬──────────┘
       │
       ▼
┌─────────────────┐
│  Exit Code      │  findings present → 1, errors → 2-8, otherwise 0
└─────────────────┘
```

### Stage details

**CLI Args → Config Loading**
- `CliConfig` is the parsed CLI surface (flat struct, all optional).
- `RunConfig` is the resolved configuration (all required fields, merged from CLI + files + defaults).
- Separating these types prevents config loading bugs and makes both types testable independently.

**Path Discovery**
- If the input is a file: skip discovery, parse that file.
- If the input is a directory: look for `Cargo.toml` → check for `[workspace]` → discover members → collect all `.rs` files.
- If `--workspace` is set: scan every workspace member and aggregate findings.
- If no Cargo.toml found: scan all `.rs` files recursively as a flat project.

**Parsing**
- Each file parsed independently via `sentinel-parser::parse_project`.
- Parse errors per file are captured in `ParsedFile::parse_error` and reported, but do not abort the scan.
- A `ParsedProject` is constructed by aggregating all successfully parsed files.

**AST → Rule Engine**
- `RuleEngine` is configured with `RuleConfig` (derived from `RunConfig`).
- Rules are filtered based on severity threshold, enabled/disabled lists, and severity overrides.
- Rule execution is currently sequential; the interface supports parallel execution via `Send + Sync` bounds.

**Suppression**
- `SuppressionEngine` filters findings based on inline comments (`// sentinel-ignore`) and file-level comments (`// sentinel-ignore-file`).
- Suppressed findings are counted in the summary but excluded from output.

**Reporter**
- The `OutputFormatter` writes findings to the configured writer.
- If `--output` is set, findings go to a file and stdout gets only the summary.
- If `--json` or `--sarif`, the full structured output goes to stdout (or file).

### Ownership

Each stage owns its data and passes ownership downstream:
1. CLI parser owns `CliConfig` → consumed by config loader
2. Config loader owns `RunConfig` → passed to scanner
3. Scanner owns `ParsedProject` → passed to rule engine (as `&dyn Ast`)
4. Rule engine owns findings → passed to formatter
5. Formatter writes findings → result discarded

No global mutable state. No singletons. No static variables.

### Error propagation

- Fatal errors (invalid config, missing path, I/O permission) are `Err` values that propagate up to `app::run()`.
- Non-fatal errors (single file parse failure, partial I/O) are collected as diagnostics and reported alongside findings.
- The exit code reflects the most severe error encountered.

---

## 6. Exit Codes

| Code | Constant | Meaning | When |
|------|----------|---------|------|
| 0 | `Success` | Clean scan | No findings at or above the severity threshold |
| 1 | `FindingsDetected` | Issues found | At least one finding meets or exceeds `--fail-on` severity (default: any finding) |
| 2 | `InvalidArguments` | Bad CLI input | Unknown flag, missing value, contradictory flags, malformed rule ID |
| 3 | `InvalidConfiguration` | Config error | Malformed TOML, unknown keys, invalid rule IDs in config |
| 4 | `ParseFailure` | Parsing error | Cannot parse any source file (all files fail, not just some) |
| 5 | `InternalError` | Unexpected failure | Panic, assertion failure, internal invariant violation |
| 6 | `PermissionDenied` | Access denied | Cannot read config file, source file, or output path |
| 7 | `ProjectNotFound` | Missing input | Specified path does not exist |
| 8 | `UnsupportedProject` | Wrong project type | Not a Rust/Soroban project, no source files found |

Exit code 0 vs 1 is the fundamental query: "are there issues?" Code 0 guarantees no findings of interest. Code 1 guarantees at least one finding.

Exit codes 2–8 are operational failures. They are mutually exclusive with code 0 or 1 — no findings were produced because the scan did not complete.

Exit code 5 is reserved for true internal errors (panics, assertion failures). It should never occur in production. If it does, it indicates a bug in Sentinel, not in the scanned project.

The exit code table is documented in the README and in `sentinel help exit-codes`.

---

## 7. Output System

### Pretty output (default)

```
╭─ Critical ──────────────────────────────────────────────────╮
│ • auth-mistake          contract.rs:42:5                   │
│   Operation after the last require_auth call may be         │
│   unauthorized.                                             │
│   → Add require_auth after the last authorized block        │
├─────────────────────────────────────────────────────────────┤
│ • missing-require-auth   token.rs:18:1                      │
│   Public function 'burn' does not call require_auth.        │
│   → Add require_auth!() at the start of this function       │
├─────────────────────────────────────────────────────────────┤
│ • contract-upgrade       upgrade.rs:7:1                     │
│   Deployer/upgrade call without authorization.              │
│   → Add require_auth!() before the deployer/upgrade call    │
╰─────────────────────────────────────────────────────────────╯

╭─ Security Score ────────────────────────────────────────────╮
│  ████████████░░░░░░░░  60/100  (3 critical, 1 high)       │
╰─────────────────────────────────────────────────────────────╯

╭─ Summary ───────────────────────────────────────────────────╮
│  Files scanned:     14                                      │
│  Rules run:         10                                      │
│  Findings:           5  (3 critical, 1 high, 1 medium)     │
│  Suppressed:         1                                      │
│  Duration:        0.213s                                    │
╰─────────────────────────────────────────────────────────────╯
```

Design choices:
- Severity groups are ordered from most to least severe (critical → info)
- Within a group, findings are ordered by file path, then line, then column
- Each finding shows: rule ID (truncated to fit), location, message, recommendation
- Color: critical=red, high=yellow, medium=blue, low=cyan, info=white
- Box-drawing characters use Unicode by default, fall back to ASCII when Unicode is unavailable
- Suppressed findings are counted but not displayed unless `--verbose`

### Compact output

```
critical  auth-mistake          contract.rs:42:5  Operation after the last require_auth call...
critical  missing-require-auth  token.rs:18:1     Public function 'burn' does not call...
high      contract-upgrade      upgrade.rs:7:1    Deployer/upgrade call without authorization...
```

Design choices:
- One line per finding
- Tab-separated fields: severity, rule-id, location, message (truncated to terminal width)
- No colors, no box drawing, no summary
- Ideal for `grep` / `awk` pipelines
- Messages truncated to avoid line wrapping in terminal

### JSON output

```json
{
  "$schema": "https://sentinel.dev/schemas/output/v1.json",
  "version": "1.0.0",
  "sentinel_version": "0.1.0",
  "started_at": "2026-07-10T10:30:00Z",
  "duration_ms": 213,
  "project": {
    "root": "/home/user/project",
    "files_scanned": 14,
    "manifest": "Cargo.toml"
  },
  "summary": {
    "total_findings": 5,
    "suppressed": 1,
    "rules_run": 10,
    "score": 60
  },
  "findings": [
    {
      "rule_id": "auth-mistake",
      "severity": "critical",
      "category": "security",
      "message": "Operation after the last require_auth call may be unauthorized",
      "recommendation": "Add require_auth after the last authorized block",
      "location": {
        "file": "contracts/token/src/contract.rs",
        "line": 42,
        "column": 5
      },
      "fix_example": null
    }
  ]
}
```

Design choices:
- Versioned schema (`$schema` URL)
- Timestamps in ISO 8601
- All paths relative to project root (reproducible across machines)
- Null fields for optional values (not omission)
- Findings sorted deterministically
- The schema is published at a stable URL and versioned with the tool

### SARIF output

Compliant with SARIF 2.1.0 (OASIS Standard). This enables:
- GitHub code scanning integration
- VS Code problem matchers
- GitLab SAST
- Any SARIF-compatible tooling

SARIF output includes:
- Rule metadata (id, name, description, documentation URL)
- Driver info (sentinel version)
- Results with locations, messages, and levels
- Invocation info (start time, duration)

### Formatting philosophy

- The `OutputFormatter` trait has exactly one method: `fn write(&self, findings: &[Finding], summary: &ScanSummary) -> Result<(), OutputError>`.
- Formatters are selected by the output module based on CLI flags: `--json` → `JsonFormatter`, `--sarif` → `SarifFormatter`, `--compact` → `CompactFormatter`, default → `PrettyFormatter`.
- Custom formatters can be added without modifying any existing formatter.
- The summary object is computed once by the scan pipeline and shared with all formatters.

---

## 8. Integration

### sentinel-core

Sentinel-cli depends on sentinel-core for the fundamental types:
- `Finding`, `Severity`, `Category`, `DiagnosticSpan`, `RuleId` — these appear in the output
- `RuleRegistry` — used to register built-in rules
- `Ast` trait — used as the interface to `ParsedProject`
- `SecurityScore` — computed from findings for the summary

The dependency is one-way: sentinel-core knows nothing about the CLI. This is non-negotiable for maintaining the core as a reusable library.

### sentinel-parser

Sentinel-cli depends on sentinel-parser for:
- `parse_project(path)` — the primary entry point
- `ParsedProject` — the curated AST that implements `Ast`

The parser is invoked from `scan.rs` and `check.rs`. It receives a path and returns a `ParsedProject` or a `ParserError`.

### sentinel-rules

Sentinel-cli depends on sentinel-rules for:
- `RuleEngine` — the orchestrator
- `RuleConfig` — configuration for which rules to run
- `SuppressionEngine` — inline suppression handling
- `register_all()` — to populate the rule registry

The rule engine is configured with a `RuleConfig` (derived from CLI + config file) and invoked with the `ParsedProject` as `&dyn Ast`.

### sentinel-utils

Sentinel-cli depends on sentinel-utils for:
- `init_logging()` — consistent tracing/logging setup across all sentinel crates

### Future crate integration

**sentinel-config** (planned)
- Currently, config loading is implemented directly in `config.rs`.
- When `sentinel-config` is created, the CLI will delegate config parsing and merging to that crate.
- The `config.rs` module will shrink to a thin adapter, calling `sentinel-config` and converting its types to `RunConfig`.
- No circular dependency: sentinel-config depends on sentinel-core but not on sentinel-cli.

**sentinel-report** (planned)
- Currently, report storage/retrieval is embedded in `sentinel report`.
- A future `sentinel-report` crate will handle the report file format, storage, listing, and comparison.
- The CLI will delegate to sentinel-report for all report subcommands.
- sentinel-report depends on sentinel-core but not on sentinel-cli.

**sentinel-ai** (planned)
- The `sentinel explain` command uses sentinel-ai when available.
- sentinel-ai is loaded lazily — if the crate is not installed, the explain command uses built-in documentation.
- The CLI communicates with sentinel-ai through a trait to keep the dependency optional.

### Ownership boundaries

```
sentinel-cli ──depends on──► sentinel-core
sentinel-cli ──depends on──► sentinel-parser
sentinel-cli ──depends on──► sentinel-rules
sentinel-cli ──depends on──► sentinel-utils

sentinel-cli ──will depend on──► sentinel-config (future)
sentinel-cli ──will depend on──► sentinel-report (future)
sentinel-cli ──will optionally depend on──► sentinel-ai (future)
```

No circular dependencies. No crate below sentinel-cli depends on sentinel-cli. All data flows from outer (CLI) to inner (core/parser/rules).

---

## 9. Performance

### Startup latency

Target: <50ms cold start, <10ms warm start (with cached filesystem metadata).

- Configuration loading: <5ms
- Path discovery: <10ms for small projects (up to 100 files), scales linearly
- CLI parsing: <1ms (clap is fast)
- Dynamic linking: ~20-30ms (Rust startup overhead)

### Memory usage

Target: <50MB for typical projects, <200MB for large workspace scans.

- The `ParsedProject` AST is 10-20x smaller than the raw `syn` AST (by design)
- Findings are streamed to the formatter, not accumulated (except for sorting)
- The sorted output buffer is the only O(f) memory cost, where f is finding count (typically <1000)

### Large repositories

For repositories with 10,000+ source files:
- Path discovery uses `walkdir` with `into_iter()` to avoid building a full list in memory when possible
- Parsing is parallelizable per-file (future enhancement)
- Rule execution is parallelizable per-rule (the `Rule` trait is `Send + Sync`)
- The AST for each file is dropped after rule execution completes

### Workspace scanning

- Each workspace member is scanned independently
- Findings are aggregated after all members are scanned
- Workspace members are scanned sequentially by default, with a `--threads` flag for parallelism
- Thread count is not an "N workers" pool but a "N concurrent members" model

### Streaming diagnostics

- Findings can be written incrementally as they are produced
- The `OutputFormatter::write` method receives a complete slice, but formatters can implement incremental writing internally
- For JSON output, the formatter writes `[` → (finding JSON + `,`) × N → `]`

### Incremental output

- `--output` sends findings to a file while summary goes to stdout
- This allows `sentinel scan > findings.json` to work naturally
- The summary (score, counts) is computed from the final finding count

### Parallel execution

The architecture supports parallel execution at multiple levels:
1. **Per-file parsing** — independent files can be parsed in parallel (IO-bound, then CPU-bound)
2. **Per-rule execution** — rules can run in parallel against the same AST (CPU-bound)
3. **Per-member workspace scanning** — workspace members are independent

However, parallel execution is deferred to a future optimization pass. The initial implementation is sequential to ensure correctness and determinism. The interfaces (`Send + Sync`, `Arc<dyn Ast>`) are designed for parallelism from day one.

### Future caching

A cache directory (`.sentinel/cache/`) could store:
- Parsed AST summaries (for unchanged files → skip re-parsing)
- Rule check results (for unchanged ASTs → skip re-checking)

This is a future enhancement. The initial implementation is always-fresh.

---

## 10. Error Model

### Error classification

| Category | Recoverable? | Examples |
|----------|-------------|---------|
| Invalid arguments | Yes | Unknown flag, missing value, contradictory options |
| Invalid configuration | Yes | Malformed TOML, invalid rule IDs in config |
| File I/O | Partial | Permission denied on one file, missing file |
| Parse error | Partial | Syntax error in one file (other files still parsed) |
| Rule engine error | No (bug) | Rule panics, invariant violation |
| Output error | Per-call | Write failure (disk full), serialization failure |

### Recoverable errors

Recoverable errors are those where the scan can continue with partial results:
- A single file has a syntax error → skip that file, report the error, continue with remaining files
- A single file is unreadable → same behavior
- A config file has unknown keys → warn and continue

Recoverable errors are collected in a `Vec<CliDiagnostic>` and reported after the scan completes.

### Unrecoverable errors

Unrecoverable errors abort the scan immediately:
- Invalid CLI arguments (cannot proceed without understanding intent)
- All source files fail to parse (nothing to scan)
- Internal invariant violation (bug — abort to prevent incorrect results)
- Disk-full writing output (cannot report findings)
- Memory allocation failure (system constraint)

Unrecoverable errors are returned as `Err(CliError)` from the pipeline and displayed by `app::run()`.

### Error display

- CLI argument errors: concise message with `--help` suggestion
- Configuration errors: the specific TOML location and a hint
- I/O errors: the path and the specific OS error
- Parse errors: the file, line, and the parser error message
- Internal errors: "This is a bug. Please report at <url>" with backtrace in verbose mode

Errors are written to stderr. Stdout contains only the requested output format (or nothing on errors).

---

## 11. Testing Strategy

### Unit tests

Every module has unit tests for its public functions:
- `config.rs`: test config loading from TOML strings, merging precedence, default generation
- `paths.rs`: test canonicalization, workspace discovery, source file collection
- `exit.rs`: test that each variant maps to the correct i32
- `diagnostics.rs`: test message formatting
- `errors.rs`: test error wrapping and display

### Integration tests

Integration tests run sentinel as a subprocess with real project fixtures:

**`tests/integration/scan_basic.rs`**
- `sentinel scan` on valid contract → exit 0 or 1
- `sentinel scan` on vulnerable contract → exit 1
- `sentinel scan` on non-existent path → exit 7
- `sentinel scan --fail-on critical` → exit 1 only for critical findings

**`tests/integration/scan_json.rs`**
- `sentinel scan --json` → valid JSON, schema-conformant
- `sentinel scan --json --output /tmp/out.json` → file contains valid JSON
- Extract finding count from JSON and verify against known fixture

**`tests/integration/scan_sarif.rs`**
- `sentinel scan --sarif` → valid SARIF 2.1.0
- Validate against SARIF JSON schema

**`tests/integration/config_loading.rs`**
- Test with `sentinel.toml` in project root
- Test with `--config path/to/config.toml`
- Test with invalid config → exit 3

**`tests/integration/exit_codes.rs`**
- Test every exit code with appropriate input
- Test that exit code 0 never has findings at the threshold
- Test `--fail-on` threshold changes

**`tests/integration/error_handling.rs`**
- Permission denied on config file
- Permission denied on source file
- Broken symlink in project
- Mixed valid/invalid files (scan continues)

### Golden snapshot tests

Snapshot tests for output formatting:
- Run `sentinel scan` on a fixed fixture project
- Compare stdout against a committed golden file in `tests/snapshots/`
- Separate snapshots for pretty, compact, JSON, and SARIF output
- Snapshots are reviewed and updated on intentional output changes

### JSON validation

- All JSON output is validated against the published schema
- The schema itself is tested for correctness (valid JSON Schema document)
- Schema tests ensure backward compatibility (new fields are optional)

### SARIF validation

- SARIF output is validated against the official SARIF 2.1.0 schema
- Edge cases: zero findings, maximum findings, all severity levels

### Cross-platform tests

- Path normalization (forward vs backward slashes)
- Line ending handling (`\n` vs `\r\n`)
- Color detection (NO_COLOR, CLICOLOR, TERM=dumb)
- Unicode fallback (box-drawing characters vs ASCII)

### Large repository benchmarks

- Create synthetic projects with 100, 1000, 10000 files
- Measure startup latency, parsing time, rule execution time
- Track memory usage
- These are benchmarks, not pass/fail tests — tracked in CI for regression

### Failure scenario tests

- Empty project (no `.rs` files) → exit 8
- Binary-only project (no lib) → succeeds
- Panic in a rule → caught, reported as bug, other rules continue
- Ctrl+C during scan → graceful shutdown, partial output on stderr

---

## 12. Future Features

### sentinel fix

A future `sentinel fix` command will auto-apply fixes for certain rules (e.g., adding `require_auth!()` to public functions). The CLI design supports this by:
- Rules optionally expose a `fix` method that returns a replacement span
- The output already includes `fix_example` fields
- The `fix` command will reuse the scan pipeline, then apply fixes to files

### sentinel explain

The `sentinel explain <rule-id>` command is already described in the command hierarchy. It reuses the rule metadata from `sentinel-rules` and the AI module from `sentinel-ai`. The command was designed in the initial hierarchy, so no new flags or arguments need to be added later.

### sentinel update

Future command to update built-in rules or download community rules. This requires:
- A rule registry/repository (not yet designed)
- Network access (currently none)
- The command is additive and affects only the `rules` subcommand group

### sentinel benchmark

Performance benchmarking command. Runs the scan pipeline multiple times and reports timing statistics. Reuses the scan pipeline with a `--bench` flag that enables additional instrumentation. No architecture changes needed.

### sentinel doctor

Described in the command hierarchy. Checks system prerequisites. Requires no changes to the core pipeline.

### Plugin manager

A plugin system would allow loading external rules as dynamic libraries. The CLI would support:
- `sentinel plugin install <name>`
- `sentinel plugin list`
- `sentinel plugin remove <name>`

This is enabled by the `Rule` trait being object-safe and the `RuleRegistry` supporting external registration.

### LSP bridge

A Language Server Protocol integration. The CLI would be invoked by an LSP server that wraps Sentinel. The `scan` and `check` commands provide the analysis results. A new `sentinel lsp` command would start the LSP server, reusing the existing analysis pipeline.

### Watch mode

`sentinel scan --watch` would:
- Scan the project immediately
- Use `notify` (file watcher) to detect changes
- Re-scan changed files incrementally
- Display a live-updating report in the terminal

No major architecture changes needed. The scan pipeline is already stateless and deterministic. Watch mode just calls it repeatedly on change events.

### Git hooks

`sentinel init --pre-commit` creates a `.git/hooks/pre-commit` script that:
- Runs `sentinel check` on staged `.rs` files
- Blocks the commit if any findings exceed the configured threshold
- Uses the `--quiet --fail-on` flags for script-friendly output

No CLI changes needed — existing `check` command handles this.

### CI integration

GitHub Action, GitLab CI, and CircleCI integrations use the existing `scan` command with `--json` or `--sarif` output. The integration is a shell script or Docker image, not a CLI change.

---

## 13. Tradeoff Analysis

### CLI framework: clap vs. hand-rolled parser

| Aspect | clap | Hand-rolled |
|--------|------|-------------|
| **Chosen** | ✓ | |
| **Pros** | Derive macros eliminate boilerplate; stable API; automatic `--help` generation; bash/zsh/fish completions; battle-tested at scale (cargo, rustc use clap) | |
| **Cons** | Dependency weight (~10 crates); compile time; occasional version churn | Total control; minimal deps; no version churn |
| **Why chosen** | The maintenance burden of a hand-rolled parser far outweighs the dependency cost. clap's derive API is mature, stable, and produces correct behavior for edge cases (quoted args, `=` syntax, `--` terminator). | |
| **Long-term** | clap 4.x has a stability commitment. If clap is ever abandoned, `gumdrop` or a hand-rolled parser are viable fallbacks — the argument parsing is isolated to `commands.rs`. | |

### Output abstraction: trait vs. enum dispatch

| Aspect | Trait (`OutputFormatter`) | Enum (`OutputFormat::Pretty(...)`) |
|--------|--------------------------|-----------------------------------|
| **Chosen** | ✓ | |
| **Pros** | New formatters don't require modifying existing code; formatters can be tested independently; formatters can have different internal states | Simpler dispatch; no dynamic dispatch; easy to serialize in config |
| **Cons** | Dynamic dispatch overhead (microseconds); trait object limitations | Matching all variants at every call site violates OCP |
| **Why chosen** | Output format extension (SARIF, HTML, etc.) is a likely future need. The trait approach lets contributors add formatters without touching a central enum. The performance difference is immeasurable at human time scales. | |
| **Long-term** | An enum wrapper (`OutputFormat::Json(JsonFormatter)`) combines the best of both: enum-level dispatch with trait-level extensibility. | |

### Configuration precedence: three-tier vs. flat

| Aspect | Three-tier (flag > file > default) | Flat (config file only) |
|--------|-----------------------------------|------------------------|
| **Chosen** | ✓ | |
| **Pros** | CLIs must respect flag overrides (UNIX convention); per-project config enables team consistency; user config enables personal preferences | Simpler to implement; fewer merge edge cases |
| **Cons** | Merge logic is non-trivial; precedence rules must be documented | Users cannot override config per-invocation; team and personal preferences conflict |
| **Why chosen** | cargo, clippy, and eslint all use multi-tier precedence. It is the established CLI convention. An explicit `--config` flag gives users full control over which config file is used. | |
| **Long-term** | The three-tier model is stable and well-understood. A future `--no-user-config` flag could skip the user tier for CI environments. | |

### Progress reporting: synchronous vs. separate thread

| Aspect | Synchronous (main thread) | Separate thread (spinner) |
|--------|--------------------------|--------------------------|
| **Chosen** | ✓ | |
| **Pros** | Simple; no thread-safety concerns; no cleanup on abort | Spinner animates while work happens |
| **Cons** | No animation; "scanning..." message blocks until complete | Thread lifecycle management; signal handling complexity |
| **Why chosen** | Sentinel scans should be fast enough (<1s typical) that a spinner is visual noise, not value. For large scans, the progress output is a log line per file, not a spinner. The complexity of a spinner thread is not justified. | |
| **Long-term** | A `--progress` flag could enable the thread-based spinner for large projects. The `Progress` abstraction makes this easy to add. | |

### Streaming output: incremental vs. buffered

| Aspect | Incremental (write findings as produced) | Buffered (collect, sort, write) |
|--------|----------------------------------------|--------------------------------|
| **Chosen** | | ✓ |
| **Pros** | O(1) memory; progressive output; works with pipes | Deterministic ordering; simpler formatters; can deduplicate |
| **Cons** | Cannot sort findings (non-deterministic order); cannot deduplicate; complex JSON streaming | O(f) memory for finding buffer |
| **Why chosen** | Deterministic ordering is a non-negotiable design goal. Finding counts are typically <1000 even for large projects, so the memory cost of buffering is trivial. | |
| **Long-term** | A `--stream` flag could enable incremental output for the compact formatter, which does not require sorting. | |

### Exit code model: findings vs. error dominance

| Aspect | Findings dominate (1 = findings) | Error dominates (non-zero for any problem) |
|--------|----------------------------------|-------------------------------------------|
| **Chosen** | ✓ | |
| **Pros** | Matches linter convention (clippy, ruff); CI can `exit 0` for clean, `exit 1` for findings; script-friendly | Simpler; single non-zero = something wrong |
| **Cons** | Mixed semantics for 1 (findings or error?) | Scripts cannot distinguish "no findings" from "couldn't run" |
| **Why chosen** | The most common CI use case is: "are there findings?" A single exit code answers that. Errors are a separate concern with their own codes (2-8). | |
| **Long-term** | The `--fail-on` flag lets users control what counts as "findings detected." A future `--warnings-as-errors` flag could upgrade low-severity findings to exit code 1. | |

### Path handling: relative vs. absolute in output

| Aspect | Relative paths | Absolute paths |
|--------|---------------|----------------|
| **Chosen** | ✓ | |
| **Pros** | Reproducible across machines; short; human-readable; no leaked user paths | Unambiguous; no path resolution needed |
| **Cons** | Requires project root for resolution | Leaks user home directories; different on every machine |
| **Why chosen** | Reproducibility in CI environments (Docker, GitHub Actions) is more important than absolute precision. The project root is always known. | |
| **Long-term** | A `--absolute-paths` flag could opt in to absolute paths for editor integration. | |

---

## 14. Conventional Commit

```
feat(sentinel-cli): add CLI crate with scan pipeline and output system

- Design CLI architecture with stable human-first and script-friendly UX
- Implement 8 exit codes with deterministic semantics
- Build multi-tier configuration loading with flag > file > default
- Create output abstraction supporting Pretty, Compact, JSON, and SARIF
- Wire full scan pipeline: path discovery → parsing → rule engine → output
- Add 9 CLI commands: scan, check, report, config, doctor, rules, explain, version, init
- Integrate with sentinel-core, sentinel-parser, sentinel-rules, sentinel-utils
- Design for future extensibility: fix, watch, LSP, plugins, caching
- Target: <50ms cold start, deterministic output, 124 existing tests remain passing
```

Closes: #6 (Phase 6 implementation)
