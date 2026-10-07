//! LIR 递归 drop 发射：结构体/数组/胖指针/闭包的释放代码生成。
use super::*;

/// 动态数组字段的长度兄弟字段：优先名为 len/length/size/count 的整数字段，
/// 其次数组字段之后最近的整数字段（如 ArrayList 的 data/len/capability）。
fn find_len_field(fields: &[(Symbol, HirType)], array_idx: usize) -> Option<usize> {
    let is_int = |t: &HirType| matches!(t, HirType::Int | HirType::IntN { bits: 64, .. });
    let by_names = |names: &[&str]| -> Option<usize> {
        fields.iter().enumerate().find(|(i, (n, t))| {
            *i != array_idx && is_int(t) && names.iter().any(|x| n.as_str() == *x)
        }).map(|(i, _)| i)
    };
    // 容量优先：pop/remove 后越界槽仍持有原值（分配时 memset 零初始化，释放安全）
    if let Some(i) = by_names(&["capability", "capacity", "cap"]) {
        return Some(i);
    }
    if let Some(i) = by_names(&["len", "length", "size", "count"]) {
        return Some(i);
    }
    let after = array_idx + 1;
    if after < fields.len() && is_int(&fields[after].1) {
        return Some(after);
    }
    None
}

/// 动态计数数组的元素递归释放（运行时长度值）。
fn emit_array_elem_drops_dyn(
    ctx: &mut LirEmitCtx,
    data: &str,
    elem: &HirType,
    count: &str,
    lines: &mut Vec<String>,
) {
    let elem_llvm = ctx.llvm_type(elem);
    let iv = ctx.tmp();
    lines.push(format!("%iv{} = alloca i64, align 8", iv));
    lines.push(format!("store i64 0, ptr %iv{}", iv));
    lines.push(format!("br label %L{}cond", iv));
    lines.push(format!("L{}cond:", iv));
    let v = ctx.tmp();
    lines.push(format!("%t{} = load i64, ptr %iv{}", v, iv));
    let c = ctx.tmp();
    lines.push(format!("%t{} = icmp ult i64 %t{}, {}", c, v, count));
    lines.push(format!("br i1 %t{}, label %L{}body, label %L{}end", c, iv, iv));
    lines.push(format!("L{}body:", iv));
    let ep = ctx.tmp();
    lines.push(format!("%t{} = getelementptr {}, ptr {}, i64 %t{}", ep, elem_llvm, data, v));
    emit_drop_value(ctx, &format!("%t{}", ep), elem, lines);
    let nv = ctx.tmp();
    lines.push(format!("%t{} = add i64 %t{}, 1", nv, v));
    lines.push(format!("store i64 %t{}, ptr %iv{}", nv, iv));
    lines.push(format!("br label %L{}cond", iv));
    lines.push(format!("L{}end:", iv));
}

/// 静态计数数组的元素递归释放：小 n 展开，大 n 用循环。
fn emit_array_elem_drops(
    ctx: &mut LirEmitCtx,
    data: &str,
    elem: &HirType,
    n: usize,
    lines: &mut Vec<String>,
) {
    let elem_llvm = ctx.llvm_type(elem);
    if n <= 4 {
        for k in 0..n {
            let ep = ctx.tmp();
            lines.push(format!("%t{} = getelementptr {}, ptr {}, i64 {}", ep, elem_llvm, data, k));
            emit_drop_value(ctx, &format!("%t{}", ep), elem, lines);
        }
        return;
    }
    let iv = ctx.tmp();
    lines.push(format!("%iv{} = alloca i64, align 8", iv));
    lines.push(format!("store i64 0, ptr %iv{}", iv));
    lines.push(format!("br label %L{}cond", iv));
    lines.push(format!("L{}cond:", iv));
    let v = ctx.tmp();
    lines.push(format!("%t{} = load i64, ptr %iv{}", v, iv));
    let c = ctx.tmp();
    lines.push(format!("%t{} = icmp ult i64 %t{}, {}", c, v, n));
    lines.push(format!("br i1 %t{}, label %L{}body, label %L{}end", c, iv, iv));
    lines.push(format!("L{}body:", iv));
    let ep = ctx.tmp();
    lines.push(format!("%t{} = getelementptr {}, ptr {}, i64 %t{}", ep, elem_llvm, data, v));
    emit_drop_value(ctx, &format!("%t{}", ep), elem, lines);
    let nv = ctx.tmp();
    lines.push(format!("%t{} = add i64 %t{}, 1", nv, v));
    lines.push(format!("store i64 %t{}, ptr %iv{}", nv, iv));
    lines.push(format!("br label %L{}cond", iv));
    lines.push(format!("L{}end:", iv));
}

/// 发射对 `slot`（指向 `ty` 值的内存指针）的递归释放代码。
pub(crate) fn emit_drop_value(
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
                // 静态计数数组：拥有元素逐个递归释放，再释放缓冲区
                HirType::ArraySized(elem, n)
                    if *n > 0 && needs_drop(elem, &ctx.prog.struct_defs) =>
                {
                    emit_array_elem_drops(ctx, &ptr, elem, *n, lines);
                }
                // 动态 [T] 无长度元数据：仅释放缓冲区（元素 drop 待长度方案）
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
                    // 数据非空时调用 vtable[0] drop glue（箱内具体值的拥有字段递归释放）
                    let n = ctx.tmp();
                    lines.push(format!("%c{} = icmp eq ptr %c{}, null", n, tmp));
                    lines.push(format!("br i1 %c{}, label %L{}skip, label %L{}vt", n, n, n));
                    lines.push(format!("L{}vt:", n));
                    let v = ctx.tmp();
                    lines.push(format!("%g{} = getelementptr inbounds {{ ptr, ptr }}, ptr {}, i32 0, i32 1", v, slot));
                    lines.push(format!("%c{} = load ptr, ptr %g{}, align 8", v, v));
                    let f = ctx.tmp();
                    lines.push(format!("%c{} = load ptr, ptr %c{}, align 8", f, v));
                    let nf = ctx.tmp();
                    lines.push(format!("%c{} = icmp eq ptr %c{}, null", nf, f));
                    lines.push(format!("br i1 %c{}, label %L{}skip, label %L{}call", nf, n, n));
                    lines.push(format!("L{}call:", n));
                    lines.push(format!("call void %c{}(ptr %c{})", f, tmp));
                    lines.push(format!("br label %L{}end", n));
                    lines.push(format!("L{}skip:", n));
                    lines.push(format!("br label %L{}end", n));
                    lines.push(format!("L{}end:", n));
                    lines.push(format!("call void @__ayanami_unique_free(i8* %c{})", tmp));
                }
                _ => {}
            }
        }
        HirType::Closure(..) => {
            // { env, vtable }：env 为空（零初始化）跳过；否则调用 vtable[0] drop glue
            let t = ctx.tmp();
            lines.push(format!("%g{} = getelementptr inbounds {{ ptr, ptr }}, ptr {}, i32 0, i32 0", t, slot));
            lines.push(format!("%c{} = load ptr, ptr %g{}, align 8", t, t));
            let isnull = ctx.tmp();
            lines.push(format!("%c{} = icmp eq ptr %c{}, null", isnull, t));
            lines.push(format!("br i1 %c{}, label %L{}skip, label %L{}call", isnull, t, t));
            lines.push(format!("L{}call:", t));
            let v = ctx.tmp();
            lines.push(format!("%g{} = getelementptr inbounds {{ ptr, ptr }}, ptr {}, i32 0, i32 1", v, slot));
            lines.push(format!("%c{} = load ptr, ptr %g{}, align 8", v, v));
            let f = ctx.tmp();
            lines.push(format!("%c{} = load ptr, ptr %c{}, align 8", f, v));
            lines.push(format!("call void %c{}(ptr %c{})", f, t));
            lines.push(format!("br label %L{}end", t));
            lines.push(format!("L{}skip:", t));
            lines.push(format!("br label %L{}end", t));
            lines.push(format!("L{}end:", t));
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
                // 动态 [T] + 拥有元素 + 长度兄弟字段（如 ArrayList.len）→ 依赖 drop
                if let HirType::Unique(inner) = ft {
                    if let HirType::Array(elem) = inner.as_ref() {
                        if needs_drop(elem, &ctx.prog.struct_defs) {
                            if let Some(len_idx) = find_len_field(&fields, idx) {
                                let data_t = ctx.tmp();
                                lines.push(format!("%c{} = load ptr, ptr %g{}, align 8", data_t, gep));
                                let lenp = ctx.tmp();
                                lines.push(format!(
                                    "%g{} = getelementptr inbounds {}, ptr {}, i32 0, i32 {}",
                                    lenp, struct_llvm, slot, len_idx
                                ));
                                let len_t = ctx.tmp();
                                lines.push(format!("%c{} = load i64, ptr %g{}, align 8", len_t, lenp));
                                emit_array_elem_drops_dyn(
                                    ctx, &format!("%c{}", data_t), elem, &format!("%c{}", len_t), lines);
                                lines.push(format!("call void @__ayanami_unique_free(i8* %c{})", data_t));
                                continue;
                            }
                        }
                    }
                }
                emit_drop_value(ctx, &format!("%g{}", gep), ft, lines);
            }
        }
        _ => {}
    }
}

/// 动态数组按外部计数释放：元素递归 drop + 缓冲释放。
impl LirNode for SLirDropArray {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "DropArray" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let mut lines = Vec::new();
        let p = ctx.tmp();
        lines.push(format!("%c{} = load ptr, ptr %v{}, align 8", p, self.var.0));
        // 空指针（未分配/已移出）跳过元素循环；free(null) 安全
        let n = ctx.tmp();
        lines.push(format!("%c{} = icmp eq ptr %c{}, null", n, p));
        lines.push(format!("br i1 %c{}, label %L{}skip, label %L{}body", n, n, n));
        lines.push(format!("L{}body:", n));
        let c = ctx.tmp();
        lines.push(format!("%c{} = load i64, ptr %v{}, align 8", c, self.count_var.0));
        emit_array_elem_drops_dyn(ctx, &format!("%c{}", p), &self.elem_ty, &format!("%c{}", c), &mut lines);
        lines.push(format!("br label %L{}end", n));
        lines.push(format!("L{}skip:", n));
        lines.push(format!("br label %L{}end", n));
        lines.push(format!("L{}end:", n));
        lines.push(format!("call void @__ayanami_unique_free(i8* %c{})", p));
        lines
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    drop_array v{} : {:?} count=v{}", self.var.0, self.elem_ty, self.count_var.0)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(40);
        put_u32(buf, self.var.0 as u32);
        put_type(buf, &self.elem_ty);
        put_u32(buf, self.count_var.0 as u32);
    }
}
