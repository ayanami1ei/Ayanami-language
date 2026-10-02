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
mod liveness;
mod loans;

use crate::error::{Error, Result};
use crate::hir::ir::{HirType, VarId};
use crate::mir::ir::*;

pub fn check_borrows(mir_fn: &MirFn) -> Result<()> {
    if matches!(mir_fn.return_type, HirType::Ref(..)) {
        return Err(Error::Borrow(format!(
            "function `{}` cannot return a reference: borrows cannot escape",
            mir_fn.name.as_str()
        )));
    }
    loans::check_fn(mir_fn)
}
