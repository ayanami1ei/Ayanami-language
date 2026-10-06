//! M6.2/M6.2c：全局常量发射（标量 / 数组 / 结构体）。
use super::*;

/// 剥去拥有包装（`Unique(T)` → `T`）
fn strip_unique(ty: &HirType) -> &HirType {
    match ty {
        HirType::Unique(inner) => inner.as_ref(),
        other => other,
    }
}

fn path_key(path: &[usize]) -> String {
    path.iter().map(|i| i.to_string()).collect::<Vec<_>>().join("_")
}

impl<'a> Emitter<'a> {
    /// M6.2：发射全局变量
    /// - 标量：`@name = global <ty> <init>`
    /// - 数组：数据数组常量 + 指针变量（语言层 `[T; N]` 为指针语义）
    /// - 结构体：内联结构体常量（值语义，取址即全局地址）；数组字段预发射数据全局并引用
    pub(super) fn emit_global_defs(&mut self) {
        use crate::hir::ir::HirLiteral;
        for g in &self.prog.globals {
            let linkage = if crate::hir::contracts::is_release() && !g.is_pub { "internal " } else { "" };
            if let HirLiteral::Struct(_) = &g.value {
                let mut arrays: std::collections::HashMap<Vec<usize>, String> = std::collections::HashMap::new();
                let mut path = Vec::new();
                self.collect_array_fields(&g.name.as_str(), &g.value, &g.ty, &mut path, &mut arrays);
                let mut path = Vec::new();
                let lit = self.const_value_at(&g.value, &g.ty, &mut path, &arrays);
                let ty = self.llvm_type(&g.ty);
                self.wln_fmt(format_args!("@{} = {}global {} {}", g.name.as_str(), linkage, ty, lit));
                continue;
            }
            if let HirLiteral::Array(_) = &g.value {
                let data = self.const_value_at(&g.value, strip_unique(&g.ty), &mut Vec::new(), &Default::default());
                self.wln_fmt(format_args!("@__ayanami_gdata_{} = private global {}", g.name.as_str(), data));
                self.wln_fmt(format_args!(
                    "@{} = {}global ptr @__ayanami_gdata_{}",
                    g.name.as_str(), linkage, g.name.as_str()
                ));
                continue;
            }
            let ty = self.llvm_type(&g.ty);
            let lit = lit_to_string(&g.value, &g.ty);
            self.wln_fmt(format_args!("@{} = {}global {} {}", g.name.as_str(), linkage, ty, lit));
        }
        for (name, ty) in &self.prog.extern_globals {
            let ty = self.llvm_type(ty);
            self.wln_fmt(format_args!("@{} = external global {}", name.as_str(), ty));
        }
        if !self.prog.globals.is_empty() || !self.prog.extern_globals.is_empty() {
            self.wln("");
        }
    }

    /// 结构体常量中的数组字段（指针语义）→ 预发射数据数组，记录 路径 → 符号名
    fn collect_array_fields(
        &mut self,
        base: &str,
        lit: &crate::hir::ir::HirLiteral,
        ty: &HirType,
        path: &mut Vec<usize>,
        out: &mut std::collections::HashMap<Vec<usize>, String>,
    ) {
        use crate::hir::ir::HirLiteral;
        match (lit, strip_unique(ty)) {
            (HirLiteral::Array(_), HirType::ArraySized(_, _) | HirType::Array(_))
                if matches!(ty, HirType::Unique(_)) =>
            {
                let name = format!("__ayanami_gdata_{}_{}", base, path_key(path));
                let data = self.const_value_at(lit, strip_unique(ty), &mut Vec::new(), &Default::default());
                self.wln_fmt(format_args!("@{} = private global {}", name, data));
                out.insert(path.clone(), name);
            }
            (HirLiteral::Struct(vals), HirType::Named(n)) => {
                if let Some(def) = self.prog.struct_defs.get(n).cloned() {
                    for (i, (v, f)) in vals.iter().zip(def.iter()).enumerate() {
                        path.push(i);
                        self.collect_array_fields(base, v, &f.1, path, out);
                        path.pop();
                    }
                }
            }
            _ => {}
        }
    }

    /// 递归生成带类型的 LLVM 常量；指针位置的数组字段引用预发射数据全局
    fn const_value_at(
        &self,
        lit: &crate::hir::ir::HirLiteral,
        ty: &HirType,
        path: &mut Vec<usize>,
        arrays: &std::collections::HashMap<Vec<usize>, String>,
    ) -> String {
        use crate::hir::ir::HirLiteral;
        match (lit, strip_unique(ty)) {
            (HirLiteral::Array(vals), HirType::ArraySized(elem, _) | HirType::Array(elem)) => {
                if matches!(ty, HirType::Unique(_)) {
                    if let Some(name) = arrays.get(path) {
                        // 裸符号：字段格式化已带 `ptr` 类型前缀
                        return format!("@{}", name);
                    }
                }
                let et = self.llvm_type(elem);
                let elems: Vec<String> = vals.iter()
                    .map(|v| format!("{} {}", et, self.const_value_at(v, elem, path, arrays)))
                    .collect();
                format!("[{} x {}] [{}]", vals.len(), et, elems.join(", "))
            }
            (HirLiteral::Struct(vals), HirType::Named(n)) => {
                if let Some(def) = self.prog.struct_defs.get(n).cloned() {
                    let parts: Vec<String> = vals.iter().zip(def.iter()).enumerate()
                        .map(|(i, (v, f))| {
                            path.push(i);
                            let s = format!("{} {}", self.llvm_type(&f.1), self.const_value_at(v, &f.1, path, arrays));
                            path.pop();
                            s
                        })
                        .collect();
                    // LLVM 结构体常量语法：`{ <typed fields> }`（不带 %name 前缀）
                    format!("{{ {} }}", parts.join(", "))
                } else {
                    "zeroinitializer".into()
                }
            }
            _ => lit_to_string(lit, ty),
        }
    }
}
