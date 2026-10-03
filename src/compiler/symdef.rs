use crate::parser::ast::Stmt;

/// A symbol definition with source location.
#[derive(Debug, Clone)]
pub struct SymDef {
    pub name: String,
    pub kind: String,
    pub file: String,
    pub line: usize,
    pub col: usize,
    /// 声明上的标注名（A0 起）
    pub attrs: Vec<String>,
    /// A3：效应 tokens（声明 + 推断事实 + 承诺 + throws 槽位）
    pub effects: Vec<String>,
}

/// A3：声明上的效应 tokens（无推断；推断由 CLI 侧接 HIR 摘要合并）。
fn stmt_effect_tokens(stmt: &Stmt) -> Vec<String> {
    let attrs = match stmt {
        Stmt::FnDecl { attrs, .. } => attrs,
        _ => return Vec::new(),
    };
    crate::hir::effects::parse(attrs)
        .map(|d| crate::hir::effects::EffectSummary { declared: d, inferred: Default::default() }.tokens())
        .unwrap_or_default()
}

/// 提取声明上的标注名列表。
fn stmt_attr_names(stmt: &Stmt) -> Vec<String> {
    let attrs = match stmt {
        Stmt::FnDecl { attrs, .. }
        | Stmt::StructDef { attrs, .. }
        | Stmt::EnumDef { attrs, .. }
        | Stmt::InterfaceDef { attrs, .. }
        | Stmt::ImplBlock { attrs, .. } => attrs,
        _ => return Vec::new(),
    };
    attrs.iter().map(|a| a.path_str()).collect()
}

/// Collect all symbol definitions from a list of statements with their locations.
pub fn collect_defs_from_stmts(
    stmts: &[Stmt],
    file: &str,
    prefix: &str,
    defs: &mut Vec<SymDef>,
) {
    for stmt in stmts {
        collect_defs_from_stmt(stmt, file, prefix, defs);
    }
}

fn collect_defs_from_stmt(stmt: &Stmt, file: &str, prefix: &str, defs: &mut Vec<SymDef>) {
    use crate::parser::ast::vis::Visibility;
    match stmt {
        Stmt::FnDecl {
            name,
            span,
            vis,
            attrs,
            ..
        } => {
            let qualified = if prefix.is_empty() {
                name.as_str().to_string()
            } else {
                format!("{}.{}", prefix, name)
            };
            let is_macro = attrs.iter().any(|a| a.is_builtin() && a.name.as_str() == "macro");
            defs.push(SymDef {
                name: qualified,
                kind: if is_macro {
                    "macro"
                } else if matches!(vis, Visibility::Pub) {
                    "pub fn"
                } else {
                    "fn"
                }
                .into(),
                file: file.into(),
                line: span.start_line,
                col: span.start_col,
                attrs: stmt_attr_names(stmt),
                effects: stmt_effect_tokens(stmt),
            });
        }
        Stmt::StructDef {
            name,
            span,
            vis,
            ..
        } => {
            let qualified = if prefix.is_empty() {
                name.as_str().to_string()
            } else {
                format!("{}.{}", prefix, name)
            };
            defs.push(SymDef {
                name: qualified,
                kind: if matches!(vis, Visibility::Pub) {
                    "pub struct"
                } else {
                    "struct"
                }
                .into(),
                file: file.into(),
                line: span.start_line,
                col: span.start_col,
                attrs: stmt_attr_names(stmt),
                effects: Vec::new(),
            });
        }
        Stmt::InterfaceDef {
            name, span, ..
        } => {
            let qualified = if prefix.is_empty() {
                name.as_str().to_string()
            } else {
                format!("{}.{}", prefix, name)
            };
            defs.push(SymDef {
                name: qualified,
                kind: "interface".into(),
                file: file.into(),
                line: span.start_line,
                col: span.start_col,
                attrs: stmt_attr_names(stmt),
                effects: Vec::new(),
            });
        }
            Stmt::EnumDef { .. } => {}
        Stmt::ImplBlock {
            type_name,
            span,
            ..
        } => {
            defs.push(SymDef {
                name: format!("impl {}", type_name),
                kind: "impl".into(),
                file: file.into(),
                line: span.start_line,
                col: span.start_col,
                attrs: stmt_attr_names(stmt),
                effects: Vec::new(),
            });
        }
        Stmt::Namespace {
            name, items, span, ..
        } => {
            let qualified = if prefix.is_empty() {
                name.as_str().to_string()
            } else {
                format!("{}.{}", prefix, name)
            };
            defs.push(SymDef {
                name: qualified,
                kind: "namespace".into(),
                file: file.into(),
                line: span.start_line,
                col: span.start_col,
                attrs: stmt_attr_names(stmt),
                effects: Vec::new(),
            });
            let ns_prefix = if prefix.is_empty() {
                name.as_str().to_string()
            } else {
                format!("{}.{}", prefix, name)
            };
            for item in items {
                collect_defs_from_stmt(item, file, &ns_prefix, defs);
            }
        }
        _ => {}
    }
}

/// Convert symbol definitions to JSON format.
pub fn defs_to_json(defs: &[SymDef]) -> String {
    let mut items: Vec<String> = defs
        .iter()
        .map(|d| {
            let attrs: Vec<String> = d
                .attrs
                .iter()
                .map(|a| format!("\"{}\"", a.replace('\\', "\\\\").replace('"', "\\\"")))
                .collect();
            let effects: Vec<String> = d
                .effects
                .iter()
                .map(|a| format!("\"{}\"", a.replace('\\', "\\\\").replace('"', "\\\"")))
                .collect();
            format!(
                r#"{{"name":"{}","kind":"{}","file":"{}","line":{},"col":{},"attrs":[{}],"effects":[{}]}}"#,
                d.name.replace('\\', "\\\\").replace('"', "\\\""),
                d.kind,
                d.file.replace('\\', "\\\\").replace('"', "\\\""),
                d.line,
                d.col,
                attrs.join(","),
                effects.join(",")
            )
        })
        .collect();
    items.sort();
    format!("[\n  {}\n]", items.join(",\n  "))
}
