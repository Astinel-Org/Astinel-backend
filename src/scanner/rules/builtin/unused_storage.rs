use std::collections::HashSet;

use crate::core::{Ast, Category, Finding, Rule, RuleId, Severity};
use crate::scanner::parser::{ParsedProject, StorageOpKind};

#[derive(Debug, Clone)]
pub struct UnusedStorage;

impl Rule for UnusedStorage {
    fn id(&self) -> RuleId {
        RuleId::new("unused-storage").unwrap()
    }

    fn name(&self) -> &'static str {
        "Unused storage write"
    }

    fn description(&self) -> &'static str {
        "Detects storage writes whose key is never read"
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

        let read_keys: HashSet<&str> = p
            .all_functions()
            .filter_map(|f| f.body.as_ref())
            .flat_map(|b| &b.storage_ops)
            .filter(|op| matches!(op.kind, StorageOpKind::Get | StorageOpKind::Has))
            .map(|op| op.key.as_str())
            .collect();

        p.all_functions()
            .filter_map(|f| f.body.as_ref().map(|b| (f, b)))
            .flat_map(|(f, b)| {
                b.storage_ops.iter().filter_map(|op| {
                    if !matches!(op.kind, StorageOpKind::Set | StorageOpKind::Update) {
                        return None;
                    }
                    if read_keys.contains(op.key.as_str()) {
                        return None;
                    }
                    Some(Finding::new(
                        self.id(),
                        self.severity(),
                        self.category(),
                        op.span.clone(),
                        format!(
                            "Storage write to '{}' in function '{}' is never read",
                            op.key, f.name
                        ),
                        "Remove unused storage writes or verify the value is read elsewhere",
                    ))
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
    fn warns_on_unread_write() {
        let body = FunctionBody {
            storage_ops: vec![storage_op(
                StorageOpKind::Set,
                StorageTier::Persistent,
                "counter",
                10,
            )],
            ..Default::default()
        };
        let func = build_public_fn("inc", body);
        let file = build_simple_contract("Test", vec![func], true);
        let project = build_project(".", vec![file]);
        let findings = UnusedStorage.check(&project);
        assert_eq!(findings.len(), 1);
    }

    #[test]
    fn skips_when_key_is_read() {
        let body = FunctionBody {
            storage_ops: vec![
                storage_op(StorageOpKind::Get, StorageTier::Persistent, "counter", 5),
                storage_op(StorageOpKind::Set, StorageTier::Persistent, "counter", 10),
            ],
            ..Default::default()
        };
        let func = build_public_fn("inc", body);
        let file = build_simple_contract("Test", vec![func], true);
        let project = build_project(".", vec![file]);
        let findings = UnusedStorage.check(&project);
        assert_eq!(findings.len(), 0);
    }
}
