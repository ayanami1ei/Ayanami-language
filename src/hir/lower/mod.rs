pub mod body;
pub mod helpers;
pub mod to_mir;
pub(crate) use helpers::*;

use crate::error::{Error, Result};
use std::collections::HashMap;
use crate::intern::Symbol;
use crate::parser::ast::*;
use crate::span::Span;
use crate::hir::*;

/// 从类型名中剥离泛型参数
/// 例如 "LinkedListNode<T>" → "LinkedListNode"，也处理 "LinkedListNode[T]"
pub(super) fn strip_generic_name(name: &Symbol) -> Symbol {
    let s = name.as_str();
    let pos = s.find('<').or_else(|| s.find('['));
    if let Some(p) = pos {
        Symbol::intern(&s[..p])
    } else {
        *name
    }
}

// ============================================================
//  类型定义：HIR 降级过程中使用的内部数据结构
// ============================================================

/// 函数签名 —— 用于函数重载解析和虚函数表构建
#[derive(Clone)]
pub(crate) struct FnSig {
    /// A3c：效应声明（包导入时来自 .lcl 摘要）
    pub(crate) effects: crate::hir::effects::EffectDecl,
    /// A3c：包导出的推断事实（导入函数）
    pub(crate) inferred: crate::hir::effects::EffectSet,
    /// 函数名称
    pub name: Symbol,
    /// 参数列表：(参数名, 参数类型)
    pub params: Vec<(Symbol, HirType)>,
    /// 返回值类型
    pub return_type: HirType,
}

/// 接口注册信息 —— 记录接口的泛型参数和方法签名
#[derive(Clone)]
pub(crate) struct InterfaceReg {
    /// 泛型参数列表：(参数名, 约束接口名)
    pub generic_params: Vec<(Symbol, Option<Symbol>)>,
    /// 接口中定义的方法列表
    pub methods: Vec<HirInterfaceMethod>,
}

mod ctx;
mod ctx_mono;
pub(crate) use ctx::Ctx;

// ============================================================
//  入口函数 —— lower_program
//  执行完整的 AST → HIR 降级流程
// ============================================================

/// 将抽象语法树（AST）降级为高级中间表示（HIR）
///
/// 执行三个阶段：
/// 1. 收集阶段（collect_fns）：注册所有函数和接口签名
/// 2. 构建阶段（build_vtables）：验证实现签名并构建虚函数表
/// 3. 降级阶段（lower_items）：递归处理所有语句/表达式，生成 HIR 节点
///
/// 同时收集过程中产生的特化泛型函数，以及从其他模块导入的函数签名。
pub fn lower_program(program: &Program) -> Result<HirProgram> {
    // A0：先做属性白名单校验（未知属性报错）
    crate::hir::attrs::validate_program(program)?;
    // A2b：#[cfg(...)] 编译期裁剪
    let program = crate::hir::cfg::filter_program(program)?;
    let mut ctx = Ctx::new();
    ctx.collect_fns(&program.stmts)?;
    // A5b：导入加载完成后校验宏引用
    crate::hir::attrs::validate_macros(&program, &ctx.imported_macros, &ctx.imported_passes)?;
    ctx.build_vtables()?;
    let mut items = ctx.lower_items(&program.stmts)?;

    // 收集已定义函数的 ID，找出哪些是外部导入的
    let defined_ids: std::collections::HashSet<_> = items.iter().filter_map(|item| {
        if let HirItem::Fn(f) = item { Some(f.fn_id) } else { None }
    }).collect();
    let imported_fns: Vec<ImportedFnSig> = ctx.fns.iter().enumerate()
        .filter(|(i, _)| !defined_ids.contains(&FnId(*i)))
        .map(|(i, sig)| ImportedFnSig {
            fn_id: FnId(i), name: sig.name,
            params: sig.params.clone(), return_type: sig.return_type.clone(),
            attrs: Vec::new(),
            effects: sig.effects.clone(),
            inferred: sig.inferred.clone(),
        })
        .collect();

    // 追加降级过程中特化的泛型函数
    for f in ctx.specialized_fns.drain(..) { items.push(HirItem::Fn(f)); }
    // 追加 lambda 表达式产生的匿名函数
    for f in ctx.lambda_fns.drain(..) { items.push(HirItem::Fn(f)); }

    Ok(HirProgram { items, vtables: ctx.vtables.clone(), struct_defs: ctx.struct_defs.clone(), generic_struct_params: ctx.generic_struct_params.clone(), imported_fns })
}
