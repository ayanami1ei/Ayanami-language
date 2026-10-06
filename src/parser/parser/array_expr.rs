use super::*;

impl Parser {
    /// M1.2/M6.2c：`[...]` 表达式 —— `[a, b]` 字面量 / `[T; n]` 未初始化定长数组 / `[v; n]` 重复字面量
    pub(super) fn parse_array_expr(&mut self, bracket_span: Span) -> Result<Expr> {
        // Check if this is a sized array: [type; count]
        // Peek: if next token is a type keyword or identifier, and the one after is ";"
        let is_sized = self.peek().map(|t| {
            matches!(&t.kind,
                TokenKind::Keyword(Keyword::Int | Keyword::Float | Keyword::Char | Keyword::Bool)
                | TokenKind::Identifier(_)
                | TokenKind::Keyword(Keyword::Unique)
            )
        }).unwrap_or(false)
        && self.pos + 1 < self.tokens.len()
        && self.tokens[self.pos + 1].kind == TokenKind::Delimiter(Delimiter::Semicolon);

        if is_sized {
            let elem_type = self.parse_type()?;
            self.expect_delimiter(Delimiter::Semicolon)?;
            let count = self.parse_expr()?;
            self.expect_delimiter(Delimiter::RBracket)?;
            return Ok(Expr::ArraySized { elem_type, count: Box::new(count), span: bracket_span });
        }
        // M6.2c：`[value; count]` 重复字面量（首元素为表达式）
        let mut elems = Vec::new();
        if self.peek().map(|t| &t.kind) != Some(&TokenKind::Delimiter(Delimiter::RBracket)) {
            loop {
                elems.push(self.parse_expr()?);
                if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::Semicolon)) {
                    self.advance();
                    let count = self.parse_expr()?;
                    self.expect_delimiter(Delimiter::RBracket)?;
                    if elems.len() != 1 {
                        return Err(self.error("repeat literal takes exactly one value: [value; count]"));
                    }
                    return Ok(Expr::ArrayRepeat {
                        value: Box::new(elems.pop().unwrap()),
                        count: Box::new(count),
                        span: bracket_span,
                    });
                }
                if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RBracket)) {
                    break;
                }
                self.expect_delimiter(Delimiter::Comma)?;
            }
        }
        self.expect_delimiter(Delimiter::RBracket)?;
        Ok(Expr::ArrayLiteral(elems, bracket_span))
    }
}
