use astinel_backend::core::{Category, DiagnosticSpan, Finding, RuleId, SecurityScore, Severity};
use astinel_backend::scanner::report::{
    CompactFormatter, JsonFormatter, MarkdownFormatter, OutputFormatter, PrettyFormatter, Report,
    ReportOptions, ReportSummary, SarifFormatter,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;

fn make_findings(count: usize) -> Vec<Finding> {
    let severities = [
        Severity::Critical,
        Severity::High,
        Severity::Medium,
        Severity::Low,
        Severity::Info,
    ];
    (0..count)
        .map(|i| {
            Finding::new(
                RuleId::new(format!("rule-{}", i % 10)).unwrap(),
                severities[i % 5],
                Category::Security,
                DiagnosticSpan::new(format!("src/file{}.rs", i % 5), (i % 100) + 1, (i % 10) + 1),
                format!("Finding number {}", i),
                "Fix this issue by following best practices",
            )
        })
        .collect()
}

fn make_report(findings: Vec<Finding>) -> Report {
    let score = SecurityScore::from_findings(&findings);
    let summary = ReportSummary {
        project_name: "test-project".to_string(),
        total_files: 10,
        total_rules: 10,
        total_findings: findings.len(),
        suppressed_findings: 0,
        duration: Duration::from_millis(100),
        parse_duration: Duration::from_millis(20),
        rule_duration: Duration::from_millis(70),
    };
    Report {
        findings,
        score,
        summary,
        options: ReportOptions {
            color: false,
            unicode: false,
            width: 80,
        },
    }
}

fn bench_pretty_10(c: &mut Criterion) {
    let report = make_report(make_findings(10));
    c.bench_function("format/pretty_10", |b| {
        b.iter(|| PrettyFormatter.format(black_box(&report)));
    });
}

fn bench_pretty_100(c: &mut Criterion) {
    let report = make_report(make_findings(100));
    c.bench_function("format/pretty_100", |b| {
        b.iter(|| PrettyFormatter.format(black_box(&report)));
    });
}

fn bench_json_10(c: &mut Criterion) {
    let report = make_report(make_findings(10));
    c.bench_function("format/json_10", |b| {
        b.iter(|| JsonFormatter.format(black_box(&report)));
    });
}

fn bench_json_100(c: &mut Criterion) {
    let report = make_report(make_findings(100));
    c.bench_function("format/json_100", |b| {
        b.iter(|| JsonFormatter.format(black_box(&report)));
    });
}

fn bench_sarif_10(c: &mut Criterion) {
    let report = make_report(make_findings(10));
    c.bench_function("format/sarif_10", |b| {
        b.iter(|| SarifFormatter.format(black_box(&report)));
    });
}

fn bench_markdown_10(c: &mut Criterion) {
    let report = make_report(make_findings(10));
    c.bench_function("format/markdown_10", |b| {
        b.iter(|| MarkdownFormatter.format(black_box(&report)));
    });
}

fn bench_compact_10(c: &mut Criterion) {
    let report = make_report(make_findings(10));
    c.bench_function("format/compact_10", |b| {
        b.iter(|| CompactFormatter.format(black_box(&report)));
    });
}

criterion_group!(
    benches,
    bench_pretty_10,
    bench_pretty_100,
    bench_json_10,
    bench_json_100,
    bench_sarif_10,
    bench_markdown_10,
    bench_compact_10
);
criterion_main!(benches);
