use sentinel_core::{Ast, Category, Finding, Rule, RuleId, Severity};
use sentinel_parser::{ArithKind, ParsedProject};

#[derive(Debug)]
pub struct IntegerOverflow;

impl Rule for IntegerOverflow {
    fn id(&self) -> RuleId {
        RuleId::new("integer-overflow").unwrap()
    }

    fn name(&self) -> &'static str {
        "Integer overflow"
    }

    fn description(&self) -> &'static str {
        "Detects unchecked arithmetic operations"
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
            .filter_map(|f| f.body.as_ref().map(|b| (f, b)))
            .flat_map(|(f, b)| {
                b.arith_ops.iter().filter_map(move |op| {
                    let is_unchecked = matches!(
                        op.kind,
                        ArithKind::Add
                            | ArithKind::Sub
                            | ArithKind::Mul
                            | ArithKind::Div
                            | ArithKind::CompoundAdd
                            | ArithKind::CompoundSub
                            | ArithKind::CompoundMul
                            | ArithKind::CompoundDiv
                    );
                    if is_unchecked {
                        Some(Finding::new(
                            self.id(),
                            self.severity(),
                            self.category(),
                            op.span.clone(),
                            format!("Unchecked {:?} in function '{}'", op.kind, f.name),
                            "Use checked arithmetic (checked_add, checked_mul, etc.) to prevent overflow",
                        ))
                    } else {
                        None
                    }
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sentinel_parser::{test_helpers::*, FunctionBody};

    #[test]
    fn detects_unchecked_add() {
        let body = FunctionBody {
            arith_ops: vec![arith_op(ArithKind::Add, false)],
            ..Default::default()
        };
        let func = build_public_fn("add", body);
        let file = build_simple_contract("Test", vec![func], true);
        let project = build_project(".", vec![file]);
        let findings = IntegerOverflow.check(&project);
        assert_eq!(findings.len(), 1);
    }

    #[test]
    fn skips_checked_ops() {
        let body = FunctionBody {
            arith_ops: vec![arith_op(ArithKind::CheckedOp, true)],
            ..Default::default()
        };
        let func = build_public_fn("safe_add", body);
        let file = build_simple_contract("Test", vec![func], true);
        let project = build_project(".", vec![file]);
        let findings = IntegerOverflow.check(&project);
        assert_eq!(findings.len(), 0);
    }

    #[test]
    fn skips_wrapping_ops() {
        let body = FunctionBody {
            arith_ops: vec![arith_op(ArithKind::WrappingOp, false)],
            ..Default::default()
        };
        let func = build_public_fn("wrap", body);
        let file = build_simple_contract("Test", vec![func], true);
        let project = build_project(".", vec![file]);
        let findings = IntegerOverflow.check(&project);
        assert_eq!(findings.len(), 0);
    }
}
