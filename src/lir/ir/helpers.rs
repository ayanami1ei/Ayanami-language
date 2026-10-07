use super::*;

pub fn sanitize_name(name: &str) -> String {
    let s = name.replace("->", "_to_")
        .replace('<', "_lt_").replace('>', "_gt_")
        .replace(',', "_c_").replace('[', "_lb_").replace(']', "_rb_")
        .replace(' ', "_");
    // LLVM 标识符兜底：括号等其余字符统一转 `_`（`Fn(int)->int` 等闭包泛型实参）
    s.chars().map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '.' { c } else { '_' }).collect()
}

pub(crate) fn lit_to_string(lit: &HirLiteral, expected_ty: &HirType) -> String {
    match (lit, expected_ty) {
        (HirLiteral::Array(v), _) => {
            // M6.2c：数组常量在 emit_global_defs 特判发射（数据数组 + 指针变量）
            let parts: Vec<String> = v.iter().map(|e| lit_to_string(e, expected_ty)).collect();
            format!("[{}]", parts.join(", "))
        }
        (HirLiteral::Struct(v), _) => {
            let parts: Vec<String> = v.iter().map(|e| lit_to_string(e, expected_ty)).collect();
            format!("{{ {} }}", parts.join(", "))
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
            HirType::FatPtr { .. } | HirType::Closure(..) => (16, 8),
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
pub(crate) fn needs_drop(ty: &HirType, struct_defs: &HashMap<Symbol, Vec<(Symbol, HirType)>>) -> bool {
    match ty {
        HirType::Unique(_) => true,
        HirType::Closure(_, _, true, _) => true,
        HirType::FatPtr { kind, .. } => !matches!(kind.as_ref(), HirType::Ref(..)),
        HirType::Named(name) => {
            let Some(fields) = struct_defs.get(name) else { return false; };
            fields.iter().any(|(_, ft)| needs_drop(ft, struct_defs))
        }
        _ => false,
    }
}
