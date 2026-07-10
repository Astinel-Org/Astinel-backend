use std::collections::HashMap;
use std::path::{Path, PathBuf};

use sentinel_core::{Category, RuleId, Severity};

/// Merged runtime configuration, combining CLI flags, config file, and defaults.
#[derive(Debug, Clone)]
pub struct RunConfig {
    /// Minimum severity to report findings for
    pub severity_threshold: Severity,
    /// Severity at or above which findings cause a non-zero exit
    pub fail_on: Severity,
    /// Only run these specific rules (empty = run all)
    pub enabled_rules: Vec<RuleId>,
    /// Exclude these specific rules from running
    pub disabled_rules: Vec<RuleId>,
    /// Per-rule severity overrides
    pub severity_overrides: HashMap<RuleId, Severity>,
    /// File path patterns to exclude from scanning (substring match)
    pub ignore_paths: Vec<String>,
    /// Only report findings in this category (if set)
    pub category_filter: Option<Category>,
    /// Number of analysis threads
    pub threads: usize,
    /// Show security score in output
    pub show_score: bool,
    /// Show timing breakdown in output
    pub show_timings: bool,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            severity_threshold: Severity::Low,
            fail_on: Severity::Info,
            enabled_rules: Vec::new(),
            disabled_rules: Vec::new(),
            severity_overrides: HashMap::new(),
            ignore_paths: Vec::new(),
            category_filter: None,
            threads: 1,
            show_score: true,
            show_timings: false,
        }
    }
}

impl RunConfig {
    /// Convert to the rule engine configuration.
    pub fn to_rule_config(&self) -> sentinel_rules::config::RuleConfig {
        sentinel_rules::config::RuleConfig {
            severity_threshold: self.severity_threshold,
            enabled: self.enabled_rules.clone(),
            disabled: self.disabled_rules.clone(),
            severity_overrides: self.severity_overrides.clone(),
            ignore_paths: self.ignore_paths.clone(),
        }
    }
}

/// Raw CLI arguments, parsed by clap.
#[derive(Debug, Clone)]
pub struct CliArgs {
    pub path: String,
    pub json: bool,
    pub quiet: bool,
    pub verbose: bool,
    pub color: String,
    pub fail_on: Option<String>,
    pub severity: Option<String>,
    pub category: Option<String>,
    pub rule: Option<String>,
    pub exclude: Option<String>,
    pub timings: bool,
    pub score: bool,
    pub threads: Option<usize>,
    pub config_path: Option<String>,
}

impl CliArgs {
    /// Merge CLI args into a [`RunConfig`], with CLI flags overriding config file values.
    pub fn merge_into_run_config(&self) -> Result<RunConfig, crate::errors::CliError> {
        let mut config = if let Some(ref config_path) = self.config_path {
            load_config_file(Path::new(config_path))?
        } else {
            load_project_config(Path::new(&self.path)).unwrap_or_default()
        };

        if let Some(ref severity_str) = self.severity {
            config.severity_threshold = Severity::parse(severity_str).ok_or_else(|| {
                crate::errors::CliError::InvalidArguments(format!("Unknown severity level: `{}`", severity_str))
            })?;
        }

        if let Some(ref fail_on_str) = self.fail_on {
            config.fail_on = Severity::parse(fail_on_str).ok_or_else(|| {
                crate::errors::CliError::InvalidArguments(format!("Unknown severity level: `{}`", fail_on_str))
            })?;
        }

        if let Some(ref rule_str) = self.rule {
            config.enabled_rules = rule_str
                .split(',')
                .filter_map(|s| {
                    let trimmed = s.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(RuleId::new(trimmed).map_err(|_| ()))
                    }
                })
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| crate::errors::CliError::InvalidArguments("Invalid rule ID in --rule".to_string()))?;
        }

        if let Some(ref exclude_str) = self.exclude {
            // --exclude populates ignore_paths (file path patterns, not rule IDs)
            for pattern in exclude_str.split(',').map(|s| s.trim()) {
                if !pattern.is_empty() && !config.ignore_paths.contains(&pattern.to_string()) {
                    config.ignore_paths.push(pattern.to_string());
                }
            }
        }

        if let Some(ref category_str) = self.category {
            config.category_filter = Some(Category::parse(category_str).ok_or_else(|| {
                crate::errors::CliError::InvalidArguments(format!(
                    "Unknown category: `{}`. Valid values: security, performance, gas, best-practice, upgrade",
                    category_str
                ))
            })?);
        }

        config.threads = self.threads.unwrap_or(1);
        config.show_score = self.score;
        config.show_timings = self.timings;

        Ok(config)
    }
}

fn load_config_file(path: &Path) -> Result<RunConfig, crate::errors::CliError> {
    if !path.exists() {
        return Err(crate::errors::CliError::Config(format!(
            "Config file not found: {}",
            path.display()
        )));
    }
    let content = std::fs::read_to_string(path).map_err(crate::errors::CliError::Io)?;
    parse_config_toml(&content)
}

fn load_project_config(scan_path: &Path) -> Option<RunConfig> {
    let config_path = find_config_file(scan_path)?;
    let content = std::fs::read_to_string(&config_path).ok()?;
    parse_config_toml(&content).ok()
}

fn find_config_file(scan_path: &Path) -> Option<PathBuf> {
    let mut current = if scan_path.is_file() {
        scan_path.parent()?.to_path_buf()
    } else {
        scan_path.to_path_buf()
    };
    loop {
        let candidate = current.join("sentinel.toml");
        if candidate.exists() {
            return Some(candidate);
        }
        if current.join("Cargo.toml").exists() {
            return None;
        }
        current = current.parent()?.to_path_buf();
    }
}

fn parse_config_toml(content: &str) -> Result<RunConfig, crate::errors::CliError> {
    let value: toml::Value = toml::from_str(content).map_err(|e| crate::errors::CliError::Config(e.to_string()))?;

    let mut config = RunConfig::default();

    if let Some(table) = value.as_table() {
        if let Some(severity) = table.get("severity_threshold").and_then(|v| v.as_str()) {
            config.severity_threshold = Severity::parse(severity)
                .ok_or_else(|| crate::errors::CliError::Config(format!("Unknown severity: {}", severity)))?;
        }
        if let Some(fail_on) = table.get("fail_on").and_then(|v| v.as_str()) {
            config.fail_on = Severity::parse(fail_on)
                .ok_or_else(|| crate::errors::CliError::Config(format!("Unknown severity: {}", fail_on)))?;
        }
        if let Some(rules) = table.get("rules").and_then(|v| v.as_array()) {
            for rule_val in rules {
                if let Some(rule_str) = rule_val.as_str() {
                    config.enabled_rules.push(
                        RuleId::new(rule_str)
                            .map_err(|e| crate::errors::CliError::Config(format!("Invalid rule ID: {}", e)))?,
                    );
                }
            }
        }
        if let Some(ignore) = table.get("ignore_paths").and_then(|v| v.as_array()) {
            for val in ignore {
                if let Some(p) = val.as_str() {
                    config.ignore_paths.push(p.to_string());
                }
            }
        }
        if let Some(cat) = table.get("category").and_then(|v| v.as_str()) {
            config.category_filter = Some(
                Category::parse(cat)
                    .ok_or_else(|| crate::errors::CliError::Config(format!("Unknown category: {}", cat)))?,
            );
        }
    }

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let config = RunConfig::default();
        assert_eq!(config.severity_threshold, Severity::Low);
        assert_eq!(config.fail_on, Severity::Info);
        assert_eq!(config.threads, 1);
        assert!(config.category_filter.is_none());
    }

    #[test]
    fn parse_valid_toml() {
        let toml = r#"
            severity_threshold = "high"
            fail_on = "critical"
            rules = ["missing-require-auth", "integer-overflow"]
        "#;
        let config = parse_config_toml(toml).unwrap();
        assert_eq!(config.severity_threshold, Severity::High);
        assert_eq!(config.fail_on, Severity::Critical);
        assert_eq!(config.enabled_rules.len(), 2);
    }

    #[test]
    fn parse_empty_toml_uses_defaults() {
        let config = parse_config_toml("").unwrap();
        assert_eq!(config.severity_threshold, Severity::Low);
        assert_eq!(config.fail_on, Severity::Info);
        assert!(config.enabled_rules.is_empty());
    }

    #[test]
    fn parse_invalid_severity_returns_error() {
        let toml = r#"severity_threshold = "unknown""#;
        let result = parse_config_toml(toml);
        assert!(result.is_err());
    }

    #[test]
    fn merge_with_cli_overrides() {
        let args = CliArgs {
            path: ".".to_string(),
            json: false,
            quiet: false,
            verbose: false,
            color: "auto".to_string(),
            fail_on: Some("high".to_string()),
            severity: Some("medium".to_string()),
            category: Some("security".to_string()),
            rule: Some("missing-require-auth,unsafe-panic".to_string()),
            exclude: Some("tests,examples".to_string()),
            timings: false,
            score: true,
            threads: Some(4),
            config_path: None,
        };
        let config = args.merge_into_run_config().unwrap();
        assert_eq!(config.severity_threshold, Severity::Medium);
        assert_eq!(config.fail_on, Severity::High);
        assert_eq!(config.threads, 4);
        assert_eq!(config.enabled_rules.len(), 2);
        assert_eq!(config.category_filter, Some(Category::Security));
        assert!(config.ignore_paths.contains(&"tests".to_string()));
        assert!(config.ignore_paths.contains(&"examples".to_string()));
    }

    #[test]
    fn parse_ignore_paths_from_toml() {
        let toml = r#"
            ignore_paths = ["target", "tests"]
        "#;
        let config = parse_config_toml(toml).unwrap();
        assert_eq!(config.ignore_paths.len(), 2);
        assert!(config.ignore_paths.contains(&"target".to_string()));
    }

    #[test]
    fn parse_category_from_toml() {
        let toml = r#"category = "security""#;
        let config = parse_config_toml(toml).unwrap();
        assert_eq!(config.category_filter, Some(Category::Security));
    }

    #[test]
    fn parse_invalid_category_returns_error() {
        let toml = r#"category = "bogus""#;
        let result = parse_config_toml(toml);
        assert!(result.is_err());
    }

    #[test]
    fn empty_exclude_adds_nothing() {
        let args = CliArgs {
            path: ".".to_string(),
            json: false,
            quiet: false,
            verbose: false,
            color: "auto".to_string(),
            fail_on: None,
            severity: None,
            category: None,
            rule: None,
            exclude: Some("".to_string()),
            timings: false,
            score: false,
            threads: None,
            config_path: None,
        };
        let config = args.merge_into_run_config().unwrap();
        assert!(config.ignore_paths.is_empty());
    }

    #[test]
    fn invalid_category_returns_error() {
        let args = CliArgs {
            path: ".".to_string(),
            json: false,
            quiet: false,
            verbose: false,
            color: "auto".to_string(),
            fail_on: None,
            severity: None,
            category: Some("bogus".to_string()),
            rule: None,
            exclude: None,
            timings: false,
            score: false,
            threads: None,
            config_path: None,
        };
        let result = args.merge_into_run_config();
        assert!(result.is_err());
    }

    #[test]
    fn invalid_rule_id_returns_error() {
        let args = CliArgs {
            path: ".".to_string(),
            json: false,
            quiet: false,
            verbose: false,
            color: "auto".to_string(),
            fail_on: None,
            severity: None,
            category: None,
            rule: Some("INVALID_RULE".to_string()),
            exclude: None,
            timings: false,
            score: false,
            threads: None,
            config_path: None,
        };
        let result = args.merge_into_run_config();
        assert!(result.is_err());
    }
}
