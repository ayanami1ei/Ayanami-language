//! 借用临时量提升：`f(ref "x")` 这类把拥有临时量（字符串字面量 / 调用结果）
//! 按引用传参的表达式，临时量的缓冲在调用后无人释放 → 提升到临时局部变量，
//! 语句结束后 drop（修复每次调用泄漏一个分配）。
use super::*;
use super::mem::needs_drop;

impl Ctx {
    /// 把 expr 中借用的拥有临时量提升为临时局部变量，返回（前置语句, 后置 drop）。
    pub(super) fn hoist_ref_temps(
        &mut self,
        expr: &mut MirNodeBox,
        span: crate::span::Span,
    ) -> (Vec<MirStmtBox>, Vec<MirStmtBox>) {
        let mut pre = Vec::new();
        let mut post = Vec::new();
        self.hoist_walk(expr, span, &mut pre, &mut post);
        (pre, post)
    }

    fn hoist_walk(
        &mut self,
        node: &mut MirNodeBox,
        span: crate::span::Span,
        pre: &mut Vec<MirStmtBox>,
        post: &mut Vec<MirStmtBox>,
    ) {
        node.for_each_child_mut(&mut |c| self.hoist_walk(c, span, pre, post));
        // 仅处理「借用的拥有临时量」：左值（局部/字段/下标/解引用/全局）由所有者释放，
        // 移动/克隆节点已转移所有权。
        let ty = {
            let Some(inner) = node.ref_expr() else { return; };
            let is_place = inner.as_local().is_some()
                || inner.as_global().is_some()
                || inner.as_deref().is_some()
                || inner.as_index().is_some()
                || inner.as_field_access().is_some();
            if is_place || inner.move_expr().is_some() {
                return;
            }
            let ty = inner.expr_type();
            if !needs_drop(&ty, &self.struct_defs) {
                return;
            }
            ty
        };
        let tmp = self.new_temp(ty.clone());
        let inner = node.ref_expr_mut().expect("ref_expr_mut");
        let old = std::mem::replace(inner, SMirLocal { var: tmp, ty: ty.clone(), moved: false }.into());
        pre.push(SMirAssignStmt {
            target: SMirLocal { var: tmp, ty: ty.clone(), moved: false }.into(),
            value: old,
            span,
        }.into());
        post.push(SMirDropStmt { var: tmp, ty, span }.into());
    }
}
