use sentinel_core::{Ast, Category, Finding, Rule, RuleId, Severity};
use sentinel_parser::ParsedProject;

#[derive(Debug)]
pub struct DeadCode;

impl Rule for DeadCode {
    fn id(&self) -> RuleId {
        RuleId::new("dead-code").unwrap()
    }

    fn name(&self) -> &'static str {
        "Dead code"
    }

    fn description(&self) -> &'static str {
        "Flags private functions that may be dead code"
    }

    fn severity(&self) -> Severity {
        Severity::Low
    }

    fn category(&self) -> Category {
        Category::BestPractice
    }

    fn check(&self, project: &dyn Ast) -> Vec<Finding> {
        let Some(p) = project.as_any().downcast_ref::<ParsedProject>() else {
            return Vec::new();
        };

        p.all_functions()
            .filter(|f| !f.visibility.is_public())
            .map(|f| {
                Finding::new(
                    self.id(),
                    self.severity(),
                    self.category(),
                    f.span.clone(),
                    format!("Private function '{}' may be dead code", f.name),
                    "Remove unused private functions or add #[test] if they are tests",
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sentinel_parser::test_helpers::*;

    #[test]
    fn flags_private_functions() {
        let func = build_private_fn("helper", build_body(vec![], vec![], vec![]));
        let file = build_simple_contract("Test", vec![func], true);
        let project = build_project(".", vec![file]);
        let findings = DeadCode.check(&project);
        assert_eq!(findings.len(), 1);
    }

    #[test]
    fn skips_public_functions() {
        let func = build_public_fn("do_thing", build_body(vec![], vec![], vec![]));
        let file = build_simple_contract("Test", vec![func], true);
        let project = build_project(".", vec![file]);
        let findings = DeadCode.check(&project);
        assert_eq!(findings.len(), 0);
    }

    #[test]
    fn multiple_private_all_flagged() {
        let f1 = build_private_fn("a", build_body(vec![], vec![], vec![]));
        let f2 = build_private_fn("b", build_body(vec![], vec![], vec![]));
        let file = build_simple_contract("Test", vec![f1, f2], true);
        let project = build_project(".", vec![file]);
        let findings = DeadCode.check(&project);
        assert_eq!(findings.len(), 2);
    }
}
