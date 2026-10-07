use std::collections::{HashMap, HashSet};
use crate::span::Span;
use std::fmt::Write as FmtWrite;

pub use crate::hir::ir::{HirLiteral, HirType};
pub use crate::parser::ast::{BinaryOp, UnaryOp};
use crate::hir::ir::{FnId, VarId};
use crate::intern::Symbol;
use crate::lir::ir::{LirValue, LirLowerCtx};

pub trait MirNode: std::fmt::Debug {
    fn clone_node(&self) -> Box<dyn MirNode>;
    fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue;
    fn display(&self, level: usize, w: &mut dyn FmtWrite) -> std::fmt::Result;
    fn expr_type(&self) -> HirType;
    fn as_local(&self) -> Option<VarId> { None }
    /// M6.2：全局变量地址（SMirGlobal）
    fn as_global(&self) -> Option<Symbol> { None }
    /// 解引用内部表达式（SMirDeref），供 place_ptr 穿透到全局
    fn as_deref(&self) -> Option<&MirNodeBox> { None }
    /// M6.2：该引用指向全局变量（SMirRef{expr: SMirGlobal}）
    fn refs_global(&self) -> Option<Symbol> { None }
    /// 索引表达式（SMirIndex）→ (对象, 下标)，供 place_ptr 取元素地址（#130）
    fn as_index(&self) -> Option<(&MirNodeBox, &MirNodeBox)> { None }
    /// 递归收集表达式引用的局部变量（默认经 for_each_child 下降）
    fn collect_var_ids(&self, vars: &mut HashSet<VarId>) {
        self.for_each_child(&mut |c| c.collect_var_ids(vars));
    }
    fn record_moves(&self, moved: &mut HashSet<VarId>) {
        self.for_each_child(&mut |c| c.record_moves(moved));
    }
    fn for_each_child(&self, _f: &mut dyn FnMut(&dyn MirNode)) {}
    /// A5d-3：可变子节点遍历（pass body 改写用）
    fn for_each_child_mut(&mut self, _f: &mut dyn FnMut(&mut MirNodeBox)) {}
    /// 借用节点（SMirRef）的内层表达式（借用临时量提升用）
    fn ref_expr(&self) -> Option<&MirNodeBox> { None }
    fn ref_expr_mut(&mut self) -> Option<&mut MirNodeBox> { None }
    fn as_string_literal(&self) -> Option<&str> { None }
    fn as_ref(&self) -> Option<(VarId, bool)> { None }
    /// #91：字段访问节点（SMirFieldAccess）的（对象, 字段下标）
    fn as_field_access(&self) -> Option<(&MirNodeBox, usize)> { None }
    /// 是否是函数/方法/函数指针调用（借用检查的求值上下文边界）
    fn is_call(&self) -> bool { false }
    /// A4b-2：结构体字面量的字段（仅 SMirStructLiteral）
    fn struct_literal_fields(&self) -> Option<&[(Symbol, MirNodeBox)]> { None }
    /// A4b-2：解包移动（仅 SMirMove），用于查看被移动的字面量
    fn move_expr(&self) -> Option<&MirNodeBox> { None }
    /// A4c：静态调用目标（仅 SMirCall）
    fn call_fn_id(&self) -> Option<crate::hir::ty::FnId> { None }
    /// A5d：是否为堆分配节点（字符串字面量/数组/装箱/克隆）
    fn is_alloc(&self) -> bool { false }
    /// A5d：是否为内联汇编
    fn is_asm(&self) -> bool { false }
    /// A5d-3：二元/一元运算符与字面量（pass 常量折叠用）
    fn binary_op(&self) -> Option<BinaryOp> { None }
    fn unary_op(&self) -> Option<UnaryOp> { None }
    fn literal_value(&self) -> Option<(&HirLiteral, &HirType)> { None }
}

#[derive(Debug)]
pub struct MirNodeBox(pub Box<dyn MirNode>);
impl Clone for MirNodeBox {
    fn clone(&self) -> Self { MirNodeBox(self.0.clone_node()) }
}
impl std::ops::Deref for MirNodeBox {
    type Target = dyn MirNode;
    fn deref(&self) -> &Self::Target { &*self.0 }
}
impl std::ops::DerefMut for MirNodeBox {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut *self.0 }
}

pub trait MirStmtNode: std::fmt::Debug {
    fn clone_stmt(&self) -> Box<dyn MirStmtNode>;
    fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx);
    fn display_stmt(&self, level: usize, w: &mut dyn FmtWrite) -> std::fmt::Result;
    fn for_each_child_expr(&self, _f: &mut dyn FnMut(&dyn MirNode)) {}
    fn for_each_child_stmt(&self, _f: &mut dyn FnMut(&dyn MirStmtNode)) {}
    /// A5d-3：可变子表达式/子语句遍历（pass body 改写用）
    fn for_each_child_expr_mut(&mut self, _f: &mut dyn FnMut(&mut MirNodeBox)) {}
    fn for_each_child_stmt_mut(&mut self, _f: &mut dyn FnMut(&mut MirStmtBox)) {}
    /// A6：语句源码位置（合成语句为默认 Span，line/col 为 0）
    fn span(&self) -> Span { Span::default() }
    fn is_return(&self) -> bool { false }
    fn return_value(&self) -> Option<&MirNodeBox> { None }
    fn as_drop(&self) -> Option<(VarId, &HirType)> { None }
    // ── 借用检查用访问器 ──
    fn as_if(&self) -> Option<IfParts<'_>> { None }
    fn as_while(&self) -> Option<WhileParts<'_>> { None }
    fn as_block(&self) -> Option<&[MirStmtBox]> { None }
    fn is_break(&self) -> bool { false }
    fn is_continue(&self) -> bool { false }
    fn assign_parts(&self) -> Option<(&MirNodeBox, &MirNodeBox)> { None }
    fn field_assign_parts(&self) -> Option<(&MirNodeBox, &MirNodeBox)> { None }
    fn index_assign_parts(&self) -> Option<(&MirNodeBox, &MirNodeBox, &MirNodeBox)> { None }
    /// 穿透引用写入：`*target = value`
    fn deref_assign_parts(&self) -> Option<(&MirNodeBox, &MirNodeBox)> { None }
    fn expr_part(&self) -> Option<&MirNodeBox> { None }
}

/// (cond, then, elifs[(cond, pre_cond, post_cond, block)], else)
pub type IfParts<'a> = (&'a MirNodeBox, &'a [MirStmtBox], &'a [(MirNodeBox, Vec<MirStmtBox>, Vec<MirStmtBox>, Vec<MirStmtBox>)], &'a Option<Vec<MirStmtBox>>);
pub type WhileParts<'a> = (&'a MirNodeBox, &'a [MirStmtBox]);

#[derive(Debug)]
pub struct MirStmtBox(pub Box<dyn MirStmtNode>);
impl Clone for MirStmtBox {
    fn clone(&self) -> Self { MirStmtBox(self.0.clone_stmt()) }
}
impl std::ops::Deref for MirStmtBox {
    type Target = dyn MirStmtNode;
    fn deref(&self) -> &Self::Target { &*self.0 }
}
impl std::ops::DerefMut for MirStmtBox {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut *self.0 }
}

macro_rules! s_mir {
    ($name:ident { $($field:ident: $ty:ty),* $(,)? }) => {
        #[derive(Debug, Clone)]
        pub struct $name { $(pub $field: $ty),* }
    };
    ($name:ident { $($field:ident: $ty:ty),* }) => { s_mir!($name { $($field: $ty),* , }); };
}

macro_rules! s_mstmt {
    ($name:ident { $($field:ident: $ty:ty),* $(,)? }) => {
        #[derive(Debug, Clone)]
        pub struct $name { $(pub $field: $ty),* }
    };
    ($name:ident { $($field:ident: $ty:ty),* }) => { s_mstmt!($name { $($field: $ty),* , }, span: Span); };
}

// 23 SMir* expression structs
s_mir!(SMirLocal { var: VarId, ty: HirType, moved: bool });
s_mir!(SMirLiteral { val: HirLiteral, ty: HirType });
s_mir!(SMirBinary { op: BinaryOp, lhs: MirNodeBox, rhs: MirNodeBox, ty: HirType });
s_mir!(SMirUnary { op: UnaryOp, arg: MirNodeBox, ty: HirType });
s_mir!(SMirCall { fn_id: FnId, args: Vec<MirNodeBox>, ty: HirType });
s_mir!(SMirMove { expr: MirNodeBox, ty: HirType });
// #155：比较（i1）→ 数值
s_mir!(SMirBoolToNum { expr: MirNodeBox, ty: HirType });
s_mir!(SMirClone { expr: MirNodeBox, ty: HirType });
s_mir!(SMirToUnique { expr: MirNodeBox, ty: HirType });
s_mir!(SMirCast { expr: MirNodeBox, ty: HirType });
s_mir!(SMirVirtualCall { receiver: MirNodeBox, interface: Symbol, method_index: usize, args: Vec<MirNodeBox>, ty: HirType });
s_mir!(SMirMakeFatPtr { value: MirNodeBox, concrete_type: Symbol, interface_name: Symbol, ty: HirType });
s_mir!(SMirEnumConstruct { enum_name: Symbol, variant_name: Symbol, variant_struct: Symbol, args: Vec<MirNodeBox>, ty: HirType });
s_mir!(SMirFnPtr { fn_id: FnId, ty: HirType });
s_mir!(SMirCallPtr { fn_ptr: MirNodeBox, args: Vec<MirNodeBox>, ty: HirType });
s_mir!(SMirEnumMatch { value: MirNodeBox, arms: Vec<(i64, MirNodeBox)>, ty: HirType });
s_mir!(SMirFieldAccess { object: MirNodeBox, field: Symbol, field_index: usize, ty: HirType });
s_mir!(SMirStructLiteral { type_name: Symbol, fields: Vec<(Symbol, MirNodeBox)>, ty: HirType });
s_mir!(SMirArrayLiteral { elems: Vec<MirNodeBox>, ty: HirType });
s_mir!(SMirArraySized { count: MirNodeBox, elem_ty: HirType, ty: HirType });
s_mir!(SMirRef { expr: MirNodeBox, mutable: bool, ty: HirType });
s_mir!(SMirDeref { expr: MirNodeBox, ty: HirType });
// M6.2：全局变量地址（ty 为 pointee；expr_type = Ref(ty, mutable)）
s_mir!(SMirGlobal { name: Symbol, ty: HirType, mutable: bool });
s_mir!(SMirIndex { object: MirNodeBox, index: MirNodeBox, ty: HirType });
s_mir!(SMirAsm { template: String, outputs: Vec<(String, MirNodeBox)>, inputs: Vec<(String, MirNodeBox)>, ty: HirType });

// 13 SMir*Stmt structs
s_mstmt!(SMirAssignStmt { target: MirNodeBox, value: MirNodeBox, span: Span });
s_mstmt!(SMirFieldAssignStmt { object: MirNodeBox, field: Symbol, field_index: usize, field_ty: HirType, value: MirNodeBox, span: Span });
s_mstmt!(SMirIndexAssignStmt { object: MirNodeBox, index: MirNodeBox, value: MirNodeBox, span: Span });
s_mstmt!(SMirDerefAssignStmt { target: MirNodeBox, value: MirNodeBox, span: Span });
s_mstmt!(SMirReturnStmt { value: Option<MirNodeBox>, span: Span });
s_mstmt!(SMirIfStmt { cond: MirNodeBox, then_block: Vec<MirStmtBox>, elifs: Vec<(MirNodeBox, Vec<MirStmtBox>, Vec<MirStmtBox>, Vec<MirStmtBox>)>, else_block: Option<Vec<MirStmtBox>>, span: Span });
s_mstmt!(SMirWhileStmt { cond: MirNodeBox, pre_cond: Vec<MirStmtBox>, post_cond: Vec<MirStmtBox>, body: Vec<MirStmtBox>, span: Span });
s_mstmt!(SMirBreakStmt { span: Span });
s_mstmt!(SMirContinueStmt { span: Span });
s_mstmt!(SMirExprStmt { expr: MirNodeBox, span: Span });
s_mstmt!(SMirBlockStmt { stmts: Vec<MirStmtBox>, span: Span });
s_mstmt!(SMirDropStmt { var: VarId, ty: HirType, span: Span });
// 动态数组按外部计数释放（字段覆盖旧值等无结构体上下文的场景）
s_mstmt!(SMirDropCounted { var: VarId, elem_ty: HirType, count_var: VarId, span: Span });
s_mstmt!(SMirAssumeStmt { cond: MirNodeBox, span: Span });
s_mstmt!(SMirContractStmt { kind: crate::hir::ContractKind, cond: MirNodeBox, line: u64, col: u64, span: Span });

macro_rules! impl_into_mir_node_box {
    ($($ty:ident),* $(,)?) => {
        $(impl From<$ty> for MirNodeBox {
            fn from(v: $ty) -> Self { MirNodeBox(Box::new(v) as Box<dyn MirNode>) }
        })*
    };
}
impl_into_mir_node_box!(SMirLocal, SMirLiteral, SMirBinary, SMirUnary, SMirCall, SMirMove, SMirClone, SMirToUnique, SMirCast, SMirVirtualCall, SMirMakeFatPtr, SMirEnumConstruct, SMirFnPtr, SMirCallPtr, SMirEnumMatch, SMirFieldAccess, SMirStructLiteral, SMirArrayLiteral, SMirArraySized, SMirRef, SMirDeref, SMirGlobal, SMirIndex, SMirAsm, SMirBoolToNum);

macro_rules! impl_into_mir_stmt_box {
    ($($ty:ident),* $(,)?) => {
        $(impl From<$ty> for MirStmtBox {
            fn from(v: $ty) -> Self { MirStmtBox(Box::new(v) as Box<dyn MirStmtNode>) }
        })*
    };
}
impl_into_mir_stmt_box!(SMirAssignStmt, SMirFieldAssignStmt, SMirIndexAssignStmt, SMirDerefAssignStmt, SMirReturnStmt, SMirIfStmt, SMirWhileStmt, SMirBreakStmt, SMirContinueStmt, SMirExprStmt, SMirBlockStmt, SMirDropStmt, SMirDropCounted, SMirAssumeStmt, SMirContractStmt);

#[derive(Debug, Clone)]
pub struct MirLocal {
    pub name: Symbol,
    pub ty: HirType,
    pub mutable: bool,
}

impl MirLocal {
    pub fn new(name: Symbol, ty: HirType, mutable: bool) -> Self {
        Self { name, ty, mutable }
    }
}

#[derive(Debug, Clone)]
pub struct MirFn {
    pub fn_id: FnId,
    pub name: Symbol,
    /// A6：函数声明位置（借用等诊断用）
    pub span: Span,
    pub is_inline: bool,
    pub extern_c: bool,
    /// 泛型特化（弱链接）
    pub is_specialized: bool,
    /// M-opt.2：pub 导出（release 内部化非导出函数）
    pub is_pub: bool,
    pub params: Vec<(Symbol, HirType)>,
    pub return_type: HirType,
    pub locals: Vec<MirLocal>,
    pub body: Vec<MirStmtBox>,
    /// 声明标注（A1 起映射 LLVM 属性）
    pub attrs: Vec<crate::parser::ast::Attr>,
    /// 形参标注（与 params 等长并行；A1b）
    pub param_attrs: Vec<Vec<crate::parser::ast::Attr>>,
    /// A3b：声明的效应（显式空集 → 自动 LLVM 属性）
    pub effects: crate::hir::effects::EffectDecl,
    /// A3：推断出的实际效应（有效集合 = 声明 ∪ 推断）
    pub inferred: crate::hir::effects::EffectSet,
    /// A5b：用户宏（LIR 符号加保留前缀）
    pub is_macro: bool,
    /// A4b：`#[follow_with]` 来源（参数名/类型名）
    pub follow_sources: Vec<crate::intern::Symbol>,
}

#[derive(Debug, Clone)]
pub enum MirItem {
    Fn(MirFn),
    StructDef {
        name: Symbol,
        fields: Vec<(Symbol, HirType)>,
    },
    Namespace {
        name: Symbol,
        items: Vec<MirItem>,
    },
}

#[derive(Debug, Clone)]
pub struct MirProgram {
    pub items: Vec<MirItem>,
    pub vtables: Vec<crate::hir::ir::VtableEntry>,
    pub struct_defs: HashMap<Symbol, Vec<(Symbol, HirType)>>,
    pub generic_struct_params: HashMap<Symbol, Vec<(Symbol, Option<Symbol>)>>,
    pub imported_fns: Vec<crate::hir::ir::ImportedFnSig>,
    /// M6.2：顶层 static（透传到 LIR globals）
    pub statics: Vec<crate::hir::HirStatic>,
}
