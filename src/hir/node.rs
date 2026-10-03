// ── HirNode trait — all HIR nodes implement this ──

use std::fmt::Write as FmtWrite;
use crate::mir::ir::MirNodeBox;
use std::collections::HashSet;
use crate::hir::ty::{VarId, HirType, HirLiteral};
use crate::intern::Symbol;

pub trait HirNode: std::fmt::Debug {
    fn clone_node(&self) -> Box<dyn HirNode>;
    fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox;
    fn display(&self, level: usize, w: &mut dyn FmtWrite) -> std::fmt::Result;
    fn expr_type(&self) -> HirType;

    // Helper methods for pattern-match-free traversal
    fn as_local(&self) -> Option<VarId> { None }
    fn is_move_or_clone(&self) -> bool { false }
    /// 遍历直接子节点（由各节点实现）。
    fn for_each_child(&self, _f: &mut dyn FnMut(&dyn HirNode)) {}
    /// 递归收集表达式引用的所有局部变量（默认经 `for_each_child` 下降）。
    fn collect_var_ids(&self, vars: &mut HashSet<VarId>) {
        self.for_each_child(&mut |c| c.collect_var_ids(vars));
    }
    /// 递归记录被消费（移动）的变量（默认经 `for_each_child` 下降）
    fn record_moves(&self, moved: &mut HashSet<VarId>) {
        self.for_each_child(&mut |c| c.record_moves(moved));
    }
    fn as_const(&self) -> Option<&HirLiteral> { None }
    fn as_move(&self) -> Option<&HirNodeBox> { None }
    /// A2c：是否为比较运算。HIR 中比较保持操作数类型，Bool 结果由 MIR→LIR 决定。
    fn is_comparison(&self) -> bool { false }
    /// A3a：静态调用目标（仅 SCall；函数指针/虚调用返回 None）。
    fn as_call(&self) -> Option<crate::hir::ty::FnId> { None }
    /// A3：是否为堆分配构造（unique/数组/字符串字面量）。
    fn is_alloc(&self) -> bool { false }
    /// A3：枚举构造的变体名（仅 SEnumC）。
    fn enum_variant(&self) -> Option<Symbol> { None }
    /// 泛型单态化：结构体字面量的可克隆视图（仅 SStruct）。
    fn as_struct_cloned(&self) -> Option<crate::hir::SStruct> { None }
    /// 泛型单态化：以新类型重建常量（仅 SConst）。
    fn with_type(&self, _ty: HirType) -> Option<HirNodeBox> { None }
    fn as_clone(&self) -> Option<&HirNodeBox> { None }
}

/// Clone-safe wrapper for Box<dyn HirNode>
#[derive(Debug)]
pub struct HirNodeBox(pub Box<dyn HirNode>);
impl Clone for HirNodeBox {
    fn clone(&self) -> Self { HirNodeBox(self.0.clone_node()) }
}
impl std::ops::Deref for HirNodeBox {
    type Target = dyn HirNode;
    fn deref(&self) -> &Self::Target { &*self.0 }
}
impl From<HirNodeBox> for Box<dyn HirNode> {
    fn from(b: HirNodeBox) -> Self { b.0 }
}

use crate::hir::{SBin, SUn, SCall, SConst, SVar, SMove, SClone, SToUnique, SCast, SField, SStruct, SArrLit, SArrSz, SAsm, SRef, SIdx, SVCall, SMFP, SEnumC, SEnumM, SFnPtr, SCallP};

// ── From<Struct> for HirNodeBox ──
macro_rules! impl_into_hir_node_box {
    ($($ty:ident),* $(,)?) => {
        $(impl From<$ty> for HirNodeBox {
            fn from(v: $ty) -> Self { HirNodeBox(Box::new(v) as Box<dyn HirNode>) }
        })*
    };
}
impl_into_hir_node_box!(SBin, SUn, SCall, SConst, SVar, SMove, SClone, SToUnique, SCast, SField, SStruct, SArrLit, SArrSz, SAsm, SRef, SIdx, SVCall, SMFP, SEnumC, SEnumM, SFnPtr, SCallP);
