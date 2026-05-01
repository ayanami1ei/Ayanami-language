use super::expr::*;
use super::stmt::*;

pub trait AstVisitor {
    type Output;

    fn visit_program(&mut self, program: &Program) -> Self::Output;
    fn visit_stmt(&mut self, stmt: &Stmt) -> Self::Output;
    fn visit_expr(&mut self, expr: &Expr) -> Self::Output;
    fn visit_block(&mut self, block: &Block) -> Self::Output;
    fn visit_literal(&mut self, lit: &Literal) -> Self::Output;
    fn visit_type(&mut self, ty: &Type) -> Self::Output;
}
