use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Keyword {
    Pub,
    Crate,
    Fn,
    Return,
    For,
    If,
    Elif,
    Else,
    In,
    While,
    Int,
    Float,
    Char,
    Bool,
    Mut,
    Shared,
    Unique,
    Weak,
    Namespace,
    Struct,
    Move,
    Clone,
    Interface,
    Impl,
    Self_,
    True,
    False,
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Keyword::Pub => "pub",
            Keyword::Crate => "crate",
            Keyword::Fn => "fn",
            Keyword::Return => "return",
            Keyword::For => "for",
            Keyword::If => "if",
            Keyword::Elif => "elif",
            Keyword::Else => "else",
            Keyword::In => "in",
            Keyword::While => "while",
            Keyword::Int => "int",
            Keyword::Float => "float",
            Keyword::Char => "char",
            Keyword::Bool => "bool",
            Keyword::Mut => "mut",
            Keyword::Shared => "shared",
            Keyword::Unique => "unique",
            Keyword::Weak => "weak",
            Keyword::Namespace => "namespace",
            Keyword::Struct => "struct",
            Keyword::Move => "move",
            Keyword::Clone => "clone",
            Keyword::Interface => "interface",
            Keyword::Impl => "impl",
            Keyword::Self_ => "self",
            Keyword::True => "true",
            Keyword::False => "false",
        };
        write!(f, "{}", s)
    }
}
