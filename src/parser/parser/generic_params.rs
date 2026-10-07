//! 泛型参数表解析：`[T, U: A + B]`（每参数可多个约束）。
use super::*;

impl Parser {
    /// 解析可选的泛型参数表；无 `[` 返回空表。
    pub(super) fn parse_generic_params(&mut self) -> Result<Vec<(Symbol, Vec<Symbol>)>> {
        let mut params = Vec::new();
        if self.peek().map(|t| &t.kind) != Some(&TokenKind::Delimiter(Delimiter::LBracket)) {
            return Ok(params);
        }
        self.advance();
        loop {
            let name = Symbol::intern(&self.expect_identifier()?);
            let mut constraints = Vec::new();
            if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::Colon)) {
                self.advance();
                loop {
                    constraints.push(Symbol::intern(&ast_type_text(&self.parse_type()?)));
                    // 多约束：`A + B + C`
                    if matches!(self.peek().map(|t| &t.kind), Some(TokenKind::Operator(op)) if op == "+") {
                        self.advance();
                        continue;
                    }
                    break;
                }
            }
            params.push((name, constraints));
            if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RBracket)) {
                break;
            }
            self.expect_delimiter(Delimiter::Comma)?;
        }
        self.expect_delimiter(Delimiter::RBracket)?;
        Ok(params)
    }
}
