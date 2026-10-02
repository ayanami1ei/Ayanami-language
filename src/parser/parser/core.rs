use super::*;

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub(super) fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    pub(super) fn advance(&mut self) -> Option<Token> {
        let tok = self.tokens.get(self.pos)?.clone();
        self.pos += 1;
        Some(tok)
    }

    pub(super) fn error(&self, msg: &str) -> Error {
        if let Some(tok) = self.peek() {
            Error::Parse(format!("{} (at {}:{})", msg, tok.line, tok.col))
        } else {
            Error::Parse(format!("{} (at end of file)", msg))
        }
    }

    pub(super) fn expect_keyword(&mut self, kw: Keyword) -> Result<()> {
        match self.peek() {
            Some(tok) if tok.kind == TokenKind::Keyword(kw) => {
                self.advance();
                Ok(())
            }
            Some(tok) => Err(self.error(&format!("expected keyword `{}`, found `{}`", kw, tok.kind))),
            None => Err(self.error(&format!("expected keyword `{}`, found EOF", kw))),
        }
    }

    pub(super) fn expect_delimiter(&mut self, d: Delimiter) -> Result<()> {
        match self.peek() {
            Some(tok) if tok.kind == TokenKind::Delimiter(d) => {
                self.advance();
                Ok(())
            }
            Some(tok) => Err(self.error(&format!("expected `{}`, found `{}`", d, tok.kind))),
            None => Err(self.error(&format!("expected `{}`, found EOF", d))),
        }
    }

    pub(super) fn expect_operator(&mut self, op: &str) -> Result<()> {
        match self.peek() {
            Some(tok) if tok.kind == TokenKind::Operator(op.to_string()) => {
                self.advance();
                Ok(())
            }
            Some(tok) => Err(self.error(&format!("expected `{}`, found `{}`", op, tok.kind))),
            None => Err(self.error(&format!("expected `{}`, found EOF", op))),
        }
    }

    /// Tokens that can never be the start of an expression.
    pub(super) fn is_stmt_only_keyword(kind: &TokenKind) -> bool {
        matches!(kind,
            TokenKind::Keyword(Keyword::Fn | Keyword::Return | Keyword::If | Keyword::For
                | Keyword::While | Keyword::Break | Keyword::Continue | Keyword::Match
                | Keyword::Interface | Keyword::Struct | Keyword::Impl
                | Keyword::Import | Keyword::Namespace | Keyword::Pub | Keyword::Inline
                | Keyword::Extern | Keyword::Mut | Keyword::Asm)
        )
    }

    /// Tokens that can start a new statement (including expression statements).
    pub(super) fn is_stmt_start(kind: &TokenKind) -> bool {
        Self::is_stmt_only_keyword(kind)
            || matches!(kind,
                TokenKind::Keyword(Keyword::True | Keyword::False | Keyword::Null | Keyword::Self_)
                | TokenKind::Identifier(_) | TokenKind::IntLiteral(_) | TokenKind::FloatLiteral(_)
                | TokenKind::StringLiteral(_) | TokenKind::CharLiteral(_)
                | TokenKind::Delimiter(Delimiter::LBrace | Delimiter::LParen | Delimiter::LBracket)
                | TokenKind::Operator(_)
            )
    }

    /// Consume `;` if present; otherwise, succeed if the next token
    /// starts a new statement or ends the current scope.
    pub(super) fn try_semicolon(&mut self) -> Result<()> {
        match self.peek().map(|t| &t.kind) {
            Some(TokenKind::Delimiter(Delimiter::Semicolon)) => { self.advance(); Ok(()) }
            Some(kind) if Self::is_stmt_start(kind) || matches!(kind,
                TokenKind::Delimiter(Delimiter::RBrace | Delimiter::RParen | Delimiter::RBracket)
            ) => Ok(()),
            None => Ok(()),
            Some(other) => Err(self.error(&format!("expected `;` or new statement, found `{}`", other))),
        }
    }

    pub(super) fn parse_visibility(&mut self) -> Visibility {
        match self.peek().map(|t| &t.kind) {
            Some(TokenKind::Keyword(Keyword::Pub)) => {
                self.advance();
                if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::LParen)) {
                    self.advance();
                    if self.peek().map(|t| &t.kind) == Some(&TokenKind::Keyword(Keyword::Crate)) {
                        self.advance();
                        if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RParen)) {
                            self.advance();
                            return Visibility::PubCrate;
                        }
                    }
                    // malformed pub(...), ignore
                }
                Visibility::Pub
            }
            _ => Visibility::Private,
        }
    }

    pub(super) fn expect_identifier(&mut self) -> Result<String> {
        match self.peek() {
            Some(tok) if matches!(&tok.kind, TokenKind::Identifier(_) | TokenKind::Keyword(Keyword::Self_)) => {
                let kind = tok.kind.clone();
                self.advance();
                match kind {
                    TokenKind::Identifier(s) => Ok(s),
                    TokenKind::Keyword(Keyword::Self_) => Ok("self".to_string()),
                    _ => unreachable!(),
                }
            }
            Some(tok) => Err(self.error(&format!("expected identifier, found `{}`", tok.kind))),
            None => Err(self.error("expected identifier, found EOF")),
        }
    }

    // ==================== Entry point ====================

    pub fn parse_program(&mut self) -> Result<Program> {
        let mut stmts = Vec::new();
        while self.pos < self.tokens.len() {
            stmts.push(self.parse_stmt()?);
        }
        Ok(Program::new(stmts))
    }
    /// 解析前导标注：`#[name]` 或 `#[name(arg, ...)]`
    pub(super) fn parse_attr_list(&mut self) -> Result<Vec<crate::parser::ast::Attr>> {
        let mut attrs = Vec::new();
        loop {
            let is_hash = matches!(
                self.peek().map(|t| &t.kind),
                Some(TokenKind::Operator(s)) if s == "#"
            );
            if !is_hash { break; }
            let span = self.peek().map(|t| t.span()).unwrap_or_default();
            self.advance(); // '#'
            self.expect_delimiter(Delimiter::LBracket)?;
            // 属性名允许关键字（如 inline）
            let name_str = match self.peek().map(|t| t.kind.clone()) {
                Some(TokenKind::Identifier(s)) => { self.advance(); s }
                Some(TokenKind::Keyword(k)) => { self.advance(); k.to_string() }
                _ => return Err(self.error("expected attribute name")),
            };
            let name = Symbol::intern(&name_str);
            let mut args = Vec::new();
            if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::LParen)) {
                self.advance();
                loop {
                    args.push(self.parse_attr_arg()?);
                    if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RParen)) {
                        break;
                    }
                    self.expect_delimiter(Delimiter::Comma)?;
                }
                self.expect_delimiter(Delimiter::RParen)?;
            }
            self.expect_delimiter(Delimiter::RBracket)?;
            attrs.push(crate::parser::ast::Attr { name, args, span });
        }
        Ok(attrs)
    }

    /// A2a：解析单个标注实参（`key = value` 或表达式）。
    fn parse_attr_arg(&mut self) -> Result<crate::parser::ast::AttrArg> {
        use crate::parser::ast::AttrArg;
        let is_kv = matches!(self.peek().map(|t| &t.kind), Some(TokenKind::Identifier(_)))
            && matches!(self.tokens.get(self.pos + 1).map(|t| &t.kind),
                        Some(TokenKind::Operator(s)) if s == "=");
        if is_kv {
            let key = match self.peek().map(|t| t.kind.clone()) {
                Some(TokenKind::Identifier(s)) => { self.advance(); Symbol::intern(&s) }
                _ => return Err(self.error("expected attribute key")),
            };
            self.advance(); // '='
            let val = self.parse_attr_arg()?;
            return Ok(AttrArg::KeyValue(key, Box::new(val)));
        }
        Ok(AttrArg::Expr(Box::new(self.parse_expr()?)))
    }
}
