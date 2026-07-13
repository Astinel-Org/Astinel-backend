use crate::core::{Ast, Category, Finding, Rule, RuleId, Severity};
use crate::scanner::parser::ParsedProject;

#[derive(Debug, Clone)]
pub struct LargeStorageWrite;

impl Rule for LargeStorageWrite {
    fn id(&self) -> RuleId {
        RuleId::new("large-storage-write").unwrap()
    }

    fn name(&self) -> &'static str {
        "Large storage write"
    }

    fn description(&self) -> &'static str {
        "Flags storage writes with keys longer than 32 bytes"
    }

    fn severity(&self) -> Severity {
        Severity::Low
    }

    fn category(&self) -> Category {
        Category::Gas
    }

    fn check(&self, project: &dyn Ast) -> Vec<Finding> {
        let Some(p) = project.as_any().downcast_ref::<ParsedProject>() else {
            return Vec::new();
        };

        p.all_functions()
            .filter_map(|f| f.body.as_ref().map(|b| (f, b)))
            .flat_map(|(f, b)| {
                b.storage_ops.iter().filter_map(move |op| {
                    if op.key.len() > 32 {
                        Some(Finding::new(
                            self.id(),
                            self.severity(),
                            self.category(),
                            op.span.clone(),
                            format!(
                                "Large storage key '{}' ({} bytes) in function '{}'",
                                op.key,
                                op.key.len(),
                                f.name
                            ),
                            "Use shorter storage keys (≤ 32 bytes) to reduce costs",
                        ))
                    } else {
                        None
                    }
                })
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
    use crate::scanner::parser::{test_helpers::*, FunctionBody, StorageOpKind, StorageTier};

    #[test]
    fn detects_large_key() {
        let body = FunctionBody {
            storage_ops: vec![storage_op(
                StorageOpKind::Set,
                StorageTier::Persistent,
                "a".repeat(64).as_str(),
                10,
            )],
            ..Default::default()
        };
        let func = build_public_fn("store", body);
        let file = build_simple_contract("Test", vec![func], true);
        let project = build_project(".", vec![file]);
        let findings = LargeStorageWrite.check(&project);
        assert_eq!(findings.len(), 1);
    }

    #[test]
    fn skips_short_key() {
        let body = FunctionBody {
            storage_ops: vec![storage_op(
                StorageOpKind::Set,
                StorageTier::Persistent,
                "short",
                10,
            )],
            ..Default::default()
        };
        let func = build_public_fn("store", body);
        let file = build_simple_contract("Test", vec![func], true);
        let project = build_project(".", vec![file]);
        let findings = LargeStorageWrite.check(&project);
        assert_eq!(findings.len(), 0);
    }
}
