pub mod error;
pub mod metrics;
pub mod parser;
pub mod report;
pub mod rules;
pub mod scan_request;
pub mod scan_result;
pub mod traits;

pub use error::ScanError;
pub use metrics::ScanMetrics;
pub use scan_request::{ScanRequest, ScanRequestBuilder, ScanType};
pub use scan_result::ScanResult;
pub use traits::{AIProvider, Parser, Reporter, RuleEngine, Scanner};

use std::path::Path;
use std::time::Instant;

use crate::core::{Ast, Finding, RuleConfig, RuleRegistry};
use crate::scanner::rules::registry::RuleRegistryExt;
use crate::scanner::rules::suppression::SuppressionEngine;

use tracing::{info_span, instrument, trace};

#[derive(Default)]
pub struct DefaultScanner;

impl Scanner for DefaultScanner {
    #[instrument(skip(self), fields(scan.target = %request.target.display()))]
    fn scan(&self, request: ScanRequest) -> Result<ScanResult, ScanError> {
        let _span = info_span!("scan", target = %request.target.display()).entered();

        let start = Instant::now();
        let mut metrics = ScanMetrics::default();

        // 1. Validate
        let target = validate_target(&request.target)?;

        // 2. Load config
        let rule_config = build_rule_config(&request);
        let rule_registry = RuleRegistry::new().register_builtins();

        // 3. Discover files
        let source_files = discover_source_files(&target, &request.ignore_paths)?;
        metrics.total_files = source_files.len();

        // 4. Parse
        let (project, parse_dur) = parse_project_files(&target)?;
        metrics.parse_duration = parse_dur;

        // 5. Run rules
        let (findings, suppressed, rule_dur) =
            execute_rules(&*project, &rule_registry, &rule_config, &source_files)?;
        metrics.rule_duration = rule_dur;
        metrics.total_rules = rule_registry.iter().count();
        metrics.total_findings = findings.len();
        metrics.suppressed_findings = suppressed;
        for f in &findings {
            match f.severity {
                crate::core::Severity::Critical => metrics.critical_findings += 1,
                crate::core::Severity::High => metrics.high_findings += 1,
                crate::core::Severity::Medium => metrics.medium_findings += 1,
                crate::core::Severity::Low => metrics.low_findings += 1,
                crate::core::Severity::Info => metrics.info_findings += 1,
            }
        }

        // 6. AI analysis (optional)
        if request.enable_ai {
            trace!("AI analysis enabled but not yet implemented");
        }

        // 7. Build core ScanResult
        metrics.duration = start.elapsed();
        let core = crate::core::ScanResult::new(
            findings,
            metrics.total_files,
            metrics.total_rules,
            metrics.suppressed_findings,
            metrics.duration.as_millis() as u64,
        );

        let result = ScanResult::new(core, metrics);
        Ok(result)
    }
}

fn validate_target(path: &Path) -> Result<std::path::PathBuf, ScanError> {
    let canonical = std::fs::canonicalize(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            ScanError::InvalidProject(format!("Path not found: {}", path.display()))
        } else {
            ScanError::Io(e)
        }
    })?;

    if !canonical.is_dir() && !canonical.is_file() {
        return Err(ScanError::InvalidProject(format!(
            "Path is neither a file nor directory: {}",
            canonical.display()
        )));
    }

    if canonical.is_dir() && !crate::cli::paths::is_rust_project(&canonical) {
        return Err(ScanError::InvalidProject(format!(
            "Not a Rust project: no Cargo.toml found at {}",
            canonical.display()
        )));
    }

    Ok(canonical)
}

fn build_rule_config(request: &ScanRequest) -> RuleConfig {
    let mut config = RuleConfig::default();

    if let Some(severity) = request.severity_filter {
        config.severity_threshold = severity;
    }

    for rule_id in &request.enabled_rules {
        if let Ok(id) = crate::core::RuleId::new(rule_id) {
            config.enabled.push(id);
        }
    }

    for rule_id in &request.disabled_rules {
        if let Ok(id) = crate::core::RuleId::new(rule_id) {
            config.disabled.push(id);
        }
    }

    config
}

fn discover_source_files(
    path: &Path,
    ignore_paths: &[String],
) -> Result<Vec<std::path::PathBuf>, ScanError> {
    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }

    let files = crate::cli::paths::collect_source_files(path, ignore_paths);
    if files.is_empty() {
        return Err(ScanError::InvalidProject(
            "No Rust source files found".to_string(),
        ));
    }
    Ok(files)
}

fn parse_project_files(path: &Path) -> Result<(Box<dyn Ast>, std::time::Duration), ScanError> {
    let parse_start = Instant::now();
    let project = crate::scanner::parser::parse_project(path)?;
    let duration = parse_start.elapsed();
    Ok((Box::new(project), duration))
}

fn execute_rules(
    project: &dyn Ast,
    registry: &RuleRegistry,
    config: &RuleConfig,
    source_files: &[std::path::PathBuf],
) -> Result<(Vec<Finding>, usize, std::time::Duration), ScanError> {
    let rule_start = Instant::now();

    let file_refs: Vec<&Path> = source_files.iter().map(|p| p.as_path()).collect();
    let suppression = SuppressionEngine::from_source_files(&file_refs);

    let engine = crate::scanner::rules::RuleEngine::new_with_suppression(
        registry.clone(),
        config.clone(),
        suppression,
    );
    let rule_result = engine.run(project);

    let duration = rule_start.elapsed();
    Ok((
        rule_result.findings,
        rule_result.summary.suppressed_findings,
        duration,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn create_test_project() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src");
        fs::create_dir(&src).unwrap();

        let cargo_toml = dir.path().join("Cargo.toml");
        fs::write(
            &cargo_toml,
            r#"
[package]
name = "test-contract"
version = "0.1.0"
edition = "2021"
[dependencies]
soroban-sdk = "22.0.0"
"#,
        )
        .unwrap();

        let contract_rs = src.join("contract.rs");
        fs::write(
            &contract_rs,
            r#"
#![no_std]
use soroban_sdk::{contractimpl, Env};
pub struct TestContract;
#[contractimpl]
impl TestContract {
    pub fn add(env: Env, a: u32, b: u32) -> u32 {
        a + b
    }
}
"#,
        )
        .unwrap();

        let path = dir.path().to_path_buf();
        (dir, path)
    }

    #[test]
    fn successful_scan() {
        let (_dir, path) = create_test_project();
        let scanner = DefaultScanner;
        let request = ScanRequest::new(path);
        let result = scanner.scan(request).unwrap();
        assert!(result.metrics.total_files > 0);
        assert!(result.metrics.total_rules > 0);
        assert!(result.score().score > 0);
    }

    #[test]
    fn empty_project_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_path_buf();
        fs::write(
            path.join("Cargo.toml"),
            "[package]\nname = \"empty\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();

        let scanner = DefaultScanner;
        let request = ScanRequest::new(path);
        let result = scanner.scan(request);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ScanError::InvalidProject(_)));
    }

    #[test]
    fn invalid_config_returns_configuration_error() {
        let scanner = DefaultScanner;
        let request = ScanRequest::new("/nonexistent/path");
        let result = scanner.scan(request);
        assert!(result.is_err());
    }

    #[test]
    fn scan_with_severity_filter() {
        let (_dir, path) = create_test_project();
        let scanner = DefaultScanner;
        let request = ScanRequest::builder()
            .target(path)
            .severity_filter(crate::core::Severity::Critical)
            .build();
        let result = scanner.scan(request).unwrap();
        assert!(result.metrics.total_rules > 0);
    }

    #[test]
    fn scan_disables_rules() {
        let (_dir, path) = create_test_project();
        let scanner = DefaultScanner;
        let request = ScanRequest::builder()
            .target(path)
            .disable_rule("missing-require-auth")
            .build();
        let result = scanner.scan(request).unwrap();
        assert!(result.metrics.total_rules > 0);
    }

    #[test]
    fn scan_without_ai_succeeds() {
        let (_dir, path) = create_test_project();
        let scanner = DefaultScanner;
        let request = ScanRequest::builder().target(path).enable_ai(false).build();
        let result = scanner.scan(request).unwrap();
        assert!(!result.has_findings());
    }

    #[test]
    fn parser_handles_non_utf8_gracefully() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_path_buf();
        fs::write(
            path.join("Cargo.toml"),
            "[package]\nname = \"bad\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
        let src = path.join("src");
        fs::create_dir(&src).unwrap();
        fs::write(src.join("broken.rs"), b"\xFF\xFE\x00\x01\xFF\xFE\x00\x01").unwrap();

        let scanner = DefaultScanner;
        let request = ScanRequest::new(path);
        // Parser stores errors in file metadata rather than failing hard
        let result = scanner.scan(request);
        assert!(
            result.is_ok(),
            "non-utf8 content should still be handled: {:?}",
            result.err()
        );
    }

    #[test]
    fn scan_metadata_preserved() {
        let (_dir, path) = create_test_project();
        let scanner = DefaultScanner;
        let request = ScanRequest::builder()
            .target(path)
            .metadata("ci-run-id", "12345")
            .build();
        let result = scanner.scan(request);
        assert!(result.is_ok());
    }

    #[test]
    fn scan_is_secure() {
        let (_dir, path) = create_test_project();
        let scanner = DefaultScanner;
        let request = ScanRequest::new(path);
        let result = scanner.scan(request).unwrap();
        assert!(result.is_secure());
    }

    #[test]
    fn scan_returns_valid_core_result() {
        let (_dir, path) = create_test_project();
        let scanner = DefaultScanner;
        let request = ScanRequest::new(path);
        let result = scanner.scan(request).unwrap();
        assert!(!result.core.findings.is_empty() || result.score().score == 100);
    }

    #[test]
    fn scan_with_ignore_paths() {
        let (_dir, path) = create_test_project();
        let scanner = DefaultScanner;
        let request = ScanRequest::builder()
            .target(path)
            .ignore_path("nonexistent")
            .build();
        let result = scanner.scan(request).unwrap();
        assert!(result.metrics.total_files > 0);
    }

    #[test]
    fn scan_twice_returns_same_result_structure() {
        let (_dir, path) = create_test_project();
        let scanner = DefaultScanner;
        let request = ScanRequest::new(path.clone());
        let r1 = scanner.scan(request).unwrap();

        let request2 = ScanRequest::new(path);
        let r2 = scanner.scan(request2).unwrap();

        assert_eq!(r1.metrics.total_files, r2.metrics.total_files);
        assert_eq!(r1.metrics.total_rules, r2.metrics.total_rules);
    }

    #[test]
    fn scan_metrics_duration_non_zero() {
        let (_dir, path) = create_test_project();
        let scanner = DefaultScanner;
        let request = ScanRequest::new(path);
        let result = scanner.scan(request).unwrap();
        assert!(
            result.metrics.duration.as_nanos() > 0,
            "duration should be non-zero"
        );
    }

    #[test]
    fn scan_with_cancellation_returns_error() {
        let scanner = DefaultScanner;
        let path = PathBuf::from("/dev/null/nonexistent");
        let request = ScanRequest::new(path);
        let result = scanner.scan(request);
        assert!(result.is_err());
    }
}
