//! 内联汇编表达式解析（`asm("template", in(reg) x, out(reg) y, ...)`）；
//! 从 atom.rs 拆出，保持文件 ≤300 行。
use super::*;

impl Parser {
    pub(super) fn parse_asm(&mut self) -> Result<Expr> {
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
                                    self.advance(); "=r".to_string()
                                }
                                // #198：early-clobber（`=&r`）——输出在读完全部输入前写入时必需
                                Some(TokenKind::Identifier(id)) if id == "early" => {
                                    self.advance();
                                    match self.peek().map(|t| &t.kind) {
                                        Some(TokenKind::Keyword(Keyword::Reg)) => {
                                            self.advance(); "=&r".to_string()
                                        }
                                        _ => return Err(self.error("expected reg after early in asm out")),
                                    }
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
}
