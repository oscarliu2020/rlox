use crate::syntax::{
    ast::VisitorError,
    token::{Literal, Token},
};
use rustc_hash::FxHashMap;
use std::{cell::RefCell, rc::Rc};
pub type EnvironmentRef = Rc<RefCell<Environment>>;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum EnvironmentError {
    #[error("Undefined variable {0}")]
    UndefinedVariable(String),
    #[error("Invalid environment distance")]
    InvalidEnvironmentDistance,
}
pub trait Envt {
    fn define(&mut self, name: String, value: Literal);
    fn get(&self, name: &Token) -> Result<Literal, VisitorError>;
    fn assign(&mut self, name: &Token, value: Literal) -> Result<(), VisitorError>;
    fn get_at(&self, distance: usize, name: &Token) -> Result<Literal, VisitorError>;
    fn assign_at(
        &mut self,
        distance: usize,
        name: &Token,
        value: Literal,
    ) -> Result<(), VisitorError>;
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
    fn get_at(&self, distance: usize, name: &Token) -> Result<Literal, VisitorError> {
        self.borrow().get_at(distance, name)
    }
    fn assign_at(
        &mut self,
        distance: usize,
        name: &Token,
        value: Literal,
    ) -> Result<(), VisitorError> {
        self.borrow_mut().assign_at(distance, name, value)
    }
}
#[derive(Default)]
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
    #[inline(always)]
    fn ancestor(&self, distance: usize) -> Result<&Environment, EnvironmentError> {
        unsafe {
            let mut env = self;
            for _ in 0..distance {
                env = match &env.enclosing {
                    Some(enclosing) => &*(enclosing.as_ptr()),
                    None => {
                        return Err(EnvironmentError::InvalidEnvironmentDistance);
                    }
                };
            }
            Ok(env)
        }
    }
    fn ancestor_mut(&mut self, distance: usize) -> Result<&mut Environment, EnvironmentError> {
        unsafe {
            let mut env = self;
            for _ in 0..distance {
                env = match &mut env.enclosing {
                    Some(enclosing) => &mut *(enclosing.as_ptr()),
                    None => {
                        return Err(EnvironmentError::InvalidEnvironmentDistance);
                    }
                };
            }
            Ok(env)
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
    fn get_at(&self, distance: usize, name: &Token) -> Result<Literal, VisitorError> {
        self.ancestor(distance).map_or_else(
            |e_| Err(EnvironmentError::InvalidEnvironmentDistance.into()),
            |env| env.get(name),
        )
    }
    fn assign_at(
        &mut self,
        distance: usize,
        name: &Token,
        value: Literal,
    ) -> Result<(), VisitorError> {
        self.ancestor_mut(distance).map_or_else(
            |_| Err(EnvironmentError::InvalidEnvironmentDistance.into()),
            |env| env.assign(name, value),
        )
    }
}

#[cfg(test)]
mod tests {}
