//! 函数级 track_caller：保留参数 `__line` / `__col` / `__file`
//! 调用点由编译器自动填充（宏的保留参数机制在函数上的对应物）。

use crate::hir::ty::HirType;
use crate::intern::Symbol;

pub(crate) fn is_hidden_param(name: &Symbol) -> bool {
    matches!(name.as_str().as_str(), "__line" | "__col" | "__file")
}

/// 末尾连续的保留参数个数（0..=3）
pub(crate) fn count_hidden_params(params: &[(Symbol, HirType)]) -> usize {
    count_hidden_names(params)
}

/// 同上，但适用于任意类型的形参列表（如 AST Type）
pub(crate) fn count_hidden_names<T>(params: &[(Symbol, T)]) -> usize {
    params.iter().rev().take_while(|(n, _)| is_hidden_param(n)).count()
}

impl crate::hir::lower::Ctx {
    /// 生成函数级 track_caller 的隐藏实参（按顺序取前 `count` 个：line/col/file）
    pub(crate) fn caller_hidden_args(&self, span: &crate::span::Span, count: usize) -> Vec<crate::hir::HirNodeBox> {
        let mut out: Vec<crate::hir::HirNodeBox> = Vec::new();
        if count > 0 {
            out.push(crate::hir::SConst { val: crate::hir::HirLiteral::Int(span.start_line as i64), ty: HirType::Int }.into());
        }
        if count > 1 {
            out.push(crate::hir::SConst { val: crate::hir::HirLiteral::Int(span.start_col as i64), ty: HirType::Int }.into());
        }
        if count > 2 {
            let path = crate::hir::lower::source_path();
            out.push(crate::hir::SConst {
                val: crate::hir::HirLiteral::String(path),
                ty: HirType::Named(Symbol::intern("String")),
            }.into());
        }
        out
    }
}

impl crate::hir::lower::Ctx {
    /// 若目标函数带保留参数（track_caller），追加调用点实参
    pub(crate) fn append_caller_args(
        &self,
        fn_id: crate::hir::ty::FnId,
        args: &mut Vec<crate::hir::HirNodeBox>,
        span: &crate::span::Span,
    ) {
        let hidden = self.fns[fn_id.0].hidden;
        if hidden > 0 {
            args.extend(self.caller_hidden_args(span, hidden));
        }
    }
}
