use super::super::{environment::EnvironmentError, resolver::ResolverError};
use super::ast::*;
use super::token::{Literal, Token};
use std::rc::Rc;
use thiserror::Error;
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
    #[error("ResolverError: {0}")]
    Resolver(#[from] ResolverError),
    #[error("line: {} {} ** Undefined property '{}'.",.0.line,.0.lexeme,.1)]
    UndefinedProperty(Token, String),
}
pub type VisitorResult<T> = Result<T, VisitorError>;
pub trait ExprVisitor {
    fn visit_binary(&mut self, token: &Token, e1: &Expr, e2: &Expr) -> VisitorResult<Literal>;
    fn visit_grouping(&mut self, expr: &Expr) -> VisitorResult<Literal>;
    fn visit_literal(&mut self, ltr: &Literal) -> VisitorResult<Literal>;
    fn visit_unary(&mut self, token: &Token, expr: &Expr) -> VisitorResult<Literal>;
    fn visit_variable(&mut self, variable: &Variable) -> VisitorResult<Literal>;
    fn visit_assign(&mut self, assign: &Assign) -> VisitorResult<Literal>;
    fn visit_logical(&mut self, left: &Expr, token: &Token, right: &Expr)
        -> VisitorResult<Literal>;
    fn visit_call(&mut self, callee: &Expr, paren: &Token, args: &[Expr])
        -> VisitorResult<Literal>;
    fn visit_get(&mut self, get: &Get) -> VisitorResult<Literal>;
    fn visitor_set(&mut self, set: &Set) -> VisitorResult<Literal>;
    fn visit_this(&mut self, token: &This) -> VisitorResult<Literal>;
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
        params: Rc<[Token]>,
        body: Rc<[Stmt]>,
    ) -> VisitorResult<()>;
    fn visit_return(&mut self, token: &Token, expr: Option<&Expr>) -> VisitorResult<()>;
    fn visit_class(&mut self, class: &ClassStmt) -> VisitorResult<()>;
}
