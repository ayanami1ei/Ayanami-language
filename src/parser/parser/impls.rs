use super::*;

impl Parser {
    // ==================== Impl block ====================

    pub(super) fn parse_import(&mut self) -> Result<Stmt> {
        let start_span = self.peek().map(|t| t.span()).unwrap_or_default();
        self.advance(); // import
        let path = match self.peek().map(|t| &t.kind) {
            Some(TokenKind::StringLiteral(s)) => {
                let s = s.clone();
                self.advance();
                s
            }
            _ => return Err(self.error("expected package path string after `import`")),
        };
        self.try_semicolon()?;
        Ok(Stmt::Import { path, span: start_span })
    }

    pub(super) fn parse_impl_block(&mut self) -> Result<Stmt> {
        let start_span = self.peek().map(|t| t.span()).unwrap_or_default();
        self.advance(); // impl
        // Parse optional generic params: [T, U: Interface]
        let mut generic_params = Vec::new();
        if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::LBracket)) {
            self.advance();
            loop {
                let gp_name = Symbol::intern(&self.expect_identifier()?);
                let gp_constraint = if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::Colon)) {
                    self.advance();
                    Some(Symbol::intern(&self.expect_identifier()?))
                } else {
                    None
                };
                generic_params.push((gp_name, gp_constraint));
                if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RBracket)) {
                    break;
                }
                self.expect_delimiter(Delimiter::Comma)?;
            }
            self.expect_delimiter(Delimiter::RBracket)?;
        }
        // 解析类型名（支持 LinkedList[T] 泛型写法）
        let impl_type = self.parse_type()?;
        // 从解析出的类型中提取类型名
        fn extract_type_name(ty: &Type) -> Symbol {
            match ty {
                Type::Named(name, _) => *name,
                Type::Generic(name, _, _) => *name,
                Type::Unique(inner, _) => {
                    extract_type_name(inner)
                }
                Type::Int(_) => Symbol::intern("int"),
                Type::Float(_) => Symbol::intern("float"),
                Type::Char(_) => Symbol::intern("char"),
                Type::Bool(_) => Symbol::intern("bool"),
                Type::Void(_) => Symbol::intern("void"),
                _ => Symbol::intern(""),
            }
        }
        let type_sym = extract_type_name(&impl_type);
        self.expect_delimiter(Delimiter::LBrace)?;
        let mut methods = Vec::new();
        loop {
            match self.peek().map(|t| &t.kind) {
                Some(TokenKind::Delimiter(Delimiter::RBrace)) | None => break,
                _ => methods.push(self.parse_impl_method(&type_sym, &generic_params)?),
            }
        }
        self.expect_delimiter(Delimiter::RBrace)?;
        Ok(Stmt::ImplBlock {
            type_name: type_sym,
            generic_params,
            methods,
            span: start_span,
        })
    }

    /// Parse a method inside an impl block.
    /// Converts `fn draw(shared self, ...)` into a regular FnDecl with
    /// the self parameter typed as `shared TypeName` (or `unique TypeName`).
    pub(super) fn parse_impl_method(&mut self, impl_type: &Symbol, impl_generic_params: &[(Symbol, Option<Symbol>)]) -> Result<Stmt> {
        // Optional pub keyword
        if self.peek().map(|t| &t.kind) == Some(&TokenKind::Keyword(Keyword::Pub)) {
            self.advance();
        }
        // Optional pub(crate)
        if self.peek().map(|t| &t.kind) == Some(&TokenKind::Keyword(Keyword::Pub)) {
            self.advance();
            if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::LParen)) {
                self.advance();
                if self.peek().map(|t| &t.kind) == Some(&TokenKind::Keyword(Keyword::Crate)) {
                    self.advance();
                    if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RParen)) {
                        self.advance();
                    }
                }
            }
        }
        let start_span = self.peek().map(|t| t.span()).unwrap_or_default();
        self.expect_keyword(Keyword::Fn)?;
        let name = Symbol::intern(&self.expect_identifier()?);

        // Generic parameters: [T: Interface, U]
        let mut generic_params = Vec::new();
        if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::LBracket)) {
            self.advance();
            loop {
                let gp_name = Symbol::intern(&self.expect_identifier()?);
                let gp_constraint = if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::Colon)) {
                    self.advance();
                    Some(Symbol::intern(&self.expect_identifier()?))
                } else {
                    None
                };
                generic_params.push((gp_name, gp_constraint));
                if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RBracket)) {
                    break;
                }
                self.expect_delimiter(Delimiter::Comma)?;
            }
            self.expect_delimiter(Delimiter::RBracket)?;
        }

        self.expect_delimiter(Delimiter::LParen)?;

        // Parse optional self parameter: ref/ref mut/unique/shared/self
        let mut params: Vec<(Symbol, Type)> = Vec::new();
        let is_self_start = matches!(self.peek().map(|t| &t.kind),
            Some(TokenKind::Keyword(Keyword::Unique))
                | Some(TokenKind::Keyword(Keyword::Ref))
                | Some(TokenKind::Keyword(Keyword::Self_)));
        if is_self_start {
            let mut ref_mut = false;
            let self_keyword = match self.peek().map(|t| &t.kind) {
                Some(TokenKind::Keyword(Keyword::Unique)) => {
                    self.advance();
                    Symbol::intern("unique")
                }
                Some(TokenKind::Keyword(Keyword::Ref)) => {
                    self.advance();
                    if self.peek().map(|t| &t.kind) == Some(&TokenKind::Keyword(Keyword::Mut)) {
                        self.advance();
                        ref_mut = true;
                    }
                    Symbol::intern("ref")
                }
                _ => Symbol::intern("self"),
            };
            let self_name = self.expect_identifier()?;
            if self_name != "self" {
                return Err(self.error("expected 'self' as first parameter name in method"));
            }
            let base_self_type = if impl_generic_params.is_empty() {
                Type::Named(*impl_type, Span::default())
            } else {
                let gp_names: Vec<Type> = impl_generic_params.iter()
                    .map(|(n, _)| Type::Named(*n, Span::default()))
                    .collect();
                Type::Generic(*impl_type, gp_names, Span::default())
            };
            let self_type = match self_keyword.as_str().as_str() {
                "unique" => Type::Unique(Box::new(base_self_type), Span::default()),
                "ref" => Type::Ref(Box::new(base_self_type), ref_mut, Span::default()),
                _ => base_self_type, // 裸 self：消费
            };
            params.push((Symbol::intern("self"), self_type));
            // Parse remaining params
            if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::Comma)) {
                self.advance();
                loop {
                    let ptype = self.parse_type()?;
                    let pname = self.expect_identifier()?;
                    params.push((Symbol::intern(&pname), ptype));
                    if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RParen)) {
                        break;
                    }
                    self.expect_delimiter(Delimiter::Comma)?;
                }
            }
        } else {
            // No self parameter — parse regular params
            loop {
                if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RParen)) {
                    break;
                }
                let ptype = self.parse_type()?;
                let pname = self.expect_identifier()?;
                params.push((Symbol::intern(&pname), ptype));
                if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::Comma)) {
                    self.advance();
                }
            }
        }

        self.expect_delimiter(Delimiter::RParen)?;

        let return_type = if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::Arrow)) {
            self.advance();
            self.parse_type()?
        } else {
            Type::Void(Span::default())
        };

        let body = self.parse_block()?;

        Ok(Stmt::FnDecl {
            vis: Visibility::Pub,
            is_inline: false,
            extern_c: false,
            generic_params,
            name,
            params,
            return_type,
            body,
            span: start_span,
        })
    }


    pub(super) fn parse_block(&mut self) -> Result<Block> {
        let start_span = self.peek().map(|t| t.span()).unwrap_or_default();
        self.expect_delimiter(Delimiter::LBrace)?;
        let mut stmts = Vec::new();
        loop {
            match self.peek().map(|t| &t.kind) {
                Some(TokenKind::Delimiter(Delimiter::RBrace)) | None => break,
                _ => stmts.push(self.parse_stmt()?),
            }
        }
        self.expect_delimiter(Delimiter::RBrace)?;
        Ok(Block::new(stmts, start_span))
    }
}
