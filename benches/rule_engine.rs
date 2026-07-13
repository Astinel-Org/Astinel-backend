use astinel_backend::core::RuleRegistry;
use astinel_backend::scanner::rules::registry::RuleRegistryExt;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn bench_rule_registry(c: &mut Criterion) {
    c.bench_function("rules/registry_all", |b| {
        b.iter(|| {
            let _registry = RuleRegistry::new().register_builtins();
        });
    });
}

fn bench_rule_engine_empty(c: &mut Criterion) {
    let registry = RuleRegistry::new().register_builtins();
    let config = astinel_backend::core::RuleConfig::default();
    let engine = astinel_backend::scanner::rules::RuleEngine::new(registry, config);
    let project_path = workspace_root().join("examples/secure-contract");
    let project =
        astinel_backend::scanner::parser::parse_project(black_box(&project_path)).unwrap();

    c.bench_function("rules/engine_scan_secure", |b| {
        b.iter(|| {
            let _result = engine.run(black_box(&project as &dyn astinel_backend::core::Ast));
        });
    });
}

fn bench_rule_engine_vulnerable(c: &mut Criterion) {
    let registry = RuleRegistry::new().register_builtins();
    let config = astinel_backend::core::RuleConfig::default();
    let engine = astinel_backend::scanner::rules::RuleEngine::new(registry, config);
    let project_path = workspace_root().join("examples/missing-require-auth");
    let project =
        astinel_backend::scanner::parser::parse_project(black_box(&project_path)).unwrap();

    c.bench_function("rules/engine_scan_vulnerable", |b| {
        b.iter(|| {
            let _result = engine.run(black_box(&project as &dyn astinel_backend::core::Ast));
        });
    });
}

criterion_group!(
    benches,
    bench_rule_registry,
    bench_rule_engine_empty,
    bench_rule_engine_vulnerable
);
criterion_main!(benches);
