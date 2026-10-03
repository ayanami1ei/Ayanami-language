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

    pub(super) fn parse_if(&mut self) -> Result<Stmt> {
        let start_span = self.peek().map(|t| t.span()).unwrap_or_default();
        self.advance();
        let cond = self.parse_expr()?;
        let then_block = self.parse_block()?;
        let mut elifs = Vec::new();
        loop {
            match self.peek().map(|t| &t.kind) {
                Some(TokenKind::Keyword(Keyword::Elif)) => {
                    self.advance();
                    let elif_cond = self.parse_expr()?;
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
        Ok(Stmt::If {
            cond,
            then_block,
            elifs,
            else_block,
            span: start_span,
        })
    }

    pub(super) fn parse_for(&mut self) -> Result<Stmt> {
        let start_span = self.peek().map(|t| t.span()).unwrap_or_default();
        self.advance();
        let iter_name = self.expect_identifier()?;
        self.expect_keyword(Keyword::In)?;
        self.expect_delimiter(Delimiter::LParen)?;
        let start = self.parse_expr()?;
        self.expect_delimiter(Delimiter::Comma)?;
        let end = self.parse_expr()?;
        let step = if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::Comma)) {
            self.advance();
            Some(self.parse_expr()?)
        } else {
            None
        };
        self.expect_delimiter(Delimiter::RParen)?;
        let body = self.parse_block()?;
        Ok(Stmt::For {
            iterator: Symbol::intern(&iter_name),
            start,
            end,
            step,
            body,
            span: start_span,
        })
    }

    pub(super) fn parse_while(&mut self) -> Result<Stmt> {
        let start_span = self.peek().map(|t| t.span()).unwrap_or_default();
        self.advance();
        let cond = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(Stmt::While { cond, body, span: start_span })
    }

    // ==================== Match statement ====================

    pub(super) fn parse_match_stmt(&mut self) -> Result<Stmt> {
        let start_span = self.peek().map(|t| t.span()).unwrap_or_default();
        self.advance(); // match
        let value = self.parse_expr()?;
        self.expect_delimiter(Delimiter::LBrace)?;
        let mut arms = Vec::new();
        loop {
            if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RBrace)) { break; }
            let variant_name = Symbol::intern(&self.expect_identifier()?);
            let mut bindings = Vec::new();
            if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::LParen)) {
                self.advance();
                loop {
                    let binding_name = self.expect_identifier()?;
                    bindings.push((Symbol::intern(&binding_name), None));
                    if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RParen)) { break; }
                    self.expect_delimiter(Delimiter::Comma)?;
                }
                self.expect_delimiter(Delimiter::RParen)?;
            }
            self.expect_delimiter(Delimiter::FatArrow)?;
            let body = self.parse_expr()?;
            arms.push(crate::parser::ast::stmt::MatchArm { variant_name, bindings, body });
            if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::Comma)) {
                self.advance();
            }
        }
        self.expect_delimiter(Delimiter::RBrace)?;
        Ok(Stmt::Match { value: Box::new(value), arms, span: start_span })
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
                    Some(Symbol::intern(&self.expect_identifier()?))
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
