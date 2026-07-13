use crate::core::{Ast, Category, Finding, Rule, RuleId, Severity};
use crate::scanner::parser::ParsedProject;

#[derive(Debug, Clone)]
pub struct MissingRequireAuth;

impl Rule for MissingRequireAuth {
    fn id(&self) -> RuleId {
        RuleId::new("missing-require-auth").unwrap()
    }

    fn name(&self) -> &'static str {
        "Missing require_auth"
    }

    fn description(&self) -> &'static str {
        "Detects public functions that do not call require_auth"
    }

    fn severity(&self) -> Severity {
        Severity::High
    }

    fn category(&self) -> Category {
        Category::Security
    }

    fn check(&self, project: &dyn Ast) -> Vec<Finding> {
        let Some(p) = project.as_any().downcast_ref::<ParsedProject>() else {
            return Vec::new();
        };

        p.all_functions()
            .filter(|f| f.visibility.is_public() && !f.is_check_auth)
            .filter(|f| f.body.as_ref().is_none_or(|b| b.auth_calls.is_empty()))
            .map(|f| {
                Finding::new(
                    self.id(),
                    self.severity(),
                    self.category(),
                    f.span.clone(),
                    format!("Public function '{}' does not call require_auth", f.name),
                    "Add require_auth!() at the start of the function",
                )
            })
            .collect()
    }

    fn clone_box(&self) -> Box<dyn Rule> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::parser::test_helpers::*;

    #[test]
    fn detects_missing_auth() {
        let func = build_public_fn("transfer", build_body(vec![], vec![], vec![]));
        let file = build_simple_contract("Token", vec![func], true);
        let project = build_project(".", vec![file]);
        let findings = MissingRequireAuth.check(&project);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("transfer"));
    }

    #[test]
    fn skips_functions_with_auth() {
        let body = build_body(vec![], vec![auth_call("admin")], vec![]);
        let func = build_public_fn("transfer", body);
        let file = build_simple_contract("Token", vec![func], true);
        let project = build_project(".", vec![file]);
        let findings = MissingRequireAuth.check(&project);
        assert_eq!(findings.len(), 0);
    }

    #[test]
    fn skips_check_auth_function() {
        let func = build_check_auth(None);
        let file = build_simple_contract("Token", vec![func], true);
        let project = build_project(".", vec![file]);
        let findings = MissingRequireAuth.check(&project);
        assert_eq!(findings.len(), 0);
    }

    #[test]
    fn skips_private_functions() {
        let func = build_private_fn("helper", build_body(vec![], vec![], vec![]));
        let file = build_simple_contract("Token", vec![func], true);
        let project = build_project(".", vec![file]);
        let findings = MissingRequireAuth.check(&project);
        assert_eq!(findings.len(), 0);
    }
}
