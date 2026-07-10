use sentinel_core::{Category, DiagnosticSpan, Finding, RuleId, Severity};

/// Builder that constructs a `Finding` with a fluent API.
pub struct FindingBuilder {
    rule_id: RuleId,
    severity: Severity,
    category: Category,
    message: String,
    recommendation: String,
    span: DiagnosticSpan,
    fix_example: Option<String>,
}

impl FindingBuilder {
    pub fn new(rule_id: RuleId, message: impl Into<String>) -> Self {
        Self {
            rule_id,
            severity: Severity::Info,
            category: Category::BestPractice,
            message: message.into(),
            recommendation: String::new(),
            span: DiagnosticSpan::new("", 0, 0),
            fix_example: None,
        }
    }

    pub fn severity(mut self, sev: Severity) -> Self {
        self.severity = sev;
        self
    }

    pub fn category(mut self, cat: Category) -> Self {
        self.category = cat;
        self
    }

    pub fn span(mut self, span: DiagnosticSpan) -> Self {
        self.span = span;
        self
    }

    pub fn recommendation(mut self, text: impl Into<String>) -> Self {
        self.recommendation = text.into();
        self
    }

    pub fn fix_example(mut self, code: impl Into<String>) -> Self {
        self.fix_example = Some(code.into());
        self
    }

    pub fn build(self) -> Finding {
        let mut f = Finding::new(
            self.rule_id,
            self.severity,
            self.category,
            self.span,
            self.message,
            self.recommendation,
        );
        f.fix_example = self.fix_example;
        f
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_constructs_finding() {
        let f = FindingBuilder::new(RuleId::new("test-rule").unwrap(), "found issue")
            .severity(Severity::High)
            .category(Category::Security)
            .span(DiagnosticSpan::new("f.rs", 10, 5))
            .recommendation("fix it")
            .build();

        assert_eq!(f.rule_id.as_str(), "test-rule");
        assert_eq!(f.severity, Severity::High);
        assert_eq!(f.category, Category::Security);
        assert_eq!(f.span.line, 10);
    }

    #[test]
    fn builder_sets_fix_example() {
        let f = FindingBuilder::new(RuleId::new("fix-rule").unwrap(), "msg")
            .severity(Severity::Critical)
            .category(Category::Gas)
            .span(DiagnosticSpan::new("a.rs", 1, 1))
            .recommendation("do x")
            .fix_example("x += 1")
            .build();

        assert_eq!(f.fix_example, Some("x += 1".to_string()));
    }
}
