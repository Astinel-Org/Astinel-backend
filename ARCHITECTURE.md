# Sentinel Architecture

## Overview

```
┌─────────────────────────────────────────────────────────┐
│                     sentinel-cli                         │
│  (clap, exit codes, stdout/stderr, color output)         │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│                    sentinel-config                       │
│  (sentinel.toml parsing, config merging, defaults)       │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│                   Project Loader                         │
│  (built into sentinel-core/cli — walks filesystem,      │
│   discovers .rs files, respects ignore lists)            │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│                   sentinel-parser                        │
│  (syn-based Rust parser → curated Soroban AST)          │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
                    AST
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│                   sentinel-rules                         │
│  (Rule trait, RuleRegistry, 10+ built-in rules)          │
│                                                          │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐              │
│  │ Rule #1  │  │ Rule #2  │  │ Rule #N  │              │
│  └──────────┘  └──────────┘  └──────────┘              │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│                   sentinel-core                          │
│  (Finding, ScanResult, Score computation)                │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│                   sentinel-report                        │
│  (Markdown, JSON, HTML output)                           │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
            ┌────────┴────────┐
            │     Output      │
            │  (stdout/file)  │
            └─────────────────┘
```

## Crate Dependency Graph

```
sentinel-cli
  ├── sentinel-core
  ├── sentinel-config
  ├── sentinel-parser
  ├── sentinel-rules
  ├── sentinel-report
  └── sentinel-utils

sentinel-report
  ├── sentinel-core
  └── sentinel-utils

sentinel-rules
  ├── sentinel-core
  └── sentinel-parser

sentinel-parser
  ├── sentinel-core
  └── sentinel-utils

sentinel-config
  ├── sentinel-core
  └── sentinel-utils

sentinel-ai
  └── sentinel-core

sentinel-core     (no sentinel dependencies)
sentinel-utils    (no sentinel dependencies)
```

## Data Flow

1. User invokes `sentinel scan [path]`
2. CLI parses args → loads config from `sentinel.toml` (or defaults)
3. Project loader discovers `.rs` files (skipping `target/`, `.git/`, etc.)
4. Parser converts each file into an `Ast` node
5. Rule engine filters eligible rules from config
6. Each rule runs against the AST → produces `Vec<Finding>`
7. Findings are collected, deduplicated, and filtered by severity threshold
8. Security score is computed from remaining findings
9. Reports are generated in requested format(s)
10. Results printed to stdout / written to files

## Design Decisions

### Why not AI in the analysis pipeline?
AI is non-deterministic. Sentinel guarantees reproducible results. AI is only used in the optional `sentinel-ai` crate to *explain* findings or suggest fixes after deterministic analysis completes.

### Why curated AST instead of raw syn AST?
A curated Soroban AST is smaller, focused, and stable across Rust versions. Rules operate on `FunctionCall`, `StorageOp`, `Attribute` etc. rather than raw `syn::Item`.

### Why workspace of many small crates?
- Fast compilation: changes to rules don't recompile the parser
- Clear ownership: each crate has a single responsibility
- Testability: small surfaces are easier to mock
- Future publishing: users depend only on what they need

### Why no async?
No I/O except filesystem reads and optional AI API calls. The analysis pipeline is synchronous, which is simpler, faster, and more predictable.
