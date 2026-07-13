use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn bench_full_scan_secure(c: &mut Criterion) {
    c.bench_function("full_scan/secure_contract", |b| {
        b.iter(|| {
            let project_path = workspace_root().join("examples/secure-contract");
            let project =
                astinel_backend::scanner::parser::parse_project(black_box(&project_path)).unwrap();
            let registry = astinel_backend::core::RuleRegistry::new().register_builtins();
            let config = astinel_backend::core::RuleConfig::default();
            let engine = astinel_backend::scanner::rules::RuleEngine::new(registry, config);
            let result = engine.run(&project as &dyn astinel_backend::core::Ast);
            let score = astinel_backend::core::SecurityScore::from_findings(&result.findings);
            let summary = astinel_backend::scanner::report::ReportSummary {
                project_name: "secure".to_string(),
                total_files: 1,
                total_rules: result.summary.total_rules_run,
                total_findings: result.findings.len(),
                suppressed_findings: result.summary.suppressed_findings,
                duration: std::time::Duration::ZERO,
                parse_duration: std::time::Duration::ZERO,
                rule_duration: std::time::Duration::ZERO,
            };
            let report = astinel_backend::scanner::report::Report {
                findings: result.findings,
                score,
                summary,
                options: astinel_backend::scanner::report::ReportOptions::default(),
            };
            use astinel_backend::scanner::report::OutputFormatter;
            let _json = astinel_backend::scanner::report::JsonFormatter.format(&report);
        });
    });
}

use astinel_backend::scanner::rules::registry::RuleRegistryExt;

criterion_group!(benches, bench_full_scan_secure);
criterion_main!(benches);
