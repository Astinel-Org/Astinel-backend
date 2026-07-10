use sentinel_core::{Ast, DiagnosticSpan};
use std::any::Any;
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Top-level
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedProject {
    pub root: PathBuf,
    pub manifest: Option<crate::project::CargoManifest>,
    pub files: Vec<ParsedFile>,
    file_paths: Vec<PathBuf>,
}

impl Ast for ParsedProject {
    fn files(&self) -> &[PathBuf] {
        &self.file_paths
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl ParsedProject {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            manifest: None,
            files: Vec::new(),
            file_paths: Vec::new(),
        }
    }

    pub fn add_file(&mut self, file: ParsedFile) {
        self.file_paths.push(file.path.clone());
        self.files.push(file);
    }

    pub fn all_contracts(&self) -> impl Iterator<Item = &ContractDef> {
        self.files.iter().flat_map(|f| f.contracts.iter())
    }

    pub fn all_functions(&self) -> impl Iterator<Item = &FunctionDef> {
        self.files
            .iter()
            .flat_map(|f| {
                f.contracts
                    .iter()
                    .flat_map(|c| c.impl_blocks.iter().flat_map(|ib| ib.functions.iter()))
            })
            .chain(self.files.iter().flat_map(|f| f.free_functions.iter()))
    }

    pub fn all_storage_ops(&self) -> impl Iterator<Item = &StorageOp> {
        self.all_functions()
            .filter_map(|f| f.body.as_ref())
            .flat_map(|b| b.storage_ops.iter())
    }
}

// ---------------------------------------------------------------------------
// File
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParsedFile {
    pub path: PathBuf,
    pub contracts: Vec<ContractDef>,
    pub error_types: Vec<ErrorTypeDef>,
    pub contract_types: Vec<ContractTypeDef>,
    pub free_functions: Vec<FunctionDef>,
    pub has_no_std: bool,
    pub parse_error: Option<String>,
}

impl ParsedFile {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            contracts: Vec::new(),
            error_types: Vec::new(),
            contract_types: Vec::new(),
            free_functions: Vec::new(),
            has_no_std: false,
            parse_error: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractDef {
    pub name: String,
    pub struct_span: DiagnosticSpan,
    pub is_contract: bool,
    pub impl_blocks: Vec<ImplBlock>,
}

impl ContractDef {
    pub fn new(name: impl Into<String>, struct_span: DiagnosticSpan) -> Self {
        Self {
            name: name.into(),
            struct_span,
            is_contract: false,
            impl_blocks: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplBlock {
    pub span: DiagnosticSpan,
    pub functions: Vec<FunctionDef>,
    pub is_trait_impl: bool,
}

impl ImplBlock {
    pub fn new(span: DiagnosticSpan) -> Self {
        Self {
            span,
            functions: Vec::new(),
            is_trait_impl: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Function
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionDef {
    pub name: String,
    pub span: DiagnosticSpan,
    pub visibility: Visibility,
    pub is_constructor: bool,
    pub is_check_auth: bool,
    pub signature: FunctionSignature,
    pub body: Option<FunctionBody>,
}

impl FunctionDef {
    pub fn new(name: impl Into<String>, span: DiagnosticSpan) -> Self {
        Self {
            name: name.into(),
            span,
            visibility: Visibility::Private,
            is_constructor: false,
            is_check_auth: false,
            signature: FunctionSignature::default(),
            body: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Visibility {
    Public,
    PublicCrate,
    PublicSuper,
    Private,
}

impl Visibility {
    pub fn is_public(&self) -> bool {
        matches!(self, Visibility::Public)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FunctionSignature {
    pub params: Vec<Parameter>,
    pub returns_result: bool,
    pub return_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub name: String,
    pub type_name: String,
    pub span: DiagnosticSpan,
}

// ---------------------------------------------------------------------------
// Function Body
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FunctionBody {
    pub storage_ops: Vec<StorageOp>,
    pub auth_calls: Vec<AuthCall>,
    pub panics: Vec<PanicOp>,
    pub arith_ops: Vec<ArithOp>,
    pub ttl_ops: Vec<TtlOp>,
    pub deployer_calls: Vec<DeployerCall>,
    pub cross_contract_calls: Vec<CrossContractCall>,
}

impl FunctionBody {
    pub fn is_empty(&self) -> bool {
        self.storage_ops.is_empty()
            && self.auth_calls.is_empty()
            && self.panics.is_empty()
            && self.arith_ops.is_empty()
            && self.ttl_ops.is_empty()
            && self.deployer_calls.is_empty()
            && self.cross_contract_calls.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Storage Operations
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageOp {
    pub kind: StorageOpKind,
    pub storage_type: StorageTier,
    pub key: String,
    pub span: DiagnosticSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageOpKind {
    Get,
    Set,
    Has,
    Remove,
    Update,
    TryUpdate,
    ExtendTtl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageTier {
    Persistent,
    Temporary,
    Instance,
}

// ---------------------------------------------------------------------------
// Authorization
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthCall {
    pub kind: AuthCallKind,
    pub target: String,
    pub span: DiagnosticSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthCallKind {
    RequireAuth,
    RequireAuthForArgs,
}

// ---------------------------------------------------------------------------
// Panic Operations
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanicOp {
    pub kind: PanicKind,
    pub message: String,
    pub span: DiagnosticSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanicKind {
    DirectPanic,
    PanicWithError,
    AssertWithError,
    Unwrap,
    Expect,
    Unreachable,
    Unimplemented,
}

// ---------------------------------------------------------------------------
// Arithmetic Operations
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArithOp {
    pub kind: ArithKind,
    pub span: DiagnosticSpan,
    pub left_type: Option<String>,
    pub has_overflow_check: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArithKind {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Neg,
    Shl,
    Shr,
    CompoundAdd,
    CompoundSub,
    CompoundMul,
    CompoundDiv,
    CompoundRem,
    WrappingOp,
    CheckedOp,
    SaturatingOp,
}

// ---------------------------------------------------------------------------
// TTL Operations
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TtlOp {
    pub kind: TtlKind,
    pub storage_type: Option<StorageTier>,
    pub has_extend_after_write: bool,
    pub span: DiagnosticSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TtlKind {
    ExtendInstance,
    ExtendPersistent,
    ExtendTemporary,
    DeployerExtend,
}

// ---------------------------------------------------------------------------
// Deployer / Upgrade
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeployerCall {
    pub kind: DeployerCallKind,
    pub span: DiagnosticSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeployerCallKind {
    UploadWasm,
    Deploy,
    DeployWithCurrentContract,
    DeployWithAddress,
    UpdateCurrentContractWasm,
}

// ---------------------------------------------------------------------------
// Cross-contract calls
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossContractCall {
    pub contract: String,
    pub function: String,
    pub span: DiagnosticSpan,
    pub is_try_call: bool,
}

// ---------------------------------------------------------------------------
// Type definitions (contracterror, contracttype)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorTypeDef {
    pub name: String,
    pub span: DiagnosticSpan,
    pub variants: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractTypeDef {
    pub name: String,
    pub span: DiagnosticSpan,
    pub kind: ContractTypeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractTypeKind {
    Struct,
    Enum,
}

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

pub mod test_helpers {
    use super::*;
    use sentinel_core::DiagnosticSpan;

    /// Build a minimal ParsedProject from source text for testing rules.
    /// The source is parsed into a single file at a fake path.
    pub fn parse_contract(_source: &str) -> ParsedProject {
        // TODO: Phase 3+ will implement full syn-based parsing.
        // For now, tests create ASTs manually using the builders below.
        unimplemented!("full parser not yet implemented; use build_ functions in tests")
    }

    /// Build a ParsedProject from a list of ParsedFiles.
    pub fn build_project(root: &str, files: Vec<ParsedFile>) -> ParsedProject {
        let file_paths = files.iter().map(|f| f.path.clone()).collect();
        ParsedProject {
            root: PathBuf::from(root),
            manifest: None,
            files,
            file_paths,
        }
    }

    /// Build a file with a single contract containing one impl block.
    pub fn build_simple_contract(name: &str, functions: Vec<FunctionDef>, is_contract: bool) -> ParsedFile {
        let span = DiagnosticSpan::new("contract.rs", 1, 1);
        let contract = ContractDef {
            name: name.to_string(),
            struct_span: span.clone(),
            is_contract,
            impl_blocks: vec![ImplBlock {
                span: span.clone(),
                functions,
                is_trait_impl: false,
            }],
        };
        ParsedFile {
            path: PathBuf::from("contract.rs"),
            contracts: vec![contract],
            error_types: vec![],
            contract_types: vec![],
            free_functions: vec![],
            has_no_std: true,
            parse_error: None,
        }
    }

    /// Build a public function with the given body.
    pub fn build_public_fn(name: &str, body: FunctionBody) -> FunctionDef {
        FunctionDef {
            name: name.to_string(),
            span: DiagnosticSpan::new("contract.rs", 10, 5),
            visibility: Visibility::Public,
            is_constructor: false,
            is_check_auth: false,
            signature: FunctionSignature::default(),
            body: Some(body),
        }
    }

    /// Build a private function with the given body.
    pub fn build_private_fn(name: &str, body: FunctionBody) -> FunctionDef {
        FunctionDef {
            name: name.to_string(),
            span: DiagnosticSpan::new("contract.rs", 10, 5),
            visibility: Visibility::Private,
            is_constructor: false,
            is_check_auth: false,
            signature: FunctionSignature::default(),
            body: Some(body),
        }
    }

    /// Build a __constructor function.
    pub fn build_constructor(body: Option<FunctionBody>) -> FunctionDef {
        FunctionDef {
            name: "__constructor".to_string(),
            span: DiagnosticSpan::new("contract.rs", 10, 5),
            visibility: Visibility::Public,
            is_constructor: true,
            is_check_auth: false,
            signature: FunctionSignature::default(),
            body,
        }
    }

    /// Build a __check_auth function.
    pub fn build_check_auth(body: Option<FunctionBody>) -> FunctionDef {
        FunctionDef {
            name: "__check_auth".to_string(),
            span: DiagnosticSpan::new("contract.rs", 10, 5),
            visibility: Visibility::Public,
            is_constructor: false,
            is_check_auth: true,
            signature: FunctionSignature::default(),
            body,
        }
    }

    /// Build a function body with the given operations.
    pub fn build_body(storage_ops: Vec<StorageOp>, auth_calls: Vec<AuthCall>, panics: Vec<PanicOp>) -> FunctionBody {
        FunctionBody {
            storage_ops,
            auth_calls,
            panics,
            ..Default::default()
        }
    }

    /// Create a storage op.
    pub fn storage_op(kind: StorageOpKind, storage_type: StorageTier, key: &str, line: usize) -> StorageOp {
        StorageOp {
            kind,
            storage_type,
            key: key.to_string(),
            span: DiagnosticSpan::new("contract.rs", line, 1),
        }
    }

    /// Create an auth call.
    pub fn auth_call(target: &str) -> AuthCall {
        AuthCall {
            kind: AuthCallKind::RequireAuth,
            target: target.to_string(),
            span: DiagnosticSpan::new("contract.rs", 15, 5),
        }
    }

    /// Create a panic op.
    pub fn panic_op(kind: PanicKind, msg: &str) -> PanicOp {
        PanicOp {
            kind,
            message: msg.to_string(),
            span: DiagnosticSpan::new("contract.rs", 20, 5),
        }
    }

    /// Create an arith op.
    pub fn arith_op(kind: ArithKind, overflow_check: bool) -> ArithOp {
        ArithOp {
            kind,
            span: DiagnosticSpan::new("contract.rs", 25, 5),
            left_type: None,
            has_overflow_check: overflow_check,
        }
    }

    /// Create a TTL op.
    pub fn ttl_op(kind: TtlKind, tier: StorageTier, extend_after_write: bool) -> TtlOp {
        TtlOp {
            kind,
            storage_type: Some(tier),
            has_extend_after_write: extend_after_write,
            span: DiagnosticSpan::new("contract.rs", 30, 5),
        }
    }

    /// Create a deployer call.
    pub fn deployer_call(kind: DeployerCallKind) -> DeployerCall {
        DeployerCall {
            kind,
            span: DiagnosticSpan::new("contract.rs", 35, 5),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sentinel_core::Ast;

    #[test]
    fn parsed_project_implements_ast() {
        let project = ParsedProject::new(PathBuf::from("/test"));
        let ast: &dyn Ast = &project;
        assert!(ast.files().is_empty());
    }

    #[test]
    fn parsed_project_downcast() {
        let project = ParsedProject::new(PathBuf::from("/test"));
        let ast: &dyn Ast = &project;
        let down = ast.as_any().downcast_ref::<ParsedProject>();
        assert!(down.is_some());
    }

    #[test]
    fn function_body_default_is_empty() {
        let body = FunctionBody::default();
        assert!(body.is_empty());
    }

    #[test]
    fn function_body_with_ops_not_empty() {
        let body = FunctionBody {
            storage_ops: vec![StorageOp {
                kind: StorageOpKind::Set,
                storage_type: StorageTier::Persistent,
                key: "counter".into(),
                span: DiagnosticSpan::new("test.rs", 1, 1),
            }],
            ..Default::default()
        };
        assert!(!body.is_empty());
    }

    #[test]
    fn visibility_is_public() {
        assert!(Visibility::Public.is_public());
        assert!(!Visibility::Private.is_public());
    }

    #[test]
    fn all_contracts_iterator() {
        let file = ParsedFile {
            path: PathBuf::from("c.rs"),
            contracts: vec![ContractDef::new("MyContract", DiagnosticSpan::new("c.rs", 1, 1))],
            ..Default::default()
        };
        let project = ParsedProject {
            root: PathBuf::from("."),
            manifest: None,
            files: vec![file],
            file_paths: vec![PathBuf::from("c.rs")],
        };
        assert_eq!(project.all_contracts().count(), 1);
    }

    #[test]
    fn all_functions_iterator() {
        let func = FunctionDef::new("test_fn", DiagnosticSpan::new("f.rs", 1, 1));
        let file = test_helpers::build_simple_contract("C", vec![func], true);
        let project = test_helpers::build_project(".", vec![file]);
        assert_eq!(project.all_functions().count(), 1);
    }
}
