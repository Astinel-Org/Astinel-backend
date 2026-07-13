use crate::core::{Ast, Category, Finding, Rule, RuleId, Severity};
use crate::scanner::parser::ParsedProject;

#[derive(Debug, Clone)]
pub struct ContractUpgrade;

impl Rule for ContractUpgrade {
    fn id(&self) -> RuleId {
        RuleId::new("contract-upgrade").unwrap()
    }

    fn name(&self) -> &'static str {
        "Contract upgrade"
    }

    fn description(&self) -> &'static str {
        "Flags unauthorized upgrade/deployer calls"
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

        let mut findings = Vec::new();

        for func in p.all_functions() {
            let Some(body) = &func.body else { continue };
            if body.deployer_calls.is_empty() {
                continue;
            }

            let has_auth = !body.auth_calls.is_empty();

            for call in &body.deployer_calls {
                if !has_auth {
                    findings.push(Finding::new(
                        self.id(),
                        Severity::Critical,
                        self.category(),
                        call.span.clone(),
                        format!(
                            "Deployer/upgrade call in '{}' without authorization",
                            func.name
                        ),
                        "Add require_auth!() before the deployer/upgrade call",
                    ));
                } else {
                    findings.push(Finding::new(
                        self.id(),
                        self.severity(),
                        self.category(),
                        call.span.clone(),
                        format!("Deployer/upgrade call in '{}'", func.name),
                        "Consider using a multi-sig or timelock pattern for upgrade safety",
                    ));
                }
            }
        }

        findings
    }

    fn clone_box(&self) -> Box<dyn Rule> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::parser::{test_helpers::*, DeployerCallKind, FunctionBody};

    #[test]
    fn flags_unauthorized_upgrade() {
        let body = FunctionBody {
            deployer_calls: vec![deployer_call(DeployerCallKind::UpdateCurrentContractWasm)],
            ..Default::default()
        };
        let func = build_public_fn("upgrade", body);
        let file = build_simple_contract("Upgradable", vec![func], true);
        let project = build_project(".", vec![file]);
        let findings = ContractUpgrade.check(&project);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Critical);
    }

    #[test]
    fn authorized_upgrade_is_lower_severity() {
        let body = FunctionBody {
            auth_calls: vec![auth_call("admin")],
            deployer_calls: vec![deployer_call(DeployerCallKind::UpdateCurrentContractWasm)],
            ..Default::default()
        };
        let func = build_public_fn("upgrade", body);
        let file = build_simple_contract("Upgradable", vec![func], true);
        let project = build_project(".", vec![file]);
        let findings = ContractUpgrade.check(&project);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::High);
    }
}
