use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn bench_parser_small(c: &mut Criterion) {
    let project_path = workspace_root().join("examples/secure-contract");
    c.bench_function("parser/small_project", |b| {
        b.iter(|| {
            let _ = astinel_backend::scanner::parser::parse_project(black_box(&project_path));
        });
    });
}

fn bench_parser_medium(c: &mut Criterion) {
    let project_path = workspace_root().join("crates/sentinel-core");
    c.bench_function("parser/medium_project", |b| {
        b.iter(|| {
            let _ = astinel_backend::scanner::parser::parse_project(black_box(&project_path));
        });
    });
}

criterion_group!(benches, bench_parser_small, bench_parser_medium);
criterion_main!(benches);
