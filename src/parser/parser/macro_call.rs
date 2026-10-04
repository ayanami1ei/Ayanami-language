use super::*;

impl Parser {
    /// A5c-2：函数宏调用 `#name(args)`，name 支持 `pkg::name`（解析后以 `.` 连接）
    pub(super) fn parse_macro_call(&mut self) -> Result<Expr> {
        let start_span = self.peek().map(|t| t.span()).unwrap_or_default();
        self.advance(); // #
        let mut name = self.expect_identifier()?;
        loop {
            let is_sep = matches!(self.peek().map(|t| &t.kind), Some(TokenKind::Operator(s)) if s == "::");
            if !is_sep { break; }
            self.advance();
            let seg = self.expect_identifier()?;
            name.push('.');
            name.push_str(&seg);
        }
        self.expect_delimiter(Delimiter::LParen)?;
        let mut args = Vec::new();
        if self.peek().map(|t| &t.kind) != Some(&TokenKind::Delimiter(Delimiter::RParen)) {
            loop {
                args.push(self.parse_expr()?);
                if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RParen)) { break; }
                self.expect_delimiter(Delimiter::Comma)?;
            }
        }
        self.expect_delimiter(Delimiter::RParen)?;
        Ok(Expr::MacroCall { name: Symbol::intern(&name), args, span: start_span })
    }
}
