pub mod body;
mod collect_copy;
mod ctx_niche;
mod ctx_env;
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
/// 泛型枚举的变体结构体名：`Base<args>_Variant` → `Base_Variant<args>`
pub(super) fn variant_struct_name(enum_name: &Symbol, variant: &Symbol) -> Symbol {
    let s = enum_name.as_str();
    if let Some(pos) = s.find('<') {
        let (base, suffix) = (&s[..pos], &s[pos..]);
        Symbol::intern(&format!("{}_{}{}", base, variant, suffix))
    } else {
        Symbol::intern(&format!("{}_{}", enum_name, variant))
    }
}

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
    /// A6：声明位置（接口签名等诊断用）
    pub span: Span,
    /// 末尾保留参数个数（函数级 track_caller：__line/__col/__file）
    pub hidden: usize,
    /// M1.9：调用不返回（`-> !` 或 `#[noreturn]`）——调用表达式类型为 `!`
    pub is_noreturn: bool,
    /// #166：extern "C" 声明（含包导入）——调用保留 C 符号名、实参借用
    pub extern_c: bool,
    /// M5：unsafe fn（调用点需 unsafe 上下文）
    pub is_unsafe: bool,
}

/// 接口注册信息 —— 记录接口的泛型参数和方法签名
#[derive(Clone)]
pub(crate) struct InterfaceReg {
    /// 泛型参数列表：(参数名, 约束接口名)
    pub generic_params: Vec<(Symbol, Vec<Symbol>)>,
    /// 接口中定义的方法列表
    pub methods: Vec<HirInterfaceMethod>,
}

mod ctx;
mod ctx_mono;
mod ctx_fields;
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
/// A5c-2：宏展开需要的当前源文件路径（`__file`）。由管线在降级前设置。
static SOURCE_PATH: std::sync::OnceLock<String> = std::sync::OnceLock::new();

pub fn set_source_path(path: &std::path::Path) {
    let _ = SOURCE_PATH.set(path.to_string_lossy().into_owned());
}

pub(crate) fn source_path() -> String {
    SOURCE_PATH.get().cloned().unwrap_or_default()
}

/// A1b-3a：async 局部类型表：函数名 → (局部名 → HIR 类型)
pub(crate) type AsyncLocalTypes = HashMap<Symbol, HashMap<Symbol, HirType>>;

pub fn lower_program(program: &Program) -> Result<HirProgram> {
    crate::hir::attrs::validate_program(program)?;
    let program = crate::hir::cfg::filter_program(program)?;
    let types = if has_async_fn(&program) {
        probe_async_local_types(&program).ok()
    } else {
        None
    };
    let program = crate::hir::async_transform::expand(&program, types.as_ref())?;
    let mut hir = lower_program_after(&program, false)?;
    crate::hir::transform::apply(&mut hir, &program.stmts)?;
    if std::env::var_os("AYANAMI_HIR_SCHEMA").is_some() {
        hir = crate::hir::schema::roundtrip(&hir)?;
    }
    Ok(hir)
}

/// 用一次“无 async 展开”的 HIR 降级收集局部类型（只读探针）。
fn probe_async_local_types(program: &Program) -> Result<AsyncLocalTypes> {
    let mut probe = Program::new(program.stmts.clone());
    // 探针只关心 async 域局部类型：排除 main，避免未展开的 Future 构造在 executor 调用处误报
    probe.stmts.retain(|st| !matches!(st, Stmt::FnDecl { name, .. } if name.as_str() == "main"));
    for st in &mut probe.stmts {
        if let Stmt::FnDecl { attrs, .. } = st {
            attrs.retain(|a| a.name.as_str() != "async");
        }
    }
    let hir = lower_program_after(&probe, true)?;
    let mut map = HashMap::new();
    for item in &hir.items {
        if let HirItem::Fn(f) = item {
            let locals = f.locals.iter().map(|l| (l.name, l.ty.clone())).collect();
            map.insert(f.name, locals);
        }
    }
    Ok(map)
}

fn has_async_fn(program: &Program) -> bool {
    program.stmts.iter().any(|st| matches!(st, Stmt::FnDecl { attrs, .. }
        if attrs.iter().any(|a| a.name.as_str() == "async")))
}

fn lower_program_after(program: &Program, async_probe: bool) -> Result<HirProgram> {
    // dld：`#[copy]` 结构体登记（校验 + 注册表；is_copy 供移动/拷贝分析）
    collect_copy::register_copy_structs(&program.stmts)?;
    let mut ctx = Ctx::new();
    ctx.async_probe = async_probe;
    ctx.collect_fns(&program.stmts)?;
    // A5b：导入加载完成后校验宏引用
    crate::hir::attrs::validate_macros(&program, &ctx.imported_macros, &ctx.imported_transforms, &ctx.imported_passes, &ctx.imported_checks, &ctx.imported_attrs)?;
    // #121：泛型体 eager 名称检查（未实例化的方法体不再完全静默）
    ctx.check_generic_bodies()?;
    ctx.build_vtables()?;
    let mut items = ctx.lower_items(&program.stmts)?;

    // M1.7：追加合成的外部运行时助手（溢出检查），使其进入 MIR/LIR 并发射 declare
    for f in ctx.synth_externs.drain(..) { items.push(HirItem::Fn(f)); }

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
            extern_c: sig.extern_c,
        })
        .collect();

    // 追加降级过程中特化的泛型函数
    for f in ctx.specialized_fns.drain(..) { items.push(HirItem::Fn(f)); }
    // 追加 lambda 表达式产生的匿名函数
    for f in ctx.lambda_fns.drain(..) { items.push(HirItem::Fn(f)); }

    let mut statics: Vec<crate::hir::HirStatic> = ctx.statics.values().cloned().collect();
    statics.sort_by(|a, b| a.name.as_str().cmp(&b.name.as_str()));
    // 别名（裸名/限定名指向同一发射符号）只保留一份
    statics.dedup_by(|a, b| a.name == b.name);
    let mut consts = ctx.const_exports.clone();
    consts.sort_by(|a, b| a.0.as_str().cmp(&b.0.as_str()));
    Ok(HirProgram { items, vtables: ctx.vtables.clone(), struct_defs: ctx.struct_defs.clone(), generic_struct_params: ctx.generic_struct_params.clone(), imported_fns, statics, consts })
}
