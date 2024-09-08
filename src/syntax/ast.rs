use super::super::environment::EnvironmentError;
use super::token::{Literal, Token};
use std::fmt::{Display, Formatter};
use thiserror::Error;
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Assign(Token, Box<Expr>),
    Binary(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Literal),
    Unary(Token, Box<Expr>),
    Variable(Token),
    Logical(Box<Expr>, Token, Box<Expr>),
    Call(Box<Expr>, Token, Vec<Expr>),
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
            Expr::Logical(left, tok, right) => {
                write!(f, "({} {} {})", left, tok.lexeme, right)
            }
            Expr::Call(callee, _, args) => {
                write!(f, "{}(", callee)?;
                for arg in args {
                    write!(f, "{},", arg)?;
                }
                write!(f, ")")
            }
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    Var(Token, Option<Expr>),
    Block(Vec<Stmt>),
    IfStmt(Expr, Box<(Stmt, Option<Stmt>)>),
    WhileStmt(Expr, Box<Stmt>),
    Function(Token, Vec<Token>, Vec<Stmt>), // name, params, body
    Return(Token, Option<Expr>),
}
#[derive(Error, Debug)]
pub enum VisitorError {
    #[error("Vistor Error")]
    VistorError,
    #[error("Environment Error")]
    EnvironmentError,
    #[error("line {}: {} ** Can only call functions and classes",.0.line,.0.lexeme)]
    NotCallable(Token),
    #[error("line {}: {} ** Expected {0} arguments but got {1}.",.2.line,.2.lexeme)]
    ArityNotMatched(usize, usize, Token),
    #[error("line {}: {} ** Operands must be two numbers",.0.line,.0.lexeme)]
    ArithmeticError(Token),
    #[error("line {}: {} ** Unknown {1} Operator",.0.line,.0.lexeme)]
    UnknownOperator(Token, &'static str),
    #[error("line {}: {} ** Unary - must be used with a number",.0.line,.0.lexeme)]
    UnaryTypeError(Token),
    #[error("line {}: {} ** Undefined variable",.0.line,.0.lexeme)]
    UndefinedVariable(Token),
    #[error("Return value: {0}")]
    ReturnValue(Literal),
    #[error("line: {} {} ** Can't read local variable in its own initializer.",.0.line,.0.lexeme)]
    NotInitialized(Token),
    #[error("EnvironmentError: {0}")]
    Variable(#[from] EnvironmentError),
}
pub type VisitorResult<T> = Result<T, VisitorError>;
pub trait ExprVisitor {
    fn visit_binary(&mut self, token: &Token, e1: &Expr, e2: &Expr) -> VisitorResult<Literal>;
    fn visit_grouping(&mut self, expr: &Expr) -> VisitorResult<Literal>;
    fn visit_literal(&mut self, ltr: &Literal) -> VisitorResult<Literal>;
    fn visit_unary(&mut self, token: &Token, expr: &Expr) -> VisitorResult<Literal>;
    fn visit_variable(&mut self, token: &Token) -> VisitorResult<Literal>;
    fn visit_assign(&mut self, token: &Token, expr: &Expr) -> VisitorResult<Literal>;
    fn visit_logical(&mut self, left: &Expr, token: &Token, right: &Expr)
        -> VisitorResult<Literal>;
    fn visit_call(&mut self, callee: &Expr, paren: &Token, args: &[Expr])
        -> VisitorResult<Literal>;
}
pub trait StmtVisitor {
    fn visit_while(&mut self, cond: &Expr, body: &Stmt) -> VisitorResult<()>;
    fn visit_expression(&mut self, expr: &Expr) -> VisitorResult<()>;
    fn visit_print(&mut self, expr: &Expr) -> VisitorResult<()>;
    fn visit_var(&mut self, token: &Token, expr: Option<&Expr>) -> VisitorResult<()>;
    fn visit_block(&mut self, stmts: &[Stmt]) -> VisitorResult<()>;
    fn visit_if(&mut self, cond: &Expr, body: &(Stmt, Option<Stmt>)) -> VisitorResult<()>;
    fn visit_function(
        &mut self,
        name: &Token,
        params: &[Token],
        body: &[Stmt],
    ) -> VisitorResult<()>;
    fn visit_return(&mut self, token: &Token, expr: Option<&Expr>) -> VisitorResult<()>;
}
