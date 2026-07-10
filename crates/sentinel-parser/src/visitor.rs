use crate::ast::*;

/// Trait for walking the curated AST without pattern matching.
///
/// Each method has a default no-op implementation. Override only the
/// methods relevant to your analysis.
pub trait AstVisitor {
    fn visit_project(&mut self, _project: &ParsedProject) {}

    fn visit_file(&mut self, _file: &ParsedFile) {}

    fn visit_contract(&mut self, _contract: &ContractDef) {}

    fn visit_impl_block(&mut self, _block: &ImplBlock) {}

    fn visit_function(&mut self, _func: &FunctionDef) {}

    fn visit_storage_op(&mut self, _op: &StorageOp) {}

    fn visit_auth_call(&mut self, _call: &AuthCall) {}

    fn visit_panic_op(&mut self, _op: &PanicOp) {}

    fn visit_arith_op(&mut self, _op: &ArithOp) {}

    fn visit_ttl_op(&mut self, _op: &TtlOp) {}

    fn visit_deployer_call(&mut self, _call: &DeployerCall) {}
}

pub fn walk_project(visitor: &mut impl AstVisitor, project: &ParsedProject) {
    visitor.visit_project(project);
    for file in &project.files {
        walk_file(visitor, file);
    }
}

pub fn walk_file(visitor: &mut impl AstVisitor, file: &ParsedFile) {
    visitor.visit_file(file);
    for contract in &file.contracts {
        walk_contract(visitor, contract);
    }
}

pub fn walk_contract(visitor: &mut impl AstVisitor, contract: &ContractDef) {
    visitor.visit_contract(contract);
    for ib in &contract.impl_blocks {
        walk_impl_block(visitor, ib);
    }
}

pub fn walk_impl_block(visitor: &mut impl AstVisitor, block: &ImplBlock) {
    visitor.visit_impl_block(block);
    for func in &block.functions {
        walk_function(visitor, func);
    }
}

pub fn walk_function(visitor: &mut impl AstVisitor, func: &FunctionDef) {
    visitor.visit_function(func);
    if let Some(body) = &func.body {
        for op in &body.storage_ops {
            visitor.visit_storage_op(op);
        }
        for call in &body.auth_calls {
            visitor.visit_auth_call(call);
        }
        for panic in &body.panics {
            visitor.visit_panic_op(panic);
        }
        for arith in &body.arith_ops {
            visitor.visit_arith_op(arith);
        }
        for ttl in &body.ttl_ops {
            visitor.visit_ttl_op(ttl);
        }
        for deploy in &body.deployer_calls {
            visitor.visit_deployer_call(deploy);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sentinel_core::DiagnosticSpan;

    struct CountingVisitor {
        functions: usize,
        storage_ops: usize,
        auth_calls: usize,
    }

    impl CountingVisitor {
        fn new() -> Self {
            Self {
                functions: 0,
                storage_ops: 0,
                auth_calls: 0,
            }
        }
    }

    impl AstVisitor for CountingVisitor {
        fn visit_function(&mut self, _func: &FunctionDef) {
            self.functions += 1;
        }
        fn visit_storage_op(&mut self, _op: &StorageOp) {
            self.storage_ops += 1;
        }
        fn visit_auth_call(&mut self, _call: &AuthCall) {
            self.auth_calls += 1;
        }
    }

    #[test]
    fn visitor_counts_correctly() {
        let body = FunctionBody {
            storage_ops: vec![StorageOp {
                kind: StorageOpKind::Set,
                storage_type: StorageTier::Persistent,
                key: "k".into(),
                span: DiagnosticSpan::new("f.rs", 1, 1),
            }],
            auth_calls: vec![AuthCall {
                kind: AuthCallKind::RequireAuth,
                target: "admin".into(),
                span: DiagnosticSpan::new("f.rs", 2, 1),
            }],
            ..Default::default()
        };

        let func = FunctionDef {
            name: "test".into(),
            span: DiagnosticSpan::new("f.rs", 1, 1),
            visibility: Visibility::Public,
            is_constructor: false,
            is_check_auth: false,
            signature: FunctionSignature::default(),
            body: Some(body),
        };

        let file = crate::ast::test_helpers::build_simple_contract("C", vec![func], true);
        let project = crate::ast::test_helpers::build_project(".", vec![file]);

        let mut visitor = CountingVisitor::new();
        walk_project(&mut visitor, &project);

        assert_eq!(visitor.functions, 1);
        assert_eq!(visitor.storage_ops, 1);
        assert_eq!(visitor.auth_calls, 1);
    }
}
