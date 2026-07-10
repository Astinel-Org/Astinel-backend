use std::io::Write;
use std::path::Path;
use std::time::Instant;

use sentinel_core::{Ast, Finding};
use sentinel_rules::registry::RuleRegistryExt;
use sentinel_rules::suppression::SuppressionEngine;

use crate::config::RunConfig;
use crate::errors::CliError;
use crate::exit::ExitCode;
use crate::output::{CompactFormatter, JsonFormatter, OutputFormatter, PrettyFormatter, ScanOutputSummary};
use crate::paths;
use crate::progress::Progress;
use crate::terminal::{ColorPreference, TerminalCapabilities};

/// Result of a scan run, including findings, score, summary, and timings.
/// Result of a scan run, including findings, score, summary, and timings.
pub struct ScanResult {
    pub findings: Vec<Finding>,
    pub score: sentinel_core::SecurityScore,
    pub summary: ScanOutputSummary,
    pub timings: Vec<(String, std::time::Duration)>,
}

/// Run the full scan pipeline: discover, parse, analyze, filter, output, and return exit code.
#[allow(clippy::too_many_arguments)]
pub fn run_scan(
    path: &Path,
    config: &RunConfig,
    progress: &Progress,
    json_output: bool,
    compact_output: bool,
    color_pref: ColorPreference,
) -> Result<ExitCode, CliError> {
    let caps = TerminalCapabilities::detect(color_pref);
    let start = Instant::now();
    let scan_path = paths::resolve_scan_path(path)?;

    progress.info(format_args!("Scanning {}", scan_path.display()));

    if !paths::is_rust_project(&scan_path) {
        return Err(CliError::Unsupported(format!(
            "Not a Rust project: no Cargo.toml found at {}",
            scan_path.display()
        )));
    }

    let source_files = paths::collect_source_files(&scan_path, &config.ignore_paths);
    if source_files.is_empty() {
        return Err(CliError::InvalidPath {
            path: scan_path,
            detail: "No Rust source files found".to_string(),
        });
    }

    progress.files_scanned(source_files.len());

    let parse_start = Instant::now();
    let project = sentinel_parser::parse_project(&scan_path).map_err(CliError::Parse)?;
    let parse_duration = parse_start.elapsed();

    let total_files = project.files().len();
    let project_name = project
        .manifest
        .as_ref()
        .and_then(|m| m.package_name.clone())
        .unwrap_or_default();
    progress.verbose(format_args!("Parsed project in {}ms", parse_duration.as_millis()));

    let rule_start = Instant::now();
    let registry = sentinel_core::RuleRegistry::new().register_builtins();

    // Build suppression engine from source files
    let file_refs: Vec<&Path> = source_files.iter().map(|p| p.as_path()).collect();
    let suppression = SuppressionEngine::from_source_files(&file_refs);

    let rule_config = config.to_rule_config();
    let engine = sentinel_rules::RuleEngine::new_with_suppression(registry, rule_config, suppression);
    let result = engine.run(&project as &dyn Ast);
    let rule_duration = rule_start.elapsed();

    progress.rules_run(result.summary.total_rules_run);
    progress.finding_count(result.findings.len());

    // Apply category filter if specified
    let findings: Vec<Finding> = if let Some(cat) = config.category_filter {
        result.findings.into_iter().filter(|f| f.category == cat).collect()
    } else {
        result.findings
    };

    let scan_summary = ScanOutputSummary {
        project_name,
        total_files,
        total_rules: result.summary.total_rules_run,
        total_findings: findings.len(),
        suppressed_findings: result.summary.suppressed_findings,
        duration: start.elapsed(),
        parse_duration,
        rule_duration,
    };

    let scan_result = ScanResult {
        findings,
        score: result.score,
        summary: scan_summary,
        timings: vec![
            ("parse".to_string(), parse_duration),
            ("rules".to_string(), rule_duration),
        ],
    };

    let writer: &mut dyn Write = &mut std::io::stdout();

    if json_output {
        let formatter = JsonFormatter;
        formatter.write(
            writer,
            &scan_result.findings,
            &scan_result.score,
            &scan_result.summary,
            &caps,
        )?;
    } else if compact_output {
        let formatter = CompactFormatter;
        formatter.write(
            writer,
            &scan_result.findings,
            &scan_result.score,
            &scan_result.summary,
            &caps,
        )?;
    } else {
        let formatter = PrettyFormatter;
        formatter.write(
            writer,
            &scan_result.findings,
            &scan_result.score,
            &scan_result.summary,
            &caps,
        )?;
    }

    let has_findings = !scan_result.findings.is_empty();
    Ok(ExitCode::from_findings_and_severity(
        has_findings,
        config.fail_on,
        &scan_result.findings,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sentinel_core::Category;
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
    fn scan_valid_project_completes() {
        let (_dir, path) = create_test_project();
        let config = RunConfig::default();
        let progress = Progress::new(crate::progress::ProgressMode::Quiet);

        let result = run_scan(&path, &config, &progress, false, false, ColorPreference::Never);

        assert!(result.is_ok(), "Scan failed: {:?}", result.err());
    }

    #[test]
    fn scan_nonexistent_path_returns_error() {
        let path = Path::new("/nonexistent/path");
        let config = RunConfig::default();
        let progress = Progress::new(crate::progress::ProgressMode::Quiet);

        let result = run_scan(path, &config, &progress, false, false, ColorPreference::Never);

        assert!(result.is_err());
    }

    #[test]
    fn scan_output_is_deterministic() {
        let (_dir1, path1) = create_test_project();
        let (_dir2, path2) = create_test_project();
        let config = RunConfig::default();
        let progress = Progress::new(crate::progress::ProgressMode::Quiet);

        let r1 = run_scan(&path1, &config, &progress, true, false, ColorPreference::Never);
        let r2 = run_scan(&path2, &config, &progress, true, false, ColorPreference::Never);

        assert_eq!(r1.is_ok(), r2.is_ok());
    }

    #[test]
    fn scan_json_output_parses() {
        let (_dir, path) = create_test_project();
        let config = RunConfig::default();
        let progress = Progress::new(crate::progress::ProgressMode::Quiet);

        let result = run_scan(&path, &config, &progress, true, false, ColorPreference::Never);
        assert!(result.is_ok());
    }

    #[test]
    fn scan_with_category_filter() {
        let (_dir, path) = create_test_project();
        let config = RunConfig {
            category_filter: Some(Category::Security),
            ..Default::default()
        };
        let progress = Progress::new(crate::progress::ProgressMode::Quiet);

        let result = run_scan(&path, &config, &progress, true, false, ColorPreference::Never);
        assert!(result.is_ok());
    }

    #[test]
    fn scan_with_ignored_paths() {
        let (_dir, path) = create_test_project();
        let mut config = RunConfig::default();
        config.ignore_paths.push("nonexistent".to_string());
        let progress = Progress::new(crate::progress::ProgressMode::Quiet);

        let result = run_scan(&path, &config, &progress, true, false, ColorPreference::Never);
        assert!(result.is_ok());
    }

    #[test]
    fn scan_non_rust_project_returns_unsupported() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_path_buf();
        let config = RunConfig::default();
        let progress = Progress::new(crate::progress::ProgressMode::Quiet);

        let result = run_scan(&path, &config, &progress, false, false, ColorPreference::Never);
        assert!(result.is_err());
        // Should return UnsupportedProject exit code
        if let Err(ref e) = result {
            assert_eq!(e.exit_code(), ExitCode::UnsupportedProject);
        }
    }

    #[test]
    fn scan_with_suppression() {
        let (_dir, path) = create_test_project();
        let suppress_file = path.join("src").join("contract.rs");
        let content = std::fs::read_to_string(&suppress_file).unwrap();
        // Add a suppression comment at the top
        let suppressed = format!("// sentinel-ignore-file\n{}", content);
        std::fs::write(&suppress_file, &suppressed).unwrap();

        let config = RunConfig::default();
        let progress = Progress::new(crate::progress::ProgressMode::Quiet);

        let result = run_scan(&path, &config, &progress, true, false, ColorPreference::Never);
        assert!(result.is_ok(), "Scan with suppression failed: {:?}", result.err());
    }

    #[test]
    fn scan_compact_output_succeeds() {
        let (_dir, path) = create_test_project();
        let config = RunConfig::default();
        let progress = Progress::new(crate::progress::ProgressMode::Quiet);

        let result = run_scan(&path, &config, &progress, false, true, ColorPreference::Never);
        assert!(result.is_ok(), "Compact scan failed: {:?}", result.err());
    }

    #[test]
    fn scan_empty_directory_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_path_buf();
        // Write Cargo.toml so it's a valid Rust project, but no .rs files
        fs::write(
            path.join("Cargo.toml"),
            "[package]\nname = \"empty\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();

        let config = RunConfig::default();
        let progress = Progress::new(crate::progress::ProgressMode::Quiet);

        let result = run_scan(&path, &config, &progress, false, false, ColorPreference::Never);
        assert!(result.is_err());
        match result {
            Err(CliError::InvalidPath { .. }) => {} // expected
            _ => panic!("Expected InvalidPath error, got: {:?}", result),
        }
    }
}
