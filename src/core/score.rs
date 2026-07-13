use serde::{Deserialize, Serialize};

use crate::core::finding::Finding;
use crate::core::severity::Severity;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityScore {
    pub score: u8,
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
    pub info: usize,
}

impl SecurityScore {
    pub fn from_findings(findings: &[Finding]) -> Self {
        let mut critical = 0usize;
        let mut high = 0usize;
        let mut medium = 0usize;
        let mut low = 0usize;
        let mut info = 0usize;

        for f in findings {
            match f.severity {
                Severity::Critical => critical += 1,
                Severity::High => high += 1,
                Severity::Medium => medium += 1,
                Severity::Low => low += 1,
                Severity::Info => info += 1,
            }
        }

        let raw = 100i16
            - (critical as i16 * Severity::Critical.weight() as i16)
            - (high as i16 * Severity::High.weight() as i16)
            - (medium as i16 * Severity::Medium.weight() as i16)
            - (low as i16 * Severity::Low.weight() as i16);

        let score = raw.clamp(0, 100) as u8;

        Self {
            score,
            critical,
            high,
            medium,
            low,
            info,
        }
    }

    pub fn perfect() -> Self {
        Self {
            score: 100,
            critical: 0,
            high: 0,
            medium: 0,
            low: 0,
            info: 0,
        }
    }
}

impl std::fmt::Display for SecurityScore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.score)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::category::Category;
    use crate::core::rule_id::RuleId;
    use crate::core::span::DiagnosticSpan;

    fn finding(severity: Severity) -> Finding {
        Finding::new(
            RuleId::new("test").unwrap(),
            severity,
            Category::Security,
            DiagnosticSpan::new("test.rs", 1, 1),
            "msg",
            "fix",
        )
    }

    #[test]
    fn perfect_score() {
        let score = SecurityScore::from_findings(&[]);
        assert_eq!(score.score, 100);
    }

    #[test]
    fn single_critical() {
        let score = SecurityScore::from_findings(&[finding(Severity::Critical)]);
        assert_eq!(score.score, 75);
        assert_eq!(score.critical, 1);
    }

    #[test]
    fn four_criticals_min() {
        let findings = vec![finding(Severity::Critical); 4];
        let score = SecurityScore::from_findings(&findings);
        assert_eq!(score.score, 0);
    }

    #[test]
    fn mixed_findings() {
        let findings = vec![
            finding(Severity::Critical),
            finding(Severity::Medium),
            finding(Severity::Medium),
            finding(Severity::Low),
        ];
        let score = SecurityScore::from_findings(&findings);
        assert_eq!(score.score, 63);
    }

    #[test]
    fn no_negative_score() {
        let findings = vec![finding(Severity::Critical); 10];
        let score = SecurityScore::from_findings(&findings);
        assert_eq!(score.score, 0);
    }

    #[test]
    fn info_does_not_affect() {
        let findings = vec![finding(Severity::Info); 100];
        let score = SecurityScore::from_findings(&findings);
        assert_eq!(score.score, 100);
    }

    #[test]
    fn correct_counts() {
        let findings = vec![
            finding(Severity::Critical),
            finding(Severity::High),
            finding(Severity::High),
            finding(Severity::Medium),
            finding(Severity::Info),
        ];
        let score = SecurityScore::from_findings(&findings);
        assert_eq!(score.critical, 1);
        assert_eq!(score.high, 2);
        assert_eq!(score.medium, 1);
        assert_eq!(score.low, 0);
        assert_eq!(score.info, 1);
    }

    #[test]
    fn perfect_constructor() {
        let score = SecurityScore::perfect();
        assert_eq!(score.score, 100);
        assert_eq!(score.critical, 0);
    }

    #[test]
    fn serde_roundtrip() {
        let score = SecurityScore::from_findings(&[finding(Severity::Critical)]);
        let json = serde_json::to_string(&score).unwrap();
        let back: SecurityScore = serde_json::from_str(&json).unwrap();
        assert_eq!(back, score);
    }

    #[test]
    fn display() {
        let score = SecurityScore::from_findings(&[]);
        assert_eq!(score.to_string(), "100");
    }
}
