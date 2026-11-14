use super::*;


/// Compiled package (.lcl) containing public symbols, generic source (reserved), LIR, and target metadata.
pub struct Package {
    pub name: String,
    pub version: String,
    pub target_types: Vec<TargetType>,
    pub symbols: Vec<PackageSymbol>,
    pub generic_sources: Vec<String>,
    pub lir_data: Vec<u8>,
    /// A5b-3：依赖包 stem 列表（构建宏插件 .so 时递归链接）
    pub deps: Vec<String>,
    /// A5j-1：插件清单（普通 .lcl 为 None）
    pub plugin_manifest: Option<super::plugin::PluginManifest>,
    /// A3c：HIR 推断摘要（仅内存；打包时用于生成 effects tokens）
    pub effect_summaries: std::collections::HashMap<String, crate::hir::effects::EffectSummary>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PackageSymbol {
    Fn {
        name: String,
        signature: String,
        /// A3c：效应 tokens（d:/i:/pure/no_error/throws:...）随包导出
        flags: Vec<String>,
    },
    Struct {
        name: String,
        copy: bool,
    },
    Namespace {
        name: String,
    },
    Interface {
        name: String,
    },
    /// A5b：导出的宏（`#[macro]` 函数）
    Macro {
        name: String,
        tokens: bool,
    },
    /// A5d：导出的 MIR 优化注解（`#[pass]` 函数）
    Pass {
        name: String,
    },
    /// A5d：导出的 MIR 只读检查注解（`#[check]` 函数）
    Check {
        name: String,
    },
    /// A5f：导出的属性插件（`#[attr]` 函数）
    Attr {
        name: String,
    },
    /// A5e/B1：导出的 HIR transform（`#[transform]` 函数）
    Transform {
        name: String,
    },
    /// 方法表（供编辑器补全/悬停；编译器导入忽略 `method=` 行）
    Method {
        type_name: String,
        name: String,
        signature: String,
    },
    /// M6.1b：导出的编译期常量（`const="name,ty,value"`）
    Const {
        name: String,
        ty: String,
        value: String,
    },
    /// M6.2b：导出的全局变量（`static="name,ty,mut"`）
    Static {
        name: String,
        ty: String,
        is_mut: bool,
    },
}

/// Parsed symbol from a .lcl package file.
#[derive(Debug, Clone)]
pub enum ImportedSymbol {
    Fn {
        name: String,
        sig: String,
        flags: Vec<String>,
    },
    Struct {
        name: String,
        copy: bool,
    },
    Namespace {
        name: String,
    },
    Interface {
        name: String,
    },
    /// A5b：包导出的宏
    Macro {
        name: String,
        tokens: bool,
    },
    /// A5d：包导出的 MIR 优化注解
    Pass {
        name: String,
    },
    /// A5d：包导出的 MIR 只读检查注解
    Check {
        name: String,
    },
    /// A5f：包导出的属性插件
    Attr {
        name: String,
    },
    /// A5e/B1：包导出的 HIR transform
    Transform {
        name: String,
    },
    /// M6.1b：包导出的编译期常量
    Const {
        name: String,
        ty: String,
        value: String,
    },
    /// M6.2b：包导出的全局变量
    Static {
        name: String,
        ty: String,
        is_mut: bool,
    },
}

pub(super) fn type_to_string(ty: &Type) -> String {
    match ty {
        Type::Default | Type::FnPtr(..) => "???".into(),
        // M2：闭包签名（与 sig_str_to_hir 的 `Fn(...)->R` 格式对应）
        Type::Closure(ps, ret, _, once) => format!("{}({})->{}",
            if *once { "FnOnce" } else { "Fn" },
            ps.iter().map(|p| type_to_string(p)).collect::<Vec<_>>().join(","),
            type_to_string(ret)),
        Type::Int(_) => "int".into(),
        Type::Float(_) => "float".into(),
        Type::Char(_) => "char".into(),
        Type::Bool(_) => "bool".into(),
        Type::Void(_) => "void".into(),
        Type::Never(_) => "!".into(),
        Type::Named(s, _) => s.as_str().to_string(),
        Type::Array(inner, _) => format!("[{}]", type_to_string(inner)),
        Type::ArraySized(inner, n, _) => format!("[{}; {}]", type_to_string(inner), n),
        Type::Unique(inner, _) => format!("unique {}", type_to_string(inner)),
        Type::RawPtr(inner, _) => format!("*{}", type_to_string(inner)),
        Type::Generic(name, args, _) => format!("{}[{}]", name, args.iter().map(|a| type_to_string(a)).collect::<Vec<_>>().join(",")),
        Type::Ref(inner, mutable, _) => {
            if *mutable {
                format!("ref mut {}", type_to_string(inner))
            } else {
                format!("ref {}", type_to_string(inner))
            }
        }
        Type::Self_(_) => "Self".into(),
    }
}
