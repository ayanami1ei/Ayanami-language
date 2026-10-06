use super::*;

impl Parser {
    /// M6.2：`static [mut] NAME [: T] = expr`（顶层全局；常量初始化，可寻址）
    pub(super) fn parse_static_decl(
        &mut self,
        vis: Visibility,
        attrs: Vec<crate::parser::ast::Attr>,
    ) -> Result<Stmt> {
        let start_span = self.peek().map(|t| t.span()).unwrap_or_default();
        self.advance(); // static
        let is_mut = self.peek().map(|t| &t.kind) == Some(&TokenKind::Keyword(Keyword::Mut));
        if is_mut { self.advance(); }
        let name = Symbol::intern(&self.expect_identifier()?);
        let ty = if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::Colon)) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        match self.peek().map(|t| &t.kind) {
            Some(TokenKind::Operator(s)) if s == "=" => { self.advance(); }
            _ => return Err(self.error("expected '=' in static declaration")),
        }
        let value = Box::new(self.parse_expr()?);
        self.try_semicolon()?;
        Ok(Stmt::StaticDecl { attrs, vis, is_mut, name, ty, value, span: start_span })
    }

    /// M6.1：`const NAME [: T] = expr`（顶层常量；行尾 `;` 可选）
    pub(super) fn parse_const_decl(
        &mut self,
        vis: Visibility,
        attrs: Vec<crate::parser::ast::Attr>,
    ) -> Result<Stmt> {
        let start_span = self.peek().map(|t| t.span()).unwrap_or_default();
        self.advance(); // const
        let name = Symbol::intern(&self.expect_identifier()?);
        let ty = if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::Colon)) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        match self.peek().map(|t| &t.kind) {
            Some(TokenKind::Operator(s)) if s == "=" => {
                self.advance();
            }
            _ => return Err(self.error("expected '=' in const declaration")),
        }
        let value = Box::new(self.parse_expr()?);
        self.try_semicolon()?;
        Ok(Stmt::ConstDecl {
            attrs,
            vis,
            name,
            ty,
            value,
            span: start_span,
        })
    }
}
