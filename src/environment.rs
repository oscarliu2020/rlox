use crate::syntax::{
    ast::VisitorError,
    token::{Literal, Token},
};
use rustc_hash::FxHashMap;
use std::{cell::RefCell, rc::Rc};
pub type EnvironmentRef = Rc<RefCell<Environment>>;
pub trait Envt {
    fn define(&mut self, name: String, value: Literal);
    fn get(&self, name: &Token) -> Result<Literal, VisitorError>;
    fn assign(&mut self, name: &Token, value: Literal) -> Result<(), VisitorError>;
}
impl<T: Envt> Envt for Rc<RefCell<T>> {
    fn define(&mut self, name: String, value: Literal) {
        self.borrow_mut().define(name, value)
    }
    fn get(&self, name: &Token) -> Result<Literal, VisitorError> {
        self.borrow().get(name)
    }
    fn assign(&mut self, name: &Token, value: Literal) -> Result<(), VisitorError> {
        self.borrow_mut().assign(name, value)
    }
}
#[derive(Default, Clone)]
pub struct Environment {
    values: FxHashMap<String, Literal>,
    pub enclosing: Option<EnvironmentRef>,
}
impl Environment {
    pub fn new(enclosing: Option<EnvironmentRef>) -> Self {
        Self {
            values: FxHashMap::default(),
            enclosing,
        }
    }
}
impl Envt for Environment {
    fn define(&mut self, name: String, value: Literal) {
        self.values.insert(name, value);
    }
    fn get(&self, name: &Token) -> Result<Literal, VisitorError> {
        self.values
            .get(&name.lexeme)
            .cloned()
            .or_else(|| {
                self.enclosing
                    .as_ref()
                    .and_then(|enclosing| enclosing.get(name).ok())
            })
            .ok_or_else(|| {
                // error(name, "Undefined variable");
                VisitorError::UndefinedVariable(name.clone())
            })
    }
    fn assign(&mut self, name: &Token, value: Literal) -> Result<(), VisitorError> {
        self.values
            .get_mut(&name.lexeme)
            .map(|v| {
                *v = value.clone();
            })
            .or_else(|| {
                self.enclosing
                    .as_mut()
                    .and_then(|enclosing| enclosing.assign(name, value).ok())
            })
            .ok_or_else(|| {
                // error(name, "Undefined variable");
                VisitorError::UndefinedVariable(name.clone())
            })
    }
}

#[cfg(test)]
mod tests {}
