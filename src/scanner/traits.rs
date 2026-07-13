use std::path::Path;

use crate::core::{Ast, Finding, RuleConfig, RuleRegistry};

use super::error::ScanError;
use super::scan_request::ScanRequest;
use super::scan_result::ScanResult;

pub trait Scanner: Send + Sync {
    fn scan(&self, request: ScanRequest) -> Result<ScanResult, ScanError>;
}

pub trait Parser: Send + Sync {
    fn parse(&self, path: &Path) -> Result<Box<dyn Ast>, ScanError>;
}

pub trait RuleEngine: Send + Sync {
    fn execute(
        &self,
        project: &dyn Ast,
        registry: &RuleRegistry,
        config: &RuleConfig,
    ) -> Result<Vec<Finding>, ScanError>;
}

pub trait Reporter: Send + Sync {
    fn generate(&self, result: &ScanResult) -> Result<String, ScanError>;
}

pub trait AIProvider: Send + Sync {
    fn analyze(&self, findings: &[Finding]) -> Result<Vec<Finding>, ScanError>;
}
