use super::*;

impl Parser {
    // ==================== Enum definition ====================

    pub(super) fn parse_enum_def(&mut self, vis: Visibility, attrs: Vec<crate::parser::ast::Attr>) -> Result<Stmt> {
        let start_span = self.peek().map(|t| t.span()).unwrap_or_default();
        self.advance(); // enum
        let name = Symbol::intern(&self.expect_identifier()?);

        let mut generic_params = Vec::new();
        if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::LBracket)) {
            self.advance();
            loop {
                let gp_name = Symbol::intern(&self.expect_identifier()?);
                let gp_constraint = if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::Colon)) {
                    self.advance();
                    Some(Symbol::intern(&self.expect_identifier()?))
                } else { None };
                generic_params.push((gp_name, gp_constraint));
                if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RBracket)) { break; }
                self.expect_delimiter(Delimiter::Comma)?;
            }
            self.expect_delimiter(Delimiter::RBracket)?;
        }

        self.expect_delimiter(Delimiter::LBrace)?;

        let mut variants = Vec::new();
        while self.peek().map(|t| &t.kind) != Some(&TokenKind::Delimiter(Delimiter::RBrace)) {
            if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::Comma)) {
                self.advance();
                continue;
            }
            let var_name = Symbol::intern(&self.expect_identifier()?);
            let fields = if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::LParen)) {
                self.advance();
                let mut tys = Vec::new();
                loop {
                    tys.push(self.parse_type()?);
                    if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RParen)) { break; }
                    self.expect_delimiter(Delimiter::Comma)?;
                }
                self.expect_delimiter(Delimiter::RParen)?;
                crate::parser::ast::stmt::EnumFields::Tuple(tys)
            } else if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::LBrace)) {
                self.advance();
                let mut named = Vec::new();
                loop {
                    if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RBrace)) { break; }
                    let fty = self.parse_type()?;
                    let fname = self.expect_identifier()?;
                    named.push((Symbol::intern(&fname), fty));
                    if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::Comma)) {
                        self.advance();
                    }
                }
                self.expect_delimiter(Delimiter::RBrace)?;
                crate::parser::ast::stmt::EnumFields::Named(named)
            } else {
                crate::parser::ast::stmt::EnumFields::None
            };
            variants.push(crate::parser::ast::stmt::EnumVariant { name: var_name, fields });
            if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::Comma)) {
                self.advance();
            }
        }
        self.expect_delimiter(Delimiter::RBrace)?;
        Ok(Stmt::EnumDef { attrs, vis, name, generic_params, variants, span: start_span })
    }

    // ==================== Interface definition ====================

    pub(super) fn parse_interface_def(&mut self, attrs: Vec<crate::parser::ast::Attr>) -> Result<Stmt> {
        let start_span = self.peek().map(|t| t.span()).unwrap_or_default();
        self.advance(); // interface
        let name = self.expect_identifier()?;

        // Generic parameters: [T, U: Constraint]
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

        self.expect_delimiter(Delimiter::LBrace)?;
        let mut methods = Vec::new();
        loop {
            match self.peek().map(|t| &t.kind) {
                Some(TokenKind::Delimiter(Delimiter::RBrace)) | None => break,
                _ => methods.push(self.parse_interface_method()?),
            }
        }
        self.expect_delimiter(Delimiter::RBrace)?;
        Ok(Stmt::InterfaceDef {
            attrs,
            name: Symbol::intern(&name),
            generic_params,
            methods,
            span: start_span,
        })
    }

    pub(super) fn parse_interface_method(&mut self) -> Result<InterfaceMethod> {
        let attrs = self.parse_attr_list()?;
        self.expect_keyword(Keyword::Fn)?;
        let name = self.expect_identifier()?;
        self.expect_delimiter(Delimiter::LParen)?;

        // Parse self parameter: ref/ref mut/unique/shared/self
        let self_keyword = match self.peek().map(|t| &t.kind) {
            Some(TokenKind::Keyword(Keyword::Unique)) => {
                return Err(self.error("`unique self` has been removed: use `self`, `ref self`, or `ref mut self`"));
            }
            Some(TokenKind::Keyword(Keyword::Ref)) => {
                self.advance();
                if self.peek().map(|t| &t.kind) == Some(&TokenKind::Keyword(Keyword::Mut)) {
                    self.advance();
                    Symbol::intern("refmut")
                } else {
                    Symbol::intern("ref")
                }
            }
            Some(TokenKind::Keyword(Keyword::Self_)) => Symbol::intern("self"),
            _ => return Err(self.error("expected 'ref', 'unique', 'shared' or 'self' for self parameter in interface method")),
        };
        let self_name = self.expect_identifier()?;
        if self_name != "self" {
            return Err(self.error("expected 'self' as first parameter name in interface method"));
        }

        // Optional additional params
        let mut params = Vec::new();
        if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::Comma)) {
            self.advance();
            loop {
                let pattrs = self.parse_attr_list()?;
                if !pattrs.is_empty() {
                    return Err(self.error("parameter attributes are not supported in interface methods"));
                }
                let ptype = self.parse_type()?;
                let pname = self.expect_identifier()?;
                params.push((Symbol::intern(&pname), ptype));
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

        self.try_semicolon()?;

        Ok(InterfaceMethod {
            attrs,
            name: Symbol::intern(&name),
            self_keyword,
            params,
            return_type,
        })
    }
}
