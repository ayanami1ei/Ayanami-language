use super::*;

impl Parser {
    // ==================== Expressions ====================

    /// A5c-2：公开的表达式入口（宏展开产物解析）
    pub fn parse_expr_entry(&mut self) -> Result<Expr> {
        self.parse_expr()
    }

    pub(super) fn parse_expr(&mut self) -> Result<Expr> {
        self.parse_or()
    }

    pub(super) fn parse_or(&mut self) -> Result<Expr> {
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

    pub(super) fn parse_and(&mut self) -> Result<Expr> {
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

    pub(super) fn parse_compare(&mut self) -> Result<Expr> {
        let mut left = self.parse_bitor()?;
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
                    let right = self.parse_bitor()?;
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

    pub(super) fn parse_bitor(&mut self) -> Result<Expr> {
        let mut left = self.parse_bitxor()?;
        while self.peek().map(|t| t.kind == TokenKind::Operator("|".to_string())) == Some(true) {
            let op_span = self.peek().unwrap().span();
            self.advance();
            let right = self.parse_bitxor()?;
            left = Expr::Binary {
                op: BinaryOp::BitOr,
                lhs: Box::new(left),
                rhs: Box::new(right),
                span: op_span,
            };
        }
        Ok(left)
    }

    pub(super) fn parse_bitxor(&mut self) -> Result<Expr> {
        let mut left = self.parse_bitand()?;
        while self.peek().map(|t| t.kind == TokenKind::Operator("^".to_string())) == Some(true) {
            let op_span = self.peek().unwrap().span();
            self.advance();
            let right = self.parse_bitand()?;
            left = Expr::Binary {
                op: BinaryOp::BitXor,
                lhs: Box::new(left),
                rhs: Box::new(right),
                span: op_span,
            };
        }
        Ok(left)
    }

    pub(super) fn parse_bitand(&mut self) -> Result<Expr> {
        let mut left = self.parse_shift()?;
        while self.peek().map(|t| t.kind == TokenKind::Operator("&".to_string())) == Some(true) {
            let op_span = self.peek().unwrap().span();
            self.advance();
            let right = self.parse_shift()?;
            left = Expr::Binary {
                op: BinaryOp::BitAnd,
                lhs: Box::new(left),
                rhs: Box::new(right),
                span: op_span,
            };
        }
        Ok(left)
    }

    pub(super) fn parse_shift(&mut self) -> Result<Expr> {
        let mut left = self.parse_sum()?;
        while let Some(tok) = self.peek() {
            let op_span = tok.span();
            let op = match &tok.kind {
                TokenKind::Operator(s) => match s.as_str() {
                    "<<" => Some(BinaryOp::Shl),
                    ">>" => Some(BinaryOp::Shr),
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

    pub(super) fn parse_sum(&mut self) -> Result<Expr> {
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

    pub(super) fn parse_cast(&mut self) -> Result<Expr> {
        let mut expr = self.parse_unary()?;
        while matches!(self.peek().map(|t| &t.kind), Some(TokenKind::Keyword(Keyword::As))) {
            let span = self.peek().unwrap().span();
            self.advance();
            let ty = self.parse_type()?;
            expr = Expr::Cast { expr: Box::new(expr), ty, span };
        }
        Ok(expr)
    }

    pub(super) fn parse_product(&mut self) -> Result<Expr> {
        let mut left = self.parse_cast()?;
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
                    let right = self.parse_cast()?;
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
}
