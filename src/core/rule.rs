use crate::core::ast::Ast;
use crate::core::category::Category;
use crate::core::finding::Finding;
use crate::core::rule_id::RuleId;
use crate::core::severity::Severity;

pub trait Rule: Send + Sync + std::fmt::Debug {
    fn id(&self) -> RuleId;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn severity(&self) -> Severity;
    fn category(&self) -> Category;
    fn check(&self, ast: &dyn Ast) -> Vec<Finding>;
    fn clone_box(&self) -> Box<dyn Rule>;
}

impl Clone for Box<dyn Rule> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ast::Ast;
    use crate::core::category::Category;
    use crate::core::finding::Finding;
    use crate::core::rule_id::RuleId;
    use crate::core::severity::Severity;
    use crate::core::span::DiagnosticSpan;
    use std::any::Any;
    use std::path::PathBuf;

    #[derive(Debug)]
    struct TestAst;

    impl Ast for TestAst {
        fn files(&self) -> &[PathBuf] {
            &[]
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[derive(Debug, Clone)]
    struct TestRule;

    impl Rule for TestRule {
        fn id(&self) -> RuleId {
            RuleId::new("test-rule").unwrap()
        }
        fn name(&self) -> &'static str {
            "Test Rule"
        }
        fn description(&self) -> &'static str {
            "A rule for testing"
        }
        fn severity(&self) -> Severity {
            Severity::High
        }
        fn category(&self) -> Category {
            Category::Security
        }
        fn check(&self, _ast: &dyn Ast) -> Vec<Finding> {
            vec![Finding::new(
                self.id(),
                self.severity(),
                self.category(),
                DiagnosticSpan::new("test.rs", 1, 1),
                "test finding",
                "do something",
            )]
        }

        fn clone_box(&self) -> Box<dyn Rule> {
            Box::new(self.clone())
        }
    }

    #[test]
    fn rule_contract() {
        let rule = TestRule;
        assert_eq!(rule.id().as_str(), "test-rule");
        assert_eq!(rule.name(), "Test Rule");
        assert_eq!(rule.severity(), Severity::High);
        assert_eq!(rule.category(), Category::Security);
    }

    #[test]
    fn rule_check_generates_findings() {
        let rule = TestRule;
        let ast = TestAst;
        let findings = rule.check(&ast);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, rule.id());
    }

    #[test]
    fn rule_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<Box<dyn Rule>>();
        assert_sync::<Box<dyn Rule>>();
    }
}
