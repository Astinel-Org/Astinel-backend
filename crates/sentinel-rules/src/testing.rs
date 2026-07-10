use sentinel_core::{Ast, Category, DiagnosticSpan, Finding, Rule, RuleId, Severity};
use std::any::Any;
use std::fmt;
use std::path::PathBuf;

/// A test-only AST that implements `Ast` for use in unit tests.
#[derive(Debug)]
pub struct TestProject {
    paths: Vec<PathBuf>,
}

impl TestProject {
    pub fn empty() -> Self {
        Self { paths: Vec::new() }
    }

    pub fn with_file(path: &str) -> Self {
        Self {
            paths: vec![PathBuf::from(path)],
        }
    }
}

impl Ast for TestProject {
    fn files(&self) -> &[PathBuf] {
        &self.paths
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// A configurable test rule that returns findings on demand.
pub struct TestRule {
    id: RuleId,
    severity: Severity,
    category: Category,
    produce_finding: bool,
}

impl fmt::Debug for TestRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TestRule").field("id", &self.id).finish()
    }
}

pub struct TestRuleBuilder {
    id: RuleId,
    severity: Severity,
    category: Category,
    produce_finding: bool,
}

impl TestRuleBuilder {
    pub fn new(id: &str) -> Self {
        Self {
            id: RuleId::new(id).unwrap(),
            severity: Severity::Medium,
            category: Category::BestPractice,
            produce_finding: false,
        }
    }

    pub fn severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_finding(mut self, produce: bool) -> Self {
        self.produce_finding = produce;
        self
    }

    pub fn build(self) -> TestRule {
        TestRule {
            id: self.id,
            severity: self.severity,
            category: self.category,
            produce_finding: self.produce_finding,
        }
    }
}

impl Rule for TestRule {
    fn id(&self) -> RuleId {
        self.id.clone()
    }

    fn name(&self) -> &'static str {
        "Test rule"
    }

    fn description(&self) -> &'static str {
        "A rule used for testing"
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn category(&self) -> Category {
        self.category
    }

    fn check(&self, _project: &dyn Ast) -> Vec<Finding> {
        if self.produce_finding {
            vec![Finding::new(
                self.id.clone(),
                self.severity,
                self.category,
                DiagnosticSpan::new("test.rs", 1, 1),
                "test finding",
                "do something",
            )]
        } else {
            vec![]
        }
    }
}
