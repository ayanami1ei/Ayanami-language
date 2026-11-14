//! A5e/B1：在 HIR 降级后、MIR 前执行第三方 `#[transform]` 插件。
//!
//! v1 语义：标注在函数/struct 上的 `#[pkg::name]`（name 由 `.lcl` 的 `transform=` 导出）
//! 触发程序级变换；泛型函数在其特化 HirFn 上触发。插件接收 HIR JSON，返回新的 HIR JSON。当前只完成 identity
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
    validate_targets(stmts, &tables)?;
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

fn validate_targets(stmts: &[Stmt], tables: &AnnotationTables) -> Result<()> {
    for stmt in stmts {
        match stmt {
            Stmt::FnDecl { attrs, generic_params, .. } => {
                // 泛型函数没有独立 HirFn；其特化实例会携带 attrs 并在这里触发。
                let _ = (attrs, generic_params, tables);
            }
            Stmt::ImplBlock { attrs, methods, .. } => {
                if let Some(attr) = transform_attr(attrs, tables)? {
                    return Err(Error::Hir(format!(
                        "transform #[{}] must be placed on a function/method, not an impl block (at {}:{})",
                        attr.path_str(), attr.span.start_line, attr.span.start_col
                    )));
                }
                for method in methods {
                    if let Stmt::FnDecl { attrs, .. } = method {
                        // 泛型方法同样只在特化 HirFn 上触发。
                        let _ = transform_attr(attrs, tables)?;
                    }
                }
            }
            Stmt::StructDef { attrs, generic_params, .. } => {
                if let Some(attr) = transform_attr(attrs, tables)? {
                    if !generic_params.is_empty() {
                        return Err(Error::Hir(format!(
                            "transform #[{}] is not supported on generic structs in v1 (at {}:{})",
                            attr.path_str(), attr.span.start_line, attr.span.start_col
                        )));
                    }
                }
            }
            Stmt::EnumDef { attrs, .. }
            | Stmt::InterfaceDef { attrs, .. } => {
                if let Some(attr) = transform_attr(attrs, tables)? {
                    return Err(Error::Hir(format!(
                        "transform #[{}] is only supported on functions and non-generic structs in v1 (at {}:{})",
                        attr.path_str(), attr.span.start_line, attr.span.start_col
                    )));
                }
            }
            Stmt::Namespace { items, .. } => validate_targets(items, tables)?,
            Stmt::Attributed { attrs, .. } => {
                if let Some(attr) = transform_attr(attrs, tables)? {
                    return Err(Error::Hir(format!(
                        "transform #[{}] is only supported on function items in v1 (at {}:{})",
                        attr.path_str(), attr.span.start_line, attr.span.start_col
                    )));
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn transform_attr<'a>(
    attrs: &'a [crate::parser::ast::Attr],
    tables: &AnnotationTables,
) -> Result<Option<&'a crate::parser::ast::Attr>> {
    for attr in attrs {
        if matches!(tables.resolve_kind(attr)?, Some(AnnKind::Transform)) {
            return Ok(Some(attr));
        }
    }
    Ok(None)
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
        HirItem::StructDef(d) => collect_attrs_markers(&d.attrs, tables, out),
        HirItem::InterfaceDef { .. } => Ok(()),
    }
}

fn collect_attrs_markers(
    attrs: &[crate::parser::ast::Attr],
    tables: &AnnotationTables,
    out: &mut Vec<(String, String)>,
) -> Result<()> {
    for attr in attrs {
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
        HirItem::StructDef(d) => strip_attrs(&mut d.attrs, tables),
        HirItem::InterfaceDef { .. } => Ok(()),
    }
}

fn strip_attrs(attrs: &mut Vec<crate::parser::ast::Attr>, tables: &AnnotationTables) -> Result<()> {
    let mut keep = Vec::with_capacity(attrs.len());
    for attr in attrs.drain(..) {
        match tables.resolve(&attr)? {
            Some((_, _, AnnKind::Transform, _)) => {}
            _ => keep.push(attr),
        }
    }
    *attrs = keep;
    Ok(())
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
