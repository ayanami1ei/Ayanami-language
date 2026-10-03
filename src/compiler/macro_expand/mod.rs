//! A5b-2：用户宏展开（M3 插件 ABI，Ayanami 优先）。
//!
//! 流程：
//! 1. 从已重写为 `.lcl` 的 import 收集宏表（`macro=` 行）与短名映射；
//! 2. 对带库宏标注的 item：格式化其源码 → 从宏库 LIR 构建插件 `.so`
//!    （llc -relocation-model=pic + C shim + runtime.c）→ dlopen 调用 → 得到新源码；
//! 3. 重新解析并递归展开（深度上限 32），输出替换原 item。

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::parser::ast::{Attr, Program, Stmt};

const MAX_DEPTH: usize = 32;

mod plugin;

/// 展开入口：在 HIR 降级前调用（此时 import 已重写为 .lcl 绝对路径）。
pub fn expand(program: &Program, src_path: &Path) -> Result<Program> {
    let ctx = MacroCtx::collect(program, src_path)?;
    if ctx.tables.is_empty() && ctx.short.is_empty() {
        return Ok(program.clone());
    }
    let stmts = expand_stmts(&program.stmts, &ctx, 0)?;
    Ok(Program::new(stmts))
}

struct MacroCtx {
    /// 包 stem → (lcl 路径, 宏名列表)
    tables: HashMap<String, (String, Vec<String>)>,
    /// import 短名 → 包 stem
    short: HashMap<String, String>,
    src_path: PathBuf,
}

impl MacroCtx {
    fn collect(program: &Program, src_path: &Path) -> Result<Self> {
        let mut tables = HashMap::new();
        let mut short = HashMap::new();
        collect_imports(&program.stmts, &mut tables, &mut short)?;
        Ok(Self { tables, short, src_path: src_path.to_path_buf() })
    }

    fn resolve(&self, a: &Attr) -> Option<(String, String)> {
        let name = a.name.as_str();
        if !a.qualifier.is_empty() {
            let pkg = a.qualifier[0].as_str();
            let rest: Vec<String> = a.qualifier[1..].iter().map(|s| s.as_str()).collect();
            let macro_name = if rest.is_empty() { name } else { format!("{}.{}", rest.join("."), name) };
            if self.tables.contains_key(&pkg) {
                return Some((pkg, macro_name));
            }
            return None;
        }
        self.short.get(&name).map(|pkg| (pkg.clone(), name))
    }
}

fn collect_imports(
    stmts: &[Stmt],
    tables: &mut HashMap<String, (String, Vec<String>)>,
    short: &mut HashMap<String, String>,
) -> Result<()> {
    for stmt in stmts {
        match stmt {
            Stmt::Import { path, macros, .. } => {
                let stem = Path::new(path)
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_else(|| path.clone());
                let (syms, _src, _lir, _tt) = crate::package::load_package(path)
                    .map_err(|e| Error::Compile(format!("macro import '{}': {}", path, e)))?;
                let names: Vec<String> = syms.iter().filter_map(|s| match s {
                    crate::package::ImportedSymbol::Macro { name } => Some(name.clone()),
                    _ => None,
                }).collect();
                for m in macros {
                    short.insert(m.as_str(), stem.clone());
                }
                if !names.is_empty() || !macros.is_empty() {
                    tables.insert(stem, (path.clone(), names));
                }
            }
            Stmt::Namespace { items, .. } => collect_imports(items, tables, short)?,
            _ => {}
        }
    }
    Ok(())
}

fn expand_stmts(stmts: &[Stmt], ctx: &MacroCtx, depth: usize) -> Result<Vec<Stmt>> {
    if depth > MAX_DEPTH {
        return Err(Error::Compile(format!(
            "macro expansion exceeded depth {} (recursive macro?) at {}",
            MAX_DEPTH, ctx.src_path.display()
        )));
    }
    let mut out = Vec::new();
    for stmt in stmts {
        match stmt {
            Stmt::Namespace { vis, name, items, span } => {
                out.push(Stmt::Namespace {
                    vis: *vis, name: *name,
                    items: expand_stmts(items, ctx, depth + 1)?, span: *span,
                });
            }
            Stmt::ImplBlock { attrs, type_name, generic_params, methods, span } => {
                out.push(Stmt::ImplBlock {
                    attrs: attrs.clone(), type_name: *type_name,
                    generic_params: generic_params.clone(),
                    methods: expand_stmts(methods, ctx, depth + 1)?, span: *span,
                });
            }
            Stmt::Attributed { attrs, stmt: inner, span } => {
                if attrs.iter().any(|a| !is_compiler_attr(a)) {
                    return Err(Error::Compile(format!(
                        "statement-level macros are not supported yet (at {}:{})",
                        span.start_line, span.start_col
                    )));
                }
                out.push(Stmt::Attributed {
                    attrs: attrs.clone(),
                    stmt: Box::new(expand_one(inner, ctx, depth)?.into_iter().next().unwrap_or_else(|| (**inner).clone())),
                    span: *span,
                });
            }
            other => out.extend(expand_one(other, ctx, depth)?),
        }
    }
    Ok(out)
}

/// 展开单个 item：无宏标注则原样返回；有则调用插件并递归展开输出。
fn expand_one(stmt: &Stmt, ctx: &MacroCtx, depth: usize) -> Result<Vec<Stmt>> {
    let macro_attr = attrs_of(stmt).and_then(|attrs| {
        attrs.iter().find(|a| !is_compiler_attr(a)).cloned()
    });
    let Some(attr) = macro_attr else { return Ok(vec![stmt.clone()]); };
    if !attr.args.is_empty() {
        return Err(Error::Compile(format!(
            "macro arguments are not supported yet: #[{}] (at {}:{})",
            attr.path_str(), attr.span.start_line, attr.span.start_col
        )));
    }
    let Some((pkg, macro_name)) = ctx.resolve(&attr) else {
        return Err(Error::Compile(format!(
            "macro #[{}] cannot be resolved (at {}:{})",
            attr.path_str(), attr.span.start_line, attr.span.start_col
        )));
    };
    let Some((lcl_path, names)) = ctx.tables.get(&pkg) else {
        return Err(Error::Compile(format!("package `{}` has no macro table", pkg)));
    };
    if !names.iter().any(|n| n == &macro_name) {
        return Err(Error::Compile(format!(
            "macro `{}` not found in package `{}` (at {}:{})",
            macro_name, pkg, attr.span.start_line, attr.span.start_col
        )));
    }

    // 去掉被调用的宏标注后格式化 item 源码
    let mut input_stmt = stmt.clone();
    if let Some(attrs) = attrs_of_mut(&mut input_stmt) {
        attrs.retain(|a| a.path_str() != attr.path_str());
    }
    let input = crate::formatter::format_program(&Program::new(vec![input_stmt]));

    let output = plugin::invoke_plugin(lcl_path, &macro_name, &input)?;

    let parsed = parse_source(&output)?;
    expand_stmts(&parsed.stmts, ctx, depth + 1)
}

fn parse_source(code: &str) -> Result<Program> {
    let mut lexer = crate::lexer::Lexer::new(code);
    let tokens = lexer.tokenize_all();
    let filtered: Vec<_> = tokens.into_iter()
        .filter(|t| !matches!(t.kind, crate::lexer::TokenKind::EOF))
        .collect();
    let mut parser = crate::parser::Parser::new(filtered);
    parser.parse_program().map_err(|e| Error::Compile(format!("macro output parse error: {}", e)))
}

fn is_compiler_attr(a: &Attr) -> bool {
    let name = a.name.as_str();
    if crate::hir::effects::is_effect(&name) {
        return true;
    }
    if a.is_builtin() {
        return crate::hir::attrs::ALLOWED.contains(&name.as_str());
    }
    false
}

fn attrs_of(stmt: &Stmt) -> Option<&Vec<Attr>> {
    match stmt {
        Stmt::FnDecl { attrs, .. }
        | Stmt::StructDef { attrs, .. }
        | Stmt::EnumDef { attrs, .. }
        | Stmt::InterfaceDef { attrs, .. }
        | Stmt::ImplBlock { attrs, .. } => Some(attrs),
        _ => None,
    }
}

fn attrs_of_mut(stmt: &mut Stmt) -> Option<&mut Vec<Attr>> {
    match stmt {
        Stmt::FnDecl { attrs, .. }
        | Stmt::StructDef { attrs, .. }
        | Stmt::EnumDef { attrs, .. }
        | Stmt::InterfaceDef { attrs, .. }
        | Stmt::ImplBlock { attrs, .. } => Some(attrs),
        _ => None,
    }
}
