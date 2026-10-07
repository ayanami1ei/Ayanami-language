use crate::intern::Symbol;
use crate::parser::ast::literal::Literal;

/// Phase 1.3：match 模式（atb.1 字面量/`_`/`|`/guard；atb.2 嵌套/解构/range）
#[derive(Debug, Clone)]
pub enum Pattern {
    /// `_`（不绑定）
    Wildcard,
    /// 字面量模式（int/float/char/bool）
    Literal(Literal),
    /// 标识符绑定（绑定整个值；作为 catch-all）
    Binding(Symbol),
    /// 枚举变体 `Variant(sub-patterns...)`（`E::Variant` 归一为 `E.Variant`）
    Enum { name: Symbol, args: Vec<Pattern> },
    /// 结构体解构 `Point { x, y }` / `Point { x = pat }`
    Struct { name: Symbol, fields: Vec<(Symbol, Pattern)> },
    /// 整数区间 `lo..hi`（半开）/ `lo..=hi`（闭）
    Range { lo: Literal, hi: Literal, inclusive: bool },
    /// 或模式 `p1 | p2`
    Or(Vec<Pattern>),
}

impl Pattern {
    /// 是否不可反驳（无 guard 时该臂必然匹配）
    pub fn is_irrefutable(&self) -> bool {
        match self {
            Pattern::Wildcard | Pattern::Binding(_) => true,
            Pattern::Struct { fields, .. } => fields.iter().all(|(_, p)| p.is_irrefutable()),
            Pattern::Or(ps) => ps.iter().any(|p| p.is_irrefutable()),
            _ => false,
        }
    }

    /// 模式引入的全部绑定名（含 or 分支、嵌套载荷与结构体字段）
    pub fn bindings(&self) -> Vec<Symbol> {
        match self {
            Pattern::Wildcard | Pattern::Literal(_) | Pattern::Range { .. } => Vec::new(),
            Pattern::Binding(n) => vec![*n],
            Pattern::Enum { args, .. } => args.iter().flat_map(|p| p.bindings()).collect(),
            Pattern::Struct { fields, .. } => fields.iter().flat_map(|(_, p)| p.bindings()).collect(),
            Pattern::Or(ps) => ps.iter().flat_map(|p| p.bindings()).collect(),
        }
    }

    /// 可读文本（格式化/诊断）
    pub fn display(&self) -> String {
        match self {
            Pattern::Wildcard => "_".into(),
            Pattern::Literal(l) => display_lit(l),
            Pattern::Binding(n) => n.as_str(),
            Pattern::Enum { name, args } => {
                if args.is_empty() {
                    name.as_str()
                } else {
                    format!("{}({})", name.as_str(),
                        args.iter().map(|p| p.display()).collect::<Vec<_>>().join(", "))
                }
            }
            Pattern::Struct { name, fields } => {
                let fs: Vec<String> = fields.iter().map(|(n, p)| {
                    if matches!(p, Pattern::Binding(b) if *b == *n) {
                        n.as_str()
                    } else {
                        format!("{} = {}", n.as_str(), p.display())
                    }
                }).collect();
                format!("{} {{ {} }}", name.as_str(), fs.join(", "))
            }
            Pattern::Range { lo, hi, inclusive } => {
                format!("{}{}{}", display_lit(lo), if *inclusive { "..=" } else { ".." }, display_lit(hi))
            }
            Pattern::Or(ps) => ps.iter().map(|p| p.display()).collect::<Vec<_>>().join(" | "),
        }
    }

    /// 枚举变体名（或模式取首个枚举分支；其余 None）
    pub fn enum_variant(&self) -> Option<&Symbol> {
        match self {
            Pattern::Enum { name, .. } => Some(name),
            Pattern::Or(ps) => ps.iter().find_map(|p| p.enum_variant()),
            _ => None,
        }
    }
}

fn display_lit(l: &Literal) -> String {
    match l {
        Literal::Int(i, _) => i.to_string(),
        Literal::Float(f, _) => f.to_string(),
        Literal::Char(c, _) => format!("'{}'", c),
        Literal::String(s, _) => format!("\"{}\"", s),
        Literal::Bool(b, _) => b.to_string(),
    }
}
