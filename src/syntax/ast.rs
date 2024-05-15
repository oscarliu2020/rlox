use super::token::{Literal, Token};
use std::fmt::{Display, Formatter};
pub enum Expr {
    Assign(Token, Box<Expr>),
    Binary(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Literal),
    Unary(Token, Box<Expr>),
    Variable(Token),
}
impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Literal(ltr) => {
                write!(f, "{}", ltr)
            }
            Expr::Grouping(expr) => {
                write!(f, "( group {})", expr)
            }
            Expr::Unary(tok, expr) => {
                write!(f, "({} {})", tok.lexeme, expr)
            }
            Expr::Binary(left, tok, right) => {
                write!(f, "({} {} {})", tok.lexeme, left, right)
            }
            Expr::Variable(tok) => {
                write!(f, "{}", tok.lexeme)
            }
            Expr::Assign(tok, expr) => {
                write!(f, "{} = {}", tok.lexeme, expr)
            }
        }
    }
}
pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    Var(Token, Option<Expr>),
    Block(Vec<Stmt>),
}
pub trait ExprVisitor {
    fn visit_binary(&mut self, token: &Token, e1: &Expr, e2: &Expr) -> Result<Literal, ()>;
    fn visit_grouping(&mut self, expr: &Expr) -> Result<Literal, ()>;
    fn visit_literal(&mut self, ltr: &Literal) -> Result<Literal, ()>;
    fn visit_unary(&mut self, token: &Token, expr: &Expr) -> Result<Literal, ()>;
    fn visit_variable(&mut self, token: &Token) -> Result<Literal, ()>;
    fn visit_assign(&mut self, token: &Token, expr: &Expr) -> Result<Literal, ()>;
}
pub trait StmtVisitor {
    fn visit_expression(&mut self, expr: &Expr) -> Result<(), ()>;
    fn visit_print(&mut self, expr: &Expr) -> Result<(), ()>;
    fn visit_var(&mut self, token: &Token, expr: Option<&Expr>) -> Result<(), ()>;
    fn visit_block(&mut self, stmts: &[Stmt]) -> Result<(), ()>;
}
