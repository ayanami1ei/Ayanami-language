use super::*;

impl Parser {
    // ==================== Types ====================

    pub(super) fn parse_type(&mut self) -> Result<Type> {
        let tok = self.peek().ok_or_else(|| self.error("expected type"))?.clone();
        let span = tok.span();
        match tok.kind {
            TokenKind::Keyword(Keyword::Unique) => {
                Err(self.error("`unique` has been removed: ownership is the default; write `[T]`/`[T; n]` for owned arrays"))
            }
            TokenKind::Keyword(Keyword::Ref) => {
                self.advance();
                let mutable = self.peek().map(|t| &t.kind) == Some(&TokenKind::Keyword(Keyword::Mut));
                if mutable { self.advance(); }
                let inner = self.parse_base_type()?;
                Ok(Type::Ref(Box::new(inner), mutable, span))
            }
            _ => self.parse_base_type(),
        }
    }

    pub(super) fn parse_base_type(&mut self) -> Result<Type> {
        let tok = self.peek().ok_or_else(|| self.error("expected type"))?.clone();
        let span = tok.span();
        match tok.kind {
            TokenKind::Delimiter(Delimiter::LBracket) => {
                self.advance();
                let inner = self.parse_type()?;
                self.expect_delimiter(Delimiter::RBracket)?;
                Ok(Type::Array(Box::new(inner), span))
            }
            TokenKind::Keyword(Keyword::Int) => {
                self.advance();
                Ok(Type::Int(span))
            }
            TokenKind::Keyword(Keyword::Float) => {
                self.advance();
                Ok(Type::Float(span))
            }
            TokenKind::Keyword(Keyword::Char) => {
                self.advance();
                Ok(Type::Char(span))
            }
            TokenKind::Keyword(Keyword::Bool) => {
                self.advance();
                Ok(Type::Bool(span))
            }
            TokenKind::Keyword(Keyword::Self_) => {
                self.advance();
                Ok(Type::Self_(span))
            }
            TokenKind::Keyword(Keyword::Fn) => {
                self.advance();
                self.expect_delimiter(Delimiter::LParen)?;
                let mut params = Vec::new();
                if self.peek().map(|t| &t.kind) != Some(&TokenKind::Delimiter(Delimiter::RParen)) {
                    loop {
                        params.push(self.parse_type()?);
                        if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RParen)) { break; }
                        self.expect_delimiter(Delimiter::Comma)?;
                    }
                }
                self.expect_delimiter(Delimiter::RParen)?;
                let ret = if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::Arrow)) {
                    self.advance();
                    self.parse_type()?
                } else {
                    Type::Void(span)
                };
                Ok(Type::FnPtr(params, Box::new(ret), span))
            }
            TokenKind::Identifier(s) => {
                let name = Symbol::intern(&s);
                self.advance();
                // Check for generic type instantiation: Foo[int]
                if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::LBracket)) {
                    self.advance();
                    let mut args = Vec::new();
                    loop {
                        args.push(self.parse_type()?);
                        if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RBracket)) { break; }
                        self.expect_delimiter(Delimiter::Comma)?;
                    }
                    self.expect_delimiter(Delimiter::RBracket)?;
                    Ok(Type::Generic(name, args, span))
                } else {
                    Ok(Type::Named(name, span))
                }
            }
            _ => Err(self.error("expected type")),
        }
    }

    /// Handle :: as path separator: consume :: pairs that form namespace paths.
    /// Stops when :: is followed by ( or { (enum construct).
    pub(super) fn handle_path_sep(&mut self, name_str: &mut String, name_sym: &mut Symbol) -> Result<()> {
        loop {
            match self.peek().map(|t| &t.kind) {
                Some(TokenKind::Operator(s)) if s == "::" => {
                    self.advance();
                    let next = self.expect_identifier()?;
                    *name_str = format!("{}.{}", name_str, next);
                    *name_sym = Symbol::intern(&name_str);
                }
                _ => break,
            }
        }
        Ok(())
    }
}
