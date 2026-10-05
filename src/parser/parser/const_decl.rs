use super::*;

impl Parser {
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
