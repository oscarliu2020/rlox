use super::token::{Literal, Token};
use std::cell::Cell;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub enum Expr {
    Assign(Assign),
    Binary(Rc<Expr>, Token, Rc<Expr>),
    Grouping(Rc<Expr>),
    Literal(Literal),
    Unary(Token, Rc<Expr>),
    Variable(Variable),
    Logical(Rc<Expr>, Token, Rc<Expr>),
    Call(Rc<Expr>, Token, Rc<[Expr]>),
    Get(Get),
    Set(Set),
    This(This),
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
                for arg in args.iter() {
                    write!(f, "{},", arg)?;
                }
                write!(f, ")")
            }
            Expr::Get(get) => {
                write!(f, "{}", get)
            }
            Expr::Set(set) => {
                write!(f, "{}", set)
            }
            Expr::This(_) => {
                write!(f, "this")
            }
        }
    }
}
#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    Var(Token, Option<Expr>),
    Block(Vec<Stmt>),
    IfStmt(Expr, Rc<(Stmt, Option<Stmt>)>),
    WhileStmt(Expr, Rc<Stmt>),
    Function(FnStmt), // name, params, body
    Return(Token, Option<Expr>),
    Class(ClassStmt),
}
#[derive(PartialEq, Debug, Clone)]
pub struct FnStmt {
    pub name: Token,
    pub params: Rc<[Token]>,
    pub body: Rc<[Stmt]>,
}
impl FnStmt {
    pub fn new(name: Token, params: Rc<[Token]>, body: Rc<[Stmt]>) -> Self {
        Self { name, params, body }
    }
}
impl Display for FnStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "fn {}(", self.name.lexeme)?;
        for param in self.params.iter() {
            write!(f, "{},", param.lexeme)?;
        }
        write!(f, ")")?;
        Ok(())
    }
}
#[derive(PartialEq, Debug)]
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
#[derive(Debug, PartialEq)]
pub struct Assign {
    pub name: Token,
    pub value: Rc<Expr>,
    pub dist: Cell<Option<usize>>,
}
impl Display for Assign {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} = {}", self.name.lexeme, self.value)
    }
}
impl Assign {
    pub fn new(name: Token, value: Rc<Expr>) -> Self {
        Self {
            name,
            value,
            dist: Cell::new(None),
        }
    }
}
#[derive(Debug, PartialEq)]
pub struct Get {
    pub object: Rc<Expr>,
    pub name: Token,
}
impl Display for Get {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.object, self.name.lexeme)
    }
}
impl Get {
    pub fn new(object: Rc<Expr>, name: Token) -> Self {
        Self { object, name }
    }
}
#[derive(Debug, PartialEq)]
pub struct ClassStmt {
    pub name: Token,
    pub methods: Rc<[FnStmt]>,
}
impl Display for ClassStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "class {}:", self.name.lexeme)?;
        for method in self.methods.iter() {
            write!(f, "\t{}", method)?;
        }
        Ok(())
    }
}
impl ClassStmt {
    pub fn new(name: Token, methods: Rc<[FnStmt]>) -> Self {
        Self { name, methods }
    }
}
#[derive(Debug, PartialEq)]
pub struct Set {
    pub object: Rc<Expr>,
    pub name: Token,
    pub value: Rc<Expr>,
}
impl Display for Set {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{} = {}", self.object, self.name.lexeme, self.value)
    }
}
impl Set {
    pub fn new(object: Rc<Expr>, name: Token, value: Rc<Expr>) -> Self {
        Self {
            object,
            name,
            value,
        }
    }
    pub fn from_get(get: Get, value: Rc<Expr>) -> Self {
        Self {
            object: get.object,
            name: get.name,
            value,
        }
    }
}
#[derive(Debug, PartialEq)]
pub struct This {
    pub token: Token,
    dist: Cell<Option<usize>>,
}
impl Display for This {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "this")
    }
}
impl This {
    pub fn new(token: Token) -> Self {
        Self {
            token,
            dist: Cell::new(None),
        }
    }
}
pub use super::visitor::*;
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
                visitor.visit_function(name, Rc::clone(params), Rc::clone(body))
            }
            Stmt::Return(token, expr) => visitor.visit_return(token, expr.as_ref()),
            Stmt::Class(class) => visitor.visit_class(class),
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
            Expr::Get(get) => visitor.visit_get(get),
            Expr::Set(set) => visitor.visitor_set(set),
            Expr::This(this) => visitor.visit_this(this),
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
impl Resolvable for This {
    fn name(&self) -> &Token {
        &self.token
    }
    fn get_dist(&self) -> Option<usize> {
        self.dist.get()
    }
    fn set_dist(&self, dist: usize) {
        self.dist.set(Some(dist));
    }
}
