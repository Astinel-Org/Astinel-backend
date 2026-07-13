use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::core::Category;
use crate::core::RuleId;
use crate::core::Severity;

use crate::config::error::ConfigError;

// ---------------------------------------------------------------------------
// TOML data model
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ConfigFile {
    #[serde(default)]
    pub scan: Option<ScanSection>,
    #[serde(default)]
    pub rules: Option<RulesSection>,
    #[serde(default)]
    pub severity: Option<SeveritySection>,
    #[serde(default)]
    pub output: Option<OutputSection>,
    #[serde(default)]
    pub performance: Option<PerformanceSection>,
    #[serde(default)]
    pub ignore: Option<IgnoreSection>,
    #[serde(default)]
    #[allow(dead_code)]
    pub experimental: Option<toml::Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ScanSection {
    #[serde(default)]
    pub threads: Option<usize>,
    #[serde(default)]
    pub exclude: Option<Vec<String>>,
    #[serde(default)]
    pub fail_on: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RulesSection {
    #[serde(default)]
    pub enable: Option<Vec<String>>,
    #[serde(default)]
    pub disable: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SeveritySection {
    #[serde(default)]
    pub minimum: Option<String>,
    #[serde(default)]
    pub overrides: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OutputSection {
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub quiet: Option<bool>,
    #[serde(default)]
    pub verbose: Option<bool>,
    #[serde(default)]
    pub show_score: Option<bool>,
    #[serde(default)]
    pub show_timings: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PerformanceSection {
    #[serde(default)]
    pub parallel: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct IgnoreSection {
    #[serde(default)]
    pub paths: Option<Vec<String>>,
}

// ---------------------------------------------------------------------------
// CliOverrides – all optional (None = not provided)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct CliOverrides {
    pub path: Option<String>,
    pub json: Option<bool>,
    pub compact: Option<bool>,
    pub quiet: Option<bool>,
    pub verbose: Option<bool>,
    pub color: Option<String>,
    pub fail_on: Option<String>,
    pub severity: Option<String>,
    pub category: Option<String>,
    pub rule: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    pub timings: Option<bool>,
    pub score: Option<bool>,
    pub threads: Option<usize>,
    pub config_path: Option<String>,
}

// ---------------------------------------------------------------------------
// RunConfig – immutable resolved config
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct RunConfig {
    // scan
    pub path: String,
    pub threads: usize,
    pub exclude: Vec<String>,
    pub fail_on: Severity,
    // rules
    pub enabled_rules: Vec<RuleId>,
    pub disabled_rules: Vec<RuleId>,
    // severity
    pub severity_threshold: Severity,
    pub severity_overrides: HashMap<RuleId, Severity>,
    // output
    pub format: OutputFormat,
    pub color: OutputColor,
    pub quiet: bool,
    pub verbose: bool,
    pub show_score: bool,
    pub show_timings: bool,
    // performance
    pub parallel: bool,
    // ignore
    pub ignore_paths: Vec<String>,
    // category filter
    pub category_filter: Option<Category>,
    // source tracking
    pub config_source: Option<ConfigSource>,
    pub project_name: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ConfigSource {
    Discovered(PathBuf),
    Explicit(PathBuf),
    Defaults,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Pretty,
    Compact,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputColor {
    Auto,
    Always,
    Never,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CategoryFilter {
    Any,
    Specific(Category),
}

// ---------------------------------------------------------------------------
// Defaults
// ---------------------------------------------------------------------------

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            path: ".".to_string(),
            threads: 4,
            exclude: Vec::new(),
            fail_on: Severity::High,
            enabled_rules: Vec::new(),
            disabled_rules: Vec::new(),
            severity_threshold: Severity::Low,
            severity_overrides: HashMap::new(),
            format: OutputFormat::Pretty,
            color: OutputColor::Auto,
            quiet: false,
            verbose: false,
            show_score: true,
            show_timings: false,
            parallel: true,
            ignore_paths: Vec::new(),
            category_filter: None,
            config_source: None,
            project_name: None,
        }
    }
}

// ---------------------------------------------------------------------------
// TOML parsing and validation
// ---------------------------------------------------------------------------

/// Parse TOML string into a validated `ConfigFile`.
pub(crate) fn parse_config_toml(input: &str, path: &Path) -> Result<ConfigFile, ConfigError> {
    let raw: toml::Value = toml::from_str(input).map_err(|e| {
        let msg = e.message().to_string();
        let span_start = e.span().map(|s| s.start).unwrap_or(0);
        let (line, col) = byte_offset_to_line_col(input, span_start);
        ConfigError::InvalidTomlSyntax {
            path: path.to_path_buf(),
            line,
            col,
            message: msg,
        }
    })?;

    validate_top_level(&raw, path)?;

    let cf: ConfigFile = raw.try_into().map_err(|e| ConfigError::InvalidToml {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;

    if let Some(ref scan) = cf.scan {
        validate_scan_section(scan, path)?;
    }
    if let Some(ref rules) = cf.rules {
        validate_rules_section(rules, path)?;
    }
    if let Some(ref sev) = cf.severity {
        validate_severity_section(sev, path)?;
    }
    if let Some(ref out) = cf.output {
        validate_output_section(out, path)?;
    }
    if let Some(ref perf) = cf.performance {
        validate_performance_section(perf, path)?;
    }
    if let Some(ref ignore) = cf.ignore {
        validate_ignore_section(ignore, path)?;
    }

    Ok(cf)
}

fn byte_offset_to_line_col(input: &str, byte_offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for (i, c) in input.char_indices() {
        if i >= byte_offset {
            break;
        }
        if c == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

fn validate_top_level(raw: &toml::Value, path: &Path) -> Result<(), ConfigError> {
    let table = match raw.as_table() {
        Some(t) => t,
        None => return Ok(()),
    };

    let known: &[&str] = &[
        "scan",
        "rules",
        "severity",
        "output",
        "performance",
        "ignore",
        "experimental",
    ];

    for key in table.keys() {
        if !known.contains(&key.as_str()) {
            return Err(ConfigError::UnknownSection {
                path: path.to_path_buf(),
                section: key.clone(),
                line: 0,
            });
        }
    }
    Ok(())
}

fn validate_scan_section(s: &ScanSection, path: &Path) -> Result<(), ConfigError> {
    if let Some(threads) = s.threads {
        if threads == 0 {
            return Err(ConfigError::NegativeThreads {
                value: 0,
                path: path.to_path_buf(),
                line: 0,
            });
        }
    }
    if let Some(ref patterns) = s.exclude {
        for (i, pattern) in patterns.iter().enumerate() {
            validate_glob(pattern, path, i + 1)?;
        }
    }
    if let Some(ref fail_on) = s.fail_on {
        Severity::parse(fail_on).ok_or_else(|| ConfigError::InvalidSeverity {
            value: fail_on.clone(),
            path: path.to_path_buf(),
            line: 0,
        })?;
    }
    Ok(())
}

fn validate_rules_section(s: &RulesSection, path: &Path) -> Result<(), ConfigError> {
    let enabled = s.enable.as_deref().unwrap_or(&[]);
    let disabled = s.disable.as_deref().unwrap_or(&[]);

    for e in enabled {
        if disabled.contains(e) {
            return Err(ConfigError::DuplicateRule {
                rule: e.clone(),
                path: path.to_path_buf(),
            });
        }
    }

    for (i, r) in enabled.iter().enumerate() {
        if !is_valid_rule_id(r) {
            return Err(ConfigError::InvalidRuleId {
                rule: r.clone(),
                path: path.to_path_buf(),
                line: i + 1,
            });
        }
    }
    for (i, r) in disabled.iter().enumerate() {
        if !is_valid_rule_id(r) {
            return Err(ConfigError::InvalidRuleId {
                rule: r.clone(),
                path: path.to_path_buf(),
                line: i + 1,
            });
        }
    }
    Ok(())
}

fn validate_severity_section(s: &SeveritySection, path: &Path) -> Result<(), ConfigError> {
    if let Some(ref min) = s.minimum {
        Severity::parse(min).ok_or_else(|| ConfigError::InvalidSeverity {
            value: min.clone(),
            path: path.to_path_buf(),
            line: 0,
        })?;
    }
    if let Some(ref overrides) = s.overrides {
        for (rule, sev_str) in overrides {
            if !is_valid_rule_id(rule) {
                return Err(ConfigError::InvalidRuleId {
                    rule: rule.clone(),
                    path: path.to_path_buf(),
                    line: 0,
                });
            }
            Severity::parse(sev_str).ok_or_else(|| ConfigError::InvalidSeverity {
                value: sev_str.clone(),
                path: path.to_path_buf(),
                line: 0,
            })?;
        }
    }
    Ok(())
}

fn validate_output_section(s: &OutputSection, path: &Path) -> Result<(), ConfigError> {
    if let Some(ref format) = s.format {
        match format.as_str() {
            "pretty" | "compact" | "json" => {}
            other => {
                return Err(ConfigError::InvalidOutputFormat {
                    value: other.to_string(),
                    path: path.to_path_buf(),
                    line: 0,
                });
            }
        }
    }
    if let Some(ref color) = s.color {
        match color.as_str() {
            "auto" | "always" | "never" => {}
            other => {
                return Err(ConfigError::InvalidColor {
                    value: other.to_string(),
                    path: path.to_path_buf(),
                    line: 0,
                });
            }
        }
    }
    Ok(())
}

fn validate_performance_section(_s: &PerformanceSection, _path: &Path) -> Result<(), ConfigError> {
    Ok(())
}

fn validate_ignore_section(s: &IgnoreSection, path: &Path) -> Result<(), ConfigError> {
    if let Some(ref paths) = s.paths {
        for (i, p) in paths.iter().enumerate() {
            validate_glob(p, path, i + 1)?;
        }
    }
    Ok(())
}

fn validate_glob(pattern: &str, path: &Path, line: usize) -> Result<(), ConfigError> {
    glob::Pattern::new(pattern).map_err(|e| ConfigError::InvalidGlob {
        pattern: pattern.to_string(),
        path: path.to_path_buf(),
        line,
        message: e.msg.to_string(),
    })?;
    Ok(())
}

pub fn is_valid_rule_id(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

// ---------------------------------------------------------------------------
// Config discovery
// ---------------------------------------------------------------------------

const CONFIG_FILENAMES: &[&str] = &["sentinel.toml", ".sentinel.toml"];

/// Search for a configuration file, returning the resolved path.
///
/// Order:
///   1. `--config <path>` (explicit)
///   2. `$CWD/sentinel.toml`
///   3. `$CWD/.sentinel.toml`
///   4. `$CWD/.config/sentinel.toml`
///   5. User config dir (`$XDG_CONFIG_HOME/sentinel.toml` or `~/.config/sentinel/sentinel.toml`)
///
/// Returns `None` if no config file is found (built-in defaults will be used).
pub fn discover_config(
    start: &Path,
    explicit: Option<&Path>,
) -> Result<Option<PathBuf>, ConfigError> {
    if let Some(path) = explicit {
        if path.exists() {
            return Ok(Some(canonicalize(path)?));
        }
        return Err(ConfigError::ConfigurationNotFound(path.to_path_buf()));
    }

    let start = if start.is_absolute() {
        start.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(ConfigError::CurrentDir)?
            .join(start)
    };

    let start = canonicalize(&start)?;

    for ancestor in start.ancestors() {
        for filename in CONFIG_FILENAMES {
            let candidate = ancestor.join(filename);
            if candidate.is_file() {
                return Ok(Some(candidate));
            }
        }
        let config_dir = ancestor.join(".config").join("sentinel.toml");
        if config_dir.is_file() {
            return Ok(Some(config_dir));
        }
    }

    if let Some(user_config) = user_config_path() {
        if user_config.is_file() {
            return Ok(Some(user_config));
        }
    }

    Ok(None)
}

fn user_config_path() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("XDG_CONFIG_HOME") {
        let p = PathBuf::from(dir).join("sentinel").join("sentinel.toml");
        if p.is_file() {
            return Some(p);
        }
    }
    if let Some(home) = dirs_home() {
        let p = home.join(".config").join("sentinel").join("sentinel.toml");
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
}

fn canonicalize(path: &Path) -> Result<PathBuf, ConfigError> {
    path.canonicalize().map_err(|e| ConfigError::IoError {
        path: path.to_path_buf(),
        source: e,
    })
}

// ---------------------------------------------------------------------------
// Build resolved RunConfig
// ---------------------------------------------------------------------------

pub fn build_run_config(
    cli: &CliOverrides,
    project: Option<PathBuf>,
    user: Option<PathBuf>,
) -> Result<RunConfig, ConfigError> {
    let mut rc = RunConfig::default();

    if let Some(ref user_path) = user {
        apply_toml_file(user_path, &mut rc)?;
    }

    if let Some(ref project_path) = project {
        apply_toml_file(project_path, &mut rc)?;
    }

    apply_cli(cli, &mut rc);

    rc.config_source = match (cli.config_path.as_ref(), project, user) {
        (Some(p), _, _) => Some(ConfigSource::Explicit(PathBuf::from(p))),
        (None, Some(p), _) => Some(ConfigSource::Discovered(p)),
        (None, None, Some(p)) => Some(ConfigSource::Discovered(p)),
        (None, None, None) => Some(ConfigSource::Defaults),
    };

    Ok(rc)
}

fn apply_toml_file(path: &Path, rc: &mut RunConfig) -> Result<(), ConfigError> {
    let content = std::fs::read_to_string(path).map_err(|e| ConfigError::IoError {
        path: path.to_path_buf(),
        source: e,
    })?;
    let cf = parse_config_toml(&content, path)?;
    apply_config_file(&cf, rc);
    Ok(())
}

fn apply_config_file(cf: &ConfigFile, rc: &mut RunConfig) {
    if let Some(ref scan) = cf.scan {
        if let Some(threads) = scan.threads {
            rc.threads = threads;
        }
        if let Some(ref exclude) = scan.exclude {
            rc.exclude = exclude.clone();
        }
        if let Some(ref fail_on) = scan.fail_on {
            if let Some(sev) = Severity::parse(fail_on) {
                rc.fail_on = sev;
            }
        }
    }

    if let Some(ref rules) = cf.rules {
        if let Some(ref enable) = rules.enable {
            rc.enabled_rules = enable.iter().filter_map(|s| RuleId::new(s).ok()).collect();
        }
        if let Some(ref disable) = rules.disable {
            rc.disabled_rules = disable.iter().filter_map(|s| RuleId::new(s).ok()).collect();
        }
    }

    if let Some(ref sev) = cf.severity {
        if let Some(ref min) = sev.minimum {
            if let Some(s) = Severity::parse(min) {
                rc.severity_threshold = s;
            }
        }
        if let Some(ref overrides) = sev.overrides {
            for (rule, sev_str) in overrides {
                if let (Ok(rid), Some(s)) = (RuleId::new(rule), Severity::parse(sev_str)) {
                    rc.severity_overrides.insert(rid, s);
                }
            }
        }
    }

    if let Some(ref out) = cf.output {
        if let Some(ref format) = out.format {
            rc.format = match format.as_str() {
                "json" => OutputFormat::Json,
                "compact" => OutputFormat::Compact,
                _ => OutputFormat::Pretty,
            };
        }
        if let Some(ref color) = out.color {
            rc.color = match color.as_str() {
                "always" => OutputColor::Always,
                "never" => OutputColor::Never,
                _ => OutputColor::Auto,
            };
        }
        if let Some(q) = out.quiet {
            rc.quiet = q;
        }
        if let Some(v) = out.verbose {
            rc.verbose = v;
        }
        if let Some(s) = out.show_score {
            rc.show_score = s;
        }
        if let Some(t) = out.show_timings {
            rc.show_timings = t;
        }
    }

    if let Some(ref perf) = cf.performance {
        if let Some(p) = perf.parallel {
            rc.parallel = p;
        }
    }

    if let Some(ref ignore) = cf.ignore {
        if let Some(ref paths) = ignore.paths {
            rc.ignore_paths = paths.clone();
        }
    }
}

fn apply_cli(cli: &CliOverrides, rc: &mut RunConfig) {
    if let Some(ref path) = cli.path {
        rc.path = path.clone();
    }
    if let Some(true) = cli.json {
        rc.format = OutputFormat::Json;
    }
    if let Some(true) = cli.compact {
        rc.format = OutputFormat::Compact;
    }
    if let Some(quiet) = cli.quiet {
        rc.quiet = quiet;
    }
    if let Some(verbose) = cli.verbose {
        rc.verbose = verbose;
    }
    if let Some(ref color) = cli.color {
        rc.color = match color.as_str() {
            "always" => OutputColor::Always,
            "never" => OutputColor::Never,
            _ => OutputColor::Auto,
        };
    }
    if let Some(ref fail_on) = cli.fail_on {
        if let Some(sev) = Severity::parse(fail_on) {
            rc.fail_on = sev;
        }
    }
    if let Some(ref sev) = cli.severity {
        if let Some(s) = Severity::parse(sev) {
            rc.severity_threshold = s;
        }
    }
    if let Some(ref category) = cli.category {
        rc.category_filter = Category::parse(category);
        if rc.category_filter.is_some() {
            rc.disabled_rules = Vec::new();
            rc.enabled_rules = Vec::new();
        }
    }
    if let Some(ref rules) = cli.rule {
        rc.enabled_rules = rules.iter().filter_map(|s| RuleId::new(s).ok()).collect();
    }
    if let Some(ref exclude) = cli.exclude {
        rc.exclude = exclude.clone();
    }
    if let Some(timings) = cli.timings {
        rc.show_timings = timings;
    }
    if let Some(score) = cli.score {
        rc.show_score = score;
    }
    if let Some(threads) = cli.threads {
        rc.threads = threads;
    }
}

// ---------------------------------------------------------------------------
// Conversion to RuleConfig for sentinel-rules
// ---------------------------------------------------------------------------

impl RunConfig {
    pub fn to_rule_config(&self) -> crate::core::RuleConfig {
        crate::core::RuleConfig {
            severity_threshold: self.severity_threshold,
            enabled: self.enabled_rules.clone(),
            disabled: self.disabled_rules.clone(),
            severity_overrides: self.severity_overrides.clone(),
            ignore_paths: self.ignore_paths.clone(),
        }
    }

    pub fn category_filter(&self) -> CategoryFilter {
        CategoryFilter::Any
    }
}

// ---------------------------------------------------------------------------
// High-level convenience entry point
// ---------------------------------------------------------------------------

/// Discover config, load user and project configs, merge with CLI overrides, return resolved `RunConfig`.
pub fn load_config(start: &Path, cli: &CliOverrides) -> Result<RunConfig, ConfigError> {
    let explicit = cli.config_path.as_ref().map(PathBuf::from);
    let explicit_ref = explicit.as_deref();

    let config_path = discover_config(start, explicit_ref)?;

    let (project_config, user_config) = if let Some(ref path) = config_path {
        if is_user_config_path(path) {
            (None, Some(path.clone()))
        } else {
            (Some(path.clone()), None)
        }
    } else {
        (None, None)
    };

    build_run_config(cli, project_config, user_config)
}

fn is_user_config_path(path: &Path) -> bool {
    if let Some(user_path) = user_config_path() {
        path == user_path
    } else {
        false
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_parse_valid_toml() {
        let toml = r#"
[scan]
threads = 8
exclude = ["target/**"]
fail_on = "high"

[rules]
enable = ["rule-one", "rule-two"]
disable = ["rule-three"]

[severity]
minimum = "medium"
overrides = { "some-rule" = "critical" }

[output]
format = "pretty"
color = "auto"
quiet = false
verbose = false
show_score = true
show_timings = false

[performance]
parallel = true

[ignore]
paths = ["examples/**"]

[experimental]
future-option = true
"#;
        let cf = parse_config_toml(toml, Path::new("test.toml")).unwrap();
        assert!(cf.scan.is_some());
        assert!(cf.rules.is_some());
        assert!(cf.severity.is_some());
        assert!(cf.output.is_some());
        assert!(cf.performance.is_some());
        assert!(cf.ignore.is_some());
        assert!(cf.experimental.is_some());
    }

    #[test]
    fn test_parse_empty_toml() {
        let cf = parse_config_toml("", Path::new("test.toml")).unwrap();
        assert!(cf.scan.is_none());
        assert!(cf.rules.is_none());
    }

    #[test]
    fn test_parse_unknown_section() {
        let err = parse_config_toml("[bogus]\nfoo = 1", Path::new("test.toml")).unwrap_err();
        assert!(matches!(err, ConfigError::UnknownSection { .. }));
    }

    #[test]
    fn test_parse_invalid_toml_syntax() {
        let err = parse_config_toml("[[broken", Path::new("test.toml")).unwrap_err();
        assert!(matches!(err, ConfigError::InvalidTomlSyntax { .. }));
    }

    #[test]
    fn test_parse_unknown_key_in_section() {
        let err =
            parse_config_toml("[scan]\nunknown_key = 42", Path::new("test.toml")).unwrap_err();
        assert!(matches!(err, ConfigError::InvalidToml { .. }));
    }

    #[test]
    fn test_validate_glob_invalid() {
        let toml = r#"
[scan]
exclude = ["[invalid"]
"#;
        let err = parse_config_toml(toml, Path::new("test.toml")).unwrap_err();
        assert!(matches!(err, ConfigError::InvalidGlob { .. }));
    }

    #[test]
    fn test_validate_severity_invalid() {
        let toml = r#"
[severity]
minimum = "super"
"#;
        let err = parse_config_toml(toml, Path::new("test.toml")).unwrap_err();
        assert!(matches!(err, ConfigError::InvalidSeverity { .. }));
    }

    #[test]
    fn test_validate_output_format_invalid() {
        let toml = r#"
[output]
format = "html"
"#;
        let err = parse_config_toml(toml, Path::new("test.toml")).unwrap_err();
        assert!(matches!(err, ConfigError::InvalidOutputFormat { .. }));
    }

    #[test]
    fn test_validate_color_invalid() {
        let toml = r#"
[output]
color = "neon"
"#;
        let err = parse_config_toml(toml, Path::new("test.toml")).unwrap_err();
        assert!(matches!(err, ConfigError::InvalidColor { .. }));
    }

    #[test]
    fn test_duplicate_rule_across_lists() {
        let toml = r#"
[rules]
enable = ["dup-rule"]
disable = ["dup-rule"]
"#;
        let err = parse_config_toml(toml, Path::new("test.toml")).unwrap_err();
        assert!(matches!(err, ConfigError::DuplicateRule { .. }));
    }

    #[test]
    fn test_invalid_rule_id() {
        let toml = r#"
[rules]
enable = ["Invalid_Rule!"]
"#;
        let err = parse_config_toml(toml, Path::new("test.toml")).unwrap_err();
        assert!(matches!(err, ConfigError::InvalidRuleId { .. }));
    }

    #[test]
    fn test_negative_threads() {
        let toml = r#"
[scan]
threads = 0
"#;
        let err = parse_config_toml(toml, Path::new("test.toml")).unwrap_err();
        assert!(matches!(err, ConfigError::NegativeThreads { .. }));
    }

    #[test]
    fn test_run_config_defaults() {
        let rc = RunConfig::default();
        assert_eq!(rc.threads, 4);
        assert_eq!(rc.path, ".");
        assert_eq!(rc.format, OutputFormat::Pretty);
        assert_eq!(rc.color, OutputColor::Auto);
        assert_eq!(rc.severity_threshold, Severity::Low);
        assert_eq!(rc.fail_on, Severity::High);
        assert!(rc.parallel);
        assert!(rc.show_score);
        assert!(!rc.show_timings);
        assert!(!rc.quiet);
        assert!(!rc.verbose);
        assert!(rc.enabled_rules.is_empty());
        assert!(rc.disabled_rules.is_empty());
        assert!(rc.severity_overrides.is_empty());
        assert!(rc.ignore_paths.is_empty());
        assert!(rc.exclude.is_empty());
    }

    #[test]
    fn test_discover_config_explicit() {
        let dir = tempfile::tempdir().unwrap();
        let cfg_path = dir.path().join("mysentinel.toml");
        let mut f = std::fs::File::create(&cfg_path).unwrap();
        writeln!(f, "[scan]\nthreads = 2").unwrap();
        drop(f);

        let result = discover_config(dir.path(), Some(&cfg_path)).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_discover_config_explicit_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("nope.toml");
        let err = discover_config(dir.path(), Some(&missing)).unwrap_err();
        assert!(matches!(err, ConfigError::ConfigurationNotFound(_)));
    }

    #[test]
    fn test_discover_config_none_found() {
        let dir = tempfile::tempdir().unwrap();
        let result = discover_config(dir.path(), None).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_build_run_config_defaults_only() {
        let cli = CliOverrides::default();
        let rc = build_run_config(&cli, None, None).unwrap();
        assert_eq!(rc.threads, 4);
        assert_eq!(rc.format, OutputFormat::Pretty);
        assert!(matches!(rc.config_source, Some(ConfigSource::Defaults)));
    }

    #[test]
    fn test_build_run_config_cli_overrides() {
        let cli = CliOverrides {
            path: Some("/my/project".to_string()),
            threads: Some(16),
            json: Some(true),
            color: Some("never".to_string()),
            quiet: Some(true),
            fail_on: Some("medium".to_string()),
            severity: Some("high".to_string()),
            exclude: Some(vec!["build/**".to_string()]),
            timings: Some(true),
            score: Some(false),
            ..Default::default()
        };
        let rc = build_run_config(&cli, None, None).unwrap();
        assert_eq!(rc.path, "/my/project");
        assert_eq!(rc.threads, 16);
        assert_eq!(rc.format, OutputFormat::Json);
        assert_eq!(rc.color, OutputColor::Never);
        assert!(rc.quiet);
        assert_eq!(rc.fail_on, Severity::Medium);
        assert_eq!(rc.severity_threshold, Severity::High);
        assert_eq!(rc.exclude, vec!["build/**"]);
        assert!(rc.show_timings);
        assert!(!rc.show_score);
    }

    #[test]
    fn test_build_run_config_with_project_toml() {
        let dir = tempfile::tempdir().unwrap();
        let cfg_path = dir.path().join("sentinel.toml");
        let mut f = std::fs::File::create(&cfg_path).unwrap();
        writeln!(
            f,
            "[scan]\nthreads = 2\n[output]\nformat = \"compact\"\n[severity]\nminimum = \"high\""
        )
        .unwrap();
        drop(f);

        let cli = CliOverrides {
            threads: Some(8),
            ..Default::default()
        };
        let rc = build_run_config(&cli, Some(cfg_path), None).unwrap();
        // CLI threads (8) should override project threads (2)
        assert_eq!(rc.threads, 8);
        // Project output format should apply
        assert_eq!(rc.format, OutputFormat::Compact);
        // Project severity should apply
        assert_eq!(rc.severity_threshold, Severity::High);
    }

    #[test]
    fn test_to_rule_config() {
        let rc = RunConfig {
            severity_threshold: Severity::Medium,
            enabled_rules: vec![RuleId::new("rule-a").unwrap()],
            disabled_rules: vec![RuleId::new("rule-b").unwrap()],
            ignore_paths: vec!["test/**".to_string()],
            ..Default::default()
        };
        let rcfg = rc.to_rule_config();
        assert_eq!(rcfg.severity_threshold, Severity::Medium);
        assert_eq!(rcfg.enabled, vec![RuleId::new("rule-a").unwrap()]);
        assert_eq!(rcfg.disabled, vec![RuleId::new("rule-b").unwrap()]);
        assert_eq!(rcfg.ignore_paths, vec!["test/**"]);
    }

    #[test]
    fn test_is_valid_rule_id_valid() {
        assert!(is_valid_rule_id("simple-rule"));
        assert!(is_valid_rule_id("rule123"));
        assert!(is_valid_rule_id("a"));
    }

    #[test]
    fn test_is_valid_rule_id_invalid() {
        assert!(!is_valid_rule_id(""));
        assert!(!is_valid_rule_id("UPPERCASE"));
        assert!(!is_valid_rule_id("has space"));
        assert!(!is_valid_rule_id("special!"));
    }

    #[test]
    fn test_parse_config_with_experimental() {
        let toml = r#"
[experimental]
feature_x = true
feature_y = "test"
"#;
        let cf = parse_config_toml(toml, Path::new("test.toml")).unwrap();
        assert!(cf.experimental.is_some());
        assert!(cf.scan.is_none());
    }

    #[test]
    fn test_build_run_config_explicit_source() {
        let cli = CliOverrides {
            config_path: Some("/some/path.toml".to_string()),
            ..Default::default()
        };
        let rc = build_run_config(&cli, None, None).unwrap();
        assert!(matches!(rc.config_source, Some(ConfigSource::Explicit(_))));
    }

    #[test]
    fn test_severity_override_invalid_value() {
        let toml = r#"
[severity]
overrides = { "my-rule" = "bogus" }
"#;
        let err = parse_config_toml(toml, Path::new("test.toml")).unwrap_err();
        assert!(matches!(err, ConfigError::InvalidSeverity { .. }));
    }

    #[test]
    fn test_severity_override_invalid_rule_id() {
        let toml = r#"
[severity]
overrides = { "Invalid!" = "high" }
"#;
        let err = parse_config_toml(toml, Path::new("test.toml")).unwrap_err();
        assert!(matches!(err, ConfigError::InvalidRuleId { .. }));
    }

    #[test]
    fn test_build_run_config_cli_category() {
        let cli = CliOverrides {
            category: Some("security".to_string()),
            ..Default::default()
        };
        let rc = build_run_config(&cli, None, None).unwrap();
        // category clears enabled/disabled lists
        assert!(rc.enabled_rules.is_empty());
        assert!(rc.disabled_rules.is_empty());
    }

    #[test]
    fn test_build_run_config_cli_rule() {
        let cli = CliOverrides {
            rule: Some(vec!["my-rule".to_string()]),
            ..Default::default()
        };
        let rc = build_run_config(&cli, None, None).unwrap();
        assert_eq!(rc.enabled_rules.len(), 1);
        assert_eq!(rc.enabled_rules[0].as_str(), "my-rule");
    }

    #[test]
    fn test_build_run_config_project_then_cli_severity() {
        let dir = tempfile::tempdir().unwrap();
        let cfg_path = dir.path().join("sentinel.toml");
        let mut f = std::fs::File::create(&cfg_path).unwrap();
        writeln!(f, "[severity]\nminimum = \"low\"").unwrap();
        drop(f);

        // CLI severity should override project severity
        let cli = CliOverrides {
            severity: Some("critical".to_string()),
            ..Default::default()
        };
        let rc = build_run_config(&cli, Some(cfg_path), None).unwrap();
        assert_eq!(rc.severity_threshold, Severity::Critical);
    }

    #[test]
    fn test_discover_config_sentinel_toml_in_cwd() {
        let dir = tempfile::tempdir().unwrap();
        let cfg_path = dir.path().join("sentinel.toml");
        let mut f = std::fs::File::create(&cfg_path).unwrap();
        writeln!(f, "[scan]").unwrap();
        drop(f);

        let result = discover_config(dir.path(), None).unwrap();
        assert!(result.is_some());
        let found = result.unwrap();
        assert!(found.ends_with("sentinel.toml"));
    }

    #[test]
    fn test_discover_config_dot_sentinel_toml() {
        let dir = tempfile::tempdir().unwrap();
        let cfg_path = dir.path().join(".sentinel.toml");
        let mut f = std::fs::File::create(&cfg_path).unwrap();
        writeln!(f, "[scan]").unwrap();
        drop(f);

        let result = discover_config(dir.path(), None).unwrap();
        assert!(result.is_some());
        let found = result.unwrap();
        assert!(found.ends_with(".sentinel.toml"));
    }

    #[test]
    fn test_parse_toml_skip_unknown_section() {
        // "unknown" is not in the known list
        let toml = r#"
[unknown]
key = "value"
"#;
        let err = parse_config_toml(toml, Path::new("test.toml")).unwrap_err();
        assert!(matches!(err, ConfigError::UnknownSection { .. }));
    }
}
