# Phase 3: `sentinel-parser` — Architecture & Implementation Plan

## 1. Overall Architecture

The parser crate is the bridge between raw Soroban Rust source files and the curated, rule-consumable AST. It discovers project structure, invokes `syn` to parse each file, converts the generic Rust AST into Soroban-specific high-level nodes, and implements `sentinel_core::Ast`.

### Module Map

```
sentinel-parser/
├── Cargo.toml
└── src/
    ├── lib.rs              # Re-exports, top-level parse function, public API surface
    ├── parser.rs           # Orchestrator: ties project discovery → file parsing → AST conversion
    ├── project.rs          # Cargo.toml discovery, workspace detection, manifest parsing
    ├── walker.rs           # Filesystem walker: finds .rs files, respects .gitignore/ignore patterns
    ├── convert.rs          # syn::File → curated SorobanAst conversion (the core transformation)
    ├── ast.rs              # Curated Soroban AST types (Contract, Function, StorageOp, etc.)
    ├── visitor.rs          # AstVisitor trait for walking the curated AST
    ├── source.rs           # SourceFile representation: path, text, parsed syn::File
    ├── error.rs            # ParserError enum (thiserror)
    └── tests/
        ├── mod.rs
        ├── parser_tests.rs       # Full pipeline tests
        ├── project_tests.rs      # Project discovery tests
        ├── convert_tests.rs      # syn → curated AST conversion tests
        └── fixtures/             # Test Soroban contracts
```

### Responsibility of each file

| File | Visibility | Responsibility |
|------|-----------|----------------|
| `lib.rs` | `pub` | Single entry point: `parse_project(path) → Result<ParsedProject>` |
| `parser.rs` | `pub(crate)` | Orchestrates the pipeline stages. Not directly exposed. |
| `project.rs` | `pub(crate)` | Discovers `Cargo.toml`, reads manifest, identifies crate structure. |
| `walker.rs` | `pub(crate)` | Recursive filesystem walk, filters to `.rs` files, respects ignore rules. |
| `convert.rs` | `pub(crate)` | The heart of the crate. Walks `syn::File`, produces curated AST. |
| `ast.rs` | `pub` | All curated AST types. Rules and report crate depend on these. |
| `visitor.rs` | `pub` | Trait for traversing the curated AST without matching. |
| `source.rs` | `pub(crate)` | Holds a file's path, source text, and parsed `syn::File`. Not exposed. |
| `error.rs` | `pub` | `ParserError` enum with `thiserror` derives. |

---

## 2. Type-by-Type Design

### 2.1 `ParsedProject` (`ast.rs`)

The top-level output of the parser. Implements `sentinel_core::Ast`.

```rust
pub struct ParsedProject {
    pub root: PathBuf,
    pub manifest: Option<CargoManifest>,
    pub files: Vec<ParsedFile>,
}
```

**Fields:**

| Field | Type | Notes |
|-------|------|-------|
| `root` | `PathBuf` | Absolute path to project root (directory containing Cargo.toml or scan root) |
| `manifest` | `Option<CargoManifest>` | Parsed Cargo.toml if found; `None` for loose file scans |
| `files` | `Vec<ParsedFile>` | Every discovered and parsed `.rs` file |

**Trait derivations:** `Debug, Clone, PartialEq, Serialize, Deserialize`

**Ownership:** Fully owned. No borrowed data. Thread-safe via `Send + Sync`.

**Design rationale:** The project is the unit of analysis. Rules receive this as `&dyn Ast` and downcast to `&ParsedProject` to access files, contracts, functions, etc.

---

### 2.2 `ParsedFile` (`ast.rs`)

Represents a single parsed Rust source file.

```rust
pub struct ParsedFile {
    pub path: PathBuf,
    pub contracts: Vec<ContractDef>,
    pub error_types: Vec<ErrorTypeDef>,
    pub contract_types: Vec<ContractTypeDef>,
    pub free_functions: Vec<FunctionDef>,
    pub has_no_std: bool,
    pub parse_error: Option<String>,
}
```

**Fields:**

| Field | Type | Notes |
|-------|------|-------|
| `path` | `PathBuf` | Absolute path to the `.rs` file |
| `contracts` | `Vec<ContractDef>` | Contract structs + their impl blocks |
| `error_types` | `Vec<ErrorTypeDef>` | Enums with `#[contracterror]` |
| `contract_types` | `Vec<ContractTypeDef>` | Structs/enums with `#[contracttype]` |
| `free_functions` | `Vec<FunctionDef>` | Top-level functions (outside impl blocks) |
| `has_no_std` | `bool` | Whether `#![no_std]` is present |
| `parse_error` | `Option<String>` | If `syn` failed on this file, the error message; other fields may be empty |

**Trait derivations:** `Debug, Clone, PartialEq, Serialize, Deserialize`

**Why `parse_error` is a field instead of returning `Err`:** Per-file parse failures should not fail the entire scan. A file with broken syntax simply yields no findings. The error is recorded for user visibility in the report.

---

### 2.3 `ContractDef` (`ast.rs`)

A Soroban contract — the struct definition plus its associated impl blocks.

```rust
pub struct ContractDef {
    pub name: String,
    pub struct_span: DiagnosticSpan,
    pub is_contract: bool,           // #[contract]
    pub impl_blocks: Vec<ImplBlock>,
}
```

**Fields:**

| Field | Type | Notes |
|-------|------|-------|
| `name` | `String` | Contract struct name |
| `struct_span` | `DiagnosticSpan` | Location of the struct definition |
| `is_contract` | `bool` | Whether marked with `#[contract]` |
| `impl_blocks` | `Vec<ImplBlock>` | All `#[contractimpl]` blocks for this contract |

**Design rationale:** A contract can have multiple `#[contractimpl]` blocks. Collecting them under a single `ContractDef` lets rules analyze the contract holistically (e.g., checking if `__check_auth` exists alongside public functions).

---

### 2.4 `ImplBlock` (`ast.rs`)

A `#[contractimpl]` block.

```rust
pub struct ImplBlock {
    pub span: DiagnosticSpan,
    pub functions: Vec<FunctionDef>,
    pub is_trait_impl: bool,           // impl Trait for Contract vs impl Contract
}
```

**Fields:**

| Field | Type | Notes |
|-------|------|-------|
| `span` | `DiagnosticSpan` | Location of the `impl` keyword |
| `functions` | `Vec<FunctionDef>` | All functions in this block |
| `is_trait_impl` | `bool` | `true` if this is `impl TraitForContract for Contract` |

**Design rationale:** `is_trait_impl` helps rules distinguish contract interface implementations from inherent methods.

---

### 2.5 `FunctionDef` (`ast.rs`)

A function with full Soroban-specific body analysis.

```rust
pub struct FunctionDef {
    pub name: String,
    pub span: DiagnosticSpan,
    pub visibility: Visibility,
    pub is_constructor: bool,
    pub is_check_auth: bool,
    pub signature: FunctionSignature,
    pub body: Option<FunctionBody>,
}
```

**Fields:**

| Field | Type | Notes |
|-------|------|-------|
| `name` | `String` | Function identifier |
| `span` | `DiagnosticSpan` | Location of `fn` keyword |
| `visibility` | `Visibility` | Public, private, or pub(crate) |
| `is_constructor` | `bool` | `true` if named `__constructor` |
| `is_check_auth` | `bool` | `true` if named `__check_auth` |
| `signature` | `FunctionSignature` | Parameter types, return type |
| `body` | `Option<FunctionBody>` | Analyzed body; `None` if only declaration |

```rust
pub enum Visibility {
    Public,
    PublicCrate,
    PublicSuper,
    Private,
}
```

**Trait derivations:** `Debug, Clone, PartialEq, Serialize, Deserialize`

**Design rationale:** `is_constructor` and `is_check_auth` are common query targets for rules. Lifting them to dedicated booleans avoids every rule having to string-match the function name.

---

### 2.6 `FunctionSignature` (`ast.rs`)

```rust
pub struct FunctionSignature {
    pub params: Vec<Parameter>,
    pub returns_result: bool,
    pub return_type: Option<String>,
}

pub struct Parameter {
    pub name: String,
    pub type_name: String,
    pub span: DiagnosticSpan,
}
```

**Design rationale:** Rules need to know whether a function returns `Result` (for error-handling analysis) and whether it takes `Env` (a core Soroban pattern). The type is stored as a `String` (not a `syn::Type`) to avoid coupling the curated AST to `syn` types. Rules parse the string if they need deeper inspection.

---

### 2.7 `FunctionBody` (`ast.rs`)

The analyzed body of a function — a summary of every operation relevant to security analysis.

```rust
pub struct FunctionBody {
    pub storage_ops: Vec<StorageOp>,
    pub auth_calls: Vec<AuthCall>,
    pub panics: Vec<PanicOp>,
    pub arith_ops: Vec<ArithOp>,
    pub ttl_ops: Vec<TtlOp>,
    pub deployer_calls: Vec<DeployerCall>,
    pub cross_contract_calls: Vec<CrossContractCall>,
}
```

**Design rationale:** Rather than exposing raw `syn::Stmt`, we pre-analyze the body into Soroban-specific operations. Each rule only looks at the operation vector it cares about. This is the core optimization of the curated AST approach.

---

### 2.8 `StorageOp` (`ast.rs`)

```rust
pub struct StorageOp {
    pub kind: StorageOpKind,
    pub storage_type: StorageTier,
    pub key: String,               // Source text representation of the key expression
    pub span: DiagnosticSpan,
}

pub enum StorageOpKind {
    Get,
    Set,
    Has,
    Remove,
    Update,
    TryUpdate,
    ExtendTtl,
}

pub enum StorageTier {
    Persistent,
    Temporary,
    Instance,
}
```

**Detection approach (in `convert.rs`):** Walk `Expr::MethodCall` chains. Match `env.storage().persistent().get(...)` by checking receiver chains. The innermost receiver is `env`, then `.storage()`, then `.persistent()|.temporary()|.instance()`, then the terminal method.

---

### 2.9 `AuthCall` (`ast.rs`)

```rust
pub struct AuthCall {
    pub kind: AuthCallKind,
    pub target: String,    // Expression text: e.g., "admin", "from"
    pub span: DiagnosticSpan,
}

pub enum AuthCallKind {
    RequireAuth,
    RequireAuthForArgs,
}
```

**Detection approach:** Match `Expr::MethodCall` where `method == "require_auth"` or `method == "require_auth_for_args"`.

---

### 2.10 `PanicOp` (`ast.rs`)

```rust
pub struct PanicOp {
    pub kind: PanicKind,
    pub message: String,
    pub span: DiagnosticSpan,
}

pub enum PanicKind {
    DirectPanic,              // panic!("msg")
    PanicWithError,           // panic_with_error!(&env, Error::Variant)
    AssertWithError,          // assert_with_error!(&env, cond, Error::Variant)
    Unwrap,                   // .unwrap()
    Expect,                   // .expect("msg")
    Unreachable,              // unreachable!()
    Unimplemented,            // todo!() / unimplemented!()
}
```

**Detection approach:**
- `panic!(...)`: `Expr::Macro` where `mac.path.is_ident("panic")`
- `panic_with_error!(...)`: `Expr::Macro` where `mac.path.is_ident("panic_with_error")`
- `.unwrap()` / `.expect(...)`: `Expr::MethodCall` with matching method name

---

### 2.11 `ArithOp` (`ast.rs`)

```rust
pub struct ArithOp {
    pub kind: ArithKind,
    pub span: DiagnosticSpan,
    pub left_type: Option<String>,   // inferred type info if available
    pub has_overflow_check: bool,    // wrapping_*, checked_*, saturating_*
}

pub enum ArithKind {
    Add, Sub, Mul, Div, Rem,
    Neg,                               // unary negation
    Shl, Shr,                          // potential overflow in shift
    CompoundAdd, CompoundSub,          // +=, -=
    CompoundMul, CompoundDiv, CompoundRem,
    WrappingOp,                        // wrapping_add, etc. (explicitly safe)
    CheckedOp,                         // checked_add, etc. (explicitly safe)
    SaturatingOp,                      // saturating_add, etc. (explicitly safe)
}
```

**Design rationale:** Rules flag `Add`/`Sub`/`Mul` on integer types without `wrapping_*`/`checked_*`/`saturating_*` prefix. The curated AST pre-computes `has_overflow_check` so rules don't need to re-parse method chains.

---

### 2.12 `TtlOp` (`ast.rs`)

```rust
pub struct TtlOp {
    pub kind: TtlKind,
    pub storage_type: Option<StorageTier>,
    pub has_extend_after_write: bool,   // extend_ttl called after set/update
    pub span: DiagnosticSpan,
}

pub enum TtlKind {
    ExtendInstance,
    ExtendPersistent,
    ExtendTemporary,
    DeployerExtend,
}
```

**Design rationale:** The `has_extend_after_write` flag is pre-computed. A rule checking "missing TTL" can immediately flag a `StorageOp::Set` on persistent storage that isn't followed by a `TtlOp`.

---

### 2.13 `DeployerCall` (`ast.rs`)

```rust
pub struct DeployerCall {
    pub kind: DeployerCallKind,
    pub span: DiagnosticSpan,
}

pub enum DeployerCallKind {
    UploadWasm,
    Deploy,
    DeployWithCurrentContract,
    DeployWithAddress,
    UpdateCurrentContractWasm,   // upgrade
}
```

---

### 2.14 `CrossContractCall` (`ast.rs`)

```rust
pub struct CrossContractCall {
    pub contract: String,           // Expression text
    pub function: String,           // Function name
    pub span: DiagnosticSpan,
    pub is_try_call: bool,          // try_invoke_contract (handles errors)
}
```

---

### 2.15 `CargoManifest` (`project.rs`)

```rust
pub struct CargoManifest {
    pub path: PathBuf,
    pub package_name: Option<String>,
    pub dependencies: Vec<String>,          // dependency crate names
    pub has_soroban_sdk: bool,
    pub is_workspace: bool,
    pub members: Vec<PathBuf>,              // workspace members
}
```

**Detection:** Parse `Cargo.toml` with `toml` crate. Check `[dependencies]` for `soroban-sdk` or `soroban-sdk-*`. Detect `[workspace]` and `members`.

---

### 2.16 Error Types (`error.rs`)

```rust
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParserError {
    #[error("I/O error: {0}")]
    Io(#[from] Arc<std::io::Error>),

    #[error("invalid project at `{path}`: {detail}")]
    InvalidProject { path: PathBuf, detail: String },

    #[error("parse error in `{path}`: {detail}")]
    ParseError { path: PathBuf, detail: String },

    #[error("unsupported Rust syntax in `{path}`: {detail}")]
    UnsupportedSyntax { path: PathBuf, detail: String },
}
```

**Why `Arc<io::Error>` instead of `io::Error`:** `ParserError` must implement `Clone` for the curated AST to be cloneable. `std::io::Error` does not implement `Clone`. Wrapping in `Arc` solves this without heap-allocating per-clone.

**Design rationale:** Errors are categorized. IO and parse failures are *recoverable* at the file level (skip the file, continue). `InvalidProject` is *unrecoverable* at the project level (abort the scan). This categorization guides the caller's error handling strategy.

---

### 2.17 Thread Safety

Every curated AST type is `Send + Sync`. The `ParsedProject` implements `sentinel_core::Ast` which requires `Send + Sync`. This enables parallel rule execution in future phases.

The `syn::File` type is kept private to `source.rs` and never exposed across threads directly. The curated AST is the thread-safe boundary.

---

## 3. Parsing Pipeline

### Complete Flow

```
Path (Dir or .rs file)
│
├── is directory?
│   ├── YES → project.rs: discover Cargo.toml
│   │         ├── found → parse manifest, detect workspace
│   │         └── not found → treat as loose file collection
│   │
│   └── NO  → treat as single file (no project context)
│
├── walker.rs: discover all .rs files
│   ├── skip: target/, .git/, node_modules/, .hg/
│   ├── skip: hidden files/dirs (.)
│   ├── skip: files in sentinel ignore list
│   └── yield: Vec<PathBuf> of .rs files
│
├── For each file, in order:
│   ├── source.rs: read file to String
│   │   ├── success → continue
│   │   └── IO error → record error, skip file
│   │
│   ├── syn::parse_file(&source)
│   │   ├── success → syn::File
│   │   └── error → record parse error, skip file
│   │
│   └── convert.rs: syn::File → ParsedFile
│       ├── walk file.items
│       │   ├── detect #![no_std]
│       │   ├── Item::Struct → check #[contract], #[contracttype]
│       │   ├── Item::Enum  → check #[contracterror], #[contracttype]
│       │   ├── Item::Impl  → check #[contractimpl]
│       │   │                → walk ImplItem::Fn for each function
│       │   │                → analyze function body
│       │   ├── Item::Fn    → top-level function analysis
│       │   └── other       → ignore
│       │
│       └── yield: ParsedFile
│
├── parser.rs: collect all ParsedFile into ParsedProject
│   └── yield: ParsedProject
│
└── Validation (lightweight):
    ├── warn: duplicate contract names
    └── warn: orphan impl blocks (no matching struct)
```

### Stage Details

#### Stage 1: Project Discovery (`project.rs`)

| Aspect | Detail |
|--------|--------|
| **Input** | `&Path` — directory or file path |
| **Output** | `Option<CargoManifest>` |
| **Errors** | `ParserError::InvalidProject` if Cargo.toml exists but is malformed |
| **Performance** | O(1) — single file read + TOML parse (~100µs) |
| **Edge cases** | Path doesn't exist, path is a file, no Cargo.toml, workspace root |

#### Stage 2: Source Discovery (`walker.rs`)

| Aspect | Detail |
|--------|--------|
| **Input** | `&Path` — project root |
| **Output** | `Vec<PathBuf>` — sorted list of `.rs` files |
| **Errors** | IO errors per-directory (skip inaccessible dirs, log warning) |
| **Performance** | O(n) where n = total files in tree. Uses `walkdir` with `filter_entry` for early pruning |
| **Ignored** | `target/`, `.git/`, `node_modules/`, hidden files/dirs |

#### Stage 3: File Parsing (`source.rs` + `syn`)

| Aspect | Detail |
|--------|--------|
| **Input** | `&Path`, file content as `String` |
| **Output** | `syn::File` |
| **Errors** | IO error → skip; `syn::Error` → record as `parse_error` in `ParsedFile` |
| **Performance** | O(f) where f = file size in bytes. `syn::parse_file` allocates ~5-10x source size. ~1-3ms per 100KB file |

#### Stage 4: AST Conversion (`convert.rs`)

| Aspect | Detail |
|--------|--------|
| **Input** | `&syn::File`, `&Path` |
| **Output** | `ParsedFile` |
| **Errors** | None (best-effort: unknown syntax is ignored, not errored) |
| **Performance** | O(s) where s = number of syntax nodes. Single-pass visit. Each node visited once. |

### Pipeline Invariants

1. **All errors are non-fatal at the file level.** A broken file results in `ParsedFile { parse_error: Some("..."), .. }` with empty collections. The scan continues.
2. **File order is deterministic.** Files are sorted alphabetically by path.
3. **The pipeline is idempotent.** Same input → same output.
4. **No external state.** No caching, no shared counters, no randomness.

---

## 4. Integration with `sentinel-core`

### Crate Boundaries

```
sentinel-core (no deps on sentinel-* crates)
├── Ast trait (Send + Sync)
│   └── fn files(&self) -> &[PathBuf]
│   └── fn as_any(&self) -> &dyn Any
├── DiagnosticSpan
├── Severity, Category, Finding, Rule, RuleRegistry, SecurityScore, ScanResult
└── CoreError

sentinel-parser (depends on sentinel-core)
├── ParsedProject  (implements Ast)
├── ParsedFile, ContractDef, FunctionDef, StorageOp, ...
└── ParserError
```

### The `Ast` Trait (in `sentinel-core`)

```rust
/// A parsed project AST that rules can analyze.
///
/// This trait is intentionally minimal. The concrete type (e.g. `ParsedProject`)
/// lives in the parser crate. Rules downcast via `as_any()` to access
/// Soroban-specific types.
pub trait Ast: Send + Sync + std::fmt::Debug {
    /// Returns the set of source files that were parsed.
    fn files(&self) -> &[PathBuf];

    /// Downcast to the concrete AST type.
    fn as_any(&self) -> &dyn std::any::Any;
}
```

### Implementation in `sentinel-parser`

```rust
impl Ast for ParsedProject {
    fn files(&self) -> &[PathBuf] {
        // Collect all file paths from self.files
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
```

### How Rules Use the AST

```rust
fn check(&self, ast: &dyn Ast) -> Vec<Finding> {
    let project = ast.as_any()
        .downcast_ref::<ParsedProject>()
        .expect("sentinel-parser AST expected");

    for file in &project.files {
        for contract in &file.contracts {
            // ... analyze and produce findings
        }
    }
    // ...
}
```

**Why `expect` is acceptable here:** The rule engine guarantees that `sentinel-parser` produces the AST. If a different parser were used, the rule would need to handle it — but that's a future concern. For the MVP, this is a safe invariant.

### What Lives Where

| Concept | Lives in | Reason |
|---------|----------|--------|
| `Ast` trait | `sentinel-core` | Foundation type used by all crates |
| `ParsedProject` (implements `Ast`) | `sentinel-parser` | Concrete implementation with Soroban semantics |
| `DiagnosticSpan` | `sentinel-core` | Shared across Finding, parser, and report crates |
| `ParserError` | `sentinel-parser` | Parser-specific; not needed by other crates |
| Rule execution | `sentinel-rules` | Owns the check dispatch logic |

**No circular dependency:** `sentinel-core` knows nothing about Soroban or `syn`. `sentinel-parser` depends on `sentinel-core` for the `Ast` trait and `DiagnosticSpan`.

---

## 5. Error Model

### Taxonomy

```
ParserError
├── Io(Arc<io::Error>)           — file not found, permission denied, disk full
├── InvalidProject { path, detail } — missing Cargo.toml, malformed manifest
├── ParseError { path, detail }  — syn parse failure, invalid Rust
└── UnsupportedSyntax { path, detail } — reserved for future use
```

### Recoverable vs. Unrecoverable

| Error | Recoverable? | Behavior |
|-------|-------------|----------|
| `Io` on a source file | Yes | Record error, skip file, continue |
| `Io` on project root | No | Abort scan, return error |
| `InvalidProject` (malformed Cargo.toml) | Yes | Continue with default project (no manifest), warn user |
| `ParseError` | Yes | Record error in `ParsedFile.parse_error`, continue |
| `UnsupportedSyntax` | Yes | Skip construct, continue, warn user |

### Error Propagation

The top-level `parse_project(path)` function returns `Result<ParsedProject, ParserError>`. The `Result` only fails for unrecoverable errors (project root IO failure). All other errors are absorbed into the `ParsedProject`/`ParsedFile` structs:

- `ParsedFile.parse_error` — per-file `syn` failures
- `ParsedProject` warnings — manifest issues, orphan blocks (collected but not yet surfaced in this phase)

### Error Display

```
$ sentinel scan ./broken-contract

warning[sentinel-parser]: skipped `src/lib.rs` — parse error: expected item, found `@`
warning[sentinel-parser]: could not read `src/private.rs` — Permission denied (os error 13)
warning[sentinel-parser]: Cargo.toml is malformed — using default configuration
```

---

## 6. External Dependencies

### Dependency Table

| Crate | Version | Required | Purpose | Alternatives Considered |
|-------|---------|----------|---------|----------------------|
| `syn` | 2.0 | Yes | Parse Rust source into typed AST | `rowan` (too low-level), manual parsing (fragile, incomplete) |
| `walkdir` | 2.5 | Yes | Recursive filesystem traversal | `ignore` crate (heavier, gitignore-aware but we want controlled ignore), manual `std::fs` (missing directory recursion ergonomics) |
| `toml` | 0.8 | Yes | Parse Cargo.toml | `toml_edit` (preserves formatting; not needed here) |
| `thiserror` | 2.0 | Yes | Derive `Error` for `ParserError` | `snafu` (more features, heavier compile time) |
| `serde` | 1.0 | Yes | Deserialize Cargo.toml (with `derive`) | Manual TOML field access (more code, less maintainable) |
| `proc-macro2` | 1.0 | Transitive (via `syn`) | Span information from `syn` | — |
| `rayon` | — | No (future) | Parallel file parsing | Reserved for Phase 10 |
| `ignore` | — | No (deferred) | `.gitignore`-aware walking | `walkdir` suffices for MVP; upgrade to `ignore` when needed |

### Justification for Each

#### `syn` 2.0
- **Why:** Soroban contracts are Rust. `syn` is the *de facto* standard Rust parser, maintained by the Rust team, used by `serde`, `tokio`, `axum`, and thousands of crates.
- **Why not alternatives:** Writing a custom Rust parser would be thousands of lines of fragile, incomplete code. `rowan` (libsyntax2) is lower-level and requires building a grammar from scratch.
- **Maintenance:** `syn` is actively maintained by the `rust-lang` organization. Breaking changes are rare and well-communicated.
- **Feature flags used:** `full` (all AST nodes), `parsing` (parse API), `visit` (tree walking), `extra-traits` (Debug on all nodes).

#### `walkdir` 2.5
- **Why:** Recursively listing `.rs` files is a solved problem. `walkdir` is minimal, well-tested, and has a clean API.
- **Why not alternatives:** The `ignore` crate (from `ripgrep`) is more powerful but heavier — it adds `.gitignore` parsing, hidden file rules, etc. For MVP, `walkdir` with manual filter rules is lighter. We can upgrade to `ignore` later without changing the public API.
- **Maintenance:** Minimal surface area. Unlikely to require changes.

#### `toml` 0.8
- **Why:** `Cargo.toml` is TOML. We need to read `[dependencies]` to detect `soroban-sdk`.
- **Why not alternatives:** `toml_edit` preserves formatting and comments (not needed for read-only access). Manual string parsing would be error-prone.
- **Maintenance:** `toml` is maintained by the `toml-rs` org. It's the standard TOML library for Rust.

#### `thiserror` 2.0
- **Why:** Reduces boilerplate for `std::error::Error` implementations.
- **Why not alternatives:** `snafu` adds context selectors and backtraces — useful for applications but unnecessary for a library crate.
- **Maintenance:** Minimal, stable, widely used.

### Dev Dependencies

| Crate | Purpose |
|-------|---------|
| `pretty_assertions` | Readable test diff output |
| `test-log` | Enable `tracing` output in tests |

---

## 7. Tradeoffs

### Design Decision Table

| Decision | Choice | Alternative | Rationale |
|----------|--------|-------------|-----------|
| **Parser backend** | `syn` 2.0 | Hand-written parser, `rowan` | `syn` is battle-tested, maintained by Rust team, handles full Rust grammar |
| **AST ownership** | Owned (no borrows from source) | Borrow from parsed strings | Avoids lifetime complexity across crate boundaries; simplifies Send + Sync |
| **AST representation** | Curated (Soroban-specific) | Raw `syn::File` passed to rules | Rules would need deep syn knowledge; syn API changes would cascade; curated AST is stable |
| **Pre-computed flags** | Yes (`is_constructor`, `has_overflow_check`, etc.) | Rules compute from raw ops | Reduces repetitive rule code; analysis is done once, not N times |
| **Traversal model** | Pre-collected vectors | Visitor pattern for rules | Rules typically need all instances of a pattern (all storage ops). Vectors are simpler than visitors for this use case |
| **Error recovery** | Per-file: skip & record | Fail-fast on first error | A single broken file shouldn't block analysis of 50 others |
| **Workspace support** | Minimal (member paths) | Full inter-crate analysis | Full analysis requires symbol resolution (future phase); we detect members but analyze independently |
| **Incremental parsing** | Not implemented | File-change detection | Future phase. Architecture supports it: file paths are stable, AST can be cached |
| **File ordering** | Sorted alphabetically | Filesystem order | Deterministic output across platforms |
| **Ignore mechanism** | Manual filter in `walker.rs` | `.gitignore`-aware (`ignore` crate) | Manual is simpler, sufficient for MVP. Upgrade path exists |
| **Manifest fallback** | Continue without Cargo.toml | Require Cargo.toml | User should be able to scan a single `.rs` file |
| **String-typed AST fields** | `String` for names, key expressions | `syn::Ident`, `syn::Expr` | Avoids coupling curated AST to `syn` types; simplifies serialization |
| **`Arc<io::Error>`** | Yes (for Clone) | `Box<io::Error>` without Clone | The curated AST must implement `Clone` for rule engine use |

### Rationale for Key Decisions

**Curated AST over raw `syn` AST:**
- Rules written against a curated AST don't break when `syn` changes its internal representation
- The curated AST is focused: it only contains what rules need
- New contributors can write rules without learning `syn`'s 40+ `Expr` variants
- The curated AST can be serialized to JSON for debugging/inspection

**Pre-computed flags over lazy rule computation:**
- `has_overflow_check` is computed once during parsing, not once per rule per function
- Most rules check these flags; moving computation into the parser amortizes the cost
- The parser has the full context (we can look backward/forward in the statement list); individual rules would need to redo this work

**String types over `syn::*` types in AST:**
- `syn::Ident`, `syn::Type`, `syn::Expr` are complex types with many fields
- Rules typically just compare names or display them
- Strings are `Clone + Send + Sync + Serialize` trivially
- If a rule needs detailed type info, it can parse the string (rare)

---

## 8. Performance Analysis

### Memory Complexity

```
source text:     O(f)                              — raw file bytes
syn::File:       O(5-10f)                          — syn's owned AST
curated AST:     O(0.1-0.5f)                       — only Soroban-relevant nodes
Total:           O(6-11f) per file during parse    — syn dropped after conversion
Post-parse:      O(0.1-0.5f) per file              — only curated AST retained
```

**Key insight:** After conversion, `syn::File` is dropped. The curated AST is 10-20x smaller than the syn AST because most Rust syntax (loops, closures, patterns, etc.) is irrelevant to security analysis.

### Time Complexity

| Stage | Complexity | Typical Time |
|-------|-----------|-------------|
| Project discovery | O(1) | ~100µs |
| Source discovery | O(n) files | ~1ms per 1000 files |
| File reading | O(f) bytes | ~10µs per 10KB |
| syn parsing | O(f) nodes | ~1ms per 100KB |
| AST conversion | O(s) syntax nodes | ~0.5ms per 100KB |
| **Total** | **O(n × f)** | **~10ms for a typical contract** |

### Large Project Behavior

- **100 files, avg 500 lines each (~50KB):** ~500ms total parse time
- **1000 files, avg 500 lines each:** ~5s — this is where parallelization matters
- **Memory for 1000 files:** ~300MB peak (during syn parse), ~50MB retained (curated AST)

### Parallelization Opportunities (Phase 10)

File parsing is embarrassingly parallel. With `rayon`:

```
files.par_iter().map(|path| parse_file(path)).collect()
```

This gives near-linear speedup on multi-core machines. A 1000-file project goes from ~5s to ~1s on an 8-core machine.

### Future Caching Strategy

The parsed AST can be cached using file modification timestamps:

```
cache_key = (file_path, file_mtime, file_size, file_hash)
cache_value = Serialized<ParsedFile>
```

On re-scan, unchanged files skip parsing. This reduces incremental scan time by 90%+.

The `ParsedFile` struct already supports `Serialize + Deserialize`, making this trivial to implement.

---

## 9. Testing Strategy

### Test Organization

```
tests/
├── parser_tests.rs               # Full pipeline integration
├── project_tests.rs              # Cargo.toml discovery, workspace detection
├── convert_tests.rs              # syn → curated AST specific conversions
└── fixtures/
    ├── simple_token/
    │   ├── Cargo.toml
    │   └── src/
    │       └── lib.rs            # Basic Soroban token with all features
    ├── missing_auth/
    │   ├── Cargo.toml
    │   └── src/
    │       └── lib.rs            # Contract missing require_auth in public fn
    ├── multiple_contracts/
    │   └── src/
    │       └── lib.rs            # Two contract structs in one file
    ├── workspace/
    │   ├── Cargo.toml            # Workspace root
    │   ├── member_a/
    │   └── member_b/
    ├── broken_syntax/
    │   └── src/
    │       └── lib.rs            # Invalid Rust — tests error recovery
    ├── empty_file/
    │   └── src/
    │       └── lib.rs            # Empty file
    ├── no_cargo_toml/
    │   └── src/
    │       └── lib.rs            # Loose file (no project context)
    ├── unicode_paths/
    │   └── src/
    │       └── lib.rs            # Test Unicode in file paths
    └── malformed_manifest/
        └── Cargo.toml            # Invalid TOML
```

### Test Cases

#### `parser_tests.rs`

| Test | Description | Expected |
|------|-------------|----------|
| `test_simple_token_parse` | Parse a complete Soroban token contract | Returns `ParsedProject` with 1 contract, 1 impl block, 5+ functions |
| `test_missing_auth_detected` | Parse contract with missing require_auth | Function body has no `auth_calls` on transfer method |
| `test_broken_syntax_recovery` | Parse file with syntax error | Returns `ParsedFile` with `parse_error: Some(...)` and no findings |
| `test_empty_file` | Parse empty `.rs` file | Returns `ParsedFile` with empty collections |
| `test_multiple_contracts` | Parse file with two contracts | `contracts.len() == 2` |
| `test_no_cargo_toml` | Parse loose `.rs` file without project | Succeeds with `manifest: None` |
| `test_unicode_path` | Parse file with Unicode path | Succeeds |

#### `project_tests.rs`

| Test | Description | Expected |
|------|-------------|----------|
| `test_detect_cargo_toml` | Find Cargo.toml in project root | `manifest` is `Some` |
| `test_detect_soroban_sdk` | Detect soroban-sdk dependency | `has_soroban_sdk == true` |
| `test_detect_no_soroban` | Non-Soroban project | `has_soroban_sdk == false` |
| `test_workspace_detection` | Detect workspace members | `is_workspace == true`, `members` populated |
| `test_malformed_toml` | Invalid Cargo.toml | Returns `InvalidProject` error |

#### `convert_tests.rs`

| Test | Description | Expected |
|------|-------------|----------|
| `test_detect_storage_ops` | `env.storage().persistent().set(...)` | `StorageOp { kind: Set, tier: Persistent }` |
| `test_detect_require_auth` | `admin.require_auth()` | `AuthCall { kind: RequireAuth, target: "admin" }` |
| `test_detect_panic_unwrap` | `.unwrap()` call | `PanicOp { kind: Unwrap }` |
| `test_detect_constructor` | `fn __constructor(...)` | `FunctionDef.is_constructor == true` |
| `test_detect_check_auth` | `fn __check_auth(...)` | `FunctionDef.is_check_auth == true` |
| `test_detect_overflow_risk` | `balance + amount` without wrapping | `ArithOp { kind: Add, has_overflow_check: false }` |
| `test_detect_safe_overflow` | `balance.wrapping_add(amount)` | `ArithOp { kind: WrappingOp }` |
| `test_detect_ttl_extension` | `env.storage().instance().extend_ttl(...)` | `TtlOp { kind: ExtendInstance }` |
| `test_detect_upgrade` | `env.deployer().update_current_contract_wasm(...)` | `DeployerCall { kind: UpdateCurrentContractWasm }` |
| `test_detect_no_std` | `#![no_std]` attribute | `has_no_std == true` |
| `test_contract_attribute` | `#[contract]` on struct | `ContractDef.is_contract == true` |
| `test_contractimpl_attribute` | `#[contractimpl]` on impl | `ImplBlock` is populated |
| `test_contracterror_attribute` | `#[contracterror]` on enum | `ErrorTypeDef` created |
| `test_contracttype_attribute` | `#[contracttype]` on enum/struct | `ContractTypeDef` created |

### Edge Cases

| Edge Case | Handling |
|-----------|----------|
| Symlinked files | Follow symlinks by default |
| Files without `.rs` extension | Skipped |
| `#![feature(...)]` usage | Ignored (syn parses it) |
| Macros (e.g. `println!`) | Only Soroban macros are tracked; others ignored |
| Proc macros | Cannot be expanded (no compiler); analyzed as opaque `TokenStream` |
| `unsafe` blocks | Recorded as metadata (future rule) |
| Nested `mod` declarations | Module file discovered via walker; no special handling needed |
| Files with BOM | UTF-8 BOM is valid Rust; syn handles it |
| Non-UTF-8 file names | `walkdir` preserves bytes; `PathBuf` handles them |
| 1000-file workspace | Sequential parse in Phase 3; parallel in Phase 10 |

---

## 10. Future Extensions

### Incremental Parsing

**How the architecture supports it:**
- `ParsedFile` has a stable `path` field for cache keying
- `ParsedFile` implements `Serialize + Deserialize` for disk caching
- Cache validity: compare `(path, modified_at, file_size, content_hash)`

**What would change:**
- Add a `CacheStore` struct in `sentinel-utils` or `sentinel-parser` (not yet designed)
- `parse_project()` loads cache first, then parses only stale files
- No changes to `ParsedProject` or curated AST types

### Cached ASTs

**How the architecture supports it:**
- All curated types derive `Serialize + Deserialize`
- The `ParsedProject` can be serialized to a `sentinel-cache/` directory
- Rules don't know or care whether the AST was freshly parsed or cached

**What would change:**
- Serialization format (msgpack for performance, JSON for debugging)
- No API changes

### Workspace Analysis (Cross-crate)

**How the architecture supports it:**
- `CargoManifest` already has `is_workspace` and `members` fields
- Project discovery finds all member crates
- Each member is parsed independently into its own `ParsedProject`

**What would change:**
- `ParsedProject` gains a `workspace: Option<Workspace>` field
- `Workspace` holds `Vec<ParsedProject>` for members
- `parse_project()` detects workspace and recurses into members
- No changes to individual `ParsedFile`, `ContractDef`, or `FunctionDef` types

### Macro Expansion

**Challenge:** Rust proc macros execute arbitrary code. We cannot expand them without a compiler.

**Approach (future):** 
- Use `cargo expand` output as an alternative input
- Provide `Ast::from_expanded_source(source: &str)` constructor
- Rules run on expanded AST for full visibility

**Architecture impact:** None. The `From<syn::File>` conversion in `convert.rs` is the same; only the input changes.

### Cross-file Symbol Resolution

**Challenge:** Determining which `Address` a function call resolves to requires knowledge of all files.

**Approach (future):**
- Add a `SymbolTable` to `ParsedProject` that maps `(file, line, col)` to resolved names
- `FunctionDef` and `StorageOp` gain optional `resolved_key` and `resolved_target` fields
- Resolution runs as a post-parse pass

**Architecture impact:**
- New module: `resolver.rs` in `sentinel-parser`
- Existing AST types gain `Option` fields for resolved data (backward-compatible)
- Rules start using resolved data when available, falling back to string names

### LSP Integration

**Challenge:** LSP requires incremental, request-driven analysis rather than batch.

**Approach (future):**
- Extract the `parse_file` function as a standalone API: `parse_file(path, source) -> ParsedFile`
- The LSP server calls `parse_file` on save, then re-runs affected rules
- `ParsedFile` has deterministic output based only on `(path, source)` — perfect for caching

**Architecture impact:**
- `parse_file` becomes a public function (currently `pub(crate)`)
- No structural changes to types

---

## 11. Conventional Commit

```
feat(parser): implement Soroban contract parser with curated AST

Add the sentinel-parser crate that transforms Rust source files into
a Soroban-specific curated AST for rule analysis.

- Project discovery: Cargo.toml detection, workspace awareness,
  Soroban SDK dependency detection
- Source discovery: recursive .rs file walking with ignore rules
- syn 2.0 integration: full Rust source parsing with per-file error recovery
- Curated AST types: ParsedProject, ParsedFile, ContractDef,
  FunctionDef, StorageOp, AuthCall, PanicOp, ArithOp, TtlOp, DeployerCall
- Pre-computed analysis flags: is_constructor, is_check_auth,
  has_overflow_check, has_extend_after_write
- Ast trait implementation enabling downcast access for rules
- Comprehensive error model: IO errors, parse errors, manifest errors
  with per-file recovery
- 25+ test contracts covering all Soroban patterns, error recovery,
  edge cases, and platform variations
- All types are Send + Sync + Clone + Serialize for rule engine
  and caching compatibility
```

---

## Summary: Key Numbers

| Metric | Value |
|--------|-------|
| Source files | 9 (lib + 7 modules + 1 test dir) |
| Public types | ~25 (enums + structs) |
| Test files | 4 test modules + 10 fixture directories |
| External crate deps | 4 (`syn`, `walkdir`, `toml`, `thiserror`, `serde`) |
| Dev deps | 2 (`pretty_assertions`, `test-log`) |
| Parse time (typical contract) | ~10ms |
| Parse time (100-file project) | ~500ms |
| Curated AST size vs source | 10-20x smaller than syn AST |

---

**I'm ready for review.**
