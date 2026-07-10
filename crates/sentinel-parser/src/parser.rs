use std::path::{Path, PathBuf};

use quote::ToTokens;
use syn::spanned::Spanned;

use crate::ast::*;
use crate::error::ParserError;
use sentinel_core::DiagnosticSpan;

pub fn parse_project(path: &Path) -> Result<ParsedProject, ParserError> {
    if !path.exists() {
        return Err(ParserError::InvalidProject {
            path: path.to_path_buf(),
            detail: "path does not exist".to_string(),
        });
    }

    let root = if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent().unwrap_or(path).to_path_buf()
    };

    let mut project = ParsedProject::new(root.clone());

    if path.is_file() && path.extension().is_some_and(|e| e == "rs") {
        let content = std::fs::read_to_string(path)?;
        let file = parse_single_file(path, &content);
        project.add_file(file);
    } else if path.is_dir() {
        let cargo_toml = path.join("Cargo.toml");
        if cargo_toml.exists() {
            match std::fs::read_to_string(&cargo_toml) {
                Ok(contents) => {
                    let manifest = parse_cargo_toml(&cargo_toml, &contents);
                    project.manifest = Some(manifest);
                }
                Err(e) => {
                    return Err(ParserError::InvalidProject {
                        path: cargo_toml,
                        detail: format!("cannot read Cargo.toml: {e}"),
                    });
                }
            }
        }
        discover_rs_files(&root, &mut project)?;
    }

    Ok(project)
}

fn parse_cargo_toml(path: &Path, contents: &str) -> crate::project::CargoManifest {
    let mut manifest = crate::project::CargoManifest::new(path.to_path_buf());
    if let Ok(toml) = contents.parse::<toml::Value>() {
        if let Some(package) = toml.get("package") {
            manifest.package_name = package.get("name").and_then(|v| v.as_str().map(String::from));
        }
        if let Some(deps) = toml.get("dependencies").and_then(|d| d.as_table()) {
            for key in deps.keys() {
                manifest.dependencies.push(key.clone());
                if key.starts_with("soroban-sdk") {
                    manifest.has_soroban_sdk = true;
                }
            }
        }
        if let Some(workspace) = toml.get("workspace") {
            manifest.is_workspace = true;
            if let Some(members) = workspace.get("members").and_then(|m| m.as_array()) {
                for member in members {
                    if let Some(m) = member.as_str() {
                        manifest.members.push(PathBuf::from(m));
                    }
                }
            }
        }
    }
    manifest
}

fn discover_rs_files(dir: &Path, project: &mut ParsedProject) -> Result<(), ParserError> {
    for entry in walkdir::WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| {
            if e.depth() == 0 {
                return true;
            }
            let name = e.file_name().to_str().unwrap_or("");
            !name.starts_with('.') && name != "target" && name != "node_modules"
        })
    {
        match entry {
            Ok(entry) => {
                if entry.file_type().is_file() && entry.path().extension().is_some_and(|e| e == "rs") {
                    let path = entry.path().to_path_buf();
                    match std::fs::read_to_string(&path) {
                        Ok(content) => {
                            let file = parse_single_file(&path, &content);
                            project.add_file(file);
                        }
                        Err(e) => {
                            tracing::warn!("skipping `{}`: {}", path.display(), e);
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!("walk error: {}", e);
            }
        }
    }
    Ok(())
}

fn parse_single_file(path: &Path, content: &str) -> ParsedFile {
    let mut file = ParsedFile::new(path.to_path_buf());

    let syn_file = match syn::parse_file(content) {
        Ok(f) => f,
        Err(e) => {
            file.parse_error = Some(e.to_string());
            return file;
        }
    };

    file.has_no_std = syn_file.attrs.iter().any(|attr| attr.path().is_ident("no_std"));

    for item in &syn_file.items {
        match item {
            syn::Item::Struct(s) => {
                if has_attr(&s.attrs, "contract") {
                    let sl = s.ident.span();
                    let span = DiagnosticSpan::new(path.to_path_buf(), sl.start().line, sl.start().column + 1);
                    let mut contract = ContractDef::new(s.ident.to_string(), span);
                    contract.is_contract = true;
                    file.contracts.push(contract);
                }
                if has_attr(&s.attrs, "contracttype") {
                    let sl = s.ident.span();
                    let span = DiagnosticSpan::new(path.to_path_buf(), sl.start().line, sl.start().column + 1);
                    file.contract_types.push(ContractTypeDef {
                        name: s.ident.to_string(),
                        span,
                        kind: ContractTypeKind::Struct,
                    });
                }
            }
            syn::Item::Enum(e) => {
                if has_attr(&e.attrs, "contracterror") {
                    let sl = e.ident.span();
                    let span = DiagnosticSpan::new(path.to_path_buf(), sl.start().line, sl.start().column + 1);
                    file.error_types.push(ErrorTypeDef {
                        name: e.ident.to_string(),
                        span,
                        variants: e.variants.iter().map(|v| v.ident.to_string()).collect(),
                    });
                }
                if has_attr(&e.attrs, "contracttype") {
                    let sl = e.ident.span();
                    let span = DiagnosticSpan::new(path.to_path_buf(), sl.start().line, sl.start().column + 1);
                    file.contract_types.push(ContractTypeDef {
                        name: e.ident.to_string(),
                        span,
                        kind: ContractTypeKind::Enum,
                    });
                }
            }
            syn::Item::Impl(imp) => {
                if has_attr(&imp.attrs, "contractimpl") {
                    let il = imp.self_ty.span();
                    let span = DiagnosticSpan::new(path.to_path_buf(), il.start().line, il.start().column + 1);
                    let mut impl_block = ImplBlock::new(span);
                    impl_block.is_trait_impl = imp.trait_.is_some();

                    for item in &imp.items {
                        if let syn::ImplItem::Fn(method) = item {
                            let fn_span = DiagnosticSpan::new(
                                path.to_path_buf(),
                                method.sig.ident.span().start().line,
                                method.sig.ident.span().start().column + 1,
                            );
                            let mut func = FunctionDef::new(method.sig.ident.to_string(), fn_span);
                            func.visibility = parse_visibility(&method.vis);
                            func.is_constructor = method.sig.ident == "__constructor";
                            func.is_check_auth = method.sig.ident == "__check_auth";

                            func.signature.params = method
                                .sig
                                .inputs
                                .iter()
                                .filter_map(|p| {
                                    if let syn::FnArg::Typed(pat_type) = p {
                                        let name = match pat_type.pat.as_ref() {
                                            syn::Pat::Ident(pat_ident) => pat_ident.ident.to_string(),
                                            _ => "_".to_string(),
                                        };
                                        let type_name = quote::quote! { #pat_type.ty }.to_string();
                                        let ps = pat_type.span();
                                        let param_span = DiagnosticSpan::new(
                                            path.to_path_buf(),
                                            ps.start().line,
                                            ps.start().column + 1,
                                        );
                                        Some(Parameter {
                                            name,
                                            type_name,
                                            span: param_span,
                                        })
                                    } else {
                                        None
                                    }
                                })
                                .collect();

                            func.signature.return_type = match &method.sig.output {
                                syn::ReturnType::Type(_, ty) => Some(quote::quote! { #ty }.to_string()),
                                syn::ReturnType::Default => None,
                            };
                            func.signature.returns_result = func
                                .signature
                                .return_type
                                .as_deref()
                                .is_some_and(|t| t.contains("Result<"));

                            let body = analyze_body(path, &method.block);
                            func.body = Some(body);
                            impl_block.functions.push(func);
                        }
                    }

                    if let Some(contract) = file.contracts.last_mut() {
                        contract.impl_blocks.push(impl_block);
                    }
                }
            }
            syn::Item::Fn(free_fn) => {
                let fn_span = DiagnosticSpan::new(
                    path.to_path_buf(),
                    free_fn.sig.ident.span().start().line,
                    free_fn.sig.ident.span().start().column + 1,
                );
                let mut func = FunctionDef::new(free_fn.sig.ident.to_string(), fn_span);
                func.visibility = parse_visibility(&free_fn.vis);

                let body = analyze_body(path, &free_fn.block);
                func.body = Some(body);
                file.free_functions.push(func);
            }
            _ => {}
        }
    }

    file
}

fn analyze_body(path: &Path, block: &syn::Block) -> FunctionBody {
    let mut body = FunctionBody::default();
    let path_buf = path.to_path_buf();

    for stmt in &block.stmts {
        if let syn::Stmt::Expr(expr, _) = stmt {
            collect_from_expr(expr, &mut body, &path_buf);
        }
        if let syn::Stmt::Local(local) = stmt {
            if let Some(init) = &local.init {
                collect_from_expr(&init.expr, &mut body, &path_buf);
            }
        }
        if let syn::Stmt::Macro(mac) = stmt {
            if mac.mac.path.is_ident("panic_with_error") {
                body.panics.push(PanicOp {
                    kind: PanicKind::PanicWithError,
                    message: quote::quote! { #mac.mac.tokens }.to_string(),
                    span: DiagnosticSpan::new(
                        path_buf.clone(),
                        mac.mac.bang_token.span.start().line,
                        mac.mac.bang_token.span.start().column + 1,
                    ),
                });
            }
        }
    }

    body
}

#[allow(clippy::too_many_lines)]
fn collect_from_expr(expr: &syn::Expr, body: &mut FunctionBody, path: &PathBuf) {
    match expr {
        syn::Expr::Call(call) => {
            if let syn::Expr::Path(path_expr) = call.func.as_ref() {
                if let Some(seg) = path_expr.path.segments.last() {
                    if seg.ident == "require_auth" {
                        body.auth_calls.push(AuthCall {
                            kind: AuthCallKind::RequireAuth,
                            target: "call".to_string(),
                            span: DiagnosticSpan::new(
                                path.clone(),
                                call.func.span().start().line,
                                call.func.span().start().column + 1,
                            ),
                        });
                    }
                }
            }
        }
        syn::Expr::MethodCall(mc) => match mc.method.to_string().as_str() {
            "require_auth" => {
                body.auth_calls.push(AuthCall {
                    kind: AuthCallKind::RequireAuth,
                    target: format_receiver(&mc.receiver),
                    span: DiagnosticSpan::new(
                        path.clone(),
                        mc.method.span().start().line,
                        mc.method.span().start().column + 1,
                    ),
                });
            }
            "require_auth_for_args" => {
                body.auth_calls.push(AuthCall {
                    kind: AuthCallKind::RequireAuthForArgs,
                    target: format_receiver(&mc.receiver),
                    span: DiagnosticSpan::new(
                        path.clone(),
                        mc.method.span().start().line,
                        mc.method.span().start().column + 1,
                    ),
                });
            }
            "unwrap" => {
                body.panics.push(PanicOp {
                    kind: PanicKind::Unwrap,
                    message: String::new(),
                    span: DiagnosticSpan::new(
                        path.clone(),
                        mc.method.span().start().line,
                        mc.method.span().start().column + 1,
                    ),
                });
            }
            "expect" => {
                let msg = mc
                    .args
                    .first()
                    .map(|a| quote::quote! { #a }.to_string())
                    .unwrap_or_default();
                body.panics.push(PanicOp {
                    kind: PanicKind::Expect,
                    message: msg,
                    span: DiagnosticSpan::new(
                        path.clone(),
                        mc.method.span().start().line,
                        mc.method.span().start().column + 1,
                    ),
                });
            }
            "get" | "set" | "has" | "remove" => {
                let kind = match mc.method.to_string().as_str() {
                    "get" => StorageOpKind::Get,
                    "set" => StorageOpKind::Set,
                    "has" => StorageOpKind::Has,
                    "remove" => StorageOpKind::Remove,
                    _ => unreachable!(),
                };
                let tier = detect_storage_tier(&mc.receiver);
                let key = mc
                    .args
                    .first()
                    .map(|a| quote::quote! { #a }.to_string())
                    .unwrap_or_default();
                body.storage_ops.push(StorageOp {
                    kind,
                    storage_type: tier,
                    key,
                    span: DiagnosticSpan::new(
                        path.clone(),
                        mc.method.span().start().line,
                        mc.method.span().start().column + 1,
                    ),
                });
            }
            "extend_ttl" => {
                let tier = detect_storage_tier(&mc.receiver);
                body.ttl_ops.push(TtlOp {
                    kind: match tier {
                        StorageTier::Persistent => TtlKind::ExtendPersistent,
                        StorageTier::Temporary => TtlKind::ExtendTemporary,
                        StorageTier::Instance => TtlKind::ExtendInstance,
                    },
                    storage_type: Some(tier),
                    has_extend_after_write: false,
                    span: DiagnosticSpan::new(
                        path.clone(),
                        mc.method.span().start().line,
                        mc.method.span().start().column + 1,
                    ),
                });
            }
            "update_current_contract_wasm" => {
                body.deployer_calls.push(DeployerCall {
                    kind: DeployerCallKind::UpdateCurrentContractWasm,
                    span: DiagnosticSpan::new(
                        path.clone(),
                        mc.method.span().start().line,
                        mc.method.span().start().column + 1,
                    ),
                });
            }
            _ => {}
        },
        syn::Expr::Macro(em) => {
            let macro_name = em.mac.path.to_token_stream().to_string();
            let span = DiagnosticSpan::new(
                path.clone(),
                em.mac.bang_token.span.start().line,
                em.mac.bang_token.span.start().column + 1,
            );
            if em.mac.path.is_ident("panic") {
                body.panics.push(PanicOp {
                    kind: PanicKind::DirectPanic,
                    message: macro_name.clone(),
                    span: span.clone(),
                });
            }
            if em.mac.path.is_ident("panic_with_error") {
                body.panics.push(PanicOp {
                    kind: PanicKind::PanicWithError,
                    message: macro_name.clone(),
                    span: span.clone(),
                });
            }
            if em.mac.path.is_ident("assert_with_error") {
                body.panics.push(PanicOp {
                    kind: PanicKind::AssertWithError,
                    message: macro_name,
                    span,
                });
            }
        }
        syn::Expr::Binary(bin) => {
            let arith_kind = match &bin.op {
                syn::BinOp::Add(_) => Some(ArithKind::Add),
                syn::BinOp::Sub(_) => Some(ArithKind::Sub),
                syn::BinOp::Mul(_) => Some(ArithKind::Mul),
                syn::BinOp::Div(_) => Some(ArithKind::Div),
                syn::BinOp::Rem(_) => Some(ArithKind::Rem),
                syn::BinOp::Shl(_) => Some(ArithKind::Shl),
                syn::BinOp::Shr(_) => Some(ArithKind::Shr),
                _ => None,
            };
            if let Some(kind) = arith_kind {
                body.arith_ops.push(ArithOp {
                    kind,
                    span: DiagnosticSpan::new(
                        path.clone(),
                        bin.op.span().start().line,
                        bin.op.span().start().column + 1,
                    ),
                    left_type: None,
                    has_overflow_check: false,
                });
            }
            collect_from_expr(&bin.left, body, path);
            collect_from_expr(&bin.right, body, path);
        }
        syn::Expr::Assign(assign) => {
            collect_from_expr(&assign.left, body, path);
            collect_from_expr(&assign.right, body, path);
        }
        syn::Expr::If(if_expr) => {
            collect_from_expr(&if_expr.cond, body, path);
            for stmt in &if_expr.then_branch.stmts {
                if let syn::Stmt::Expr(e, _) = stmt {
                    collect_from_expr(e, body, path);
                }
                if let syn::Stmt::Local(l) = stmt {
                    if let Some(init) = &l.init {
                        collect_from_expr(&init.expr, body, path);
                    }
                }
            }
            if let Some((_, else_branch)) = &if_expr.else_branch {
                match else_branch.as_ref() {
                    syn::Expr::Block(block) => {
                        for stmt in &block.block.stmts {
                            if let syn::Stmt::Expr(e, _) = stmt {
                                collect_from_expr(e, body, path);
                            }
                        }
                    }
                    other => {
                        collect_from_expr(other, body, path);
                    }
                }
            }
        }
        syn::Expr::Block(block) => {
            for stmt in &block.block.stmts {
                if let syn::Stmt::Expr(e, _) = stmt {
                    collect_from_expr(e, body, path);
                }
            }
        }
        _ => {}
    }
}

fn detect_storage_tier(receiver: &syn::Expr) -> StorageTier {
    if let syn::Expr::MethodCall(mc) = receiver {
        match mc.method.to_string().as_str() {
            "persistent" => StorageTier::Persistent,
            "temporary" => StorageTier::Temporary,
            "instance" => StorageTier::Instance,
            _ => StorageTier::Persistent,
        }
    } else {
        StorageTier::Persistent
    }
}

fn format_receiver(expr: &syn::Expr) -> String {
    match expr {
        syn::Expr::Path(p) => p.path.segments.last().map(|s| s.ident.to_string()).unwrap_or_default(),
        syn::Expr::MethodCall(mc) => mc.method.to_string(),
        syn::Expr::Field(field) => quote::quote! { #field.member }.to_string(),
        _ => quote::quote! { #expr }.to_string(),
    }
}

fn has_attr(attrs: &[syn::Attribute], name: &str) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident(name))
}

fn parse_visibility(vis: &syn::Visibility) -> crate::ast::Visibility {
    match vis {
        syn::Visibility::Public(_) => crate::ast::Visibility::Public,
        syn::Visibility::Restricted(restricted) => {
            if restricted.path.is_ident("crate") {
                crate::ast::Visibility::PublicCrate
            } else {
                crate::ast::Visibility::PublicSuper
            }
        }
        syn::Visibility::Inherited => crate::ast::Visibility::Private,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn parse_non_existent_path() {
        let result = parse_project(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    #[test]
    fn parse_rust_file() {
        let dir = std::env::temp_dir().join("sentinel_test_parse");
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("empty.rs");
        std::fs::write(&file, "").unwrap();

        let project = parse_project(&file).unwrap();
        assert_eq!(project.files.len(), 1);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_contract_file() {
        let dir = std::env::temp_dir().join("sentinel_test_contract2");
        let _ = std::fs::create_dir_all(dir.join("src"));
        let file = dir.join("src").join("lib.rs");
        std::fs::write(
            &file,
            r#"
            #![no_std]
            use soroban_sdk::{contract, contractimpl, Env, Address};

            #[contract]
            pub struct MyContract;

            #[contractimpl]
            impl MyContract {
                pub fn hello(env: Env, to: Address) {}
            }
            "#,
        )
        .unwrap();

        let project = parse_project(&file).unwrap();
        assert_eq!(project.files.len(), 1);
        let pf = &project.files[0];
        assert!(pf.parse_error.is_none(), "parse error: {:?}", pf.parse_error);
        assert_eq!(pf.contracts.len(), 1);
        assert_eq!(pf.contracts[0].name, "MyContract");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
