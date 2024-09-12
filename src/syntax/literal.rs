use super::ast::{FnStmt, Stmt};
use super::token::Token;
use crate::environment::EnvironmentRef;
use std::fmt::{self, Display};
use std::rc::Rc;
#[derive(Clone)]
pub struct Func {
    pub decl: Rc<FnStmt>,
    pub closure: EnvironmentRef,
}
impl PartialEq for Func {
    fn eq(&self, other: &Self) -> bool {
        self.decl == other.decl
    }
}
impl Func {
    pub fn name(&self) -> &str {
        // match &*self.decl {
        //     Stmt::Function(name, _, _) => &name.lexeme,
        //     _ => panic!("Not a function"),
        // }
        &self.decl.name.lexeme
    }
    pub fn params(&self) -> &[Token] {
        // match &*self.decl {
        //     Stmt::Function(_, params, _) => params,
        //     _ => panic!("Not a function"),
        // }
        &self.decl.params
    }
    pub fn body(&mut self) -> &[Stmt] {
        // match &*self.decl {
        //     Stmt::Function(_, _, body) => body,
        //     _ => panic!("Not a function"),
        // }
        &self.decl.body
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct NativeFunc {
    pub name: String,
    pub func: fn() -> Literal,
    pub arity: usize,
}
#[derive(Clone, PartialEq)]
pub enum Function {
    Function(Func), //0:parameters,1:body
    Native(NativeFunc),
}
impl Function {
    fn display(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Function::Function(func) => {
                write!(f, "function {}", func.name())
            }
            Function::Native(native) => {
                write!(f, "native function {}", native.name)
            }
        }
    }
}
impl Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f)
    }
}
impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f)
    }
}
// impl<T> Func for T where T: fn() -> Literal + Clone {}
// struct LoxFn {
//     function_type: Function,
//     closure:Box<dyn Func>
// }
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Number(f64),
    String(String),
    Boolean(bool),
    Callable(Function),
    Nil,
}
impl Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Nil => write!(f, "nil"),
            Literal::Boolean(b) => write!(f, "{}", b),
            Literal::Number(n) => write!(f, "{:.}", *n),
            Literal::String(s) => write!(f, "{}", s),
            Literal::Callable(ff) => write!(f, "{}", ff),
        }
    }
}
impl Literal {
    pub fn is_truthy(&self) -> bool {
        match self {
            Literal::Nil => false,
            Literal::Boolean(b) => *b,
            _ => true,
        }
    }
}
pub struct Class {
    name: String,
}
impl Class {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}
impl Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
