use crate::intern::Symbol;
use crate::parser::ast::literal::Literal;

/// Phase 1.3（atb.1）：match 模式
#[derive(Debug, Clone)]
pub enum Pattern {
    /// `_`（不绑定）
    Wildcard,
    /// 字面量模式（int/float/char/bool）
    Literal(Literal),
    /// 标识符绑定（绑定整个值；作为 catch-all）
    Binding(Symbol),
    /// 枚举变体 `Variant(bindings)`（`E::Variant` 归一为 `E.Variant`）
    Enum { name: Symbol, bindings: Vec<Symbol> },
    /// 或模式 `p1 | p2`
    Or(Vec<Pattern>),
}

impl Pattern {
    /// 是否不可反驳（无 guard 时该臂必然匹配）
    pub fn is_irrefutable(&self) -> bool {
        match self {
            Pattern::Wildcard | Pattern::Binding(_) => true,
            Pattern::Or(ps) => ps.iter().any(|p| p.is_irrefutable()),
            _ => false,
        }
    }

    /// 模式引入的全部绑定名（含 or 分支与枚举载荷）
    pub fn bindings(&self) -> Vec<Symbol> {
        match self {
            Pattern::Wildcard | Pattern::Literal(_) => Vec::new(),
            Pattern::Binding(n) => vec![*n],
            Pattern::Enum { bindings, .. } => bindings.clone(),
            Pattern::Or(ps) => ps.iter().flat_map(|p| p.bindings()).collect(),
        }
    }

    /// 可读文本（格式化/诊断）
    pub fn display(&self) -> String {
        match self {
            Pattern::Wildcard => "_".into(),
            Pattern::Literal(l) => match l {
                Literal::Int(i, _) => i.to_string(),
                Literal::Float(f, _) => f.to_string(),
                Literal::Char(c, _) => format!("'{}'", c),
                Literal::String(s, _) => format!("\"{}\"", s),
                Literal::Bool(b, _) => b.to_string(),
            },
            Pattern::Binding(n) => n.as_str(),
            Pattern::Enum { name, bindings } => {
                if bindings.is_empty() {
                    name.as_str()
                } else {
                    format!("{}({})", name.as_str(),
                        bindings.iter().map(|b| b.as_str()).collect::<Vec<_>>().join(", "))
                }
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
