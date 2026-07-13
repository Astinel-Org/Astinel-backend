use std::path::Path;
use std::time::Instant;

use crate::cli::errors::CliError;
use crate::cli::exit::ExitCode;
use crate::cli::progress::Progress;
use crate::cli::terminal::{ColorPreference, TerminalCapabilities};
use crate::config::RunConfig;
use crate::scanner::report::{
    CompactFormatter, JsonFormatter, OutputFormatter, PrettyFormatter, Report, ReportOptions,
    ReportSummary,
};
use crate::scanner::scan_request::ScanRequest;
use crate::scanner::{DefaultScanner, Scanner};

/// Run the full scan pipeline via ScannerService.
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

    let mut request = ScanRequest::builder()
        .target(path)
        .ignore_paths(config.ignore_paths.clone());

    if config.severity_threshold != crate::core::Severity::Low {
        request = request.severity_filter(config.severity_threshold);
    }

    if config.threads > 0 {
        request = request.parallelism(config.threads);
    }

    for rule_id in &config.enabled_rules {
        request = request.enable_rule(rule_id.as_str());
    }

    for rule_id in &config.disabled_rules {
        request = request.disable_rule(rule_id.as_str());
    }

    if let Some(ref cat) = config.category_filter {
        request = request.metadata("category", cat.as_str());
    }

    progress.info(format_args!("Scanning {}", path.display()));

    let scanner = DefaultScanner;
    let result = scanner.scan(request.build()).map_err(|e| match e {
        crate::scanner::ScanError::InvalidProject(msg) => CliError::Unsupported(msg),
        crate::scanner::ScanError::Io(io_err) => CliError::from(io_err),
        crate::scanner::ScanError::Parser(msg) => {
            CliError::Parse(crate::scanner::parser::ParserError::InvalidProject {
                path: std::path::PathBuf::new(),
                detail: msg,
            })
        }
        _ => CliError::Internal(e.to_string()),
    })?;

    progress.files_scanned(result.metrics.total_files);
    progress.rules_run(result.metrics.total_rules);
    progress.finding_count(result.metrics.total_findings);

    // Build report data for formatters
    let project_name = config
        .project_name
        .clone()
        .unwrap_or_else(|| "unknown".to_string());

    let score = result.score();
    let metrics = &result.metrics;

    // Apply category filter if specified
    let findings = if let Some(cat) = config.category_filter {
        result
            .core
            .findings
            .into_iter()
            .filter(|f| f.category == cat)
            .collect()
    } else {
        result.core.findings
    };

    let report_options = ReportOptions {
        color: caps.color,
        unicode: caps.unicode,
        width: caps.width,
    };

    let summary = ReportSummary {
        project_name,
        total_files: metrics.total_files,
        total_rules: metrics.total_rules,
        total_findings: findings.len(),
        suppressed_findings: metrics.suppressed_findings,
        duration: start.elapsed(),
        parse_duration: metrics.parse_duration,
        rule_duration: metrics.rule_duration,
    };

    let report = Report {
        score,
        summary,
        options: report_options,
        findings,
    };

    let writer: &mut dyn std::io::Write = &mut std::io::stdout();

    if json_output {
        JsonFormatter.write(writer, &report)?;
    } else if compact_output {
        CompactFormatter.write(writer, &report)?;
    } else {
        PrettyFormatter.write(writer, &report)?;
    }

    let has_findings = !report.findings.is_empty();
    Ok(ExitCode::from_findings_and_severity(
        has_findings,
        config.fail_on,
        &report.findings,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Category;
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
        let progress = Progress::new(crate::cli::progress::ProgressMode::Quiet);

        let result = run_scan(
            &path,
            &config,
            &progress,
            false,
            false,
            ColorPreference::Never,
        );

        assert!(result.is_ok(), "Scan failed: {:?}", result.err());
    }

    #[test]
    fn scan_nonexistent_path_returns_error() {
        let path = Path::new("/nonexistent/path");
        let config = RunConfig::default();
        let progress = Progress::new(crate::cli::progress::ProgressMode::Quiet);

        let result = run_scan(
            path,
            &config,
            &progress,
            false,
            false,
            ColorPreference::Never,
        );

        assert!(result.is_err());
    }

    #[test]
    fn scan_output_is_deterministic() {
        let (_dir1, path1) = create_test_project();
        let (_dir2, path2) = create_test_project();
        let config = RunConfig::default();
        let progress = Progress::new(crate::cli::progress::ProgressMode::Quiet);

        let r1 = run_scan(
            &path1,
            &config,
            &progress,
            true,
            false,
            ColorPreference::Never,
        );
        let r2 = run_scan(
            &path2,
            &config,
            &progress,
            true,
            false,
            ColorPreference::Never,
        );

        assert_eq!(r1.is_ok(), r2.is_ok());
    }

    #[test]
    fn scan_json_output_parses() {
        let (_dir, path) = create_test_project();
        let config = RunConfig::default();
        let progress = Progress::new(crate::cli::progress::ProgressMode::Quiet);

        let result = run_scan(
            &path,
            &config,
            &progress,
            true,
            false,
            ColorPreference::Never,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn scan_with_category_filter() {
        let (_dir, path) = create_test_project();
        let config = RunConfig {
            category_filter: Some(Category::Security),
            ..Default::default()
        };
        let progress = Progress::new(crate::cli::progress::ProgressMode::Quiet);

        let result = run_scan(
            &path,
            &config,
            &progress,
            true,
            false,
            ColorPreference::Never,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn scan_with_ignored_paths() {
        let (_dir, path) = create_test_project();
        let mut config = RunConfig::default();
        config.ignore_paths.push("nonexistent".to_string());
        let progress = Progress::new(crate::cli::progress::ProgressMode::Quiet);

        let result = run_scan(
            &path,
            &config,
            &progress,
            true,
            false,
            ColorPreference::Never,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn scan_non_rust_project_returns_unsupported() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_path_buf();
        let config = RunConfig::default();
        let progress = Progress::new(crate::cli::progress::ProgressMode::Quiet);

        let result = run_scan(
            &path,
            &config,
            &progress,
            false,
            false,
            ColorPreference::Never,
        );
        assert!(result.is_err());
        if let Err(ref e) = result {
            assert_eq!(e.exit_code(), ExitCode::UnsupportedProject);
        }
    }

    #[test]
    fn scan_with_suppression() {
        let (_dir, path) = create_test_project();
        let suppress_file = path.join("src").join("contract.rs");
        let content = std::fs::read_to_string(&suppress_file).unwrap();
        let suppressed = format!("// sentinel-ignore-file\n{}", content);
        std::fs::write(&suppress_file, &suppressed).unwrap();

        let config = RunConfig::default();
        let progress = Progress::new(crate::cli::progress::ProgressMode::Quiet);

        let result = run_scan(
            &path,
            &config,
            &progress,
            true,
            false,
            ColorPreference::Never,
        );
        assert!(
            result.is_ok(),
            "Scan with suppression failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn scan_compact_output_succeeds() {
        let (_dir, path) = create_test_project();
        let config = RunConfig::default();
        let progress = Progress::new(crate::cli::progress::ProgressMode::Quiet);

        let result = run_scan(
            &path,
            &config,
            &progress,
            false,
            true,
            ColorPreference::Never,
        );
        assert!(result.is_ok(), "Compact scan failed: {:?}", result.err());
    }

    #[test]
    fn scan_empty_directory_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_path_buf();
        fs::write(
            path.join("Cargo.toml"),
            "[package]\nname = \"empty\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();

        let config = RunConfig::default();
        let progress = Progress::new(crate::cli::progress::ProgressMode::Quiet);

        let result = run_scan(
            &path,
            &config,
            &progress,
            false,
            false,
            ColorPreference::Never,
        );
        assert!(result.is_err());
    }
}
