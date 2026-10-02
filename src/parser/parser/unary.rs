use super::*;

impl Parser {
    pub(super) fn parse_unary(&mut self) -> Result<Expr> {
        let tok = self.peek().ok_or_else(|| self.error("expected expression"))?.clone();
        let span = tok.span();
        match &tok.kind {
            TokenKind::Operator(s) if s == "-" => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Unary {
                    op: UnaryOp::Neg,
                    arg: Box::new(expr),
                    span,
                })
            }
            TokenKind::Operator(s) if s == "!" => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Unary {
                    op: UnaryOp::Not,
                    arg: Box::new(expr),
                    span,
                })
            }
            TokenKind::Keyword(Keyword::Move) => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Move(Box::new(expr), span))
            }
            TokenKind::Keyword(Keyword::Clone) => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Clone(Box::new(expr), span))
            }
            TokenKind::Keyword(Keyword::Unique) => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::ToUnique(Box::new(expr), span))
            }
            TokenKind::Keyword(Keyword::Ref) => {
                self.advance();
                let mutable = self.peek().map(|t| t.kind == TokenKind::Keyword(Keyword::Mut)).unwrap_or(false);
                if mutable { self.advance(); }
                let expr = self.parse_unary()?;
                Ok(Expr::Ref(Box::new(expr), mutable, span))
            }
            _ => self.parse_postfix(),
        }
    }

    /// Parse postfix operations: function calls, indexing, method calls, field access.
    pub(super) fn parse_postfix(&mut self) -> Result<Expr> {
        let mut expr = self.parse_atom()?;
        loop {
            let tok = self.peek().cloned();
            let span = tok.as_ref().map(|t| t.span()).unwrap_or_default();
            match tok.as_ref().map(|t| &t.kind) {
                // Function call: expr(args) — currently only used for Ident(args)
                // which is handled inside parse_atom. This branch handles cases
                // like (expr)(args) for parenthesized expressions.
                Some(TokenKind::Delimiter(Delimiter::LParen)) => {
                    self.advance();
                    let mut args = Vec::new();
                    if self.peek().map(|t| &t.kind) != Some(&TokenKind::Delimiter(Delimiter::RParen)) {
                        loop {
                            args.push(self.parse_expr()?);
                            if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RParen)) {
                                break;
                            }
                            self.expect_delimiter(Delimiter::Comma)?;
                        }
                    }
                    self.expect_delimiter(Delimiter::RParen)?;
                    // If expr is an Ident, convert to FnCall
                    if let Expr::Ident(name, _) = &expr {
                        let name = *name;
                        expr = Expr::FnCall { name, args, span };
                    } else {
                        expr = Expr::CallExpr { target: Box::new(expr), args, span };
                    }
                }
                // Indexing: expr[index]
                Some(TokenKind::Delimiter(Delimiter::LBracket)) => {
                    self.advance();
                    let index = self.parse_expr()?;
                    self.expect_delimiter(Delimiter::RBracket)?;
                    expr = Expr::Index {
                        object: Box::new(expr), index: Box::new(index), span
                    };
                }
                // Try operator: expr?
                Some(TokenKind::Operator(s)) if s == "?" => {
                    self.advance();
                    expr = Expr::TryOp(Box::new(expr), span);
                }
                // Enum construction: EnumType::Variant(args)
                Some(TokenKind::Operator(s)) if s == "::" => {
                    self.advance();
                    let variant_name = Symbol::intern(&self.expect_identifier()?);
                    if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::LParen)) {
                        self.advance();
                        let mut args = Vec::new();
                        if self.peek().map(|t| &t.kind) != Some(&TokenKind::Delimiter(Delimiter::RParen)) {
                            loop {
                                args.push(self.parse_expr()?);
                                if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RParen)) { break; }
                                self.expect_delimiter(Delimiter::Comma)?;
                            }
                        }
                        self.expect_delimiter(Delimiter::RParen)?;
                        let enum_name = match &expr {
                            Expr::Ident(n, _) => *n,
                            _ => return Err(self.error("expected enum type name before ::")),
                        };
                        expr = Expr::EnumConstruct {
                            enum_name,
                            variant_name,
                            tuple_args: args,
                            named_args: vec![],
                            span,
                        };
                    } else if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::LBrace)) {
                        self.advance();
                        let mut fields = Vec::new();
                        loop {
                            if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RBrace)) { break; }
                            let fexpr = self.parse_expr()?;
                            let fname = self.expect_identifier()?;
                            fields.push((Symbol::intern(&fname), fexpr));
                            if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::Comma)) {
                                self.advance();
                            }
                        }
                        self.expect_delimiter(Delimiter::RBrace)?;
                        let enum_name = match &expr {
                            Expr::Ident(n, _) => *n,
                            _ => return Err(self.error("expected enum type name before ::")),
                        };
                        expr = Expr::EnumConstruct {
                            enum_name,
                            variant_name,
                            tuple_args: vec![],
                            named_args: fields,
                            span,
                        };
                    } else {
                        return Err(self.error("expected '(' or '{' after enum variant name"));
                    }
                }
                // Method call: expr.method(args) or field access: expr.field
                Some(TokenKind::Delimiter(Delimiter::Dot)) => {
                    self.advance();
                    let name = self.expect_identifier()?;
                    if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::LParen)) {
                        self.advance();
                        let mut args = Vec::new();
                        loop {
                            if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RParen)) { break; }
                            args.push(self.parse_expr()?);
                            if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RParen)) { break; }
                            self.expect_delimiter(Delimiter::Comma)?;
                        }
                        self.expect_delimiter(Delimiter::RParen)?;
                        expr = Expr::MethodCall {
                            object: Box::new(expr), method: Symbol::intern(&name), args, span
                        };
                    } else {
                        expr = Expr::FieldAccess {
                            object: Box::new(expr), field: Symbol::intern(&name), span
                        };
                    }
                }
                _ => break,
            }
        }
        Ok(expr)
    }
}
