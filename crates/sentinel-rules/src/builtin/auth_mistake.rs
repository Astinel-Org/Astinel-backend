use sentinel_core::{Ast, Category, Finding, Rule, RuleId, Severity};
use sentinel_parser::ParsedProject;

#[derive(Debug)]
pub struct AuthMistake;

impl Rule for AuthMistake {
    fn id(&self) -> RuleId {
        RuleId::new("auth-mistake").unwrap()
    }

    fn name(&self) -> &'static str {
        "Authorization mistake"
    }

    fn description(&self) -> &'static str {
        "Detects authorization gaps and suspicious auth patterns"
    }

    fn severity(&self) -> Severity {
        Severity::Critical
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
            if body.auth_calls.is_empty() {
                continue;
            }

            // Check for wide gaps between auth calls
            for pair in body.auth_calls.windows(2) {
                let distance = pair[1].span.line.saturating_sub(pair[0].span.line);
                if distance > 20 {
                    findings.push(Finding::new(
                        self.id(),
                        self.severity(),
                        self.category(),
                        pair[0].span.clone(),
                        format!(
                            "require_auth calls are {} lines apart in '{}', possible auth gap",
                            distance, func.name
                        ),
                        "Check if code between require_auth calls is properly authorized",
                    ));
                }
            }

            // Check for storage ops after last auth call
            let last_auth_line = body.auth_calls.iter().map(|a| a.span.line).max().unwrap_or(0);
            for op in &body.storage_ops {
                if op.span.line > last_auth_line {
                    findings.push(Finding::new(
                        self.id(),
                        Severity::High,
                        self.category(),
                        op.span.clone(),
                        format!(
                            "Storage operation after last require_auth in '{}' may be unauthorized",
                            func.name
                        ),
                        "Move this before require_auth or add another require_auth call",
                    ));
                }
            }
        }

        findings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sentinel_parser::{test_helpers::*, FunctionBody, StorageOpKind, StorageTier};

    #[test]
    fn detects_storage_after_auth() {
        let body = FunctionBody {
            auth_calls: vec![auth_call("admin")],
            storage_ops: vec![storage_op(StorageOpKind::Set, StorageTier::Persistent, "balance", 30)],
            ..Default::default()
        };
        let func = build_public_fn("transfer", body);
        let file = build_simple_contract("Token", vec![func], true);
        let project = build_project(".", vec![file]);
        let findings = AuthMistake.check(&project);
        assert!(!findings.is_empty());
    }
}
