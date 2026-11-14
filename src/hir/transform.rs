//! A5e/B1：在 HIR 降级后、MIR 前执行第三方 `#[transform]` 插件。
//!
//! v1 语义：标注在函数上的 `#[pkg::name]`（name 由 `.lcl` 的 `transform=` 导出）
//! 触发一次程序级变换；插件接收 HIR JSON，返回新的 HIR JSON。当前只完成 identity
//! 通路，结构化 mutation API 见 B2。

use crate::annotation_table::{AnnKind, AnnotationTables};
use crate::error::{Error, Result};
use crate::hir::{HirFn, HirItem, HirProgram};
use crate::parser::ast::Stmt;

pub(crate) fn apply(program: &mut HirProgram, stmts: &[Stmt]) -> Result<()> {
    let tables = AnnotationTables::collect(stmts)?;
    if tables.is_empty() {
        return Ok(());
    }
    let mut markers = Vec::new();
    collect_markers(program, &tables, &mut markers)?;
    if std::env::var_os("AYANAMI_TRANSFORM_TRACE").is_some() {
        eprintln!("transform trace: {} marker(s): {:?}", markers.len(), markers);
    }
    if markers.is_empty() {
        return Ok(());
    }

    for (lcl_path, name) in markers {
        let input = crate::hir::schema::to_json(program)?;
        let output = crate::compiler::macro_expand::invoke_transform(&lcl_path, &name, &input)?;
        if std::env::var_os("AYANAMI_TRANSFORM_TRACE").is_some() {
            eprintln!("transform trace: plugin {} returned {} bytes", name, output.len());
        }
        *program = crate::hir::schema::from_json(&output).map_err(|e| {
            Error::Hir(format!("transform `{}` returned invalid HIR schema: {}", name, e))
        })?;
    }
    strip_transform_attrs(program, &tables)?;
    Ok(())
}

fn collect_markers(
    program: &HirProgram,
    tables: &AnnotationTables,
    out: &mut Vec<(String, String)>,
) -> Result<()> {
    for item in &program.items {
        collect_item_markers(item, tables, out)?;
    }
    Ok(())
}

fn collect_item_markers(
    item: &HirItem,
    tables: &AnnotationTables,
    out: &mut Vec<(String, String)>,
) -> Result<()> {
    match item {
        HirItem::Fn(f) => collect_fn_markers(f, tables, out),
        HirItem::Namespace { items, .. } => {
            for child in items {
                collect_item_markers(child, tables, out)?;
            }
            Ok(())
        }
        HirItem::StructDef(_) | HirItem::InterfaceDef { .. } => Ok(()),
    }
}

fn collect_fn_markers(
    f: &HirFn,
    tables: &AnnotationTables,
    out: &mut Vec<(String, String)>,
) -> Result<()> {
    for attr in &f.attrs {
        let Some((pkg, name, AnnKind::Transform, _)) = tables.resolve(attr)? else {
            continue;
        };
        let lcl_path = tables.lcl_path(&pkg).ok_or_else(|| {
            Error::Hir(format!("transform package `{}` has no .lcl path", pkg))
        })?;
        out.push((lcl_path, name));
    }
    Ok(())
}

fn strip_transform_attrs(program: &mut HirProgram, tables: &AnnotationTables) -> Result<()> {
    for item in &mut program.items {
        strip_item_attrs(item, tables)?;
    }
    Ok(())
}

fn strip_item_attrs(item: &mut HirItem, tables: &AnnotationTables) -> Result<()> {
    match item {
        HirItem::Fn(f) => strip_fn_attrs(f, tables),
        HirItem::Namespace { items, .. } => {
            for child in items {
                strip_item_attrs(child, tables)?;
            }
            Ok(())
        }
        HirItem::StructDef(_) | HirItem::InterfaceDef { .. } => Ok(()),
    }
}

fn strip_fn_attrs(f: &mut HirFn, tables: &AnnotationTables) -> Result<()> {
    let mut keep = Vec::with_capacity(f.attrs.len());
    for attr in f.attrs.drain(..) {
        match tables.resolve(&attr)? {
            Some((_, _, AnnKind::Transform, _)) => {}
            _ => keep.push(attr),
        }
    }
    f.attrs = keep;
    Ok(())
}
