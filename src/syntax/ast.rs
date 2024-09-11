use super::super::{environment::EnvironmentError, resolver::ResolverError};
use super::token::{Literal, Token};
use std::cell::Cell;
use std::fmt::{Display, Formatter};
use thiserror::Error;
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Assign(Assign),
    Binary(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Literal),
    Unary(Token, Box<Expr>),
    Variable(Variable),
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
            Expr::Variable(variable) => {
                write!(f, "{}", variable)
            }
            Expr::Assign(assign) => {
                write!(f, "{}", assign)
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
    Function(FnStmt), // name, params, body
    Return(Token, Option<Expr>),
}
#[derive(Clone, PartialEq, Debug)]
pub struct FnStmt {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Vec<Stmt>,
}
impl FnStmt {
    pub fn new(name: Token, params: Vec<Token>, body: Vec<Stmt>) -> Self {
        Self { name, params, body }
    }
}
#[derive(Clone, PartialEq, Debug)]
pub struct Variable {
    pub name: Token,
    pub dist: Cell<Option<usize>>,
}
impl Display for Variable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name.lexeme)
    }
}
impl Variable {
    pub fn new(name: Token) -> Self {
        Self {
            name,
            dist: Cell::new(None),
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Assign {
    pub name: Token,
    pub value: Box<Expr>,
    pub dist: Cell<Option<usize>>,
}
impl Display for Assign {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} = {}", self.name.lexeme, self.value)
    }
}
impl Assign {
    pub fn new(name: Token, value: Box<Expr>) -> Self {
        Self {
            name,
            value,
            dist: Cell::new(None),
        }
    }
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
    #[error("ResolverError: {0}")]
    Resolver(#[from] ResolverError),
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
impl Stmt {
    pub fn accept(&self, visitor: &mut impl StmtVisitor) -> VisitorResult<()> {
        match self {
            Stmt::Expression(expr) => visitor.visit_expression(expr),
            Stmt::Print(expr) => visitor.visit_print(expr),
            Stmt::Var(token, expr) => visitor.visit_var(token, expr.as_ref()),
            Stmt::Block(stmts) => visitor.visit_block(stmts),
            Stmt::IfStmt(cond, body) => visitor.visit_if(cond, body),
            Stmt::WhileStmt(cond, body) => visitor.visit_while(cond, body),
            Stmt::Function(FnStmt { name, params, body }) => {
                visitor.visit_function(name, params, body)
            }
            Stmt::Return(token, expr) => visitor.visit_return(token, expr.as_ref()),
        }
    }
}
impl Expr {
    pub fn accept(&self, visitor: &mut impl ExprVisitor) -> VisitorResult<Literal> {
        match self {
            Expr::Binary(e1, token, e2) => visitor.visit_binary(token, e1, e2),
            Expr::Grouping(expr) => visitor.visit_grouping(expr),
            Expr::Literal(ltr) => visitor.visit_literal(ltr),
            Expr::Unary(token, expr) => visitor.visit_unary(token, expr),
            Expr::Variable(variable) => visitor.visit_variable(variable),
            Expr::Assign(assign) => visitor.visit_assign(assign),
            Expr::Logical(left, token, right) => visitor.visit_logical(left, token, right),
            Expr::Call(callee, paren, args) => visitor.visit_call(callee, paren, args),
        }
    }
}
use super::super::resolver::Resolvable;
impl Resolvable for Variable {
    fn name(&self) -> &Token {
        &self.name
    }
    fn get_dist(&self) -> Option<usize> {
        self.dist.get()
    }
    fn set_dist(&self, dist: usize) {
        self.dist.set(Some(dist));
    }
}
impl Resolvable for Assign {
    fn name(&self) -> &Token {
        &self.name
    }
    fn get_dist(&self) -> Option<usize> {
        self.dist.get()
    }
    fn set_dist(&self, dist: usize) {
        self.dist.set(Some(dist));
    }
}
