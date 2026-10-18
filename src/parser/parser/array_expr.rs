use super::*;

impl Parser {
    /// M1.2/M6.2c：`[...]` 表达式 —— `[a, b]` 字面量 / `[T; n]` 未初始化定长数组 / `[v; n]` 重复字面量
    pub(super) fn parse_array_expr(&mut self, bracket_span: Span) -> Result<Expr> {
        // `[T; n]` 未初始化定长数组：先按「类型 + `;`」试解析（支持 `Ptr[T]` 等复合元素类型；
        // 失败/不匹配则回退表达式路径）。歧义消解：关键字类型或「大写开头标识符」才尝试
        // 类型路径；小写标识符仅当其紧跟 `;`（旧行为）视为类型——避免 `[a[b]; n]` 误判。
        let type_like = match self.peek().map(|t| &t.kind) {
            Some(TokenKind::Keyword(Keyword::Int | Keyword::Float | Keyword::Char | Keyword::Bool
                | Keyword::Unique)) => true,
            Some(TokenKind::Identifier(id)) => {
                let upper = id.as_str().chars().next().map(|c| c.is_ascii_uppercase()).unwrap_or(false);
                upper || self.tokens.get(self.pos + 1)
                    .map(|t| t.kind == TokenKind::Delimiter(Delimiter::Semicolon))
                    .unwrap_or(false)
            }
            _ => false,
        };
        if type_like {
            let saved = self.pos;
            if let Ok(elem_type) = self.parse_type() {
                if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::Semicolon)) {
                    self.advance();
                    let count = self.parse_expr()?;
                    self.expect_delimiter(Delimiter::RBracket)?;
                    return Ok(Expr::ArraySized { elem_type, count: Box::new(count), span: bracket_span });
                }
            }
            self.pos = saved;
        }
        // M6.2c：`[value; count]` 重复字面量（首元素为表达式）
        let mut elems = Vec::new();
        if self.peek().map(|t| &t.kind) != Some(&TokenKind::Delimiter(Delimiter::RBracket)) {
            loop {
                elems.push(self.parse_expr()?);
                if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::Semicolon)) {
                    self.advance();
                    let count = self.parse_expr()?;
                    self.expect_delimiter(Delimiter::RBracket)?;
                    if elems.len() != 1 {
                        return Err(self.error("repeat literal takes exactly one value: [value; count]"));
                    }
                    return Ok(Expr::ArrayRepeat {
                        value: Box::new(elems.pop().unwrap()),
                        count: Box::new(count),
                        span: bracket_span,
                    });
                }
                if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RBracket)) {
                    break;
                }
                self.expect_delimiter(Delimiter::Comma)?;
            }
        }
        self.expect_delimiter(Delimiter::RBracket)?;
        Ok(Expr::ArrayLiteral(elems, bracket_span))
    }
}
