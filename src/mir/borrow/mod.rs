//! NLL 风格借用检查。
//!
//! 模型：
//! - 借用（loan）由 `ref` / `ref mut` 创建；
//! - 引用局部变量（`r = ref x`）的借用在 `r` 最后一次使用后失效（数据流 liveness）；
//! - 临时借用按表达式求值顺序存活，调用结束时释放；
//! - 冲突：同一变量同时存在可变借用与任意借用；借用期间写入/移动/读取（可变时）。
//!
//! 事实模型与 Polonius 兼容（loan / liveness / kill / invalidation），
//! 后续如需可替换为 polonius-engine 求解。
//!
//! 限制：引用不可返回、不可存入字段/数组（无生命周期注解）。

mod cfg;
mod collect;
mod follow;
mod liveness;
mod loans;
mod walk;

use crate::error::{Error, Result};
use std::collections::HashMap;

use crate::hir::ir::{HirType, VarId};
use crate::mir::ir::*;

/// A4c：跨函数 `follow_with` 表（FnId → 来源解析）。
pub type FollowTable = HashMap<crate::hir::ty::FnId, follow::FollowInfo>;

/// 从 MIR 程序构建来源表。
pub fn build_follow_table(mir: &MirProgram) -> FollowTable {
    follow::build_table(mir)
}

pub fn check_borrows(mir_fn: &MirFn, table: &FollowTable) -> Result<()> {
    // 引用参数（生命周期省略需要恰好一个）
    let ref_params: Vec<(VarId, bool)> = mir_fn
        .params
        .iter()
        .enumerate()
        .filter_map(|(i, (_, ty))| match ty {
            HirType::Ref(_, m) => Some((VarId(i), *m)),
            _ => None,
        })
        .collect();
    if matches!(mir_fn.return_type, HirType::Ref(..)) && ref_params.len() != 1 {
        if mir_fn.follow_sources.is_empty() {
            let at = if mir_fn.span.start_line > 0 {
                format!(" (at {}:{})", mir_fn.span.start_line, mir_fn.span.start_col)
            } else {
                String::new()
            };
            return Err(Error::Borrow(format!(
                "function `{}` returns a reference but has {} reference parameters; add #[follow_with(param, ...)] to declare the source(s){}",
                mir_fn.name.as_str(),
                ref_params.len(),
                at
            )));
        }
        // A4b：多来源由 #[follow_with] 声明（存在性/歧义在 loans 中校验）
    }
    loans::check_fn(mir_fn, &ref_params, table)
}
