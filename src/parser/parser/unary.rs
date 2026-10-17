use super::*;

impl Parser {
    pub(super) fn parse_unary(&mut self) -> Result<Expr> {
        let tok = self.peek().ok_or_else(|| self.error("expected expression"))?.clone();
        let span = tok.span();
        match &tok.kind {
            TokenKind::Operator(s) if s == "-" => {
                self.advance();
                let expr = self.parse_unary()?;
                // M1.5：负号折进带后缀字面量，保证 -128i8 的范围检查正确
                if let Expr::Suffixed { lit, suffix, span: sp } = expr {
                    let neg = match lit {
                        Literal::Int(n, lsp) => Literal::Int(-n, lsp),
                        Literal::Float(f, lsp) => Literal::Float(-f, lsp),
                        other => {
                            return Ok(Expr::Unary { op: UnaryOp::Neg, arg: Box::new(Expr::Literal(other)), span });
                        }
                    };
                    return Ok(Expr::Suffixed { lit: neg, suffix, span: sp });
                }
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
            // M5 P2：裸指针解引用 `*p`
            TokenKind::Operator(s) if s == "*" => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Unary { op: UnaryOp::Deref, arg: Box::new(expr), span })
            }
            TokenKind::Operator(s) if s == "~" => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Unary {
                    op: UnaryOp::BitNot,
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
                Err(self.error("`unique` has been removed: values are owned by default; array literals are written `[..]`"))
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
    pub(super) fn parse_postfix(&mut self) -> Result<Expr> {
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
                        expr = Expr::FnCall { name, args, generic_args: Vec::new(), span };
                    } else {
                        expr = Expr::CallExpr { target: Box::new(expr), args, span };
                    }
                }
                // 显式泛型调用：`ns.fn[T](args)`（限定名）或 `f[T](args)`（单标识符，P4a）
                Some(TokenKind::Delimiter(Delimiter::LBracket)) => {
                    if let Expr::Ident(name, _) = &expr {
                        let dotted = name.as_str().contains('.');
                        if let Some((tys, args)) = self.try_parse_generic_call_args(!dotted)? {
                            expr = Expr::FnCall { name: *name, args, generic_args: tys, span };
                            continue;
                        }
                    }
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
                // Enum construction: EnumType::Variant(args)
                Some(TokenKind::Operator(s)) if s == "::" => {
                    self.advance();
                    let variant_name = Symbol::intern(&self.expect_identifier()?);
                    if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::LParen)) {
                        self.advance();
                        let mut args = Vec::new();
                        if self.peek().map(|t| &t.kind) != Some(&TokenKind::Delimiter(Delimiter::RParen)) {
                            loop {
                                args.push(self.parse_expr()?);
                                if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RParen)) { break; }
                                self.expect_delimiter(Delimiter::Comma)?;
                            }
                        }
                        self.expect_delimiter(Delimiter::RParen)?;
                        let enum_name = match &expr {
                            Expr::Ident(n, _) => *n,
                            _ => return Err(self.error("expected enum type name before ::")),
                        };
                        expr = Expr::EnumConstruct {
                            enum_name,
                            variant_name,
                            tuple_args: args,
                            named_args: vec![],
                            span,
                        };
                    } else if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::LBrace)) {
                        self.advance();
                        let mut fields = Vec::new();
                        loop {
                            if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RBrace)) { break; }
                            let fexpr = self.parse_expr()?;
                            let fname = self.expect_member_name()?;
                            fields.push((Symbol::intern(&fname), fexpr));
                            if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::Comma)) {
                                self.advance();
                            }
                        }
                        self.expect_delimiter(Delimiter::RBrace)?;
                        let enum_name = match &expr {
                            Expr::Ident(n, _) => *n,
                            _ => return Err(self.error("expected enum type name before ::")),
                        };
                        expr = Expr::EnumConstruct {
                            enum_name,
                            variant_name,
                            tuple_args: vec![],
                            named_args: fields,
                            span,
                        };
                    } else {
                        return Err(self.error("expected '(' or '{' after enum variant name"));
                    }
                }
                // Method call: expr.method(args) or field access: expr.field
                Some(TokenKind::Delimiter(Delimiter::Dot)) => {
                    self.advance();
                    // #172：成员位置允许关键字（如 `.union(...)`、`.type`）
                    let name = self.expect_member_name()?;
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
}

impl Parser {
    /// P4a：尝试解析显式泛型调用实参 `[T1, T2](args)`（回退不消耗 token）。
    ///
    /// `require_type_like`：单标识符场景下要求类型列表「像类型」
    /// （关键字类型 / `ref`/`*`/数组/嵌套泛型 / 大写开头标识符），
    /// 避免把 `a[i](x)`（索引后调用）误判为泛型调用。
    pub(super) fn try_parse_generic_call_args(
        &mut self,
        require_type_like: bool,
    ) -> Result<Option<(Vec<Type>, Vec<Expr>)>> {
        let saved = self.pos;
        if self.peek().map(|t| &t.kind) != Some(&TokenKind::Delimiter(Delimiter::LBracket)) {
            return Ok(None);
        }
        self.advance(); // [
        let mut tys: Vec<Type> = Vec::new();
        let mut ok = true;
        loop {
            match self.parse_type() {
                Ok(t) => tys.push(t),
                Err(_) => { ok = false; break; }
            }
            match self.peek().map(|t| &t.kind) {
                Some(TokenKind::Delimiter(Delimiter::RBracket)) => break,
                Some(TokenKind::Delimiter(Delimiter::Comma)) => { self.advance(); }
                _ => { ok = false; break; }
            }
        }
        if !ok || self.peek().map(|t| &t.kind) != Some(&TokenKind::Delimiter(Delimiter::RBracket)) {
            self.pos = saved;
            return Ok(None);
        }
        if require_type_like && !tys.iter().any(is_type_like_arg) {
            self.pos = saved;
            return Ok(None);
        }
        self.advance(); // ]
        if self.peek().map(|t| &t.kind) != Some(&TokenKind::Delimiter(Delimiter::LParen)) {
            self.pos = saved;
            return Ok(None);
        }
        self.advance(); // (
        let mut args = Vec::new();
        if self.peek().map(|t| &t.kind) != Some(&TokenKind::Delimiter(Delimiter::RParen)) {
            loop {
                args.push(self.parse_expr()?);
                if self.peek().map(|t| &t.kind) == Some(&TokenKind::Delimiter(Delimiter::RParen)) { break; }
                self.expect_delimiter(Delimiter::Comma)?;
            }
        }
        self.expect_delimiter(Delimiter::RParen)?;
        Ok(Some((tys, args)))
    }
}

/// 类型列表「像类型」判定（P4a 单标识符歧义消解）：关键字类型/引用/指针/数组/
/// 嵌套泛型/大写开头标识符（类型参数与类型名约定大写）。
fn is_type_like_arg(t: &Type) -> bool {
    match t {
        Type::Int(_) | Type::Float(_) | Type::Char(_) | Type::Bool(_) | Type::Void(_)
        | Type::Never(_) | Type::Ref(..) | Type::RawPtr(..) | Type::Array(..)
        | Type::ArraySized(..) | Type::FnPtr(..) | Type::Closure(..) | Type::Unique(..)
        | Type::Generic(..) | Type::Self_(..) => true,
        Type::Named(s, _) => s.as_str().chars().next().map(|c| c.is_uppercase()).unwrap_or(false),
        Type::Default => false,
    }
}
