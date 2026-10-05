use std::collections::HashMap;
use crate::intern::Symbol;
use crate::span::Span;
use crate::hir::*;

#[derive(Debug, Clone)]
pub struct HirStructField {
    pub name: Symbol,
    pub ty: HirType,
}

#[derive(Debug, Clone)]
pub struct HirStructDef {
    pub name: Symbol,
    pub fields: Vec<HirStructField>,
}

#[derive(Debug, Clone)]
pub struct HirLocal {
    pub name: Symbol,
    pub ty: HirType,
    pub mutable: bool,
}

impl HirLocal {
    pub fn new(name: Symbol, ty: HirType, mutable: bool) -> Self {
        Self { name, ty, mutable }
    }
}

/// Vtable entry: maps a (concrete_type, interface) pair to the function IDs
/// implementing each interface method (index 0 = destructor/drop, 1..N = methods).
#[derive(Debug, Clone)]
pub struct VtableEntry {
    pub concrete_type: Symbol,
    pub interface: Symbol,
    pub method_fn_ids: Vec<FnId>,
}

/// A method signature inside an interface definition
#[derive(Debug, Clone)]
pub struct HirInterfaceMethod {
    pub name: Symbol,
    pub self_keyword: Symbol, // "shared" or "unique"
    pub params: Vec<(Symbol, HirType)>,
    pub return_type: HirType,
}

#[derive(Debug, Clone)]
pub struct HirFn {
    pub span: Span,
    /// 声明上的标注（A0 起透传；A1 优化/A3 效应读取）
    pub attrs: Vec<crate::parser::ast::Attr>,
    /// A3a：声明的效应集合（注解权威）
    pub effects: crate::hir::effects::EffectDecl,
    pub fn_id: FnId,
    pub name: Symbol,
    pub is_inline: bool,
    pub extern_c: bool,
    /// 泛型单态化产生的特化函数（弱链接 linkonce_odr，可跨 .lcl 重复）
    pub is_specialized: bool,
    /// A3：是否导出接口（告警只在接口层提示）
    pub is_pub: bool,
    /// A5b：是否为用户宏（LIR 符号加保留前缀，避免与展开产物重名）
    pub is_macro: bool,
    /// A4b：`#[follow_with(...)]` 声明的引用来源（参数名或类型名）
    pub follow_sources: Vec<Symbol>,
    /// A3：本轮编译推断出的实际效应（默认最好情况；由 effects::analyze 填充）
    pub inferred: crate::hir::effects::EffectSet,
    pub params: Vec<(Symbol, HirType)>,
    /// 形参标注（与 params 等长并行；A1b 起映射 LLVM 参数属性）
    pub param_attrs: Vec<Vec<crate::parser::ast::Attr>>,
    pub return_type: HirType,
    pub locals: Vec<HirLocal>,
    pub body: HirBlock,
}

#[derive(Debug, Clone)]
pub enum HirItem {
    Fn(HirFn),
    StructDef(HirStructDef),
    Namespace {
        name: Symbol,
        items: Vec<HirItem>,
    },
    InterfaceDef {
        name: Symbol,
        generic_params: Vec<(Symbol, Option<Symbol>)>,
        methods: Vec<HirInterfaceMethod>,
    },
}

/// Signature of an imported function (from a package).
#[derive(Debug, Clone)]
pub struct ImportedFnSig {
    pub fn_id: FnId,
    pub name: Symbol,
    pub params: Vec<(Symbol, HirType)>,
    pub return_type: HirType,
    /// 包导入暂无标注（源码 extern 声明的标注走 HirFn.attrs）
    pub attrs: Vec<crate::parser::ast::Attr>,
    /// A3c：包导出的效应摘要（声明）
    pub effects: crate::hir::effects::EffectDecl,
    /// A3c：包导出的推断事实
    pub inferred: crate::hir::effects::EffectSet,
}

/// M6.2：全局变量（常量初始化，可寻址）
#[derive(Debug, Clone)]
pub struct HirStatic {
    pub name: Symbol,
    pub ty: HirType,
    pub value: HirLiteral,
    pub is_mut: bool,
}

#[derive(Debug, Clone)]
pub struct HirProgram {
    pub items: Vec<HirItem>,
    pub vtables: Vec<VtableEntry>,
    pub struct_defs: HashMap<Symbol, Vec<HirStructField>>,
    pub generic_struct_params: HashMap<Symbol, Vec<(Symbol, Option<Symbol>)>>,
    pub imported_fns: Vec<ImportedFnSig>,
    /// M6.2：顶层 static 列表（按名排序，发射为 LLVM global）
    pub statics: Vec<HirStatic>,
}
