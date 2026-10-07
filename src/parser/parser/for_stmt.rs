use super::*;

impl Parser {
    pub(super) fn parse_for(&mut self) -> Result<Stmt> {
        let start_span = self.peek().map(|t| t.span()).unwrap_or_default();
        self.advance();
        let iter_name = self.expect_identifier()?;
        self.expect_keyword(Keyword::In)?;
        // M3：`(start, end[, step])` 区间形式 或 普通表达式（迭代器协议）
        if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::LParen)) {
            self.advance();
            // 括号内允许结构体字面量（iterable/区间边界）
            let saved_depth = self.struct_lit_depth;
            self.struct_lit_depth = 0;
            let first = self.parse_expr();
            self.struct_lit_depth = saved_depth;
            let first = first?;
            if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::Comma)) {
                self.advance();
                let end = self.parse_expr()?;
                let step = if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::Comma)) {
                    self.advance();
                    Some(self.parse_expr()?)
                } else {
                    None
                };
                self.expect_delimiter(Delimiter::RParen)?;
                let body = self.parse_block()?;
                return Ok(Stmt::For {
                    iterator: Symbol::intern(&iter_name),
                    start: first,
                    end,
                    step,
                    body,
                    span: start_span,
                });
            }
            self.expect_delimiter(Delimiter::RParen)?;
            let body = self.parse_block()?;
            return Ok(Stmt::ForIn {
                iterator: Symbol::intern(&iter_name),
                iterable: first,
                body,
                span: start_span,
            });
        }
        // 与 if 条件一致：禁止 `ident {` 被当作结构体字面量（`{` 即循环体）；
        // 结构体字面量作 iterable 时用括号：`for x in (S { ... }) { ... }`
        let iterable = self.parse_cond_expr()?;
        let body = self.parse_block()?;
        Ok(Stmt::ForIn {
            iterator: Symbol::intern(&iter_name),
            iterable,
            body,
            span: start_span,
        })
    }

}
