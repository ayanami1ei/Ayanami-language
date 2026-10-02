use super::*;

impl Parser {
    // ==================== Expressions ====================

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

    pub(super) fn parse_product(&mut self) -> Result<Expr> {
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
}
