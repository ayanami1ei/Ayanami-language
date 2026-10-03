//! A5b-2/A5b-3：用户宏展开（M3 插件 ABI v2，Ayanami 优先）。
//!
//! 流程：
//! 1. 从已重写为 `.lcl` 的 import 收集宏表（`macro=` 行）与短名映射；
//! 2. 对带库宏标注的 item：格式化其源码 + 渲染标注实参 → 从宏库 LIR 构建插件 `.so`
//!    （llc -relocation-model=pic + C shim + runtime.c）→ dlopen 调用 → 得到新源码；
//! 3. 重新解析并递归展开（深度上限 32），输出替换原 item。

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::parser::ast::{Attr, Block, Program, Stmt};

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
        for s in expand_one(stmt, ctx, depth)? {
            expand_nested(s, &mut out, ctx, depth)?;
        }
    }
    Ok(out)
}

/// 展开 item 后的下降：递归处理函数体/控制流块内的语句级宏（A5c-1）。
fn expand_nested(stmt: Stmt, out: &mut Vec<Stmt>, ctx: &MacroCtx, depth: usize) -> Result<()> {
    match stmt {
        // 语句级宏：去掉目标宏标注后作为输入，输出重新解析为语句序列并递归展开
        Stmt::Attributed { attrs, stmt: inner, span }
            if attrs.iter().any(|a| !is_compiler_attr(a)) =>
        {
            let attr = attrs.iter().find(|a| !is_compiler_attr(a)).cloned().unwrap();
            let (lcl_path, macro_name, args) = resolve_macro_call(ctx, &attr)?;
            let remaining: Vec<Attr> = attrs.iter()
                .filter(|a| a.path_str() != attr.path_str())
                .cloned()
                .collect();
            let input = if remaining.is_empty() {
                crate::formatter::format_program(&Program::new(vec![*inner]))
            } else {
                crate::formatter::format_program(&Program::new(vec![Stmt::Attributed {
                    attrs: remaining,
                    stmt: inner,
                    span,
                }]))
            };
            let output = plugin::invoke_plugin(&lcl_path, &macro_name, &input, &args)?;
            let parsed = parse_source(&output)?;
            for s in &parsed.stmts {
                for e in expand_one(s, ctx, depth + 1)? {
                    expand_nested(e, out, ctx, depth + 1)?;
                }
            }
            Ok(())
        }
        // 仅编译器标注（cfg/invariant）：保留包装，下降内部
        Stmt::Attributed { attrs, stmt: inner, span } => {
            let mut inner_out = Vec::new();
            expand_nested(*inner, &mut inner_out, ctx, depth)?;
            let mut iter = inner_out.into_iter();
            if let Some(first) = iter.next() {
                out.push(Stmt::Attributed { attrs, stmt: Box::new(first), span });
                out.extend(iter);
            }
            Ok(())
        }
        Stmt::FnDecl {
            attrs, vis, is_inline, extern_c, name, generic_params,
            params, param_attrs, return_type, body, span,
        } => {
            let body = Block::new(expand_stmts(&body.stmts, ctx, depth + 1)?, body.span);
            out.push(Stmt::FnDecl {
                attrs, vis, is_inline, extern_c, name, generic_params,
                params, param_attrs, return_type, body, span,
            });
            Ok(())
        }
        Stmt::If { cond, then_block, elifs, else_block, span } => {
            let then_block = Block::new(expand_stmts(&then_block.stmts, ctx, depth + 1)?, then_block.span);
            let elifs = elifs.into_iter().map(|(c, b)| {
                let nb = Block::new(expand_stmts(&b.stmts, ctx, depth + 1)?, b.span);
                Ok((c, nb))
            }).collect::<Result<Vec<_>>>()?;
            let else_block = match else_block {
                Some(b) => Some(Block::new(expand_stmts(&b.stmts, ctx, depth + 1)?, b.span)),
                None => None,
            };
            out.push(Stmt::If { cond, then_block, elifs, else_block, span });
            Ok(())
        }
        Stmt::For { iterator, start, end, step, body, span } => {
            let body = Block::new(expand_stmts(&body.stmts, ctx, depth + 1)?, body.span);
            out.push(Stmt::For { iterator, start, end, step, body, span });
            Ok(())
        }
        Stmt::While { cond, body, span } => {
            let body = Block::new(expand_stmts(&body.stmts, ctx, depth + 1)?, body.span);
            out.push(Stmt::While { cond, body, span });
            Ok(())
        }
        Stmt::Namespace { vis, name, items, span } => {
            let items = expand_stmts(&items, ctx, depth + 1)?;
            out.push(Stmt::Namespace { vis, name, items, span });
            Ok(())
        }
        Stmt::ImplBlock { attrs, type_name, generic_params, methods, span } => {
            let methods = expand_stmts(&methods, ctx, depth + 1)?;
            out.push(Stmt::ImplBlock { attrs, type_name, generic_params, methods, span });
            Ok(())
        }
        other => {
            out.push(other);
            Ok(())
        }
    }
}

/// 解析并校验宏标注 → (lcl 路径, 宏名, 实参源码文本)
fn resolve_macro_call(ctx: &MacroCtx, attr: &Attr) -> Result<(String, String, Vec<String>)> {
    let args: Vec<String> = attr.args.iter().map(crate::formatter::format_attr_arg).collect();
    let Some((pkg, macro_name)) = ctx.resolve(attr) else {
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
    Ok((lcl_path.clone(), macro_name, args))
}

/// 展开单个 item：无宏标注则原样返回；有则调用插件并递归展开输出。
fn expand_one(stmt: &Stmt, ctx: &MacroCtx, depth: usize) -> Result<Vec<Stmt>> {
    let macro_attr = attrs_of(stmt).and_then(|attrs| {
        attrs.iter().find(|a| !is_compiler_attr(a)).cloned()
    });
    let Some(attr) = macro_attr else { return Ok(vec![stmt.clone()]); };
    let (lcl_path, macro_name, args) = resolve_macro_call(ctx, &attr)?;

    // 去掉被调用的宏标注后格式化 item 源码
    let mut input_stmt = stmt.clone();
    if let Some(attrs) = attrs_of_mut(&mut input_stmt) {
        attrs.retain(|a| a.path_str() != attr.path_str());
    }
    let input = crate::formatter::format_program(&Program::new(vec![input_stmt]));

    let output = plugin::invoke_plugin(&lcl_path, &macro_name, &input, &args)?;

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
