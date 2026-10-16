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
        // A5a：可选短名列表 `{ macro1, macro2 }`
        let mut macros = Vec::new();
        if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::LBrace)) {
            self.advance();
            loop {
                let name = self.expect_identifier()?;
                macros.push(Symbol::intern(&name));
                match self.peek().map(|t| &t.kind) {
                    Some(TokenKind::Delimiter(Delimiter::RBrace)) => { self.advance(); break; }
                    Some(TokenKind::Delimiter(Delimiter::Comma)) => { self.advance(); }
                    _ => return Err(self.error("expected ',' or '}' in import list")),
                }
            }
        }
        self.try_semicolon()?;
        Ok(Stmt::Import { path, macros, span: start_span })
    }

    pub(super) fn parse_impl_block(&mut self, attrs: Vec<crate::parser::ast::Attr>) -> Result<Stmt> {
        let start_span = self.peek().map(|t| t.span()).unwrap_or_default();
        self.advance(); // impl
        // Parse optional generic params: [T, U: Interface]
        let generic_params = self.parse_generic_params()?;        // 解析类型名（支持 LinkedList[T] 泛型写法）
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
            attrs,
            type_name: type_sym,
            generic_params,
            methods,
            span: start_span,
        })
    }

    /// Parse a method inside an impl block.
    /// Converts `fn draw(shared self, ...)` into a regular FnDecl with
    /// the self parameter typed as `shared TypeName` (or `unique TypeName`).
    pub(super) fn parse_impl_method(&mut self, impl_type: &Symbol, impl_generic_params: &[(Symbol, Vec<Symbol>)]) -> Result<Stmt> {
        let attrs = self.parse_attr_list()?;
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
        let generic_params = self.parse_generic_params()?;
        self.expect_delimiter(Delimiter::LParen)?;

        // Parse optional self parameter: ref/ref mut/unique/shared/self
        let mut params: Vec<(Symbol, Type)> = Vec::new();
        let mut param_attrs: Vec<Vec<crate::parser::ast::Attr>> = Vec::new();
        // `Self` 在签名中代表 impl 目标类型（含泛型实参）：返回类型/其余形参统一替换
        let base_self_type = if impl_generic_params.is_empty() {
            Type::Named(*impl_type, Span::default())
        } else {
            let gp_names: Vec<Type> = impl_generic_params.iter()
                .map(|(n, _)| Type::Named(*n, Span::default()))
                .collect();
            Type::Generic(*impl_type, gp_names, Span::default())
        };
        let is_self_start = matches!(self.peek().map(|t| &t.kind),
            Some(TokenKind::Keyword(Keyword::Unique))
                | Some(TokenKind::Keyword(Keyword::Ref))
                | Some(TokenKind::Keyword(Keyword::Self_)));
        if is_self_start {
            let mut ref_mut = false;
            let self_keyword = match self.peek().map(|t| &t.kind) {
                Some(TokenKind::Keyword(Keyword::Unique)) => {
                    return Err(self.error("`unique self` has been removed: use `self`, `ref self`, or `ref mut self`"));
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
            let self_type = match self_keyword.as_str().as_str() {
                "unique" => Type::Unique(Box::new(base_self_type.clone()), Span::default()),
                "ref" => Type::Ref(Box::new(base_self_type.clone()), ref_mut, Span::default()),
                _ => base_self_type.clone(), // 裸 self：消费
            };
            params.push((Symbol::intern("self"), self_type));
            param_attrs.push(Vec::new());
            // Parse remaining params
            if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::Comma)) {
                self.advance();
                loop {
                    param_attrs.push(self.parse_attr_list()?);
                    let ptype = self_type::subst_self_in_type(&self.parse_type()?, &base_self_type);
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
                param_attrs.push(self.parse_attr_list()?);
                let ptype = self_type::subst_self_in_type(&self.parse_type()?, &base_self_type);
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
            self_type::subst_self_in_type(&self.parse_type()?, &base_self_type)
        } else {
            Type::Void(Span::default())
        };

        let body = self.parse_block()?;

        Ok(Stmt::FnDecl {
            attrs,
            vis: Visibility::Pub,
            is_inline: false,
            extern_c: false,
            is_unsafe: false,
            generic_params,
            name,
            params,
            param_attrs,
            return_type,
            body,
            span: start_span,
        })
    }


    pub(super) fn parse_block(&mut self) -> Result<Block> {
        let start_span = self.peek().map(|t| t.span()).unwrap_or_default();
        self.expect_delimiter(Delimiter::LBrace)?;
        let mut stmts = Vec::new();
        let mut tail: Option<Box<Expr>> = None;
        loop {
            match self.peek().map(|t| &t.kind) {
                Some(TokenKind::Delimiter(Delimiter::RBrace)) | None => break,
                _ => {
                    let stmt = self.parse_stmt()?;
                    // 裸尾表达式：语句后无分号且紧邻 `}`
                    let had_semi = self.pos > 0
                        && self.tokens[self.pos - 1].kind == TokenKind::Delimiter(Delimiter::Semicolon);
                    let at_end = matches!(self.peek().map(|t| &t.kind),
                        Some(TokenKind::Delimiter(Delimiter::RBrace)) | None);
                    if at_end && !had_semi {
                        match stmt {
                            Stmt::ExprStmt { expr, .. } => { tail = Some(Box::new(expr)); break; }
                            Stmt::Match { value, arms, span } => {
                                tail = Some(Box::new(Expr::Match { value, arms, span }));
                                break;
                            }
                            // bbp：带 else 且至少一个分支含尾值的 if 才作为块尾表达式
                            //（否则保持语句形态，避免 void 函数里的 if/else 被当值处理）
                            Stmt::If { cond, then_block, elifs, else_block: Some(eb), span }
                                if then_block.tail.is_some()
                                    || elifs.iter().any(|(_, b)| b.tail.is_some())
                                    || eb.tail.is_some() => {
                                tail = Some(Box::new(Expr::If {
                                    cond: Box::new(cond),
                                    then_block,
                                    elifs,
                                    else_block: Some(eb),
                                    span,
                                }));
                                break;
                            }
                            other => stmts.push(other),
                        }
                    } else {
                        stmts.push(stmt);
                    }
                }
            }
        }
        self.expect_delimiter(Delimiter::RBrace)?;
        Ok(Block { stmts, tail, span: start_span })
    }
}
