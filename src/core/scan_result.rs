use serde::{Deserialize, Serialize};

use crate::core::finding::Finding;
use crate::core::score::SecurityScore;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanResult {
    pub findings: Vec<Finding>,
    pub score: SecurityScore,
    pub summary: ScanSummary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanSummary {
    pub total_files: usize,
    pub total_rules: usize,
    pub total_findings: usize,
    pub suppressed_findings: usize,
    pub duration_ms: u64,
}

impl ScanResult {
    pub fn new(
        findings: Vec<Finding>,
        total_files: usize,
        total_rules: usize,
        suppressed_findings: usize,
        duration_ms: u64,
    ) -> Self {
        let total_findings = findings.len();
        let score = SecurityScore::from_findings(&findings);

        Self {
            findings,
            score,
            summary: ScanSummary {
                total_files,
                total_rules,
                total_findings,
                suppressed_findings,
                duration_ms,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::category::Category;
    use crate::core::rule_id::RuleId;
    use crate::core::severity::Severity;
    use crate::core::span::DiagnosticSpan;

    fn finding(severity: Severity) -> Finding {
        Finding::new(
            RuleId::new("test-rule").unwrap(),
            severity,
            Category::Security,
            DiagnosticSpan::new("f.rs", 1, 1),
            "msg",
            "fix",
        )
    }

    #[test]
    fn scan_result_construction() {
        let findings = vec![finding(Severity::Critical)];
        let result = ScanResult::new(findings, 5, 10, 0, 100);
        assert_eq!(result.score.score, 75);
        assert_eq!(result.summary.total_files, 5);
        assert_eq!(result.summary.total_rules, 10);
        assert_eq!(result.summary.total_findings, 1);
        assert_eq!(result.summary.duration_ms, 100);
    }

    #[test]
    fn scan_result_empty() {
        let result = ScanResult::new(vec![], 0, 0, 0, 0);
        assert_eq!(result.score.score, 100);
        assert!(result.findings.is_empty());
    }

    #[test]
    fn scan_result_serde() {
        let result = ScanResult::new(vec![finding(Severity::High)], 1, 1, 0, 50);
        let json = serde_json::to_string(&result).unwrap();
        let back: ScanResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.score.score, 90);
        assert_eq!(back.summary.duration_ms, 50);
    }
}
