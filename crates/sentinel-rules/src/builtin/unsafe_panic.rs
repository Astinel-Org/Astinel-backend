use sentinel_core::{Ast, Category, Finding, Rule, RuleId, Severity};
use sentinel_parser::ParsedProject;

#[derive(Debug)]
pub struct UnsafePanic;

impl Rule for UnsafePanic {
    fn id(&self) -> RuleId {
        RuleId::new("unsafe-panic").unwrap()
    }

    fn name(&self) -> &'static str {
        "Unsafe panic"
    }

    fn description(&self) -> &'static str {
        "Detects panics in contract functions"
    }

    fn severity(&self) -> Severity {
        Severity::Medium
    }

    fn category(&self) -> Category {
        Category::Performance
    }

    fn check(&self, project: &dyn Ast) -> Vec<Finding> {
        let Some(p) = project.as_any().downcast_ref::<ParsedProject>() else {
            return Vec::new();
        };

        p.all_functions()
            .filter_map(|f| f.body.as_ref().map(|b| (f, b)))
            .flat_map(|(f, b)| {
                b.panics.iter().map(move |op| {
                    Finding::new(
                        self.id(),
                        self.severity(),
                        self.category(),
                        op.span.clone(),
                        format!("Panic in function '{}'", f.name),
                        "Use proper error handling with Err() returns instead of panicking",
                    )
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sentinel_parser::{test_helpers::*, PanicKind};

    #[test]
    fn detects_panics() {
        let body = build_body(vec![], vec![], vec![panic_op(PanicKind::DirectPanic, "oops")]);
        let func = build_public_fn("risky", body);
        let file = build_simple_contract("Test", vec![func], true);
        let project = build_project(".", vec![file]);
        let findings = UnsafePanic.check(&project);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("risky"));
    }

    #[test]
    fn no_findings_when_no_panics() {
        let func = build_public_fn("safe", build_body(vec![], vec![], vec![]));
        let file = build_simple_contract("Test", vec![func], true);
        let project = build_project(".", vec![file]);
        let findings = UnsafePanic.check(&project);
        assert_eq!(findings.len(), 0);
    }
}
