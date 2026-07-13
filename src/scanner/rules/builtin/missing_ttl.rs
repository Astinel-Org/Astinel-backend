use crate::core::{Ast, Category, Finding, Rule, RuleId, Severity};
use crate::scanner::parser::{ParsedProject, StorageOpKind};

#[derive(Debug, Clone)]
pub struct MissingTtl;

impl Rule for MissingTtl {
    fn id(&self) -> RuleId {
        RuleId::new("missing-ttl").unwrap()
    }

    fn name(&self) -> &'static str {
        "Missing TTL management"
    }

    fn description(&self) -> &'static str {
        "Flags functions that write to storage without setting TTL"
    }

    fn severity(&self) -> Severity {
        Severity::Medium
    }

    fn category(&self) -> Category {
        Category::BestPractice
    }

    fn check(&self, project: &dyn Ast) -> Vec<Finding> {
        let Some(p) = project.as_any().downcast_ref::<ParsedProject>() else {
            return Vec::new();
        };

        p.all_functions()
            .filter_map(|f| f.body.as_ref().map(|b| (f, b)))
            .filter(|(_, b)| {
                b.ttl_ops.is_empty()
                    && b.storage_ops
                        .iter()
                        .any(|op| matches!(op.kind, StorageOpKind::Set | StorageOpKind::Update))
            })
            .map(|(f, _)| {
                Finding::new(
                    self.id(),
                    self.severity(),
                    self.category(),
                    f.span.clone(),
                    format!(
                        "Function '{}' writes to storage but does not set TTL",
                        f.name
                    ),
                    "Call extend_ttl or set persistent storage TTL after writing",
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
    use crate::scanner::parser::{
        test_helpers::*, FunctionBody, FunctionDef, StorageOpKind, StorageTier, TtlKind,
    };

    #[test]
    fn flags_write_without_ttl() {
        let body = FunctionBody {
            storage_ops: vec![storage_op(
                StorageOpKind::Set,
                StorageTier::Persistent,
                "data",
                10,
            )],
            ..Default::default()
        };
        let func = build_public_fn("store", body);
        let file = build_simple_contract("Test", vec![func], true);
        let project = build_project(".", vec![file]);
        let findings = MissingTtl.check(&project);
        assert_eq!(findings.len(), 1);
    }

    #[test]
    fn skips_when_ttl_present() {
        let body = FunctionBody {
            storage_ops: vec![storage_op(
                StorageOpKind::Set,
                StorageTier::Persistent,
                "data",
                10,
            )],
            ttl_ops: vec![ttl_op(
                TtlKind::ExtendPersistent,
                StorageTier::Persistent,
                true,
            )],
            ..Default::default()
        };
        let func = build_public_fn("store", body);
        let file = build_simple_contract("Test", vec![func], true);
        let project = build_project(".", vec![file]);
        let findings = MissingTtl.check(&project);
        assert_eq!(findings.len(), 0);
    }

    #[test]
    fn skips_constructor() {
        let func = FunctionDef {
            is_constructor: true,
            ..build_public_fn("__constructor", build_body(vec![], vec![], vec![]))
        };
        let file = build_simple_contract("Test", vec![func], true);
        let project = build_project(".", vec![file]);
        let findings = MissingTtl.check(&project);
        assert_eq!(findings.len(), 0);
    }
}
