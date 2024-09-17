use rustc_hash::FxHashMap;

use super::ast::{FnStmt, Stmt};
use super::token::Token;
use crate::environment::{Environment, EnvironmentRef, Envt};
use std::cell::RefCell;
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
        &self.decl.name.lexeme
    }
    pub fn params(&self) -> &[Token] {
        &self.decl.params
    }
    pub fn body(&mut self) -> &[Stmt] {
        &self.decl.body
    }
    pub fn bind(&mut self, instance: Rc<RefCell<Instance>>) -> Self {
        let mut envrionment = Environment::new(Some(self.closure.clone()));
        envrionment.define("this".to_owned(), Literal::Instance(instance));
        Func {
            decl: Rc::clone(&self.decl),
            closure: Rc::new(RefCell::new(envrionment)),
        }
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
    Initializer(Class),
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
            Function::Initializer(class) => {
                write!(f, "{} initializer", class)
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
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Number(f64),
    String(String),
    Boolean(bool),
    Callable(Function),
    Nil,
    Instance(Rc<RefCell<Instance>>),
}
impl Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Nil => write!(f, "nil"),
            Literal::Boolean(b) => write!(f, "{}", b),
            Literal::Number(n) => write!(f, "{:.}", *n),
            Literal::String(s) => write!(f, "{}", s),
            Literal::Callable(ff) => write!(f, "{}", ff),
            Literal::Instance(i) => write!(f, "{}", i.borrow()),
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
#[derive(Debug, Clone, PartialEq)]
pub struct Class {
    name: String,
    methods: FxHashMap<String, Literal>,
}
impl Class {
    pub fn new(name: String, methods: FxHashMap<String, Literal>) -> Self {
        Self { name, methods }
    }
    fn get_method(&self, name: &str) -> Option<Literal> {
        self.methods.get(name).cloned()
    }
}
impl Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Instance {
    pub class: Class,
    fields: FxHashMap<String, Literal>,
}
impl Instance {
    pub fn new(class: Class) -> Self {
        Self {
            class,
            fields: FxHashMap::default(),
        }
    }
    pub fn get(name: &Token, instance: &Rc<RefCell<Instance>>) -> Option<Literal> {
        instance
            .borrow()
            .fields
            .get(&name.lexeme)
            .cloned()
            .or_else(|| {
                let Literal::Callable(Function::Function(mut method)) =
                    instance.borrow().class.get_method(&name.lexeme)?
                else {
                    unreachable!()
                };
                Some(Literal::Callable(Function::Function(
                    method.bind(Rc::clone(instance)),
                )))
            })
    }
    pub fn set(&mut self, name: &str, value: Literal) {
        self.fields.insert(name.to_string(), value);
    }
}
impl Display for Instance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} instance", self.class)
    }
}
