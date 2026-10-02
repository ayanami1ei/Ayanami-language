use super::*;

pub fn sanitize_name(name: &str) -> String {
    name.replace('<', "_lt_").replace('>', "_gt_")
        .replace(',', "_c_").replace('[', "_lb_").replace(']', "_rb_").replace(' ', "_")
}

pub(crate) fn lit_to_string(lit: &HirLiteral, expected_ty: &HirType) -> String {
    match (lit, expected_ty) {
        (HirLiteral::Int(0), ty) if is_pointer_type(ty) && !matches!(ty, HirType::Named(_)) => "null".into(),
        (HirLiteral::Int(0), HirType::Named(_)) => "zeroinitializer".into(),
        (HirLiteral::Int(n), _) => format!("{}", n),
        (HirLiteral::Float(n), _) => {
            let s = format!("{}", n);
            if !s.contains('.') { format!("{}.0", s) } else { s }
        }
        (HirLiteral::Char(c), _) => format!("{}", *c as u8),
        (HirLiteral::Bool(b), _) => if *b { "1".into() } else { "0".into() },
        (HirLiteral::String(_), _) => "null".into(),
    }
}

pub(crate) fn llvm_type_size(ty: &HirType) -> &'static str {
    match ty {
        HirType::Int => "8",
        HirType::Float => "8",
        HirType::Char => "1",
        HirType::Bool => "1",
        HirType::Void => "0",
        HirType::Named(_) | HirType::FatPtr { .. } => "16",
        HirType::Unique(inner) => llvm_type_size(inner),
        HirType::Array(_) | HirType::ArraySized(_, _) => "16",
        HirType::FnPtr(..) => "8",
        HirType::Ref(_, _) => "16",
    }
}

/// Compute the actual size of a Named struct type from its field definitions.
pub(crate) fn struct_llvm_size(ty: &HirType, struct_defs: &std::collections::HashMap<Symbol, Vec<(Symbol, HirType)>>) -> String {
    if let HirType::Named(name) = ty {
        if let Some(fields) = struct_defs.get(name) {
            let total: u64 = fields.iter().map(|(_, ft)| {
                let s = llvm_type_size(ft);
                let n: u64 = s.parse().unwrap_or(8);
                if matches!(ft, HirType::Unique(_)) { 8u64 } else { n }
            }).sum();
            return total.to_string();
        }
    }
    if let HirType::Unique(inner) = ty {
        return struct_llvm_size(inner, struct_defs);
    }
    llvm_type_size(ty).to_string()
}

pub(super) fn needs_heap_ops(ty: &HirType) -> bool {
    match ty {
        HirType::FatPtr { .. } => true,
        HirType::Unique(inner) => {
            matches!(inner.as_ref(), HirType::Named(_) | HirType::FatPtr { .. })
        }
        HirType::Named(_) => false,
        HirType::Array(_) | HirType::ArraySized(_, _) => false,
        HirType::Ref(_, _) => false,
        _ => false,
    }
}

pub(super) fn is_pointer_type(ty: &HirType) -> bool {
    matches!(ty,
        HirType::Named(_) | HirType::FatPtr { .. } | HirType::Array(_)
        | HirType::Unique(_) | HirType::Ref(_, _) | HirType::FnPtr(..)
    )
}

/// 类型离开作用域时是否需要释放（递归判断）。
///
/// 注意：
/// - 普通 `[T]` / `[T; n]` 视为值语义，不在作用域结束时释放（避免双释放）；
///   只有 `unique [T]` 会释放缓冲区。
/// - 枚举（首字段为 `_tag`）暂不递归释放 payload（P1 已知缺口）。
pub(super) fn needs_drop(ty: &HirType, struct_defs: &HashMap<Symbol, Vec<(Symbol, HirType)>>) -> bool {
    match ty {
        HirType::Unique(_) => true,
        HirType::FatPtr { kind, .. } => !matches!(kind.as_ref(), HirType::Ref(..)),
        HirType::Named(name) => {
            let Some(fields) = struct_defs.get(name) else { return false; };
            fields.iter().any(|(_, ft)| needs_drop(ft, struct_defs))
        }
        _ => false,
    }
}

/// 发射对 `slot`（指向 `ty` 值的内存指针）的递归释放代码。
pub(super) fn emit_drop_value(
    ctx: &mut LirEmitCtx,
    slot: &str,
    ty: &HirType,
    lines: &mut Vec<String>,
) {
    match ty {
        HirType::Unique(inner) => {
            if ctx.llvm_type(ty) != "ptr" {
                return; // unique 基本类型当前不是指针表示，暂不释放
            }
            let tmp = ctx.tmp();
            lines.push(format!("%c{} = load ptr, ptr {}, align 8", tmp, slot));
            let ptr = format!("%c{}", tmp);
            match inner.as_ref() {
                // 数组缓冲区由 ptr 直接指向，释放 pointee 即释放缓冲区本身
                HirType::Array(_) | HirType::ArraySized(_, _) => {}
                _ => emit_drop_value(ctx, &ptr, inner, lines),
            }
            lines.push(format!("call void @__ayanami_unique_free(i8* {})", ptr));
        }
        HirType::FatPtr { kind, .. } => {
            match kind.as_ref() {
                // 借用胖指针不拥有数据，无需释放
                HirType::Ref(..) => {}
                HirType::Unique(_) => {
                    let tmp = ctx.tmp();
                    lines.push(format!("%g{} = getelementptr inbounds {{ ptr, ptr }}, ptr {}, i32 0, i32 0", tmp, slot));
                    lines.push(format!("%c{} = load ptr, ptr %g{}, align 8", tmp, tmp));
                    lines.push(format!("call void @__ayanami_unique_free(i8* %c{})", tmp));
                }
                _ => {}
            }
        }
        HirType::Named(name) => {
            let Some(fields) = ctx.prog.struct_defs.get(name).cloned() else { return; };
            // 枚举也按字段递归释放：非活跃变体在构造时零初始化，null 释放是安全的
            let struct_llvm = ctx.llvm_type(ty);
            for (idx, (_, ft)) in fields.iter().enumerate() {
                if !needs_drop(ft, &ctx.prog.struct_defs) {
                    continue;
                }
                let gep = ctx.tmp();
                lines.push(format!(
                    "%g{} = getelementptr inbounds {}, ptr {}, i32 0, i32 {}",
                    gep, struct_llvm, slot, idx
                ));
                emit_drop_value(ctx, &format!("%g{}", gep), ft, lines);
            }
        }
        _ => {}
    }
}
