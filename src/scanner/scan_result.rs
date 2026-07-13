use std::path::PathBuf;

use crate::core::ScanResult as CoreScanResult;
use crate::core::SecurityScore;

use super::metrics::ScanMetrics;

#[derive(Debug, Clone)]
pub struct ScanResult {
    pub core: CoreScanResult,
    pub metrics: ScanMetrics,
    pub report: Option<String>,
    pub report_location: Option<PathBuf>,
    pub report_hash: Option<String>,
}

impl ScanResult {
    pub fn new(core: CoreScanResult, metrics: ScanMetrics) -> Self {
        Self {
            core,
            metrics,
            report: None,
            report_location: None,
            report_hash: None,
        }
    }

    pub fn score(&self) -> SecurityScore {
        self.core.score.clone()
    }

    pub fn with_report(mut self, report: String) -> Self {
        self.report = Some(report);
        self
    }

    pub fn with_report_location(mut self, path: PathBuf) -> Self {
        self.report_location = Some(path);
        self
    }

    pub fn with_report_hash(mut self, hash: String) -> Self {
        self.report_hash = Some(hash);
        self
    }

    pub fn is_secure(&self) -> bool {
        self.core.score.clone().critical == 0 && self.core.score.clone().high == 0
    }

    pub fn has_findings(&self) -> bool {
        !self.core.findings.is_empty()
    }
}
