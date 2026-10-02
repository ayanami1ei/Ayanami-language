use super::*;

impl Parser {
    pub(super) fn parse_atom(&mut self) -> Result<Expr> {
        let tok = self.peek().ok_or_else(|| self.error("expected expression"))?.clone();
        let span = tok.span();
        match tok.kind {
            TokenKind::IntLiteral(s) => {
                self.advance();
                let n = s.parse::<i64>().map_err(|_| self.error("invalid integer literal"))?;
                Ok(Expr::Literal(Literal::Int(n, span)))
            }
            TokenKind::FloatLiteral(s) => {
                self.advance();
                let n = s.parse::<f64>().map_err(|_| self.error("invalid float literal"))?;
                Ok(Expr::Literal(Literal::Float(n, span)))
            }
            TokenKind::CharLiteral(s) => {
                self.advance();
                let c = s.chars().next().unwrap_or('\0');
                Ok(Expr::Literal(Literal::Char(c, span)))
            }
            TokenKind::StringLiteral(s) => {
                self.advance();
                Ok(Expr::Literal(Literal::String(s, span)))
            }
            TokenKind::Identifier(ref name) => {
                let tok = self.peek().cloned().unwrap();
                let span = tok.span();
                let mut name_str = name.clone();
                let mut name_sym = Symbol::intern(&name_str);
                self.advance();
                // Handle :: as path separator only if followed by another :: (namespace chain)
                // or if it's NOT followed by ( or { (which would be enum construct)
                self.handle_path_sep(&mut name_str, &mut name_sym)?;
                // Check for generic struct literal: Name[T] { field = val }
                // Only trigger if we can find ]{ ident = pattern
                let is_generic_struct = self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::LBracket))
                    && self.pos + 4 < self.tokens.len()
                    && self.tokens[self.pos + 1].kind != TokenKind::Delimiter(Delimiter::RBracket)
                    && self.tokens[self.pos + 2].kind == TokenKind::Delimiter(Delimiter::RBracket)
                    && self.tokens[self.pos + 3].kind == TokenKind::Delimiter(Delimiter::LBrace)
                    && matches!(&self.tokens[self.pos + 4].kind, TokenKind::Identifier(_) | TokenKind::Keyword(Keyword::Self_))
                    && self.pos + 5 < self.tokens.len()
                    && self.tokens[self.pos + 5].kind == TokenKind::Operator("=".to_string());
                if is_generic_struct {
                    self.advance(); // consume [
                    let mut generic_args = Vec::new();
                    loop {
                        generic_args.push(self.parse_type()?);
                        if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RBracket)) { break; }
                        self.expect_delimiter(Delimiter::Comma)?;
                    }
                    self.expect_delimiter(Delimiter::RBracket)?;
                    // Now check for { — struct literal
                    if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::LBrace)) {
                            let is_struct_lit = self.pos + 2 < self.tokens.len()
                                && matches!(&self.tokens[self.pos + 1].kind, TokenKind::Identifier(_) | TokenKind::Keyword(Keyword::Self_))
                                && self.tokens[self.pos + 2].kind == TokenKind::Operator("=".to_string());
                            if is_struct_lit {
                                self.advance(); // consume {
                                let mut fields = Vec::new();
                                if self.peek().map(|t| &t.kind) != Some(&TokenKind::Delimiter(Delimiter::RBrace)) {
                                    loop {
                                        let field_name = Symbol::intern(&self.expect_identifier()?);
                                        self.expect_operator("=")?;
                                        let field_val = self.parse_expr()?;
                                        fields.push((field_name, field_val));
                                        if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RBrace)) { break; }
                                        self.expect_delimiter(Delimiter::Comma)?;
                                    }
                                }
                                self.expect_delimiter(Delimiter::RBrace)?;
                                return Ok(Expr::StructLiteral { type_name: name_sym, generic_args, fields, span });
                            }
                    }
                }
                match self.peek().map(|t| &t.kind) {
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
                        Ok(Expr::FnCall { name: name_sym, args, span })
                    }
                    Some(TokenKind::Delimiter(Delimiter::LBrace)) => {
                        // Try to parse as struct literal: ident { field = val, ... }
                        // If the first field ends with , → definitely struct
                        // If it ends with } → one-field struct
                        // Otherwise → backtrack: return ident, leave { in stream
                        let saved_pos = self.pos;

                        // Peek: ident { ident = pattern
                        let looks_like_struct = self.pos + 2 < self.tokens.len()
                            && matches!(&self.tokens[self.pos + 1].kind, TokenKind::Identifier(_) | TokenKind::Keyword(Keyword::Self_))
                            && self.tokens[self.pos + 2].kind == TokenKind::Operator("=".to_string());
                        if !looks_like_struct {
                            return Ok(Expr::Ident(name_sym, span));
                        }

                        // Tentatively parse as struct literal: consume {, field, =, expr
                        self.advance(); // consume {
                        let field_name = Symbol::intern(&self.expect_identifier()?);
                        self.expect_operator("=")?;
                        let field_val = self.parse_expr()?;
                        let next = self.peek().map(|t| &t.kind);
                        match next {
                            Some(TokenKind::Delimiter(Delimiter::Comma)) => {
                                // Multi-field struct: parse remaining fields
                                let mut fields = vec![(field_name, field_val)];
                                loop {
                                    self.expect_delimiter(Delimiter::Comma)?;
                                    if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RBrace)) {
                                        break;
                                    }
                                    let fn2 = Symbol::intern(&self.expect_identifier()?);
                                    self.expect_operator("=")?;
                                    let fv2 = self.parse_expr()?;
                                    fields.push((fn2, fv2));
                                    if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RBrace)) {
                                        break;
                                    }
                                }
                                self.expect_delimiter(Delimiter::RBrace)?;
                                Ok(Expr::StructLiteral { type_name: name_sym, generic_args: Vec::new(), fields, span })
                            }
                            Some(TokenKind::Delimiter(Delimiter::RBrace)) => {
                                // Single-field struct literal: Foo { x = expr }
                                self.advance(); // consume }
                                Ok(Expr::StructLiteral { type_name: name_sym, generic_args: Vec::new(), fields: vec![(field_name, field_val)], span })
                            }
                            _ => {
                                // Ambiguous: might be block body { var = expr; ... }
                                // Restore position and return ident (let caller handle {)
                                self.pos = saved_pos;
                                Ok(Expr::Ident(name_sym, span))
                            }
                        }
                    }
                    _ => Ok(Expr::Ident(name_sym, span)),
                }
            }
            TokenKind::Keyword(Keyword::True) => {
                let tok = self.peek().cloned().unwrap();
                self.advance();
                Ok(Expr::Literal(Literal::Bool(true, tok.span())))
            }
            TokenKind::Keyword(Keyword::False) => {
                let tok = self.peek().cloned().unwrap();
                self.advance();
                Ok(Expr::Literal(Literal::Bool(false, tok.span())))
            }
            TokenKind::Keyword(Keyword::Null) => {
                let tok = self.peek().cloned().unwrap();
                self.advance();
                Ok(Expr::Null(tok.span()))
            }
            TokenKind::Keyword(Keyword::Self_) => {
                let tok = self.peek().cloned().unwrap();
                self.advance();
                Ok(Expr::Ident(Symbol::intern("self"), tok.span()))
            }
            TokenKind::Keyword(Keyword::Asm) => {
                let tok = self.peek().cloned().unwrap();
                let span = tok.span();
                self.advance();
                self.expect_delimiter(Delimiter::LParen)?;
                let template = match self.peek().map(|t| &t.kind) {
                    Some(TokenKind::StringLiteral(s)) => { let s = s.clone(); self.advance(); s }
                    _ => return Err(self.error("expected string literal in asm")),
                };
                let mut outputs = Vec::new();
                let mut inputs = Vec::new();
                loop {
                    match self.peek().map(|t| &t.kind) {
                        Some(TokenKind::Delimiter(Delimiter::RParen)) | None => break,
                        _ => {
                            self.expect_delimiter(Delimiter::Comma)?;
                            match self.peek().map(|t| &t.kind) {
                                Some(TokenKind::Keyword(Keyword::Out)) => {
                                    self.advance();
                                    self.expect_delimiter(Delimiter::LParen)?;
                                    let constraint = match self.peek().map(|t| &t.kind) {
                                        Some(TokenKind::Keyword(Keyword::Reg)) => {
                                            self.advance(); "r".to_string()
                                        }
                                        _ => return Err(self.error("expected reg in asm out")),
                                    };
                                    self.expect_delimiter(Delimiter::RParen)?;
                                    let output = self.parse_expr()?;
                                    outputs.push((constraint, Box::new(output)));
                                }
                                Some(TokenKind::Keyword(Keyword::In)) => {
                                    self.advance();
                                    self.expect_delimiter(Delimiter::LParen)?;
                                    let constraint = match self.peek().map(|t| &t.kind) {
                                        Some(TokenKind::Keyword(Keyword::Reg)) => {
                                            self.advance(); "r".to_string()
                                        }
                                        _ => return Err(self.error("expected reg in asm in")),
                                    };
                                    self.expect_delimiter(Delimiter::RParen)?;
                                    let input = self.parse_expr()?;
                                    inputs.push((constraint, Box::new(input)));
                                }
                                _ => break,
                            }
                        }
                    }
                }
                self.expect_delimiter(Delimiter::RParen)?;
                Ok(Expr::Asm { template, outputs, inputs, span })
            }
            TokenKind::Delimiter(Delimiter::LParen) => {
                // Check if this is a lambda: (type name, ...) -> ret_type { ... }
                let is_lambda = self.pos + 1 < self.tokens.len()
                    && self.is_type_start(self.pos + 1);
                if is_lambda {
                    return self.parse_lambda();
                }
                self.advance();
                let expr = self.parse_expr()?;
                self.expect_delimiter(Delimiter::RParen)?;
                Ok(expr)
            }
            TokenKind::Delimiter(Delimiter::LBracket) => {
                let tok = self.peek().cloned().unwrap();
                let bracket_span = tok.span();
                self.advance();
                // Check if this is a sized array: [type; count]
                // Peek: if next token is a type keyword or identifier, and the one after is ";"
                let is_sized = self.peek().map(|t| {
                    matches!(&t.kind,
                        TokenKind::Keyword(Keyword::Int | Keyword::Float | Keyword::Char | Keyword::Bool)
                        | TokenKind::Identifier(_)
                        | TokenKind::Keyword(Keyword::Shared | Keyword::Unique | Keyword::Weak)
                    )
                }).unwrap_or(false)
                && self.pos + 1 < self.tokens.len()
                && self.tokens[self.pos + 1].kind == TokenKind::Delimiter(Delimiter::Semicolon);

                if is_sized {
                    let elem_type = self.parse_type()?;
                    self.expect_delimiter(Delimiter::Semicolon)?;
                    let count = self.parse_expr()?;
                    self.expect_delimiter(Delimiter::RBracket)?;
                    Ok(Expr::ArraySized { elem_type, count: Box::new(count), span: bracket_span })
                } else {
                    let mut elems = Vec::new();
                    if self.peek().map(|t| &t.kind) != Some(&TokenKind::Delimiter(Delimiter::RBracket)) {
                        loop {
                            elems.push(self.parse_expr()?);
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
            _ => Err(self.error("expected expression")),
        }
    }
}
