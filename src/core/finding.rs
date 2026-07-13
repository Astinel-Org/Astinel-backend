use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

use crate::core::category::Category;
use crate::core::rule_id::RuleId;
use crate::core::severity::Severity;
use crate::core::span::DiagnosticSpan;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    pub rule_id: RuleId,
    pub severity: Severity,
    pub category: Category,
    pub span: DiagnosticSpan,
    pub message: String,
    pub recommendation: String,
    pub fix_example: Option<String>,
}

impl Finding {
    pub fn new(
        rule_id: RuleId,
        severity: Severity,
        category: Category,
        span: DiagnosticSpan,
        message: impl Into<String>,
        recommendation: impl Into<String>,
    ) -> Self {
        Self {
            rule_id,
            severity,
            category,
            span,
            message: message.into(),
            recommendation: recommendation.into(),
            fix_example: None,
        }
    }

    pub fn with_fix(mut self, example: impl Into<String>) -> Self {
        self.fix_example = Some(example.into());
        self
    }
}

impl PartialOrd for Finding {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Finding {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .severity
            .cmp(&self.severity)
            .then_with(|| self.span.file.cmp(&other.span.file))
            .then_with(|| self.span.line.cmp(&other.span.line))
            .then_with(|| self.span.column.cmp(&other.span.column))
            .then_with(|| self.rule_id.cmp(&other.rule_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::category::Category;
    use crate::core::rule_id::RuleId;
    use crate::core::severity::Severity;

    fn finding(severity: Severity, file: &str, line: usize, col: usize) -> Finding {
        Finding::new(
            RuleId::new("test-rule").unwrap(),
            severity,
            Category::Security,
            DiagnosticSpan::new(file, line, col),
            "message",
            "recommendation",
        )
    }

    #[test]
    fn ordering_severity_first() {
        let critical = finding(Severity::Critical, "a.rs", 1, 1);
        let high = finding(Severity::High, "a.rs", 1, 1);
        assert!(critical > high);
    }

    #[test]
    fn ordering_same_severity_by_file() {
        let a = finding(Severity::High, "b.rs", 1, 1);
        let b = finding(Severity::High, "a.rs", 1, 1);
        assert!(b < a);
    }

    #[test]
    fn ordering_same_file_by_line() {
        let a = finding(Severity::High, "a.rs", 10, 1);
        let b = finding(Severity::High, "a.rs", 5, 1);
        assert!(b < a);
    }

    #[test]
    fn ordering_same_line_by_column() {
        let a = finding(Severity::High, "a.rs", 5, 10);
        let b = finding(Severity::High, "a.rs", 5, 5);
        assert!(b < a);
    }

    #[test]
    fn dedup_identical_findings() {
        let f1 = finding(Severity::High, "a.rs", 5, 5);
        let f2 = finding(Severity::High, "a.rs", 5, 5);
        let mut vec = vec![f1.clone(), f2];
        vec.sort();
        vec.dedup();
        assert_eq!(vec.len(), 1);
    }

    #[test]
    fn serde_roundtrip() {
        let f = finding(Severity::Critical, "src/main.rs", 42, 10);
        let json = serde_json::to_string_pretty(&f).unwrap();
        let back: Finding = serde_json::from_str(&json).unwrap();
        assert_eq!(back, f);
    }

    #[test]
    fn with_fix() {
        let f = finding(Severity::High, "test.rs", 1, 1).with_fix("use checked_add()");
        assert_eq!(f.fix_example, Some("use checked_add()".to_string()));
    }
}
