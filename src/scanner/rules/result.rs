use std::time::Duration;

use crate::core::{Finding, SecurityScore};

#[derive(Debug, Clone)]
pub struct RuleResult {
    pub findings: Vec<Finding>,
    pub score: SecurityScore,
    pub summary: ExecutionSummary,
}

#[derive(Debug, Clone)]
pub struct ExecutionSummary {
    pub total_rules_run: usize,
    pub total_files: usize,
    pub total_findings: usize,
    pub suppressed_findings: usize,
    pub duration: Duration,
}

impl RuleResult {
    pub fn from_findings(
        findings: Vec<Finding>,
        suppressed_count: usize,
        total_files: usize,
        total_rules: usize,
        duration: Duration,
    ) -> Self {
        let total_findings = findings.len();
        let score = SecurityScore::from_findings(&findings);

        Self {
            findings,
            score,
            summary: ExecutionSummary {
                total_rules_run: total_rules,
                total_files,
                total_findings,
                suppressed_findings: suppressed_count,
                duration,
            },
        }
    }

    pub fn is_secure(&self) -> bool {
        self.score.critical == 0 && self.score.high == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Category, DiagnosticSpan, RuleId, Severity};

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
    fn empty_result() {
        let result = RuleResult::from_findings(vec![], 0, 0, 0, Duration::from_millis(0));
        assert_eq!(result.score.score, 100);
        assert!(result.is_secure());
    }

    #[test]
    fn result_with_findings() {
        let findings = vec![finding(Severity::Critical)];
        let result = RuleResult::from_findings(findings, 0, 1, 5, Duration::from_millis(50));
        assert_eq!(result.score.score, 75);
        assert_eq!(result.summary.total_rules_run, 5);
        assert_eq!(result.summary.total_files, 1);
        assert_eq!(result.summary.duration.as_millis(), 50);
        assert!(!result.is_secure());
    }

    #[test]
    fn suppressed_count_tracked() {
        let result = RuleResult::from_findings(vec![], 3, 1, 10, Duration::ZERO);
        assert_eq!(result.summary.suppressed_findings, 3);
    }
}
