use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use crate::core::Severity;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ScanType {
    #[default]
    Full,
    Quick,
    Custom(Vec<String>),
}

#[derive(Debug, Clone)]
pub struct ScanRequest {
    pub target: PathBuf,
    pub scan_type: ScanType,
    pub severity_filter: Option<Severity>,
    pub enable_ai: bool,
    pub parallelism: Option<usize>,
    pub timeout: Option<Duration>,
    pub metadata: HashMap<String, String>,
    pub ignore_paths: Vec<String>,
    pub enabled_rules: Vec<String>,
    pub disabled_rules: Vec<String>,
}

impl Default for ScanRequest {
    fn default() -> Self {
        Self {
            target: PathBuf::from("."),
            scan_type: ScanType::Full,
            severity_filter: None,
            enable_ai: false,
            parallelism: None,
            timeout: None,
            metadata: HashMap::new(),
            ignore_paths: Vec::new(),
            enabled_rules: Vec::new(),
            disabled_rules: Vec::new(),
        }
    }
}

impl ScanRequest {
    pub fn builder() -> ScanRequestBuilder {
        ScanRequestBuilder::default()
    }

    pub fn new(target: impl Into<PathBuf>) -> Self {
        Self {
            target: target.into(),
            ..Default::default()
        }
    }
}

#[derive(Default)]
pub struct ScanRequestBuilder {
    target: Option<PathBuf>,
    scan_type: ScanType,
    severity_filter: Option<Severity>,
    enable_ai: bool,
    parallelism: Option<usize>,
    timeout: Option<Duration>,
    metadata: HashMap<String, String>,
    ignore_paths: Vec<String>,
    enabled_rules: Vec<String>,
    disabled_rules: Vec<String>,
}

impl ScanRequestBuilder {
    pub fn target(mut self, path: impl Into<PathBuf>) -> Self {
        self.target = Some(path.into());
        self
    }

    pub fn scan_type(mut self, scan_type: ScanType) -> Self {
        self.scan_type = scan_type;
        self
    }

    pub fn severity_filter(mut self, severity: Severity) -> Self {
        self.severity_filter = Some(severity);
        self
    }

    pub fn enable_ai(mut self, enable: bool) -> Self {
        self.enable_ai = enable;
        self
    }

    pub fn parallelism(mut self, threads: usize) -> Self {
        self.parallelism = Some(threads);
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn ignore_path(mut self, pattern: impl Into<String>) -> Self {
        self.ignore_paths.push(pattern.into());
        self
    }

    pub fn ignore_paths(mut self, patterns: Vec<String>) -> Self {
        self.ignore_paths = patterns;
        self
    }

    pub fn enable_rule(mut self, rule_id: impl Into<String>) -> Self {
        self.enabled_rules.push(rule_id.into());
        self
    }

    pub fn disable_rule(mut self, rule_id: impl Into<String>) -> Self {
        self.disabled_rules.push(rule_id.into());
        self
    }

    pub fn build(self) -> ScanRequest {
        ScanRequest {
            target: self.target.unwrap_or_else(|| PathBuf::from(".")),
            scan_type: self.scan_type,
            severity_filter: self.severity_filter,
            enable_ai: self.enable_ai,
            parallelism: self.parallelism,
            timeout: self.timeout,
            metadata: self.metadata,
            ignore_paths: self.ignore_paths,
            enabled_rules: self.enabled_rules,
            disabled_rules: self.disabled_rules,
        }
    }
}
