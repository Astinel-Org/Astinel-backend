use crate::core::{Ast, Category, Finding, Rule, RuleId, Severity};
use crate::scanner::parser::{ParsedProject, StorageOpKind};

#[derive(Debug, Clone)]
pub struct GasOptimization;

impl Rule for GasOptimization {
    fn id(&self) -> RuleId {
        RuleId::new("gas-optimization").unwrap()
    }

    fn name(&self) -> &'static str {
        "Gas optimization"
    }

    fn description(&self) -> &'static str {
        "Suggests gas optimizations for storage access patterns"
    }

    fn severity(&self) -> Severity {
        Severity::Info
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
            .filter(|(_, b)| {
                b.storage_ops
                    .iter()
                    .filter(|op| matches!(op.kind, StorageOpKind::Get))
                    .count()
                    > 5
            })
            .map(|(f, b)| {
                let read_count = b
                    .storage_ops
                    .iter()
                    .filter(|op| matches!(op.kind, StorageOpKind::Get))
                    .count();
                Finding::new(
                    self.id(),
                    self.severity(),
                    self.category(),
                    f.span.clone(),
                    format!(
                        "Function '{}' has {} storage reads, consider batching",
                        f.name, read_count
                    ),
                    "Use a single bulk read or cache values locally to reduce gas costs",
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
    use crate::scanner::parser::{test_helpers::*, FunctionBody, StorageOpKind, StorageTier};

    #[test]
    fn suggests_optimization_for_many_reads() {
        let ops = (0..7)
            .map(|i| storage_op(StorageOpKind::Get, StorageTier::Persistent, "key", 10 + i))
            .collect();
        let body = FunctionBody {
            storage_ops: ops,
            ..Default::default()
        };
        let func = build_public_fn("heavy", body);
        let file = build_simple_contract("Test", vec![func], true);
        let project = build_project(".", vec![file]);
        let findings = GasOptimization.check(&project);
        assert_eq!(findings.len(), 1);
    }

    #[test]
    fn no_optimization_for_few_reads() {
        let ops = (0..3)
            .map(|i| storage_op(StorageOpKind::Get, StorageTier::Persistent, "key", 10 + i))
            .collect();
        let body = FunctionBody {
            storage_ops: ops,
            ..Default::default()
        };
        let func = build_public_fn("light", body);
        let file = build_simple_contract("Test", vec![func], true);
        let project = build_project(".", vec![file]);
        let findings = GasOptimization.check(&project);
        assert_eq!(findings.len(), 0);
    }
}
