use super::*;

impl Parser {

    pub(super) fn parse_fn_decl(&mut self, vis: Visibility, is_inline: bool, extern_c: bool, attrs: Vec<crate::parser::ast::Attr>) -> Result<Stmt> {
        let start_span = self.peek().map(|t| t.span()).unwrap_or_default();
        self.advance();
        let name = self.expect_identifier()?;

        // Generic parameters: [T: Interface, U]
        let mut generic_params = Vec::new();
        if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::LBracket)) {
            self.advance();
            loop {
                let gp_name = Symbol::intern(&self.expect_identifier()?);
                let gp_constraint = if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::Colon)) {
                    self.advance();
                    Some(Symbol::intern(&ast_type_text(&self.parse_type()?)))
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

        // M2：extern "C"（含 #[export]）签名允许裸 `fn(...)` 类型；其余位置报错
        let raw_ok = extern_c || attrs.iter().any(|a| a.is_builtin() && a.name.as_str() == "export");
        let saved_raw = self.allow_raw_fn;
        self.allow_raw_fn = raw_ok;

        self.expect_delimiter(Delimiter::LParen)?;
        let mut params = Vec::new();
        let mut param_attrs: Vec<Vec<crate::parser::ast::Attr>> = Vec::new();
        if self.peek().map(|t| &t.kind) != Some(&TokenKind::Delimiter(Delimiter::RParen)) {
            loop {
                // A1b：形参前导标注（#[noalias]/#[nonnull]）
                param_attrs.push(self.parse_attr_list()?);
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

        let return_type = if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::Arrow))
        {
            self.advance();
            self.parse_type()?
        } else if name == "main" {
            Type::Int(Span::default())
        } else {
            Type::Void(Span::default())
        };

        // M2：签名解析结束，恢复裸 fn 限制（函数体内不允许 `fn(...)` 类型）
        self.allow_raw_fn = saved_raw;

        // Extern "C" declarations end with ; instead of a body
        let body = if extern_c && self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::Semicolon)) {
            self.advance();
            Block::new(Vec::new(), Span::default())
        } else {
            self.parse_block()?
        };

        Ok(Stmt::FnDecl {
            attrs,
            vis, is_inline, extern_c,
            generic_params,
            name: Symbol::intern(&name),
            params,
            param_attrs,
            return_type,
            body,
            span: start_span,
        })
    }

    pub(super) fn parse_return(&mut self) -> Result<Stmt> {
        let start_span = self.peek().map(|t| t.span()).unwrap_or_default();
        self.advance();
        let value = match self.peek().map(|t| &t.kind) {
            Some(TokenKind::Delimiter(Delimiter::Semicolon)) => { self.advance(); None }
            Some(TokenKind::Delimiter(Delimiter::RBrace | Delimiter::RParen | Delimiter::RBracket))
                | None => None,
            Some(kind) if Self::is_stmt_only_keyword(kind) => None,
            _ => Some(self.parse_expr()?),
        };
        if value.is_some() {
            self.try_semicolon()?;
        }
        Ok(Stmt::Return { value, span: start_span })
    }

    /// 条件表达式：禁止 `ident {` 被解析为结构体字面量
    pub(super) fn parse_cond_expr(&mut self) -> Result<Expr> {
        self.struct_lit_depth += 1;
        let r = self.parse_expr();
        self.struct_lit_depth -= 1;
        r
    }

    pub(super) fn parse_if(&mut self) -> Result<Stmt> {
        let (cond, then_block, elifs, else_block, span) = self.parse_if_parts()?;
        Ok(Stmt::If { cond, then_block, elifs, else_block, span })
    }

    /// bbp：if 表达式入口
    pub(super) fn parse_if_expr(&mut self) -> Result<Expr> {
        let (cond, then_block, elifs, else_block, span) = self.parse_if_parts()?;
        Ok(Expr::If { cond: Box::new(cond), then_block, elifs, else_block, span })
    }

    fn parse_if_parts(&mut self) -> Result<(Expr, Block, Vec<(Expr, Block)>, Option<Block>, Span)> {
        let start_span = self.peek().map(|t| t.span()).unwrap_or_default();
        self.advance();
        let cond = self.parse_cond_expr()?;
        let then_block = self.parse_block()?;
        let mut elifs = Vec::new();
        loop {
            match self.peek().map(|t| &t.kind) {
                Some(TokenKind::Keyword(Keyword::Elif)) => {
                    self.advance();
                    let elif_cond = self.parse_cond_expr()?;
                    let elif_block = self.parse_block()?;
                    elifs.push((elif_cond, elif_block));
                }
                _ => break,
            }
        }
        let else_block = if self.peek().map(|t| &t.kind) == Some(&TokenKind::Keyword(Keyword::Else)) {
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };
        Ok((cond, then_block, elifs, else_block, start_span))
    }

    pub(super) fn parse_while(&mut self) -> Result<Stmt> {
        let start_span = self.peek().map(|t| t.span()).unwrap_or_default();
        self.advance();
        let cond = self.parse_cond_expr()?;
        let body = self.parse_block()?;
        Ok(Stmt::While { cond, body, span: start_span })
    }

    // ==================== Match statement ====================

    pub(super) fn parse_match_stmt(&mut self) -> Result<Stmt> {
        match self.parse_match_expr()? {
            Expr::Match { value, arms, span } => Ok(Stmt::Match { value, arms, span }),
            _ => unreachable!(),
        }
    }

    /// `match` 作为表达式（`return match ...` 等位置）
    pub(super) fn parse_match_expr(&mut self) -> Result<Expr> {
        let start_span = self.peek().map(|t| t.span()).unwrap_or_default();
        self.advance(); // match
        self.struct_lit_depth += 1;
        let value = self.parse_expr();
        self.struct_lit_depth -= 1;
        let value = value?;
        self.expect_delimiter(Delimiter::LBrace)?;
        let mut arms = Vec::new();
        loop {
            if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RBrace)) { break; }
            let pattern = self.parse_pattern()?;
            // Phase 1.3：`pattern if guard => body`
            let guard = if self.peek().map(|t| &t.kind) == Some(&TokenKind::Keyword(Keyword::If)) {
                self.advance();
                Some(self.parse_expr()?)
            } else {
                None
            };
            self.expect_delimiter(Delimiter::FatArrow)?;
            let body = self.parse_match_arm_body()?;
            arms.push(crate::parser::ast::stmt::MatchArm { pattern, guard, body });
            if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::Comma)) {
                self.advance();
            }
        }
        self.expect_delimiter(Delimiter::RBrace)?;
        Ok(Expr::Match { value: Box::new(value), arms, span: start_span })
    }

    pub(super) fn parse_namespace(&mut self, vis: Visibility) -> Result<Stmt> {
        let start_span = self.peek().map(|t| t.span()).unwrap_or_default();
        self.advance();
        let name = self.expect_identifier()?;
        self.expect_delimiter(Delimiter::LBrace)?;
        let mut items = Vec::new();
        loop {
            match self.peek().map(|t| &t.kind) {
                Some(TokenKind::Delimiter(Delimiter::RBrace)) | None => break,
                _ => items.push(self.parse_stmt()?),
            }
        }
        self.expect_delimiter(Delimiter::RBrace)?;
        Ok(Stmt::Namespace {
            vis,
            name: Symbol::intern(&name),
            items,
            span: start_span,
        })
    }

    // ==================== Struct definition ====================

    pub(super) fn parse_struct_def(&mut self, vis: Visibility, attrs: Vec<crate::parser::ast::Attr>) -> Result<Stmt> {
        let start_span = self.peek().map(|t| t.span()).unwrap_or_default();
        self.advance(); // struct
        let name = Symbol::intern(&self.expect_identifier()?);
        let mut generic_params = Vec::new();
        if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::LBracket)) {
            self.advance();
            loop {
                let gp_name = Symbol::intern(&self.expect_identifier()?);
                let gp_constraint = if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::Colon)) {
                    self.advance();
                    Some(Symbol::intern(&ast_type_text(&self.parse_type()?)))
                } else {
                    None
                };
                generic_params.push((gp_name, gp_constraint));
                if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RBracket)) { break; }
                self.expect_delimiter(Delimiter::Comma)?;
            }
            self.expect_delimiter(Delimiter::RBracket)?;
        }
        self.expect_delimiter(Delimiter::LBrace)?;
        let mut fields = Vec::new();
        let mut field_attrs: Vec<Vec<crate::parser::ast::Attr>> = Vec::new();
        loop {
            match self.peek().map(|t| &t.kind) {
                Some(TokenKind::Delimiter(Delimiter::RBrace)) | None => break,
                _ => {
                    // A4a：字段级属性（#[follow_with(...)]）
                    field_attrs.push(self.parse_attr_list()?);
                    let field_type = self.parse_type()?;
                    let field_name = self.expect_identifier()?;
                    fields.push((Symbol::intern(&field_name), field_type));
                }
            }
        }
        self.expect_delimiter(Delimiter::RBrace)?;
        Ok(Stmt::StructDef { attrs, vis, name, generic_params, fields, field_attrs, span: start_span })
    }
}
