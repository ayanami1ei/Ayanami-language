use super::*;

impl Parser {
    // ==================== Statements ====================

    pub(super) fn parse_stmt(&mut self) -> Result<Stmt> {
        let attrs = self.parse_attr_list()?;
        let vis = self.parse_visibility();
        let is_inline = self.peek().map(|t| &t.kind) == Some(&TokenKind::Keyword(Keyword::Inline));
        if is_inline { self.advance(); }
        let extern_c = self.peek().map(|t| &t.kind) == Some(&TokenKind::Keyword(Keyword::Extern));
        if extern_c {
            self.advance();
            // Expect "C" string literal
            match self.peek().map(|t| &t.kind) {
                Some(TokenKind::StringLiteral(s)) if s == "C" => { self.advance(); }
                _ => return Err(self.error("expected \"C\" after extern")),
            }
        }
        // extern "C" { ... } block
        if extern_c && self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::LBrace)) {
            self.advance();
            let mut items = Vec::new();
            loop {
                match self.peek().map(|t| &t.kind) {
                    Some(TokenKind::Delimiter(Delimiter::RBrace)) | None => break,
                    _ => {
                        // Expect fn declarations inside extern block
                        let attrs2 = self.parse_attr_list()?;
                        let vis2 = self.parse_visibility();
                        let is_inline2 = false;
                        let fn_stmt = self.parse_fn_decl(vis2, is_inline2, true, attrs2)?;
                        items.push(fn_stmt);
                    }
                }
            }
            self.expect_delimiter(Delimiter::RBrace)?;
            // Flatten extern block items — they're regular function declarations
            // For now, just return the first one (wrap in a block if multiple)
            // Actually, return items one by one — this is a limitation
            return if items.is_empty() {
                Err(self.error("empty extern block"))
            } else if items.len() == 1 {
                Ok(items.into_iter().next().unwrap())
            } else {
                // For multiple declarations, we can only return one; this is a simplification
                Ok(items.into_iter().next().unwrap())
            };
        }
        let tok = self.peek().ok_or_else(|| self.error("expected statement"))?.clone();
        // A2f：语句级标注（#[cfg]/#[invariant]）包装非声明语句
        let stmt = match tok.kind {
            TokenKind::Keyword(Keyword::Const) => {
                // M6.3：`const fn`（编译期可求值，运行期普通调用）vs `const NAME = ...`
                if self.tokens.get(self.pos + 1).map(|t| &t.kind)
                    == Some(&TokenKind::Keyword(Keyword::Fn))
                {
                    self.advance(); // const
                    let mut stmt = self.parse_fn_decl(vis, false, false, attrs)?;
                    if let Stmt::FnDecl { is_const, .. } = &mut stmt { *is_const = true; }
                    return Ok(stmt);
                }
                return self.parse_const_decl(vis, attrs);
            }
            TokenKind::Keyword(Keyword::Static) => return self.parse_static_decl(vis, attrs),
            TokenKind::Keyword(Keyword::Fn) => return self.parse_fn_decl(vis, is_inline, extern_c, attrs),
            TokenKind::Keyword(Keyword::Struct) => return self.parse_struct_def(vis, attrs),
            TokenKind::Keyword(Keyword::Enum) => return self.parse_enum_def(vis, attrs),
            TokenKind::Keyword(Keyword::Interface) => return self.parse_interface_def(attrs),
            TokenKind::Keyword(Keyword::Impl) => return self.parse_impl_block(attrs),
            TokenKind::Keyword(Keyword::Namespace) => {
                if !attrs.is_empty() { return Err(self.error("attributes are not allowed on namespace")); }
                return self.parse_namespace(vis);
            }
            TokenKind::Keyword(Keyword::Import) => {
                if !attrs.is_empty() { return Err(self.error("attributes are not allowed on import")); }
                return self.parse_import();
            }
            TokenKind::Keyword(Keyword::Return) => self.parse_return()?,
            TokenKind::Keyword(Keyword::If) => self.parse_if()?,
            TokenKind::Keyword(Keyword::For) => self.parse_for()?,
            TokenKind::Keyword(Keyword::While) => self.parse_while()?,
            TokenKind::Keyword(Keyword::Break) => {
                self.advance();
                self.try_semicolon()?;
                Stmt::Break { span: tok.span() }
            }
            TokenKind::Keyword(Keyword::Continue) => {
                self.advance();
                self.try_semicolon()?;
                Stmt::Continue { span: tok.span() }
            }
            TokenKind::Keyword(Keyword::Match) => self.parse_match_stmt()?,
            _ => self.parse_any_assign_or_expr()?,
        };
        if attrs.is_empty() {
            Ok(stmt)
        } else {
            let span = stmt.span();
            Ok(Stmt::Attributed { attrs, stmt: Box::new(stmt), span })
        }
    }

    /// Parse an assignment (with optional mut) or expression statement.
    /// Handles: mut v = expr, v = expr, expr.field = expr, expr[i] = expr, expr;
    pub(super) fn parse_any_assign_or_expr(&mut self) -> Result<Stmt> {
        let is_mut = self.peek().map(|t| &t.kind) == Some(&TokenKind::Keyword(Keyword::Mut));
        if is_mut { self.advance(); }

        let expr = self.parse_expr()?;

        if let Some(TokenKind::Operator(s)) = self.peek().map(|t| &t.kind) {
            if s == "=" {
                self.advance();
                let value = self.parse_expr()?;
                self.try_semicolon()?;
                return match expr {
                    Expr::Ident(name, id_span) => Ok(Stmt::Assign { name, is_mut, value, span: id_span }),
                    Expr::FieldAccess { object, field, span: fa_span } =>
                        Ok(Stmt::FieldAssign { object, field, value, span: fa_span }),
                    Expr::Index { object, index, span: ix_span } =>
                        Ok(Stmt::IndexAssign { object, index, value, span: ix_span }),
                    _ => Err(self.error("invalid assignment target")),
                };
            }
        }
        self.try_semicolon()?;
        let expr_span = expr.span();
        Ok(Stmt::ExprStmt { expr, span: expr_span })
    }

    pub(super) fn is_type_start(&self, pos: usize) -> bool {
        self.tokens.get(pos).map(|t| matches!(&t.kind,
            TokenKind::Keyword(Keyword::Int | Keyword::Float | Keyword::Char | Keyword::Bool
                | Keyword::Ref | Keyword::Mut | Keyword::Fn)
            | TokenKind::Identifier(_)
        )).unwrap_or(false)
    }

    pub(super) fn parse_lambda(&mut self) -> Result<Expr> {
        let start_span = self.peek().map(|t| t.span()).unwrap_or_default();
        // Already at LParen from caller
        self.advance(); // consume (
        let mut params = Vec::new();
        if self.peek().map(|t| &t.kind) != Some(&TokenKind::Delimiter(Delimiter::RParen)) {
            loop {
                let pattrs = self.parse_attr_list()?;
                if !pattrs.is_empty() {
                    return Err(self.error("parameter attributes are not supported in lambdas"));
                }
                let param_type = self.parse_type()?;
                let param_name = self.expect_identifier()?;
                params.push((Symbol::intern(&param_name), param_type));
                if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RParen)) {
                    break;
                }
                self.expect_delimiter(Delimiter::Comma)?;
            }
        }
        self.expect_delimiter(Delimiter::RParen)?;

        let return_type = if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::Arrow)) {
            self.advance();
            self.parse_type()?
        } else {
            Type::Void(Span::default())
        };

        let mut body = self.parse_block()?;
        // 裸尾表达式在 lambda 中暂按语句求值（lambda 值语义后续支持）
        if let Some(t) = body.tail.take() {
            let tspan = t.span();
            body.stmts.push(Stmt::ExprStmt { expr: *t, span: tspan });
        }
        let span = start_span.merge(body.span);
        Ok(Expr::Lambda {
            params,
            return_type,
            body: body.stmts,
            span,
        })
    }
}
