use crate::lexer::{Delimiter, Keyword, Token, TokenKind};
use crate::intern::Symbol;
use crate::parser::ast::{BinaryOp, Block, Expr, InterfaceMethod, Literal, Program, Stmt, Type, UnaryOp};
use crate::parser::ast::vis::Visibility;
use crate::span::Span;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<Token> {
        let tok = self.tokens.get(self.pos)?.clone();
        self.pos += 1;
        Some(tok)
    }

    fn error(&self, msg: &str) -> String {
        if let Some(tok) = self.peek() {
            format!("{} (at {}:{})", msg, tok.line, tok.col)
        } else {
            format!("{} (at end of file)", msg)
        }
    }

    fn expect_keyword(&mut self, kw: Keyword) -> Result<(), String> {
        match self.peek() {
            Some(tok) if tok.kind == TokenKind::Keyword(kw) => {
                self.advance();
                Ok(())
            }
            Some(tok) => Err(self.error(&format!("expected keyword `{}`, found `{}`", kw, tok.kind))),
            None => Err(self.error(&format!("expected keyword `{}`, found EOF", kw))),
        }
    }

    fn expect_delimiter(&mut self, d: Delimiter) -> Result<(), String> {
        match self.peek() {
            Some(tok) if tok.kind == TokenKind::Delimiter(d) => {
                self.advance();
                Ok(())
            }
            Some(tok) => Err(self.error(&format!("expected `{}`, found `{}`", d, tok.kind))),
            None => Err(self.error(&format!("expected `{}`, found EOF", d))),
        }
    }

    fn expect_operator(&mut self, op: &str) -> Result<(), String> {
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
    fn is_stmt_only_keyword(kind: &TokenKind) -> bool {
        matches!(kind,
            TokenKind::Keyword(Keyword::Fn | Keyword::Return | Keyword::If | Keyword::For
                | Keyword::While | Keyword::Interface | Keyword::Struct | Keyword::Impl
                | Keyword::Import | Keyword::Namespace | Keyword::Pub | Keyword::Inline
                | Keyword::Extern | Keyword::Mut | Keyword::Asm)
        )
    }

    /// Tokens that can start a new statement (including expression statements).
    fn is_stmt_start(kind: &TokenKind) -> bool {
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
    fn try_semicolon(&mut self) -> Result<(), String> {
        match self.peek().map(|t| &t.kind) {
            Some(TokenKind::Delimiter(Delimiter::Semicolon)) => { self.advance(); Ok(()) }
            Some(kind) if Self::is_stmt_start(kind) || matches!(kind,
                TokenKind::Delimiter(Delimiter::RBrace | Delimiter::RParen | Delimiter::RBracket)
            ) => Ok(()),
            None => Ok(()),
            Some(other) => Err(self.error(&format!("expected `;` or new statement, found `{}`", other))),
        }
    }

    fn parse_visibility(&mut self) -> Visibility {
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

    fn expect_identifier(&mut self) -> Result<String, String> {
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

    pub fn parse_program(&mut self) -> Result<Program, String> {
        let mut stmts = Vec::new();
        while self.pos < self.tokens.len() {
            stmts.push(self.parse_stmt()?);
        }
        Ok(Program::new(stmts))
    }

    // ==================== Statements ====================

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
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
                        let vis2 = self.parse_visibility();
                        let is_inline2 = false;
                        let fn_stmt = self.parse_fn_decl(vis2, is_inline2, true)?;
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
        match tok.kind {
            TokenKind::Keyword(Keyword::Fn) => self.parse_fn_decl(vis, is_inline, extern_c),
            TokenKind::Keyword(Keyword::Return) => self.parse_return(),
            TokenKind::Keyword(Keyword::If) => self.parse_if(),
            TokenKind::Keyword(Keyword::For) => self.parse_for(),
            TokenKind::Keyword(Keyword::While) => self.parse_while(),
            TokenKind::Keyword(Keyword::Namespace) => self.parse_namespace(vis),
            TokenKind::Keyword(Keyword::Struct) => self.parse_struct_def(vis),
            TokenKind::Keyword(Keyword::Interface) => self.parse_interface_def(),
            TokenKind::Keyword(Keyword::Impl) => self.parse_impl_block(),
            TokenKind::Keyword(Keyword::Import) => self.parse_import(),
            _ => self.parse_any_assign_or_expr(),
        }
    }

    /// Parse an assignment (with optional mut) or expression statement.
    /// Handles: mut v = expr, v = expr, expr.field = expr, expr[i] = expr, expr;
    fn parse_any_assign_or_expr(&mut self) -> Result<Stmt, String> {
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

    fn parse_fn_decl(&mut self, vis: Visibility, is_inline: bool, extern_c: bool) -> Result<Stmt, String> {
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
        if self.peek().map(|t| &t.kind) != Some(&TokenKind::Delimiter(Delimiter::RParen)) {
            loop {
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
            vis, is_inline, extern_c,
            generic_params,
            name: Symbol::intern(&name),
            params,
            return_type,
            body,
            span: start_span,
        })
    }

    fn parse_return(&mut self) -> Result<Stmt, String> {
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

    fn parse_if(&mut self) -> Result<Stmt, String> {
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

    fn parse_for(&mut self) -> Result<Stmt, String> {
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

    fn parse_while(&mut self) -> Result<Stmt, String> {
        let start_span = self.peek().map(|t| t.span()).unwrap_or_default();
        self.advance();
        let cond = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(Stmt::While { cond, body, span: start_span })
    }

    fn parse_namespace(&mut self, vis: Visibility) -> Result<Stmt, String> {
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

    fn parse_struct_def(&mut self, vis: Visibility) -> Result<Stmt, String> {
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
        loop {
            match self.peek().map(|t| &t.kind) {
                Some(TokenKind::Delimiter(Delimiter::RBrace)) | None => break,
                _ => {
                    let field_type = self.parse_type()?;
                    let field_name = self.expect_identifier()?;
                    fields.push((Symbol::intern(&field_name), field_type));
                }
            }
        }
        self.expect_delimiter(Delimiter::RBrace)?;
        Ok(Stmt::StructDef { vis, name, generic_params, fields, span: start_span })
    }

    // ==================== Interface definition ====================

    fn parse_interface_def(&mut self) -> Result<Stmt, String> {
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
            name: Symbol::intern(&name),
            generic_params,
            methods,
            span: start_span,
        })
    }

    fn parse_interface_method(&mut self) -> Result<InterfaceMethod, String> {
        self.expect_keyword(Keyword::Fn)?;
        let name = self.expect_identifier()?;
        self.expect_delimiter(Delimiter::LParen)?;

        // Parse self parameter: shared self or unique self
        let self_keyword = match self.peek().map(|t| &t.kind) {
            Some(TokenKind::Keyword(Keyword::Shared)) => {
                self.advance();
                Symbol::intern("shared")
            }
            Some(TokenKind::Keyword(Keyword::Unique)) => {
                self.advance();
                Symbol::intern("unique")
            }
            _ => return Err(self.error("expected 'shared' or 'unique' for self parameter in interface method")),
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
            name: Symbol::intern(&name),
            self_keyword,
            params,
            return_type,
        })
    }

    // ==================== Impl block ====================

    fn parse_import(&mut self) -> Result<Stmt, String> {
        let start_span = self.peek().map(|t| t.span()).unwrap_or_default();
        self.advance(); // import
        let path = match self.peek().map(|t| &t.kind) {
            Some(TokenKind::StringLiteral(s)) => {
                let s = s.clone();
                self.advance();
                s
            }
            _ => return Err(self.error("expected package path string after `import`")),
        };
        self.try_semicolon()?;
        Ok(Stmt::Import { path, span: start_span })
    }

    fn parse_impl_block(&mut self) -> Result<Stmt, String> {
        let start_span = self.peek().map(|t| t.span()).unwrap_or_default();
        self.advance(); // impl
        // Parse optional generic params: [T, U: Interface]
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
        let type_sym = match self.peek().map(|t| &t.kind) {
            Some(TokenKind::Identifier(name)) => {
                let name = name.clone();
                self.advance();
                Symbol::intern(&name)
            }
            Some(TokenKind::Keyword(kw)) => {
                let s = kw.to_string();
                self.advance();
                Symbol::intern(&s)
            }
            _ => return Err(self.error("expected type name after `impl`")),
        };
        self.expect_delimiter(Delimiter::LBrace)?;
        let mut methods = Vec::new();
        loop {
            match self.peek().map(|t| &t.kind) {
                Some(TokenKind::Delimiter(Delimiter::RBrace)) | None => break,
                _ => methods.push(self.parse_impl_method(&type_sym, &generic_params)?),
            }
        }
        self.expect_delimiter(Delimiter::RBrace)?;
        Ok(Stmt::ImplBlock {
            type_name: type_sym,
            generic_params,
            methods,
            span: start_span,
        })
    }

    /// Parse a method inside an impl block.
    /// Converts `fn draw(shared self, ...)` into a regular FnDecl with
    /// the self parameter typed as `shared TypeName` (or `unique TypeName`).
    fn parse_impl_method(&mut self, impl_type: &Symbol, impl_generic_params: &[(Symbol, Option<Symbol>)]) -> Result<Stmt, String> {
        // Optional pub keyword
        if self.peek().map(|t| &t.kind) == Some(&TokenKind::Keyword(Keyword::Pub)) {
            self.advance();
        }
        // Optional pub(crate)
        if self.peek().map(|t| &t.kind) == Some(&TokenKind::Keyword(Keyword::Pub)) {
            self.advance();
            if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::LParen)) {
                self.advance();
                if self.peek().map(|t| &t.kind) == Some(&TokenKind::Keyword(Keyword::Crate)) {
                    self.advance();
                    if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RParen)) {
                        self.advance();
                    }
                }
            }
        }
        let start_span = self.peek().map(|t| t.span()).unwrap_or_default();
        self.expect_keyword(Keyword::Fn)?;
        let name = Symbol::intern(&self.expect_identifier()?);

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

        // Parse self parameter: shared self or unique self
        let self_keyword = match self.peek().map(|t| &t.kind) {
            Some(TokenKind::Keyword(Keyword::Shared)) => {
                self.advance();
                Symbol::intern("shared")
            }
            Some(TokenKind::Keyword(Keyword::Unique)) => {
                self.advance();
                Symbol::intern("unique")
            }
            _ => return Err(self.error("expected 'shared' or 'unique' for self parameter in method")),
        };
        let self_name = self.expect_identifier()?;
        if self_name != "self" {
            return Err(self.error("expected 'self' as first parameter name in method"));
        }

        let base_self_type = if impl_generic_params.is_empty() {
            Type::Named(*impl_type, Span::default())
        } else {
            let gp_names: Vec<Type> = impl_generic_params.iter()
                .map(|(n, _)| Type::Named(*n, Span::default()))
                .collect();
            Type::Generic(*impl_type, gp_names, Span::default())
        };
        let self_type = if self_keyword.as_str() == "shared" {
            Type::Shared(Box::new(base_self_type), Span::default())
        } else {
            Type::Unique(Box::new(base_self_type), Span::default())
        };

        let mut params = vec![(Symbol::intern("self"), self_type)];

        // Parse remaining params
        if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::Comma)) {
            self.advance();
            loop {
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

        let body = self.parse_block()?;

        Ok(Stmt::FnDecl {
            vis: Visibility::Pub,
            is_inline: false,
            extern_c: false,
            generic_params,
            name,
            params,
            return_type,
            body,
            span: start_span,
        })
    }


    fn parse_block(&mut self) -> Result<Block, String> {
        let start_span = self.peek().map(|t| t.span()).unwrap_or_default();
        self.expect_delimiter(Delimiter::LBrace)?;
        let mut stmts = Vec::new();
        loop {
            match self.peek().map(|t| &t.kind) {
                Some(TokenKind::Delimiter(Delimiter::RBrace)) | None => break,
                _ => stmts.push(self.parse_stmt()?),
            }
        }
        self.expect_delimiter(Delimiter::RBrace)?;
        Ok(Block::new(stmts, start_span))
    }

    // ==================== Expressions ====================

    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_and()?;
        while self.peek().map(|t| t.kind == TokenKind::Operator("||".to_string())) == Some(true) {
            let op_span = self.peek().unwrap().span();
            self.advance();
            let right = self.parse_and()?;
            left = Expr::Binary {
                op: BinaryOp::Or,
                lhs: Box::new(left),
                rhs: Box::new(right),
                span: op_span,
            };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_compare()?;
        while self.peek().map(|t| t.kind == TokenKind::Operator("&&".to_string())) == Some(true) {
            let op_span = self.peek().unwrap().span();
            self.advance();
            let right = self.parse_compare()?;
            left = Expr::Binary {
                op: BinaryOp::And,
                lhs: Box::new(left),
                rhs: Box::new(right),
                span: op_span,
            };
        }
        Ok(left)
    }

    fn parse_compare(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_sum()?;
        while let Some(tok) = self.peek() {
            let op_span = tok.span();
            let op = match &tok.kind {
                TokenKind::Operator(s) => match s.as_str() {
                    "==" => Some(BinaryOp::Eq),
                    "!=" => Some(BinaryOp::Neq),
                    "<=" => Some(BinaryOp::Le),
                    ">=" => Some(BinaryOp::Ge),
                    "<" => Some(BinaryOp::Lt),
                    ">" => Some(BinaryOp::Gt),
                    _ => None,
                },
                _ => None,
            };
            match op {
                Some(op) => {
                    self.advance();
                    let right = self.parse_sum()?;
                    left = Expr::Binary {
                        op,
                        lhs: Box::new(left),
                        rhs: Box::new(right),
                        span: op_span,
                    };
                }
                None => break,
            }
        }
        Ok(left)
    }

    fn parse_sum(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_product()?;
        while let Some(tok) = self.peek() {
            let op_span = tok.span();
            let op = match &tok.kind {
                TokenKind::Operator(s) => match s.as_str() {
                    "+" => Some(BinaryOp::Add),
                    "-" => Some(BinaryOp::Sub),
                    _ => None,
                },
                _ => None,
            };
            match op {
                Some(op) => {
                    self.advance();
                    let right = self.parse_product()?;
                    left = Expr::Binary {
                        op,
                        lhs: Box::new(left),
                        rhs: Box::new(right),
                        span: op_span,
                    };
                }
                None => break,
            }
        }
        Ok(left)
    }

    fn parse_product(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;
        while let Some(tok) = self.peek() {
            let op_span = tok.span();
            let op = match &tok.kind {
                TokenKind::Operator(s) => match s.as_str() {
                    "*" => Some(BinaryOp::Mul),
                    "/" => Some(BinaryOp::Div),
                    "%" => Some(BinaryOp::Mod),
                    _ => None,
                },
                _ => None,
            };
            match op {
                Some(op) => {
                    self.advance();
                    let right = self.parse_unary()?;
                    left = Expr::Binary {
                        op,
                        lhs: Box::new(left),
                        rhs: Box::new(right),
                        span: op_span,
                    };
                }
                None => break,
            }
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        let tok = self.peek().ok_or_else(|| self.error("expected expression"))?.clone();
        let span = tok.span();
        match &tok.kind {
            TokenKind::Operator(s) if s == "-" => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Unary {
                    op: UnaryOp::Neg,
                    arg: Box::new(expr),
                    span,
                })
            }
            TokenKind::Operator(s) if s == "!" => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Unary {
                    op: UnaryOp::Not,
                    arg: Box::new(expr),
                    span,
                })
            }
            TokenKind::Keyword(Keyword::Move) => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Move(Box::new(expr), span))
            }
            TokenKind::Keyword(Keyword::Clone) => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Clone(Box::new(expr), span))
            }
            TokenKind::Keyword(Keyword::Unique) => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::ToUnique(Box::new(expr), span))
            }
            TokenKind::Keyword(Keyword::Shared) => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::ToShared(Box::new(expr), span))
            }
            TokenKind::Keyword(Keyword::Weak) => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::ToWeak(Box::new(expr), span))
            }
            TokenKind::Keyword(Keyword::Ref) => {
                self.advance();
                let mutable = self.peek().map(|t| t.kind == TokenKind::Keyword(Keyword::Mut)).unwrap_or(false);
                if mutable { self.advance(); }
                let expr = self.parse_unary()?;
                Ok(Expr::Ref(Box::new(expr), mutable, span))
            }
            _ => self.parse_postfix(),
        }
    }

    /// Parse postfix operations: function calls, indexing, method calls, field access.
    fn parse_postfix(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_atom()?;
        loop {
            let tok = self.peek().cloned();
            let span = tok.as_ref().map(|t| t.span()).unwrap_or_default();
            match tok.as_ref().map(|t| &t.kind) {
                // Function call: expr(args) — currently only used for Ident(args)
                // which is handled inside parse_atom. This branch handles cases
                // like (expr)(args) for parenthesized expressions.
                Some(TokenKind::Delimiter(Delimiter::LParen)) => {
                    self.advance();
                    let mut args = Vec::new();
                    if self.peek().map(|t| &t.kind) != Some(&TokenKind::Delimiter(Delimiter::RParen)) {
                        loop {
                            args.push(self.parse_expr()?);
                            if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RParen)) {
                                break;
                            }
                            self.expect_delimiter(Delimiter::Comma)?;
                        }
                    }
                    self.expect_delimiter(Delimiter::RParen)?;
                    // If expr is an Ident, convert to FnCall
                    if let Expr::Ident(name, _) = &expr {
                        let name = *name;
                        expr = Expr::FnCall { name, args, span };
                    } else {
                        expr = Expr::CallExpr { target: Box::new(expr), args, span };
                    }
                }
                // Indexing: expr[index]
                Some(TokenKind::Delimiter(Delimiter::LBracket)) => {
                    self.advance();
                    let index = self.parse_expr()?;
                    self.expect_delimiter(Delimiter::RBracket)?;
                    expr = Expr::Index {
                        object: Box::new(expr), index: Box::new(index), span
                    };
                }
                // Try operator: expr?
                Some(TokenKind::Operator(s)) if s == "?" => {
                    self.advance();
                    expr = Expr::TryOp(Box::new(expr), span);
                }
                // Method call: expr.method(args) or field access: expr.field
                Some(TokenKind::Delimiter(Delimiter::Dot)) => {
                    self.advance();
                    let name = self.expect_identifier()?;
                    if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::LParen)) {
                        self.advance();
                        let mut args = Vec::new();
                        loop {
                            if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RParen)) { break; }
                            args.push(self.parse_expr()?);
                            if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RParen)) { break; }
                            self.expect_delimiter(Delimiter::Comma)?;
                        }
                        self.expect_delimiter(Delimiter::RParen)?;
                        expr = Expr::MethodCall {
                            object: Box::new(expr), method: Symbol::intern(&name), args, span
                        };
                    } else {
                        expr = Expr::FieldAccess {
                            object: Box::new(expr), field: Symbol::intern(&name), span
                        };
                    }
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_atom(&mut self) -> Result<Expr, String> {
        let tok = self.peek().ok_or_else(|| self.error("expected expression"))?.clone();
        let span = tok.span();
        match tok.kind {
            TokenKind::IntLiteral(s) => {
                self.advance();
                let n = s.parse::<i64>().map_err(|_| self.error("invalid integer literal"))?;
                Ok(Expr::Literal(Literal::Int(n, span)))
            }
            TokenKind::FloatLiteral(s) => {
                self.advance();
                let n = s.parse::<f64>().map_err(|_| self.error("invalid float literal"))?;
                Ok(Expr::Literal(Literal::Float(n, span)))
            }
            TokenKind::CharLiteral(s) => {
                self.advance();
                let c = s.chars().next().unwrap_or('\0');
                Ok(Expr::Literal(Literal::Char(c, span)))
            }
            TokenKind::StringLiteral(s) => {
                self.advance();
                Ok(Expr::Literal(Literal::String(s, span)))
            }
            TokenKind::Identifier(ref name) => {
                let tok = self.peek().cloned().unwrap();
                let span = tok.span();
                let mut name_str = name.clone();
                let mut name_sym = Symbol::intern(&name_str);
                self.advance();
                while self.peek().map(|t| &t.kind) == Some(&TokenKind::Operator("::".to_string())) {
                    self.advance();
                    let next = self.expect_identifier()?;
                    name_str = format!("{}.{}", name_str, next);
                    name_sym = Symbol::intern(&name_str);
                }
                // Check for generic struct literal: Name[T] { field = val }
                // Only trigger if we can find ]{ ident = pattern
                let is_generic_struct = self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::LBracket))
                    && self.pos + 4 < self.tokens.len()
                    && self.tokens[self.pos + 1].kind != TokenKind::Delimiter(Delimiter::RBracket)
                    && self.tokens[self.pos + 2].kind == TokenKind::Delimiter(Delimiter::RBracket)
                    && self.tokens[self.pos + 3].kind == TokenKind::Delimiter(Delimiter::LBrace)
                    && matches!(&self.tokens[self.pos + 4].kind, TokenKind::Identifier(_) | TokenKind::Keyword(Keyword::Self_))
                    && self.pos + 5 < self.tokens.len()
                    && self.tokens[self.pos + 5].kind == TokenKind::Operator("=".to_string());
                if is_generic_struct {
                    self.advance(); // consume [
                    let mut generic_args = Vec::new();
                    loop {
                        generic_args.push(self.parse_type()?);
                        if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RBracket)) { break; }
                        self.expect_delimiter(Delimiter::Comma)?;
                    }
                    self.expect_delimiter(Delimiter::RBracket)?;
                    // Now check for { — struct literal
                    if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::LBrace)) {
                            let is_struct_lit = self.pos + 2 < self.tokens.len()
                                && matches!(&self.tokens[self.pos + 1].kind, TokenKind::Identifier(_) | TokenKind::Keyword(Keyword::Self_))
                                && self.tokens[self.pos + 2].kind == TokenKind::Operator("=".to_string());
                            if is_struct_lit {
                                self.advance(); // consume {
                                let mut fields = Vec::new();
                                if self.peek().map(|t| &t.kind) != Some(&TokenKind::Delimiter(Delimiter::RBrace)) {
                                    loop {
                                        let field_name = Symbol::intern(&self.expect_identifier()?);
                                        self.expect_operator("=")?;
                                        let field_val = self.parse_expr()?;
                                        fields.push((field_name, field_val));
                                        if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RBrace)) { break; }
                                        self.expect_delimiter(Delimiter::Comma)?;
                                    }
                                }
                                self.expect_delimiter(Delimiter::RBrace)?;
                                return Ok(Expr::StructLiteral { type_name: name_sym, generic_args, fields, span });
                            }
                    }
                }
                match self.peek().map(|t| &t.kind) {
                    Some(TokenKind::Delimiter(Delimiter::LParen)) => {
                        self.advance();
                        let mut args = Vec::new();
                        if self.peek().map(|t| &t.kind) != Some(&TokenKind::Delimiter(Delimiter::RParen)) {
                            loop {
                                args.push(self.parse_expr()?);
                                if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RParen)) {
                                    break;
                                }
                                self.expect_delimiter(Delimiter::Comma)?;
                            }
                        }
                        self.expect_delimiter(Delimiter::RParen)?;
                        Ok(Expr::FnCall { name: name_sym, args, span })
                    }
                    Some(TokenKind::Delimiter(Delimiter::LBrace)) => {
                        let is_struct_lit = self.pos + 2 < self.tokens.len()
                            && matches!(&self.tokens[self.pos + 1].kind, TokenKind::Identifier(_) | TokenKind::Keyword(Keyword::Self_))
                            && self.tokens[self.pos + 2].kind == TokenKind::Operator("=".to_string());
                        if !is_struct_lit {
                            return Ok(Expr::Ident(name_sym, span));
                        }
                        self.advance();
                        let mut fields = Vec::new();
                        if self.peek().map(|t| &t.kind) != Some(&TokenKind::Delimiter(Delimiter::RBrace)) {
                            loop {
                                let field_name = Symbol::intern(&self.expect_identifier()?);
                                self.expect_operator("=")?;
                                let field_val = self.parse_expr()?;
                                fields.push((field_name, field_val));
                                if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RBrace)) {
                                    break;
                                }
                                self.expect_delimiter(Delimiter::Comma)?;
                            }
                        }
                        self.expect_delimiter(Delimiter::RBrace)?;
                        Ok(Expr::StructLiteral { type_name: name_sym, generic_args: Vec::new(), fields, span })
                    }
                    _ => Ok(Expr::Ident(name_sym, span)),
                }
            }
            TokenKind::Keyword(Keyword::True) => {
                let tok = self.peek().cloned().unwrap();
                self.advance();
                Ok(Expr::Literal(Literal::Bool(true, tok.span())))
            }
            TokenKind::Keyword(Keyword::False) => {
                let tok = self.peek().cloned().unwrap();
                self.advance();
                Ok(Expr::Literal(Literal::Bool(false, tok.span())))
            }
            TokenKind::Keyword(Keyword::Null) => {
                let tok = self.peek().cloned().unwrap();
                self.advance();
                Ok(Expr::Null(tok.span()))
            }
            TokenKind::Keyword(Keyword::Self_) => {
                let tok = self.peek().cloned().unwrap();
                self.advance();
                Ok(Expr::Ident(Symbol::intern("self"), tok.span()))
            }
            TokenKind::Keyword(Keyword::Asm) => {
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
                                            self.advance(); "r".to_string()
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
            TokenKind::Delimiter(Delimiter::LParen) => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect_delimiter(Delimiter::RParen)?;
                Ok(expr)
            }
            TokenKind::Delimiter(Delimiter::LBracket) => {
                let tok = self.peek().cloned().unwrap();
                let bracket_span = tok.span();
                self.advance();
                // Check if this is a sized array: [type; count]
                // Peek: if next token is a type keyword or identifier, and the one after is ";"
                let is_sized = self.peek().map(|t| {
                    matches!(&t.kind,
                        TokenKind::Keyword(Keyword::Int | Keyword::Float | Keyword::Char | Keyword::Bool)
                        | TokenKind::Identifier(_)
                        | TokenKind::Keyword(Keyword::Shared | Keyword::Unique | Keyword::Weak)
                    )
                }).unwrap_or(false)
                && self.pos + 1 < self.tokens.len()
                && self.tokens[self.pos + 1].kind == TokenKind::Delimiter(Delimiter::Semicolon);

                if is_sized {
                    let elem_type = self.parse_type()?;
                    self.expect_delimiter(Delimiter::Semicolon)?;
                    let count = self.parse_expr()?;
                    self.expect_delimiter(Delimiter::RBracket)?;
                    Ok(Expr::ArraySized { elem_type, count: Box::new(count), span: bracket_span })
                } else {
                    let mut elems = Vec::new();
                    if self.peek().map(|t| &t.kind) != Some(&TokenKind::Delimiter(Delimiter::RBracket)) {
                        loop {
                            elems.push(self.parse_expr()?);
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
            _ => Err(self.error("expected expression")),
        }
    }

    // ==================== Types ====================

    fn parse_type(&mut self) -> Result<Type, String> {
        let tok = self.peek().ok_or_else(|| self.error("expected type"))?.clone();
        let span = tok.span();
        match tok.kind {
            TokenKind::Keyword(Keyword::Unique) => {
                self.advance();
                let inner = self.parse_base_type()?;
                Ok(Type::Unique(Box::new(inner), span))
            }
            TokenKind::Keyword(Keyword::Ref) => {
                self.advance();
                let mutable = self.peek().map(|t| &t.kind) == Some(&TokenKind::Keyword(Keyword::Mut));
                if mutable { self.advance(); }
                let inner = self.parse_base_type()?;
                Ok(Type::Ref(Box::new(inner), mutable, span))
            }
            TokenKind::Keyword(Keyword::Shared) => {
                self.advance();
                let inner = self.parse_base_type()?;
                Ok(Type::Shared(Box::new(inner), span))
            }
            TokenKind::Keyword(Keyword::Weak) => {
                self.advance();
                let inner = self.parse_base_type()?;
                Ok(Type::Weak(Box::new(inner), span))
            }
            _ => self.parse_base_type(),
        }
    }

    fn parse_base_type(&mut self) -> Result<Type, String> {
        let tok = self.peek().ok_or_else(|| self.error("expected type"))?.clone();
        let span = tok.span();
        match tok.kind {
            TokenKind::Delimiter(Delimiter::LBracket) => {
                self.advance();
                let inner = self.parse_type()?;
                self.expect_delimiter(Delimiter::RBracket)?;
                Ok(Type::Array(Box::new(inner), span))
            }
            TokenKind::Keyword(Keyword::Int) => {
                self.advance();
                Ok(Type::Int(span))
            }
            TokenKind::Keyword(Keyword::Float) => {
                self.advance();
                Ok(Type::Float(span))
            }
            TokenKind::Keyword(Keyword::Char) => {
                self.advance();
                Ok(Type::Char(span))
            }
            TokenKind::Keyword(Keyword::Bool) => {
                self.advance();
                Ok(Type::Bool(span))
            }
            TokenKind::Keyword(Keyword::Self_) => {
                self.advance();
                Ok(Type::Self_(span))
            }
            TokenKind::Identifier(s) => {
                let name = Symbol::intern(&s);
                self.advance();
                // Check for generic type instantiation: Foo[int]
                if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::LBracket)) {
                    self.advance();
                    let mut args = Vec::new();
                    loop {
                        args.push(self.parse_type()?);
                        if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RBracket)) { break; }
                        self.expect_delimiter(Delimiter::Comma)?;
                    }
                    self.expect_delimiter(Delimiter::RBracket)?;
                    Ok(Type::Generic(name, args, span))
                } else {
                    Ok(Type::Named(name, span))
                }
            }
            _ => Err(self.error("expected type")),
        }
    }
}
