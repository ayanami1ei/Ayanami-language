use super::*;

use super::literal_text::parse_int_text;
use crate::parser::ast::pattern::Pattern;

impl Parser {
    /// Phase 1.3（atb.1）：模式 —— `_` / 字面量 / 绑定 / 枚举变体 / 或模式 `p1 | p2`
    pub(super) fn parse_pattern(&mut self) -> Result<Pattern> {
        let mut pats = vec![self.parse_pattern_atom()?];
        while matches!(self.peek().map(|t| &t.kind), Some(TokenKind::Operator(op)) if op == "|") {
            self.advance();
            pats.push(self.parse_pattern_atom()?);
        }
        Ok(if pats.len() == 1 { pats.pop().unwrap() } else { Pattern::Or(pats) })
    }

    fn parse_pattern_atom(&mut self) -> Result<Pattern> {
        let tok = self.peek().ok_or_else(|| self.error("expected pattern"))?.clone();
        match tok.kind {
            // `_` 通配
            TokenKind::Identifier(ref s) if s == "_" => {
                self.advance();
                Ok(Pattern::Wildcard)
            }
            // 负整数字面量
            TokenKind::Operator(ref op) if op == "-" => {
                self.advance();
                match self.peek().map(|t| t.kind.clone()) {
                    Some(TokenKind::IntLiteral(s)) => {
                        let span = self.peek().unwrap().span();
                        self.advance();
                        let digits = s.replace('_', "");
                        let n = parse_int_text(&digits)
                            .ok_or_else(|| self.error("invalid integer literal"))?;
                        Ok(Pattern::Literal(crate::parser::ast::literal::Literal::Int(-n, span)))
                    }
                    _ => Err(self.error("expected integer literal after `-` in pattern")),
                }
            }
            // 字面量模式：复用原子表达式解析（含 bool 关键字 true/false）
            TokenKind::IntLiteral(_) | TokenKind::FloatLiteral(_)
            | TokenKind::CharLiteral(_) | TokenKind::StringLiteral(_)
            | TokenKind::Keyword(Keyword::True) | TokenKind::Keyword(Keyword::False) => {
                match self.parse_atom()? {
                    Expr::Literal(lit) => Ok(Pattern::Literal(lit)),
                    Expr::Suffixed { lit, .. } => Ok(Pattern::Literal(lit)),
                    _ => Err(self.error("invalid literal pattern")),
                }
            }
            // 标识符：绑定或枚举变体（`E::V(...)` / `V(...)`）
            TokenKind::Identifier(ref s) => {
                let mut name_str = s.clone();
                let mut name = Symbol::intern(&name_str);
                self.advance();
                self.handle_path_sep(&mut name_str, &mut name)?;
                if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::LParen)) {
                    self.advance();
                    let mut bindings = Vec::new();
                    if self.peek().map(|t| &t.kind) != Some(&TokenKind::Delimiter(Delimiter::RParen)) {
                        loop {
                            bindings.push(Symbol::intern(&self.expect_identifier()?));
                            if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RParen)) { break; }
                            self.expect_delimiter(Delimiter::Comma)?;
                        }
                    }
                    self.expect_delimiter(Delimiter::RParen)?;
                    Ok(Pattern::Enum { name, bindings })
                } else {
                    Ok(Pattern::Binding(name))
                }
            }
            _ => Err(self.error("expected pattern")),
        }
    }
}
