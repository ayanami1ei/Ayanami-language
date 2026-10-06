use super::*;

pub fn sanitize_name(name: &str) -> String {
    name.replace('<', "_lt_").replace('>', "_gt_")
        .replace(',', "_c_").replace('[', "_lb_").replace(']', "_rb_").replace(' ', "_")
}

pub(crate) fn lit_to_string(lit: &HirLiteral, expected_ty: &HirType) -> String {
    match (lit, expected_ty) {
        (HirLiteral::Array(v), _) => {
            // M6.2c：数组常量在 emit_global_defs 特判发射（数据数组 + 指针变量）
            let parts: Vec<String> = v.iter().map(|e| lit_to_string(e, expected_ty)).collect();
            format!("[{}]", parts.join(", "))
        }
        (HirLiteral::Int(0), ty) if is_pointer_type(ty) && !matches!(ty, HirType::Named(_)) => "null".into(),
        (HirLiteral::Int(0), HirType::Named(_)) => "zeroinitializer".into(),
        (HirLiteral::Int(n), _) => format!("{}", n),
        (HirLiteral::Float(n), HirType::F32) => {
            let f = *n as f32;
            let s = format!("{}", f);
            // LLVM 对 float 十进制常量要求精确表示；否则用 16 位十六进制（double 位型，可精确转回）
            if s.parse::<f64>().map(|v| v == f as f64).unwrap_or(false) {
                if s.contains('.') { s } else { format!("{}.0", s) }
            } else {
                format!("0x{:016X}", (f as f64).to_bits())
            }
        }
        (HirLiteral::Float(n), _) => {
            let s = format!("{}", n);
            if !s.contains('.') { format!("{}.0", s) } else { s }
        }
        (HirLiteral::Char(c), _) => format!("{}", *c as u8),
        (HirLiteral::Bool(b), _) => if *b { "1".into() } else { "0".into() },
        (HirLiteral::String(_), _) => "null".into(),
    }
}

/// 浮点二元运算指令（fadd/fsub/fmul/fdiv/frem）
pub(crate) fn float_binop(op: BinaryOp, ty: &HirType, dest: u64, l: &str, r: &str) -> String {
    let t = if matches!(ty, HirType::F32) { "float" } else { "double" };
    let inst = match op {
        BinaryOp::Add => "fadd", BinaryOp::Sub => "fsub", BinaryOp::Mul => "fmul",
        BinaryOp::Div => "fdiv", _ => "frem",
    };
    format!("%t{} = {} {} {}, {}", dest, inst, t, l, r)
}

/// 浮点比较指令（fcmp）
pub(crate) fn float_cmp(op: BinaryOp, ty: &HirType, dest: u64, l: &str, r: &str) -> String {
    let t = if matches!(ty, HirType::F32) { "float" } else { "double" };
    let pred = match op {
        BinaryOp::Eq => "oeq", BinaryOp::Neq => "one", BinaryOp::Lt => "olt",
        BinaryOp::Gt => "ogt", BinaryOp::Le => "ole", _ => "oge",
    };
    format!("%t{} = fcmp {} {} {}, {}", dest, pred, t, l, r)
}

/// 整数类类型 → (位宽, 是否有符号)；非整数返回 None
pub(crate) fn int_info(ty: &HirType) -> Option<(u32, bool)> {
    match ty {
        HirType::Int => Some((64, true)),
        HirType::IntN { bits, signed } => Some((*bits as u32, *signed)),
        HirType::Char => Some((8, false)),
        HirType::Bool => Some((1, false)),
        _ => None,
    }
}

pub(crate) fn llvm_type_size(ty: &HirType) -> &'static str {
    match ty {
        HirType::Int => "8",
        HirType::Float => "8",
        HirType::F32 => "4",
        HirType::Char => "1",
        HirType::Bool => "1",
        HirType::Void | HirType::Never => "0",
        HirType::IntN { bits, .. } => match bits { 8 => "1", 16 => "2", 32 => "4", 64 => "8", 128 => "16", _ => "8" },
        HirType::Named(_) | HirType::FatPtr { .. } => "16",
        HirType::Unique(inner) => llvm_type_size(inner),
        HirType::Array(_) | HirType::ArraySized(_, _) => "16",
        HirType::FnPtr(..) => "8",
        HirType::Ref(_, _) => "16",
    }
}

/// 按 LLVM 布局计算类型大小（含对齐填充），用于数组分配。
/// 与 `llvm_type` 保持一致：`[T]`/`Unique`/`Array` 字段是指针（8），FatPtr 是 {ptr,ptr}（16）。
pub(crate) fn elem_layout_size(
    ty: &HirType,
    struct_defs: &std::collections::HashMap<Symbol, Vec<(Symbol, HirType)>>,
) -> u64 {
    fn layout(ty: &HirType, defs: &std::collections::HashMap<Symbol, Vec<(Symbol, HirType)>>) -> (u64, u64) {
        match ty {
            HirType::Void | HirType::Never => (0, 1),
            HirType::Char | HirType::Bool => (1, 1),
            HirType::Int | HirType::Float | HirType::Ref(_, _) | HirType::FnPtr(..) => (8, 8),
            HirType::F32 => (4, 4),
            HirType::IntN { bits, .. } => { let sz = (*bits / 8) as u64; (sz, sz.min(16)) }
            HirType::FatPtr { .. } => (16, 8),
            HirType::Array(_) | HirType::ArraySized(_, _) => (8, 8),
            HirType::Unique(inner) => match &**inner {
                HirType::Named(_) | HirType::FatPtr { .. }
                | HirType::Array(_) | HirType::ArraySized(_, _) => (8, 8),
                _ => layout(inner, defs),
            },
            HirType::Named(name) => {
                let fields = defs.get(name).or_else(|| {
                    let n = name.as_str();
                    n.find('<').map(|pos| Symbol::intern(&n[..pos])).and_then(|b| defs.get(&b))
                });
                match fields {
                    Some(fs) => {
                        let mut off = 0u64;
                        let mut align = 1u64;
                        for (_, ft) in fs {
                            let (sz, a) = layout(ft, defs);
                            off = (off + a - 1) / a * a + sz;
                            align = align.max(a);
                        }
                        ((off + align - 1) / align * align, align)
                    }
                    None => (16, 8), // 未知结构保守回退
                }
            }
        }
    }
    layout(ty, struct_defs).0
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
