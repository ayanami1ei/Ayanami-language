//! LIR 深拷贝（clone）发射：拥有堆数据的值复制到新缓冲。
//!
//! 语义：数组/集合的元素读取按 clone（非破坏性，源保留），配合元素递归 drop
//! （源集合销毁时释放原值）——读取多次、迭代、pop/remove 位移均安全无泄漏。
use super::*;

/// 类型是否需要深拷贝（拥有堆数据）。
fn deep(ty: &HirType, defs: &HashMap<Symbol, Vec<(Symbol, HirType)>>) -> bool {
    needs_drop(ty, defs)
}

/// 兄弟字段查找：按名称优先，其次数组字段后最近的 64 位整数字段。
fn count_field(fields: &[(Symbol, HirType)], array_idx: usize, names: &[&str]) -> Option<usize> {
    let is_int = |t: &HirType| matches!(t, HirType::Int | HirType::IntN { bits: 64, .. });
    for (i, (n, t)) in fields.iter().enumerate() {
        if i != array_idx && is_int(t) {
            let name = n.as_str();
            if names.iter().any(|x| name == *x) {
                return Some(i);
            }
        }
    }
    let after = array_idx + 1;
    if after < fields.len() && is_int(&fields[after].1) {
        return Some(after);
    }
    None
}

/// 发射深拷贝：把 `*src` 复制到 `*dst`（两者均为指向 `ty` 值的指针）。
/// `count_hint`：动态数组字段的长度值（如 ArrayList.len），无则浅拷贝指针。
pub(crate) fn emit_clone_value(
    ctx: &mut LirEmitCtx,
    dst: &str,
    src: &str,
    ty: &HirType,
    count_hint: Option<&str>,
    lines: &mut Vec<String>,
) {
    let llvm_ty = ctx.llvm_type(ty);
    if !deep(ty, &ctx.prog.struct_defs) {
        let t = ctx.tmp();
        lines.push(format!("%t{} = load {}, ptr {}", t, llvm_ty, src));
        lines.push(format!("store {} %t{}, ptr {}", llvm_ty, t, dst));
        return;
    }
    match ty {
        HirType::Named(name) => {
            let Some(fields) = ctx.prog.struct_defs.get(name).cloned() else { return; };
            for (idx, (_, ft)) in fields.iter().enumerate() {
                let ds = ctx.tmp(); let ss = ctx.tmp();
                lines.push(format!("%g{} = getelementptr inbounds {}, ptr {}, i32 0, i32 {}", ds, llvm_ty, dst, idx));
                lines.push(format!("%g{} = getelementptr inbounds {}, ptr {}, i32 0, i32 {}", ss, llvm_ty, src, idx));
                // 动态数组字段：长度取兄弟字段（clone 只复制活跃元素，越界槽为零）
                let hint = match ft {
                    HirType::Unique(inner) if matches!(inner.as_ref(), HirType::Array(_)) => {
                        count_field(&fields, idx, &["len", "length", "size", "count"]).map(|li| {
                            let lp = ctx.tmp(); let lv = ctx.tmp();
                            lines.push(format!("%g{} = getelementptr inbounds {}, ptr {}, i32 0, i32 {}", lp, llvm_ty, src, li));
                            lines.push(format!("%t{} = load i64, ptr %g{}", lv, lp));
                            format!("%t{}", lv)
                        })
                    }
                    _ => None,
                };
                emit_clone_value(ctx, &format!("%g{}", ds), &format!("%g{}", ss), ft, hint.as_deref(), lines);
            }
        }
        HirType::Unique(inner) => {
            let elem_size = elem_layout_size(inner, &ctx.prog.struct_defs);
            match inner.as_ref() {
                HirType::ArraySized(elem, n) => {
                    emit_clone_array(ctx, dst, src, elem, &n.to_string(), elem_size, lines);
                }
                HirType::Array(elem) => {
                    let Some(count) = count_hint else {
                        // 无长度信息：浅拷贝指针（不支持场景，保持可用）
                        let t = ctx.tmp();
                        lines.push(format!("%t{} = load ptr, ptr {}", t, src));
                        lines.push(format!("store ptr %t{}, ptr {}", t, dst));
                        return;
                    };
                    emit_clone_array(ctx, dst, src, elem, count, elem_size, lines);
                }
                _ => {
                    // 拥有非数组值（unique T）：分配 + 深拷贝
                    let buf = ctx.tmp(); let dstp = ctx.tmp(); let srcp = ctx.tmp();
                    lines.push(format!("%t{} = call i8* @__ayanami_unique_alloc(i64 {})", buf, elem_size));
                    lines.push(format!("%t{} = bitcast i8* %t{} to ptr", dstp, buf));
                    lines.push(format!("%t{} = load ptr, ptr {}", srcp, src));
                    emit_clone_value(ctx, &format!("%t{}", dstp), &format!("%t{}", srcp), inner, None, lines);
                    lines.push(format!("store ptr %t{}, ptr {}", dstp, dst));
                }
            }
        }
        // 拥有胖指针/闭包：暂浅拷贝（别名；数组元素场景罕见）
        _ => {
            let t = ctx.tmp();
            lines.push(format!("%t{} = load {}, ptr {}", t, llvm_ty, src));
            lines.push(format!("store {} %t{}, ptr {}", llvm_ty, t, dst));
        }
    }
}

/// 数组缓冲深拷贝：分配 count*elem_size，逐元素 clone（Copy 元素 memcpy）。
fn emit_clone_array(
    ctx: &mut LirEmitCtx,
    dst: &str,
    src: &str,
    elem: &HirType,
    count: &str,
    elem_size: u64,
    lines: &mut Vec<String>,
) {
    let count_t = ctx.tmp();
    lines.push(format!("%t{} = add i64 0, {}", count_t, count));
    let size_t = ctx.tmp();
    lines.push(format!("%t{} = mul i64 %t{}, {}", size_t, count_t, elem_size));
    let buf = ctx.tmp(); let dstp = ctx.tmp(); let srcp = ctx.tmp();
    lines.push(format!("%t{} = call i8* @__ayanami_unique_alloc(i64 %t{})", buf, size_t));
    lines.push(format!("%t{} = bitcast i8* %t{} to ptr", dstp, buf));
    lines.push(format!("%t{} = load ptr, ptr {}", srcp, src));
    if !deep(elem, &ctx.prog.struct_defs) {
        // Copy 元素：整体 memcpy（零长度跳过，源可能为 null）
        let z = ctx.tmp();
        lines.push(format!("%t{} = icmp eq i64 %t{}, 0", z, size_t));
        lines.push(format!("br i1 %t{}, label %L{}skip, label %L{}do", z, z, z));
        lines.push(format!("L{}do:", z));
        lines.push(format!("call void @llvm.memcpy.p0.p0.i64(ptr %t{}, ptr %t{}, i64 %t{}, i1 false)", dstp, srcp, size_t));
        lines.push(format!("br label %L{}end", z));
        lines.push(format!("L{}skip:", z));
        lines.push(format!("br label %L{}end", z));
        lines.push(format!("L{}end:", z));
    } else {
        let elem_llvm = ctx.llvm_type(elem);
        let iv = ctx.tmp();
        lines.push(format!("%iv{} = alloca i64, align 8", iv));
        lines.push(format!("store i64 0, ptr %iv{}", iv));
        lines.push(format!("br label %L{}cond", iv));
        lines.push(format!("L{}cond:", iv));
        let v = ctx.tmp();
        lines.push(format!("%t{} = load i64, ptr %iv{}", v, iv));
        let c = ctx.tmp();
        lines.push(format!("%t{} = icmp ult i64 %t{}, %t{}", c, v, count_t));
        lines.push(format!("br i1 %t{}, label %L{}body, label %L{}end", c, iv, iv));
        lines.push(format!("L{}body:", iv));
        let dp = ctx.tmp(); let sp = ctx.tmp();
        lines.push(format!("%t{} = getelementptr {}, ptr %t{}, i64 %t{}", dp, elem_llvm, dstp, v));
        lines.push(format!("%t{} = getelementptr {}, ptr %t{}, i64 %t{}", sp, elem_llvm, srcp, v));
        emit_clone_value(ctx, &format!("%t{}", dp), &format!("%t{}", sp), elem, None, lines);
        let nv = ctx.tmp();
        lines.push(format!("%t{} = add i64 %t{}, 1", nv, v));
        lines.push(format!("store i64 %t{}, ptr %iv{}", nv, iv));
        lines.push(format!("br label %L{}cond", iv));
        lines.push(format!("L{}end:", iv));
    }
    lines.push(format!("store ptr %t{}, ptr {}", dstp, dst));
}

/// `clone` 节点：src 指向源值，复制到新 alloca 并 load 出值。
impl LirNode for SLirClone {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "Clone" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let llvm_ty = ctx.llvm_type(&self.ty);
        let src = ctx.value_ref(&self.src, &HirType::Unique(Box::new(self.ty.clone())));
        let mut lines = vec![format!("%t{} = alloca {}, align 8", self.alloca_tmp, llvm_ty)];
        emit_clone_value(ctx, &format!("%t{}", self.alloca_tmp), &src, &self.ty, None, &mut lines);
        lines.push(format!("%t{} = load {}, ptr %t{}", self.dest, llvm_ty, self.alloca_tmp));
        lines
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    t{} = clone : {:?}", self.dest, self.ty)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(39);
        put_u64(buf, self.dest);
        put_u64(buf, self.alloca_tmp);
        put_value(buf, &self.src);
        put_type(buf, &self.ty);
    }
}
