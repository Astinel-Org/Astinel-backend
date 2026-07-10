# First Commit Proposal

## Proposed Commit Message

```
feat: initialize Sentinel workspace and crate scaffolding

Set up the Cargo workspace with 8 crate stubs, Rust toolchain
configuration, formatting rules, CI pipeline, and foundational
documentation (README, LICENSE, CONTRIBUTING, SECURITY).

This commit establishes the project skeleton that all subsequent
phases will build upon. No functional code yet — only structure
and tooling configuration.
```

## What's Included

### Root Files
- `Cargo.toml` — workspace definition with 8 members
- `rust-toolchain.toml` — pin to stable (1.96.0)
- `rustfmt.toml` — 4-space indent, edition 2024 style
- `.gitignore` — `target/`, editor files, OS artifacts
- `.gitattributes` — Rust files marked linguist-vendored

### Crates (8 stubs)
Each crate has a minimal `Cargo.toml` and `src/lib.rs`:

| Crate | Dependency role |
|-------|----------------|
| `sentinel-core` | No sentinel deps (foundation) |
| `sentinel-utils` | No sentinel deps (utilities) |
| `sentinel-config` | Depends on `sentinel-core` |
| `sentinel-parser` | Depends on `sentinel-core`, `sentinel-utils` |
| `sentinel-rules` | Depends on `sentinel-core`, `sentinel-parser` |
| `sentinel-report` | Depends on `sentinel-core`, `sentinel-utils` |
| `sentinel-ai` | Depends on `sentinel-core` (stub) |
| `sentinel-cli` | Depends on all others |

### Documentation
- `README.md` — placeholder with badges and description
- `CONTRIBUTING.md` — contribution guidelines
- `SECURITY.md` — vulnerability disclosure policy
- `CHANGELOG.md` — initial changelog entry

### CI
- `.github/workflows/ci.yml` — runs `fmt`, `clippy`, `test`, `doc`
- `.github/PULL_REQUEST_TEMPLATE.md`

### License
- `LICENSE` — Apache 2.0

## What's NOT Included (Deferred)

- Any implementation code (all `src/lib.rs` are empty with `#![allow(unused)]`)
- Tests, examples, benchmarks (added in relevant phases)
- mdBook configuration (added in documentation phase)
- `sentinel-ai` implementation (post-MVP)

## Rationale

This commit is intentionally minimal:
1. **Reviewable** — 20 files, none exceeding 50 lines of meaningful content
2. **Compilable** — `cargo build` succeeds immediately (no missing deps)
3. **Green CI** — all checks pass from the first commit
4. **Forkable** — anyone can fork and start contributing in any crate
5. **Historical** — clean baseline for `git blame` going forward

---

**Action:** Respond with "approve" to proceed with creating the scaffolding.
