# Phase 4: `sentinel-rules` — Architecture & Implementation Plan

## 1. Overall Architecture

The rules crate is the heart of Sentinel's analysis capability. It houses the rule engine (runner, context, filtering, suppression), the diagnostic system, and all 10 built-in rules. It depends on `sentinel-core` (types, traits) and `sentinel-parser` (curated AST).

### Crate Dependency Position

```
sentinel-core  ←  sentinel-parser  ←  sentinel-rules  →  sentinel-report
                                                           sentinel-cli
```

sentry-rules is a **library crate** — it produces `RuleResult` that the CLI and report crates consume. It never owns the entry point.

### Module Map

```
sentinel-rules/
├── Cargo.toml
└── src/
    ├── lib.rs                # Re-exports, public API facade
    ├── registry.rs           # RuleRegistry extension: bulk registration, listing, metadata queries
    ├── runner.rs             # RuleRunner: orchestrates rule execution (single-threaded MVP)
    ├── context.rs            # RuleContext: AST + config + suppression state for a single run
    ├── metadata.rs           # RuleMetadata: description, tags, doc URL, CWE, version, since
    ├── config.rs             # RuleConfig: per-rule overrides, severity bump, enable/disable
    ├── filter.rs             # RuleFilter: config-driven rule selection (by ID, severity, category)
    ├── suppression.rs        # SuppressionEngine: inline, file-level, and config suppression
    ├── diagnostic.rs         # Diagnostic: rich rule output (multi-span, CWE, notes, fix, confidence)
    ├── result.rs             # RuleResult, ExecutionSummary
    └── builtin/
        ├── mod.rs            # register_all(): registers every built-in rule into a RuleRegistry
        ├── missing_auth.rs   # S-001: Missing require_auth()
        ├── unsafe_panic.rs   # S-002: Unsafe panic!/unwrap/expect
        ├── large_storage.rs  # S-003: Large storage writes
        ├── dead_code.rs      # S-004: Dead code / unused functions
        ├── unused_storage.rs # S-005: Storage writes that are never read
        ├── missing_ttl.rs    # S-006: Missing TTL extension on persistent storage
        ├── auth_mistake.rs   # S-007: Authorization mistakes
        ├── integer_overflow.rs # S-008: Integer overflow risks
        ├── gas_optimization.rs # S-009: Gas optimization opportunities
        └── contract_upgrade.rs # S-010: Contract upgrade risks
```

### Responsibility of Each File

| File | Visibility | Responsibility |
|------|-----------|----------------|
| `lib.rs` | `pub` | Re-exports key types; provides `RuleEngine` facade |
| `registry.rs` | `pub` | `RuleRegistryExt` trait with bulk ops; `register_builtins()` |
| `runner.rs` | `pub` | `RuleRunner`: takes registry + AST + config → `RuleResult` |
| `context.rs` | `pub(crate)` | `RuleContext`: assembled per-run, holds AST reference, config overrides, suppress state |
| `metadata.rs` | `pub` | `RuleMetadata`: static rule info (used by `sentinel rules` CLI command) |
| `config.rs` | `pub` | `RuleConfig`: deserialized per-rule config from `sentinel.toml` |
| `filter.rs` | `pub(crate)` | `RuleFilter`: applies config rules → ordered vec of enabled rules |
| `suppression.rs` | `pub` | `SuppressionEngine`: checks if a span is suppressed |
| `diagnostic.rs` | `pub` | `Diagnostic`: rich output type that converts into `sentinel_core::Finding` |
| `result.rs` | `pub` | `RuleResult`: aggregated findings + execution summary + security score |
| `builtin/mod.rs` | `pub(crate)` | `register_all()` — one-stop registration |

### Non-goals (Phase 4)

- Parallel execution (planned Phase 10)
- WASM / plugin rules (post-MVP)
- Auto-fix (future feature)
- LSP integration (future feature)

---

## 2. Rule Lifecycle

### Complete Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                      sentinel scan                              │
│  CLI parses args, loads config, calls rules crate               │
└──────────────────────────┬──────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│  sentinel-parser                                                │
│  parse_project(path) → ParsedProject (implements Ast trait)     │
└──────────────────────────┬──────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│  RuleEngine::new(config)                                        │
│  ├── Create RuleRunner from registry + config                   │
│  └── Register built-in rules (unless disabled)                  │
└──────────────────────────┬──────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│  RuleRunner::run(&self, ast: &dyn Ast) → RuleResult             │
│                                                                  │
│  Step 1: Build RuleContext                                       │
│  │  ├── Wrap AST reference                                      │
│  │  ├── Merge global config → per-rule overrides                │
│  │  └── Initialize SuppressionEngine                            │
│  │                                                              │
│  Step 2: Filter rules                                           │
│  │  ├── Start with all registered rules                         │
│  │  ├── Remove disabled rules                                   │
│  │  ├── Remove rules below severity threshold                   │
│  │  ├── Apply severity overrides                                │
│  │  └── Sort by RuleId (deterministic)                          │
│  │                                                              │
│  Step 3: Execute each rule (sequential)                         │
│  │  ├── rule.check(ast) → Vec<Finding>                          │
│  │  ├── Check each finding against suppression                  │
│  │  ├── Remove suppressed findings                              │
│  │  └── Collect into Vec<Diagnostic> (via Diagnostic::from)     │
│  │                                                              │
│  Step 4: Post-process                                           │
│  │  ├── Deduplicate findings (same rule + same span)            │
│  │  ├── Sort findings (severity desc → file → line → col)       │
│  │  └── Compute security score                                  │
│  │                                                              │
│  Step 5: Return RuleResult                                      │
│     ├── findings: Vec<Finding>                                  │
│     ├── score: SecurityScore                                    │
│     └── summary: ExecutionSummary                               │
└──────────────────────────┬──────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│  sentinel-report / sentinel-cli                                 │
│  Display or serialize RuleResult                                │
└─────────────────────────────────────────────────────────────────┘
```

### Ownership & Borrowing

```
RuleRegistry
  │
  │  owned (holds Box<dyn Rule>)
  │       registry rules are borrowed (&dyn Rule) during execution
  ▼
RuleRunner
  │
  │  borrows &RuleRegistry
  │  borrows &dyn Ast (from parser)
  │  borrows &RuleConfig (from CLI)
  ▼
RuleContext (temporary, per-run)
  │
  │  borrowed: &dyn Ast, &SuppressionEngine
  │  owned:   severity overrides map
  ▼
Rule::check(&self, ast: &dyn Ast) → Vec<Finding>
  │
  │  Rule is &self (stateless)
  │  Ast is &dyn Ast (shared, immutable)
  ▼
Vec<Finding> → Vec<Diagnostic> → RuleResult (fully owned output)
```

### Thread Safety

- `Rule` trait is `Send + Sync` (designed in Phase 2)
- `RuleRegistry` is `Send + Sync` (holds `Send + Sync` rules)
- `RuleRunner` is `Send + Sync` (no mutable shared state)
- `RuleContext` is `Send + Sync`
- `ParsedProject` is `Send + Sync`
- The runner creates per-rule contexts; no data races possible

**Result:** The entire execution pipeline is thread-safe by construction. Parallel execution in Phase 10 only requires adding `par_iter()` — no type changes.

---

## 3. Public Types

### 3.1 `RuleEngine` (`lib.rs`)

The top-level facade for downstream consumers (CLI, report).

```rust
pub struct RuleEngine {
    runner: RuleRunner,
    config: RuleConfig,
    builtins_registered: bool,
}
```

**Public API:**

```rust
impl RuleEngine {
    /// Create a new engine from CLI config. Lazily registers built-ins.
    pub fn new(config: RuleConfig) -> Self;

    /// Register an additional (non-builtin) rule.
    pub fn register(&mut self, rule: Box<dyn Rule>) -> Result<(), CoreError>;

    /// Run analysis on a parsed project.
    pub fn run(&self, ast: &dyn Ast) -> RuleResult;

    /// List all enabled rules with their metadata.
    pub fn list_rules(&self) -> Vec<&dyn RuleMetaProvider>;
}
```

**Trait derivations:** `Debug`

**Design rationale:** `RuleEngine` is the single entry point. Consumers never touch `RuleRunner` or `RuleRegistry` directly. This hides internal complexity and makes future refactoring (e.g., adding parallel execution) transparent.

---

### 3.2 `RuleContext` (`context.rs`)

Per-scan immutable context assembled by the runner.

```rust
pub(crate) struct RuleContext<'ast> {
    pub ast: &'ast dyn Ast,
    pub suppression: SuppressionEngine,
    pub severity_overrides: HashMap<RuleId, Severity>,
}
```

**Why `pub(crate)`:** The context is an internal implementation detail. Rules don't receive it directly — they receive `&dyn Ast` per the Rule trait. The runner uses the context internally for suppression checks and severity adjustments.

**Design rationale:** Keeps the Rule trait interface stable. All context is passed through the runner, not through rules.

**Alternatives rejected:**
- Passing `&RuleContext` to rules: Violates the Rule trait signature (which takes `&dyn Ast`), couples rules to runner internals.

---

### 3.3 `RuleMetadata` (`metadata.rs`)

Static, compile-time metadata associated with every rule. Used by the `sentinel rules` CLI command and report generation.

```rust
pub struct RuleMetadata {
    pub id: RuleId,
    pub name: &'static str,
    pub description: &'static str,
    pub severity: Severity,
    pub category: Category,
    pub tags: &'static [&'static str],        // e.g., ["security", "soroban", "authorization"]
    pub documentation_url: Option<&'static str>,
    pub cwe_id: Option<&'static str>,         // e.g., "CWE-862"
    pub confidence: Confidence,
    pub since_version: &'static str,          // e.g., "0.1.0"
}
```

```rust
pub enum Confidence {
    High,    // Exact pattern match; no false positives expected
    Medium,  // Heuristic; may flag benign code in edge cases
    Low,     // Speculative; designed for discovery, may be noisy
}
```

**Trait derivations:** `Debug, Clone, PartialEq, Eq`

**Trait for rules to implement:**

```rust
pub trait RuleMetaProvider {
    fn metadata(&self) -> RuleMetadata;
}
```

**Design rationale:** Separates static metadata from the Rule trait. The Rule trait stays focused on analysis. The metadata trait is optional for logging/listing purposes.

**Alternatives rejected:**
- Embedding metadata in the Rule trait: Adds more required methods to every rule, clutters the core trait.
- Using `const` associated values: Can't be accessed through `&dyn Rule`.

---

### 3.4 `RuleConfig` (`config.rs`)

Deserialized from `sentinel.toml` `[rules]` and `[rules.*]` sections.

```rust
pub struct RuleConfig {
    /// Global severity threshold — findings below this are excluded.
    pub severity_threshold: Severity,

    /// Explicitly enabled rule IDs (takes priority over disabled).
    pub enabled: Vec<RuleId>,

    /// Explicitly disabled rule IDs.
    pub disabled: Vec<RuleId>,

    /// Per-rule severity overrides.
    pub severity_overrides: HashMap<RuleId, Severity>,

    /// Glob patterns for files to ignore during scanning.
    pub ignore_paths: Vec<String>,
}
```

**Default:**

```rust
impl Default for RuleConfig {
    fn default() -> Self {
        Self {
            severity_threshold: Severity::Low,
            enabled: vec![],        // all rules enabled
            disabled: vec![],
            severity_overrides: HashMap::new(),
            ignore_paths: vec![],
        }
    }
}
```

**Merging logic (CLI overrides config file):**

```rust
impl RuleConfig {
    /// Merge CLI overrides on top of file config.
    pub fn merge(mut self, cli_overrides: RuleConfig) -> Self;
}
```

**Design rationale:** Config is pure data, no behavior. Serialized from TOML. Merging supports CLI flags overriding sentinel.toml values (e.g., `--severity critical` overrides file config).

---

### 3.5 `RuleRegistryExt` (`registry.rs`)

Extension trait adding built-in registration and querying to `sentinel_core::RuleRegistry`.

```rust
pub trait RuleRegistryExt {
    /// Register all built-in rules.
    fn register_builtins(&mut self) -> Result<(), CoreError>;

    /// Register a rule with its metadata.
    fn register_with_metadata(
        &mut self,
        rule: Box<dyn Rule>,
        metadata: RuleMetadata,
    ) -> Result<(), CoreError>;

    /// Get metadata for a registered rule.
    fn metadata(&self, id: &RuleId) -> Option<&RuleMetadata>;
}
```

**Design rationale:** Uses an extension trait (not inheritance) to add methods to the core `RuleRegistry`. The core registry stays minimal; the rules crate adds domain-specific ops.

---

### 3.6 `RuleRunner` (`runner.rs`)

The orchestrator. Stateless — can be reused across scans.

```rust
pub struct RuleRunner {
    registry: RuleRegistry,
    config: RuleConfig,
    metadata_registry: HashMap<RuleId, RuleMetadata>,
}
```

**Public API:**

```rust
impl RuleRunner {
    /// Create a new runner with a registry and config.
    pub fn new(registry: RuleRegistry, config: RuleConfig) -> Self;

    /// Execute all enabled rules against the AST.
    pub fn run(&self, ast: &dyn Ast) -> RuleResult;

    /// Get the list of enabled rules (after filtering).
    pub fn enabled_rules(&self) -> Vec<&dyn Rule>;

    /// Number of registered rules.
    pub fn rule_count(&self) -> usize;
}
```

**Internal flow:**

```rust
fn run(&self, ast: &dyn Ast) -> RuleResult {
    let filter = RuleFilter::new(&self.config, &self.registry);
    let rules: Vec<&dyn Rule> = filter.apply();

    let mut all_findings = Vec::new();
    let suppression = SuppressionEngine::new(&self.config);

    for rule in &rules {
        let findings = rule.check(ast);
        // Apply severity overrides
        let overrides = &self.config.severity_overrides;
        let findings: Vec<Finding> = findings
            .into_iter()
            .map(|f| {
                if let Some(sev) = overrides.get(&f.rule_id) {
                    Finding { severity: *sev, ..f }
                } else { f }
            })
            .filter(|f| !suppression.is_suppressed(f))
            .collect();
        all_findings.extend(findings);
    }

    // Deduplicate
    all_findings.sort();
    all_findings.dedup();

    let score = SecurityScore::from_findings(&all_findings);
    let summary = ExecutionSummary { total_rules: rules.len(), total_files: ast.files().len(), .. };

    RuleResult { findings: all_findings, score, summary }
}
```

**Performance note:** `Rule::check` is called sequentially. Each rule re-walks the AST. For MVP this is acceptable (10 rules × ~1ms per rule = ~10ms). Phase 10 parallelizes this.

---

### 3.7 `RuleFilter` (`filter.rs`)

Takes config + registry → ordered list of rules to execute.

```rust
pub(crate) struct RuleFilter<'a> {
    registry: &'a RuleRegistry,
    config: &'a RuleConfig,
}

impl<'a> RuleFilter<'a> {
    pub fn new(config: &'a RuleConfig, registry: &'a RuleRegistry) -> Self;

    /// Returns rules sorted by ID for deterministic execution.
    pub fn apply(&self) -> Vec<&'a dyn Rule> {
        let all_rules: Vec<_> = self.registry.iter().collect();
        let threshold = self.config.severity_threshold;

        all_rules
            .into_iter()
            .filter(|r| {
                let id = r.id();
                let severity = self.config.severity_overrides.get(&id).copied()
                    .unwrap_or_else(|| r.severity());

                // Rule is enabled iff:
                (self.config.enabled.is_empty() || self.config.enabled.contains(&id))
                && !self.config.disabled.contains(&id)
                && severity >= threshold  // severity below threshold = excluded
            })
            .sorted_by_key(|r| r.id().clone())  // deterministic order
            .collect()
    }
}
```

**Design rationale:** Filtering is in a dedicated module so the filtering logic can be unit-tested independently of rule execution. Sorting by ID ensures deterministic execution order regardless of registration order.

---

### 3.8 `SuppressionEngine` (`suppression.rs`)

Three-tier suppression: inline comments, file-level, and config-based.

```rust
pub struct SuppressionEngine {
    file_suppressions: HashMap<PathBuf, Vec<RuleId>>,
    config_ignores: Vec<(GlobPattern, Option<RuleId>)>,
}
```

**Suppression methods (in priority order):**

| Method | Syntax | Scope | Priority |
|--------|--------|-------|----------|
| Inline comment | `// sentinel-ignore[rule-id]` on the **preceding line** | Single line | Highest |
| Inline comment (all) | `// sentinel-ignore` | Single line (all rules) | |
| File-level | `// sentinel-ignore-file[rule-id]` at line 1 or 2 | Entire file | |
| Config-level | `ignore = ["rule-id:src/file.rs"]` in sentinel.toml | Pattern match | Lowest |

```rust
impl SuppressionEngine {
    /// Build from config and parsed project.
    pub fn new(config: &RuleConfig, files: &[&Path]) -> Self;

    /// Check if a finding is suppressed.
    pub fn is_suppressed(&self, finding: &Finding) -> bool;

    /// Parse inline suppression comments from source text.
    fn parse_inline_suppressions(source: &str, path: &Path) -> Vec<(usize, Vec<RuleId>)>;
}
```

**Inline suppression detection:**

The engine parses each source file (at scan time, not parse time) looking for `// sentinel-ignore[...]` comments. It records `(line_number - 1, vec_of_rule_ids)`. When checking a finding, it computes `suppressed == finding.line in suppression_lines`.

**Performance:** Suppression parsing is O(source_lines) per file, done once per scan. For a 1000-line file this is ~10µs.

**Design rationale:** Three tiers give users control at every level. Inline suppression is critical for developer workflow — analogous to `#[allow(clippy::*)]`.

**Alternatives rejected:**
- Attribute-based suppression (`#[allow(sentinel::rule)]`): Requires Rust parser changes, couples sentinel to the compiler.
- YAML/JSON suppression file: Additional file to maintain, harder to keep in sync with code changes.

---

### 3.9 `Diagnostic` (`diagnostic.rs`)

Rich rule output. Rules construct `Diagnostic` values internally (via builder), then convert to `Finding` for the Rule trait return type.

```rust
pub struct Diagnostic {
    pub rule_id: RuleId,
    pub severity: Severity,
    pub category: Category,
    pub message: String,
    pub recommendation: String,
    pub primary_span: DiagnosticSpan,
    pub secondary_spans: Vec<DiagnosticSpan>,
    pub notes: Vec<String>,
    pub fix_example: Option<String>,
    pub documentation_url: Option<String>,
    pub cwe_id: Option<String>,
    pub confidence: Confidence,
}
```

**Builder:**

```rust
pub struct DiagnosticBuilder { /* private fields */ }

impl DiagnosticBuilder {
    pub fn new(rule_id: RuleId, message: impl Into<String>) -> Self;
    pub fn severity(mut self, severity: Severity) -> Self;
    pub fn category(mut self, category: Category) -> Self;
    pub fn span(mut self, span: DiagnosticSpan) -> Self;
    pub fn secondary_span(mut self, span: DiagnosticSpan) -> Self;
    pub fn note(mut self, note: impl Into<String>) -> Self;
    pub fn recommendation(mut self, text: impl Into<String>) -> Self;
    pub fn fix_example(mut self, code: impl Into<String>) -> Self;
    pub fn documentation_url(mut self, url: impl Into<String>) -> Self;
    pub fn cwe_id(mut self, id: impl Into<String>) -> Self;
    pub fn confidence(mut self, confidence: Confidence) -> Self;
    pub fn build(self) -> Diagnostic;
}
```

**Conversion to `Finding`:**

```rust
impl From<Diagnostic> for Finding {
    fn from(d: Diagnostic) -> Self {
        Finding {
            rule_id: d.rule_id,
            severity: d.severity,
            category: d.category,
            span: d.primary_span,
            message: d.message,
            recommendation: d.recommendation,
            fix_example: d.fix_example,
        }
    }
}
```

**Design rationale:** Diagnostic is the *authoring* format — rich, flexible, builder-friendly. Finding is the *storage* format — compact, serializable, stable. The conversion drops rich fields that don't need to survive serialization (secondary spans, notes) but these are still available in the report if we want them (we can attach them to Findings as optional fields in a future phase).

**Alternatives rejected:**
- `Diagnostic implements Rule`: Would change the core Rule trait signature, creating a circular dependency.
- All fields on `Finding`: Would bloat the core type with fields only used during rule construction.

---

### 3.10 `RuleResult` (`result.rs`)

The aggregated output of a scan.

```rust
pub struct RuleResult {
    pub findings: Vec<Finding>,
    pub score: SecurityScore,
    pub summary: ExecutionSummary,
}

pub struct ExecutionSummary {
    pub total_rules_run: usize,
    pub total_files: usize,
    pub total_findings: usize,
    pub suppressed_findings: usize,
    pub duration: std::time::Duration,
}
```

**Trait derivations:** `Debug, Clone, PartialEq, Serialize, Deserialize`

**Factory method:**

```rust
impl RuleResult {
    /// Compute result from raw findings.
    pub fn from_findings(
        findings: Vec<Finding>,
        suppressed_count: usize,
        total_files: usize,
        total_rules: usize,
        duration: Duration,
    ) -> Self;
}
```

---

### 3.11 `SecurityScore` (in `sentinel-core`, used here)

Already defined in Phase 2. The runner calls:

```rust
SecurityScore::from_findings(&all_findings)
```

Which computes:

```
weight = count_critical * 25 + count_high * 10 + count_medium * 5 + count_low * 2
score  = max(0, 100 - weight)
```

---

## 4. Built-in Rules

### 4.1 Architecture

Each built-in rule is a **single file** in `builtin/` containing:

1. A **struct** (unit struct or config struct)
2. `Rule` trait implementation
3. `RuleMetaProvider` trait implementation
4. Helper/analysis functions (private)
5. Unit tests (in the same file, `#[cfg(test)]`)
6. **Optional:** A config struct for rule-specific options

```rust
// builtin/missing_auth.rs (conceptual structure)

pub struct MissingRequireAuth;

impl Rule for MissingRequireAuth {
    fn id(&self) -> RuleId { RuleId::new("missing-require-auth").unwrap() }
    fn name(&self) -> &'static str { "missing-require-auth" }
    fn description(&self) -> &'static str {
        "Detects public contract functions that are missing authorization checks"
    }
    fn severity(&self) -> Severity { Severity::Critical }
    fn category(&self) -> Category { Category::Security }
    fn check(&self, ast: &dyn Ast) -> Vec<Finding> {
        // 1. Downcast ast to ParsedProject
        // 2. For each file, find public contract functions
        // 3. Check if function body has auth_calls
        // 4. Return Finding for each unprotected function
    }
}

impl RuleMetaProvider for MissingRequireAuth {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            id: self.id(),
            name: "Missing Authorization",
            description: self.description(),
            severity: self.severity(),
            category: self.category(),
            tags: &["security", "authorization", "critical"],
            documentation_url: Some("https://docs.sentinel.dev/rules/missing-require-auth"),
            cwe_id: Some("CWE-862"),
            confidence: Confidence::High,
            since_version: "0.1.0",
        }
    }
}
```

### 4.2 Rule-by-rule Specification

| # | ID | Severity | Category | Detection Strategy | CWE |
|---|-----|----------|----------|-------------------|-----|
| 1 | `missing-require-auth` | Critical | Security | For each public function in a `#[contractimpl]` block, check if `body.auth_calls` is empty (excluding `__constructor` and `__check_auth`) | CWE-862 |
| 2 | `unsafe-panic` | High | Security | For each function body, count `PanicOp` where kind is `DirectPanic`, `Unwrap`, or `Expect`. Flag functions with >0 | CWE-754 |
| 3 | `large-storage-write` | Medium | Performance | Detect `StorageOp::Set` where the value expression exceeds heuristic size threshold (e.g., >1KB). Flag for review | — |
| 4 | `dead-code` | Low | BestPractice | Find functions marked `pub` in `#[contractimpl]` that have zero call sites within the project | — |
| 5 | `unused-storage` | Medium | BestPractice | Detect `StorageOp::Set` where the key is never read via `StorageOp::Get` elsewhere in the contract | — |
| 6 | `missing-ttl` | High | Security | For each `StorageOp::Set` on `Persistent` or `Temporary` tier, check if a `TtlOp::Extend*` exists for the same key within the same function | CWE-1059 |
| 7 | `auth-mistake` | Critical | Security | Detect patterns like `require_auth()` on a variable that isn't an `Address`, or `require_auth_for_args` with empty args | CWE-863 |
| 8 | `integer-overflow` | High | Security | Flag `ArithOp` with `kind: Add/Sub/Mul` and `has_overflow_check: false` on signed/unsigned integer types | CWE-190 |
| 9 | `gas-optimization` | Low | Gas | Detect repeated `env.storage().persistent().get(key)` calls that could be cached in a local variable | — |
| 10 | `contract-upgrade` | High | Upgrade | Detect `DeployerCallKind::UpdateCurrentContractWasm` and verify `require_auth()` is called before it | CWE-306 |

### 4.3 Registration (`builtin/mod.rs`)

```rust
/// Register all built-in rules into a registry.
pub fn register_all(registry: &mut RuleRegistry) -> Result<(), CoreError> {
    // Security rules
    registry.register(Box::new(MissingRequireAuth))?;
    registry.register(Box::new(UnsafePanic))?;
    registry.register(Box::new(AuthMistake))?;
    registry.register(Box::new(IntegerOverflow))?;
    registry.register(Box::new(MissingTtl))?;
    registry.register(Box::new(ContractUpgrade))?;

    // Performance / Best Practice
    registry.register(Box::new(LargeStorageWrite))?;
    registry.register(Box::new(UnusedStorage))?;
    registry.register(Box::new(DeadCode))?;

    // Gas
    registry.register(Box::new(GasOptimization))?;

    Ok(())
}
```

### 4.4 Adding a New Rule (Contributor Workflow)

1. Create `src/builtin/my_rule.rs`
2. Define a struct `pub struct MyRule;`
3. Implement `Rule` and `RuleMetaProvider`
4. Write unit tests
5. Add `registry.register(Box::new(MyRule))?;` in `builtin/mod.rs`
6. Done — no other code changes needed

**Design rationale:** The registration bottleneck in `mod.rs` is intentional — it ensures every built-in rule is visible in one place for security review. No dynamic discovery.

---

## 5. Configuration System

### 5.1 TOML Schema

```toml
[rules]
# Minimum severity to report
severity = "medium"

# Only run these rules (empty = all enabled)
enable = ["missing-require-auth", "unsafe-panic"]

# Never run these rules
disable = ["gas-optimization"]

# Per-rule severity overrides
[rules.severity]
missing-require-auth = "high"
integer-overflow = "critical"

# File patterns to ignore
ignore = [
    "CWE-862:src/test/*",
    "tests/**",
]

# Per-rule options (future extensibility)
# [rules.options.integer-overflow]
# ignore-signed = true
```

### 5.2 Deserialization

```rust
#[derive(Debug, Deserialize)]
struct RawRuleConfig {
    severity: Option<String>,
    enable: Option<Vec<String>>,
    disable: Option<Vec<String>>,
    #[serde(rename = "severity")]
    severity_overrides: Option<HashMap<String, String>>,
    ignore: Option<Vec<String>>,
}
```

Converted to `RuleConfig` with validation:

```rust
impl TryFrom<RawRuleConfig> for RuleConfig {
    type Error = ConfigError;
    fn try_from(raw: RawRuleConfig) -> Result<Self, ConfigError> {
        // Parse severity strings into Severity enums
        // Validate rule IDs are known
        // Build ignore patterns
    }
}
```

### 5.3 Config Merging

When CLI flags are provided alongside sentinel.toml:

```rust
fn merge(file_config: RuleConfig, cli_config: RuleConfig) -> RuleConfig {
    RuleConfig {
        severity_threshold: cli_config.severity_threshold,  // CLI wins
        enabled: if cli_config.enabled.is_empty() { file_config.enabled } else { cli_config.enabled },
        disabled: [file_config.disabled, cli_config.disabled].concat(),
        severity_overrides: file_config.severity_overrides.merged_with(cli_config.severity_overrides),
        ignore_paths: [file_config.ignore_paths, cli_config.ignore_paths].concat(),
    }
}
```

**Principle:** CLI overrides are additive to file config for `disable` and `ignore`. CLI replaces file config for `enable` and `severity_threshold`.

---

## 6. Rule Execution

### 6.1 Single-threaded (Phase 4)

```rust
pub fn run(&self, ast: &dyn Ast) -> RuleResult {
    let timer = std::time::Instant::now();
    let filter = RuleFilter::new(&self.config, &self.registry);
    let rules = filter.apply();
    let suppression = SuppressionEngine::new(&self.config, ast.files());

    let mut all_findings = Vec::with_capacity(rules.len() * 2);
    let mut suppressed_count = 0;

    for rule in &rules {
        let findings = rule.check(ast);
        let overrides = &self.config.severity_overrides;

        for finding in findings {
            let sev = overrides.get(&finding.rule_id).copied().unwrap_or(finding.severity);
            let mut f = Finding { severity: sev, ..finding };

            if suppression.is_suppressed(&f) {
                suppressed_count += 1;
                continue;
            }
            all_findings.push(f);
        }
    }

    all_findings.sort();
    all_findings.dedup();

    let score = SecurityScore::from_findings(&all_findings);
    let duration = timer.elapsed();

    RuleResult::from_findings(all_findings, suppressed_count, ast.files().len(), rules.len(), duration)
}
```

**Error handling:** Rule execution errors are caught per-rule:

```rust
for rule in &rules {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        rule.check(ast)
    }));
    match result {
        Ok(findings) => { /* process */ }
        Err(panic) => {
            tracing::error!("Rule `{}` panicked: {:?}", rule.id(), panic);
            // Continue to next rule — one panicking rule doesn't fail the scan
        }
    }
}
```

### 6.2 Future Parallel Execution (Phase 10)

```rust
use rayon::prelude::*;

let results: Vec<(RuleId, Vec<Finding>)> = rules
    .par_iter()
    .map(|rule| {
        let findings = rule.check(ast);
        (rule.id(), findings)
    })
    .collect();
```

**Why this works:** Rules are stateless (`&self`). The AST is read-only (`&dyn Ast`). No locking required. `rayon`'s work-stealing thread pool handles load balancing.

**Safety:** The `Rule` trait requires `Send + Sync`. `&dyn Ast` is `Send + Sync`. `&dyn Rule` is `Send + Sync`. The parallel map is trivially safe.

### 6.3 Deterministic Ordering

Rules are sorted by `RuleId` before execution. Within a single rule, findings are produced in file-walk order. The final output is sorted by severity descending, then file path, then line, then column.

This guarantees that **identical projects scanned on any machine, any OS, any time → identical output**.

---

## 7. Diagnostics

### 7.1 Diagnostic Types

| Type | Field in Diagnostic | In Finding? | Purpose |
|------|-------------------|-------------|---------|
| Primary span | `primary_span` | Yes (`.span`) | The main location of the issue |
| Secondary spans | `secondary_spans` | No (dropped) | Related locations (e.g., where auth is missing vs where it should be) |
| Notes | `notes` | No | Additional human-readable context |
| Fix example | `fix_example` | Yes | Code snippet showing how to fix |
| Doc URL | `documentation_url` | No | Link to rule documentation |
| CWE ID | `cwe_id` | No | CWE identifier for security standards |
| Confidence | `confidence` | No | How sure the rule is about this finding |

### 7.2 Display Format

```
error[S-001]: Missing authorization check
  ┌─ src/contract.rs:42:5
  │
42 │   pub fn transfer(to: Address, amount: i128) {
  │   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ public function without authorization
  │
  │   note: this function modifies caller state but never calls `require_auth()`
  │   help: add `from.require_auth();` at the beginning of this function
  │
  │   CWE-862: Missing Authorization
  │   https://docs.sentinel.dev/rules/missing-require-auth
  │
  │   example fix:
  │   │ pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
  │   │     from.require_auth();
  │   │     // ...
  │   │ }
```

### 7.3 Future SARIF Support

The `Diagnostic → Finding` conversion is lossy for SARIF purposes. When SARIF is required (Phase 10+), the conversion will retain secondary spans and notes:

```toml
# Future sentinel.toml option
format = "sarif"
```

The SARIF serializer will:
1. Map `diagnostic.primary_span` → `SARIF.region`
2. Map `diagnostic.secondary_spans` → `SARIF.relatedLocations`
3. Map `diagnostic.notes` → `SARIF.message`
4. Map `diagnostic.cwe_id` → `SARIF.properties`

---

## 8. Performance

### Asymptotic Complexity

| Component | Complexity | Factor |
|-----------|-----------|--------|
| Rule filtering | O(r log r) | r = registered rules (<100) |
| Suppression engine build | O(f × s) | f = files, s = avg source lines |
| Per-rule execution | O(r × n) | r = enabled rules, n = AST node count |
| Deduplication + sorting | O(f log f) | f = total findings |
| Security score | O(f) | f = total findings |
| **Total** | **O(r × n + f log f)** | Dominated by rule execution |

### Constant Factors

- Each rule walks the full AST independently → O(r × n) where r=10 rules × n=~5000 AST nodes = 50,000 node visits
- Each visit is a cheap pattern match on enum variant + field access
- Suppression parse: O(total source lines) — single pass, ~10µs/1000 lines

### Large Project Profile

| Metric | Value |
|--------|-------|
| 10 built-in rules | 10 `check()` calls |
| 1000-file workspace, 200K AST nodes | ~200M node visits total |
| Sequential execution estimate | ~200ms (at ~1M node visits/ms) |
| Parallel execution estimate (8 cores) | ~25ms |
| Memory | O(findings) — typically <1MB |

### Caching Opportunities (Future)

| Strategy | Saving | Complexity |
|----------|--------|------------|
| AST-level: cache parsed files | 90% on re-scan | O(files) |
| Rule-level: cache rule-specific indexes | 50% for repeated scans | O(r) |
| Finding-level: cache across runs | 100% if no changes | Requires content hash |

### Parallelization Roadmap

```
Phase 4:  Sequential        (10 rules × ~1ms = ~10ms)
Phase 10: Parallel (rayon)  (10 rules / 8 cores × ~1ms = ~1.25ms)
```

---

## 9. Testing Strategy

### 9.1 Test Layers

```
┌──────────────────────────────────────┐
│  Integration tests (end-to-end scan) │  tests/integration/
├──────────────────────────────────────┤
│  Rule-specific tests (each builtin)  │  src/builtin/*.rs #[cfg(test)]
├──────────────────────────────────────┤
│  Engine tests (filter, suppress)     │  src/filter.rs, suppression.rs
├──────────────────────────────────────┤
│  Core type tests (Diagnostic, etc.)  │  src/diagnostic.rs, result.rs
└──────────────────────────────────────┘
```

### 9.2 Unit Tests

#### `filter.rs` tests
- All rules enabled by default when config is empty
- Disabled rule is excluded from results
- Enabled list overrides disabled list
- Severity threshold excludes low-severity rules
- Severity override changes rule severity
- Rules are returned in deterministic order (sorted by ID)

#### `suppression.rs` tests
- Inline `// sentinel-ignore[rule-id]` suppresses matching finding
- Inline `// sentinel-ignore` suppresses all rules
- `// sentinel-ignore-file[rule-id]` suppresses entire file
- Config ignore pattern matches path glob
- Non-matching suppression doesn't suppress
- Multiple suppression reasons combine
- Suppression on wrong line number doesn't match
- Empty source text produces no suppressions

#### `diagnostic.rs` tests
- Builder constructs valid Diagnostic
- `Diagnostic → Finding` conversion preserves primary span, rule_id, severity, category
- Secondary spans and notes are dropped in conversion
- All builder methods return correct values

#### `runner.rs` tests
- Empty registry produces empty result
- Single rule execution returns its findings
- Panicking rule is caught and logged (doesn't fail scan)
- Execution duration is recorded
- Suppressed findings are counted correctly

### 9.3 Rule-specific Tests

Each built-in rule has its own test module with:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use sentinel_parser::test_helpers::parse_contract;

    #[test]
    fn detects_missing_auth() {
        let contract = parse_contract("
            #[contract]
            pub struct MyContract;

            #[contractimpl]
            impl MyContract {
                pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
                    // No require_auth!
                }
            }
        ");
        let rule = MissingRequireAuth;
        let findings = rule.check(&contract);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Critical);
    }

    #[test]
    fn skips_constructor() {
        let contract = parse_contract("
            #[contract]
            pub struct MyContract;

            #[contractimpl]
            impl MyContract {
                pub fn __constructor(env: Env, admin: Address) {
                    // Constructor doesn't need auth
                }
            }
        ");
        let rule = MissingRequireAuth;
        let findings = rule.check(&contract);
        assert_eq!(findings.len(), 0);
    }

    #[test]
    fn skips_check_auth() {
        // __check_auth is an auth handler, not a user-facing function
    }

    #[test]
    fn ignores_private_functions() {
        // Private functions don't need require_auth
    }
}
```

### 9.4 Integration Tests

```rust
#[test]
fn full_scan_simple_token() {
    let engine = RuleEngine::new(RuleConfig::default());
    let project = parse_project("tests/fixtures/simple_token").unwrap();
    let result = engine.run(&project);

    assert_eq!(result.score.score, 100);  // Secure token should have 0 findings
    assert!(result.findings.is_empty());
}

#[test]
fn full_scan_vulnerable_contract() {
    let engine = RuleEngine::new(RuleConfig::default());
    let project = parse_project("tests/fixtures/all_vulnerabilities").unwrap();
    let result = engine.run(&project);

    assert!(result.score.score < 100);
    assert!(result.findings.len() >= 5);  // Multiple vulnerabilities
    assert!(result.summary.total_rules_run == 10);
}
```

### 9.5 Golden Tests

Rule output is compared to golden files:

```bash
tests/golden/
├── missing_auth.json     # Expected findings for missing_auth contract
├── unsafe_panic.json
├── all_vulnerabilities.md
└── ...
```

Update golden files with:

```bash
UPDATE_EXPECT=1 cargo test
```

### 9.6 Benchmarking (criterion)

```rust
fn bench_full_scan(c: &mut Criterion) {
    let engine = RuleEngine::new(RuleConfig::default());
    let project = parse_project("benches/fixtures/large_contract").unwrap();

    c.bench_function("full_scan_large_contract", |b| {
        b.iter(|| engine.run(&project));
    });
}
```

### 9.7 Fuzz Testing Opportunities

- Fuzz `RuleConfig::try_from(raw_toml)` with random TOML input
- Fuzz `SuppressionEngine::new(config, files)` with random suppression patterns
- Not in Phase 4 scope, but the types are designed for it

---

## 10. Future Extensions

### 10.1 Plugin System

```rust
pub trait RulePlugin: Send + Sync {
    fn name(&self) -> &str;
    fn rules(&self) -> Vec<Box<dyn Rule>>;
}
```

**Loading:**
- Dynamic library loading via `libloading` (Phase 11+)
- Directory scan: `~/.sentinel/plugins/*.so` / `*.dll` / `*.dylib`
- Each plugin exposes a `sentinel_plugin_init()` function returning `Vec<Box<dyn Rule>>`
- Plugin rules register into the same `RuleRegistry`

**Security boundary:** Plugins run in-process. Future: WASM sandbox.

### 10.2 WASM Rules

```rust
pub struct WasmRule {
    id: RuleId,
    wasm_bytes: Vec<u8>,
    instance: wasmtime::Instance,
}
```

**Interface:**
- Host provides `Ast` data via WASM memory
- WASM module exports `check() → JSON bytes`
- Host deserializes `Vec<Finding>` from WASM response

**Benefits:** Memory-safe, sandboxed, language-agnostic, verifiable.

### 10.3 Auto-fix

```rust
pub trait RuleFix: Rule {
    fn fix(&self, finding: &Finding) -> Option<String>;
}
```

**Output:** A diff/patch that can be applied to the source file. CLI flag: `sentinel scan --fix`.

### 10.4 LSP Integration

The rule engine is already a library. LSP integration:
1. Call `RuleEngine::new(config)` once
2. On file save: `parser::parse_file(path, source) → ParsedFile`
3. `runner.run_on_file(&parsed_file, &project_context) → Vec<Diagnostic>`
4. Convert diagnostics to LSP `Diagnostic` type
5. Publish diagnostics to client

**No engine changes needed.**

### 10.5 Rule Marketplace

Metadata-driven:

```toml
# In a community rule package
[package]
name = "sentinel-rules-stellar"
version = "0.1.0"

[[rules]]
id = "my-custom-rule"
name = "My Custom Rule"
severity = "high"
```

Consumed via:

```toml
# sentinel.toml
[rules.extensions]
sentinel-rules-stellar = "0.1.0"
```

---

## 11. Tradeoff Table

| Decision | Choice | Alternatives | Rationale |
|----------|--------|-------------|-----------|
| **Rule trait location** | In sentinel-core (carried forward) | Move to sentinel-rules | Core trait can't depend on rules crate; no circular deps |
| **Rule return type** | `Vec<Finding>` (carried forward) | `Vec<Diagnostic>` | Keeps core API stable; Diagnostic converts → Finding |
| **Diagnostic type** | In sentinel-rules (builder pattern) | All fields on Finding | Diagnostic is authoring format; Finding is storage format |
| **Metadata separation** | `RuleMetaProvider` trait separate from `Rule` | Metadata on Rule trait | Rule trait stays lean; metadata is optional |
| **Registry extension** | Extension trait on core registry | New registry in rules crate | Avoids duplicate rules; single source of truth |
| **Parallel execution** | Deferred to Phase 10 | Implement now with rayon | MVP doesn't need it; keeps deps minimal |
| **Rule filtering** | Config-driven with severity threshold | CLI-only, no config | `sentinel.toml` is the standard config; CLI overrides extend it |
| **Suppression method** | 3-tier (inline, file, config) | Only config | Developers need inline suppression for workflow |
| **Suppression parsing** | Regex-free manual line scan | `regex` crate, tree-sitter comment parser | Manual scan is O(n) with zero deps; regex is overkill |
| **Panic isolation** | `catch_unwind` per rule | Let panics propagate | One panicking rule shouldn't kill the scan |
| **Rule ID format** | Kebab-case string (carried forward) | Uuid, enum | Human-readable, CLI-friendly, extensible |
| **Built-in rule file** | One file per rule | One file per category | Independent; easy to review, test, and maintain |
| **Security score** | Linear weighted sum (carried forward) | Logarithmic, ML-based | Deterministic, transparent, verifiable |
| **Config merging** | CLI overlays on file config | Only file, only CLI | Flexible UX: set policy in file, override per-run |
| **Dependency: `indexmap`** | Yes (for registry, carried forward) | `HashMap` + vec | Deterministic iteration for consistent rule ordering |
| **Dependency: `rayon`** | No (deferred to Phase 10) | Yes | Keeps Phase 4 dependency-free; additive in Phase 10 |

### Dependencies (Phase 4)

| Crate | Required? | Purpose |
|-------|-----------|---------|
| `sentinel-core` | Yes | Rule trait, RuleRegistry, Finding, Severity, etc. |
| `sentinel-parser` | Yes | ParsedProject, FunctionDef, StorageOp, etc. |
| `tracing` | Yes | Structured logging in runner and rules |
| `glob` | Yes | Path pattern matching for suppression config |
| `thiserror` | Yes | Derived `Error` for any new error types |
| `serde` | Yes | RuleConfig deserialization |

**No new external dependencies** beyond what sentinel-core and sentinel-parser already pull in. `glob` is lightweight and widely used.

---

## 12. Conventional Commit

```
feat(rules): implement rule engine with 10 built-in Soroban security rules

Add sentinel-rules crate implementing the analysis engine and initial
rule set for Soroban smart contract auditing.

Rule engine:
- RuleRunner: sequential execution with deterministic ordering by RuleId
- RuleFilter: config-driven rule enable/disable, severity threshold, overrides
- SuppressionEngine: 3-tier suppression (inline comments, file-level, config)
- Panic isolation: per-rule catch_unwind prevents single rule failure
- ExecutionSummary with timing, suppressed count, and rule/file totals

Diagnostic system:
- Diagnostic type with builder pattern (primary + secondary spans, notes,
  CWE ID, confidence, documentation URL, fix examples)
- Conversion to sentinel_core::Finding for stable serialization
- RuleMetaProvider trait separating static metadata from analysis logic

Built-in rules (10):
- S-001 missing-require-auth (Critical): public functions without authorization
- S-002 unsafe-panic (High): panic!/unwrap/expect in contract code
- S-003 large-storage-write (Medium): oversized storage writes
- S-004 dead-code (Low): unused public functions
- S-005 unused-storage (Medium): storage writes never read
- S-006 missing-ttl (High): persistent storage without TTL extension
- S-007 auth-mistake (Critical): incorrect authorization patterns
- S-008 integer-overflow (High): arithmetic without overflow protection
- S-009 gas-optimization (Low): repeated storage access patterns
- S-010 contract-upgrade (High): unsafe upgrade patterns

Configuration:
- RuleConfig deserialized from sentinel.toml with severity, enable/disable
- CLI-to-file merging with precedence rules
- Glob-based path ignore patterns

Testing:
- Rule-specific unit tests for each built-in (nominal + edge cases)
- Filter and suppression engine unit tests
- Integration tests with full contract fixtures
- Golden test framework for output validation

Future-proofing:
- All types Send + Sync for parallel execution in Phase 10
- Plugin and WASM rule support enabled by registry design
- LSP integration without engine changes
```

---

**I'm ready for review and approval to implement sentinel-rules.**
