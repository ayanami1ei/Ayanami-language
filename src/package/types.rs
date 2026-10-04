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
    },
    /// A5d：导出的 MIR 优化注解（`#[pass]` 函数）
    Pass {
        name: String,
    },
    /// A5d：导出的 MIR 只读检查注解（`#[check]` 函数）
    Check {
        name: String,
    },
    /// 方法表（供编辑器补全/悬停；编译器导入忽略 `method=` 行）
    Method {
        type_name: String,
        name: String,
        signature: String,
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
    },
    /// A5d：包导出的 MIR 优化注解
    Pass {
        name: String,
    },
    /// A5d：包导出的 MIR 只读检查注解
    Check {
        name: String,
    },
}
