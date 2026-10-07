use super::*;

use super::literal_text::parse_int_text;
use crate::parser::ast::pattern::Pattern;

impl Parser {
    /// Phase 1.3：模式 —— `_` / 字面量 / 区间 / 绑定 / 枚举变体 / 结构体解构 / 或模式
    pub(super) fn parse_pattern(&mut self) -> Result<Pattern> {
        let mut pats = vec![self.parse_pattern_atom()?];
        while matches!(self.peek().map(|t| &t.kind), Some(TokenKind::Operator(op)) if op == "|") {
            self.advance();
            pats.push(self.parse_pattern_atom()?);
        }
        Ok(if pats.len() == 1 { pats.pop().unwrap() } else { Pattern::Or(pats) })
    }

    /// 字面量（含 `-int`）→ Literal
    fn parse_pattern_literal(&mut self) -> Result<crate::parser::ast::literal::Literal> {
        match self.peek().map(|t| t.kind.clone()) {
            Some(TokenKind::Operator(ref op)) if op == "-" => {
                self.advance();
                match self.peek().map(|t| t.kind.clone()) {
                    Some(TokenKind::IntLiteral(s)) => {
                        let span = self.peek().unwrap().span();
                        self.advance();
                        let n = parse_int_text(&s.replace('_', ""))
                            .ok_or_else(|| self.error("invalid integer literal"))?;
                        Ok(crate::parser::ast::literal::Literal::Int(-n, span))
                    }
                    _ => Err(self.error("expected integer literal after `-` in pattern")),
                }
            }
            _ => match self.parse_atom()? {
                Expr::Literal(lit) => Ok(lit),
                Expr::Suffixed { lit, .. } => Ok(lit),
                _ => Err(self.error("invalid literal pattern")),
            },
        }
    }

    fn parse_pattern_atom(&mut self) -> Result<Pattern> {
        let tok = self.peek().ok_or_else(|| self.error("expected pattern"))?.clone();
        // 字面量 / 区间：`0`、`-1`、`'a'`、`1..10`、`1..=10`
        let is_literal = matches!(tok.kind,
            TokenKind::IntLiteral(_) | TokenKind::FloatLiteral(_)
            | TokenKind::CharLiteral(_) | TokenKind::StringLiteral(_)
            | TokenKind::Keyword(Keyword::True) | TokenKind::Keyword(Keyword::False))
            || matches!(&tok.kind, TokenKind::Operator(op) if op == "-");
        if is_literal {
            let lo = self.parse_pattern_literal()?;
            let range_op = match self.peek().map(|t| &t.kind) {
                Some(TokenKind::Operator(op)) if op == ".." => Some(false),
                Some(TokenKind::Operator(op)) if op == "..=" => Some(true),
                _ => None,
            };
            if let Some(inclusive) = range_op {
                self.advance();
                let hi = self.parse_pattern_literal()?;
                return Ok(Pattern::Range { lo, hi, inclusive });
            }
            return Ok(Pattern::Literal(lo));
        }
        match tok.kind {
            // `_` 通配
            TokenKind::Identifier(ref s) if s == "_" => {
                self.advance();
                Ok(Pattern::Wildcard)
            }
            // 标识符：绑定 / 枚举变体（可嵌套）/ 结构体解构
            TokenKind::Identifier(ref s) => {
                let mut name_str = s.clone();
                let mut name = Symbol::intern(&name_str);
                self.advance();
                self.handle_path_sep(&mut name_str, &mut name)?;
                if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::LParen)) {
                    self.advance();
                    let mut args = Vec::new();
                    if self.peek().map(|t| &t.kind) != Some(&TokenKind::Delimiter(Delimiter::RParen)) {
                        loop {
                            args.push(self.parse_pattern()?);
                            if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RParen)) { break; }
                            self.expect_delimiter(Delimiter::Comma)?;
                        }
                    }
                    self.expect_delimiter(Delimiter::RParen)?;
                    Ok(Pattern::Enum { name, args })
                } else if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::LBrace)) {
                    // 结构体解构：`Point { x, y }` / `Point { x = pat }` / `Point { x: pat }`
                    self.advance();
                    let mut fields = Vec::new();
                    if self.peek().map(|t| &t.kind) != Some(&TokenKind::Delimiter(Delimiter::RBrace)) {
                        loop {
                            let fname = Symbol::intern(&self.expect_identifier()?);
                            let sub = match self.peek().map(|t| &t.kind) {
                                Some(TokenKind::Operator(op)) if op == "=" => {
                                    self.advance();
                                    self.parse_pattern()?
                                }
                                Some(TokenKind::Delimiter(Delimiter::Colon)) => {
                                    self.advance();
                                    self.parse_pattern()?
                                }
                                _ => Pattern::Binding(fname),
                            };
                            fields.push((fname, sub));
                            if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RBrace)) { break; }
                            self.expect_delimiter(Delimiter::Comma)?;
                        }
                    }
                    self.expect_delimiter(Delimiter::RBrace)?;
                    Ok(Pattern::Struct { name, fields })
                } else {
                    Ok(Pattern::Binding(name))
                }
            }
            _ => Err(self.error("expected pattern")),
        }
    }
}
