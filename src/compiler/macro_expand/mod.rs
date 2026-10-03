//! A5b-2/A5b-3/A5d-1：用户注解展开（宏，M3 插件 ABI v3，Ayanami 优先）。
//!
//! 流程：
//! 1. 从已重写为 `.lcl` 的 import 收集注解表（`macro=` / `pass=`）；
//! 2. 宏注解：格式化 item 源码 + 渲染标注实参 → 从宏库 LIR 构建插件 `.so`
//!    （llc -relocation-model=pic + C shim + runtime.c）→ dlopen 调用 → 得到新源码；
//!    优化注解（`#[pass]`）不在此展开，留给 MIR 阶段；
//! 3. 重新解析并递归展开（深度上限 32），输出替换原 item。

use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::parser::ast::{Attr, Block, Program, Stmt};

use annotations::{AnnKind, AnnotationTables};

const MAX_DEPTH: usize = 32;

mod annotations;
mod plugin;

/// 展开入口：在 HIR 降级前调用（此时 import 已重写为 .lcl 绝对路径）。
pub fn expand(program: &Program, src_path: &Path) -> Result<Program> {
    let ctx = MacroCtx::collect(program, src_path)?;
    if ctx.tables.is_empty() {
        return Ok(program.clone());
    }
    let stmts = expand_stmts(&program.stmts, &ctx, 0)?;
    Ok(Program::new(stmts))
}

struct MacroCtx {
    tables: AnnotationTables,
    src_path: PathBuf,
}

impl MacroCtx {
    fn collect(program: &Program, src_path: &Path) -> Result<Self> {
        let tables = AnnotationTables::collect(&program.stmts)?;
        Ok(Self { tables, src_path: src_path.to_path_buf() })
    }
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
    match ctx.tables.resolve(attr)? {
        Some((pkg, macro_name, AnnKind::Macro)) => {
            let lcl_path = ctx.tables.lcl_path(&pkg).ok_or_else(|| Error::Compile(format!(
                "package `{}` has no annotation table", pkg
            )))?;
            Ok((lcl_path, macro_name, args))
        }
        Some((pkg, name, AnnKind::Pass)) => Err(Error::Compile(format!(
            "`#[{}]` is an optimization annotation (`{}::{}`), not a macro (at {}:{})",
            attr.path_str(), pkg, name, attr.span.start_line, attr.span.start_col
        ))),
        None => Err(Error::Compile(format!(
            "macro #[{}] cannot be resolved (at {}:{})",
            attr.path_str(), attr.span.start_line, attr.span.start_col
        ))),
    }
}

/// 展开单个 item：无宏标注则原样返回；有则调用插件并递归展开输出。
fn expand_one(stmt: &Stmt, ctx: &MacroCtx, depth: usize) -> Result<Vec<Stmt>> {
    // 只挑「解析为宏」的标注展开；pass 注解与未知标注留给后续阶段/校验
    let mut macro_attr = None;
    if let Some(attrs) = attrs_of(stmt) {
        for a in attrs.iter().filter(|a| !is_compiler_attr(a)) {
            if let Some((_, _, AnnKind::Macro)) = ctx.tables.resolve(a)? {
                macro_attr = Some(a.clone());
                break;
            }
        }
    }
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
