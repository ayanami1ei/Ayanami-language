// ============================================================
//  辅助函数集 —— 供 HIR 降级过程中使用的工具函数
//  包括：表达式类型推断、所有权转换、泛型参数推断、
//  类型替换、类型格式化显示、运算符名映射等
// ============================================================


use std::collections::HashMap;
use crate::intern::Symbol;
use crate::parser::ast::*;
use crate::span::Span;
use crate::hir::*;
use super::strip_generic_name;
use super::InterfaceReg;

    /// 自动插入 Move 包装：如果表达式是 unique 类型且尚未包装，则包装为 Move

mod caller;
mod closure;
mod closure_checks;
mod closure_lower;
mod closure_static;
mod coerce;
mod overflow;
mod convert;
mod fn_type;
mod substitute;
mod types;
mod wrap;

pub(crate) use caller::*;
pub(crate) use closure::*;
pub(crate) use closure_lower::*;
pub(crate) use overflow::*;
pub(crate) use coerce::*;
pub(crate) use convert::*;
pub(crate) use fn_type::*;
pub(crate) use substitute::*;
pub(crate) use types::*;
pub(crate) use wrap::*;
