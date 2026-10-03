//! A5d-3：MIR 优化注解执行器（schema v1：分析 + 编辑面）。
//!
//! - 编译器把 `MirFn` 展平为 preorder 视图（v0 结构数组 + 运算符/字面量 + 编辑数组）；
//! - 插件就地修改：改效应、按节点下标标记「表达式替换为整型字面量」；
//! - 编译器只接受编辑数组的变化，其余结构变化报错；应用编辑后由管线重跑借用检查。

use std::collections::HashMap;
use std::path::Path;

use super::*;
use crate::hir::ir::{FnId, HirItem, HirProgram};
use crate::mir::ir::MirItem;
use crate::parser::ast::{Attr, Program};

mod apply;
mod flat;

use apply::apply_edits;
use flat::{deserialize_view, parse_output, serialize_fn};

/// 函数纯度表：声明 pure，或本轮推断无效应且无未知 extern
fn purity_map(hir: &HirProgram) -> HashMap<FnId, bool> {
    let mut m = HashMap::new();
    for item in &hir.items {
        if let HirItem::Fn(f) = item {
            let pure = f.effects.pure
                || (f.inferred.effects.is_empty() && !f.inferred.may_unknown_effects);
            m.insert(f.fn_id, pure);
        }
    }
    for imp in &hir.imported_fns {
        let pure = imp.effects.pure
            || (imp.inferred.effects.is_empty() && !imp.inferred.may_unknown_effects);
        m.entry(imp.fn_id).or_insert(pure);
    }
    m
}

fn is_builtin_attr(a: &Attr) -> bool {
    let name = a.name.as_str();
    (a.is_builtin() || (a.qualifier.len() == 1 && a.qualifier[0].as_str() == "core"))
        && (crate::hir::attrs::ALLOWED.contains(&name.as_str()) || crate::hir::effects::is_effect(&name))
}

/// 对 MIR 程序应用所有 `#[pass]` 注解
pub(super) fn apply_passes(
    mir: &mut crate::mir::ir::MirProgram,
    hir: &HirProgram,
    program: &Program,
    _src_path: &Path,
) -> Result<()> {
    let tables = crate::compiler::macro_expand::annotation_tables(&program.stmts)?;
    if tables.is_empty() {
        return Ok(());
    }
    let purity = purity_map(hir);
    for item in &mut mir.items {
        let MirItem::Fn(f) = item else { continue };
        let attrs: Vec<Attr> = f.attrs.iter().filter(|a| !is_builtin_attr(a)).cloned().collect();
        for a in attrs {
            let resolved = tables.resolve(&a)?;
            let Some((pkg, name, kind)) = resolved else {
                return Err(Error::Compile(format!(
                    "unknown annotation #[{}] at {}:{}", a.path_str(), a.span.start_line, a.span.start_col
                )));
            };
            if kind != crate::compiler::macro_expand::AnnKind::Pass {
                continue;
            }
            let lcl = tables.lcl_path(&pkg).ok_or_else(|| Error::Compile(format!(
                "package `{}` has no annotation table", pkg
            )))?;
            let blob = serialize_fn(f, &purity);
            let out = crate::compiler::macro_expand::invoke_pass(&lcl, &name, &blob)
                .map_err(|e| Error::Compile(format!(
                    "pass `#[{}]` at {}:{} failed: {}",
                    a.path_str(), a.span.start_line, a.span.start_col, e
                )))?;
            let parsed = parse_output(&blob, &out, &a)?;
            let view = deserialize_view(&blob);
            apply_edits(f, &view, &parsed.edits, &a)?;
            f.effects.pure = parsed.pure;
            f.effects.no_error = parsed.no_error;
        }
    }
    Ok(())
}
