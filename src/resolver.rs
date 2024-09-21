use super::syntax::{ast::*, token::*};
use rustc_hash::FxHashMap;
use std::rc::Rc;
pub struct Resolver {
    scopes: Vec<FxHashMap<String, bool>>,
    cur_func: FunctionType,
    cur_class: ClassType,
}
use thiserror::Error;
#[derive(Error, Debug)]
pub enum ResolverError {
    #[error("line {}: ** Variable {} not initialized", .0.line, .0.lexeme)]
    NotInitialized(Token),
    #[error("line {}: {} ** Already a variable with this name in this scope.", .0.line, .0.lexeme)]
    AlreadyDeclared(Token),
    #[error("line {}: ** Can't return from top-level code.", .0.line)]
    ReturnFromTopLevel(Token),
    #[error("line {}: ** Can't use 'this' outside of a class.", .0.line)]
    InvalidThis(Token),
    #[error("line {0}: ** Can't return a value from an initializer.")]
    ReturnFromInitializer(usize),
    #[error("line {}: {} ** A class can't inherit from itself.", .0.line, .0.lexeme)]
    InheritFromSelf(Token),
}
impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}
#[derive(Clone, Copy, PartialEq)]
enum FunctionType {
    None,
    Function,
    Method,
    Initializer,
}
#[derive(Clone, Copy, PartialEq)]
enum ClassType {
    None,
    Class,
}
impl Resolver {
    pub fn new() -> Self {
        Resolver {
            scopes: vec![],
            cur_func: FunctionType::None,
            cur_class: ClassType::None,
        }
    }
    pub fn resolve(&mut self, stmts: &[Stmt]) -> VisitorResult<()> {
        for stmt in stmts {
            self.resolve_stmt(stmt)?;
        }
        Ok(())
    }
    fn resolve_stmt(&mut self, stmt: &Stmt) -> VisitorResult<()> {
        stmt.accept(self)
    }
    fn resolve_expr(&mut self, expr: &Expr) -> VisitorResult<()> {
        expr.accept(self).map(|_| ())
    }
    fn resolve_local(&mut self, token: &impl Resolvable) -> VisitorResult<()> {
        for (i, scope) in self.scopes.iter().enumerate().rev() {
            if scope.contains_key(&token.name().lexeme) {
                token.set_dist(self.scopes.len() - 1 - i);
                return Ok(());
            }
        }
        Ok(())
    }
    fn begin_scope(&mut self) {
        self.scopes.push(FxHashMap::default());
    }
    fn end_scope(&mut self) {
        self.scopes.pop();
    }
    fn declare(&mut self, name: &Token) -> Result<(), ResolverError> {
        if self.scopes.is_empty() {
            return Ok(());
        }
        let scope = self.scopes.last_mut().unwrap();
        match scope.entry(name.lexeme.clone()) {
            std::collections::hash_map::Entry::Occupied(_) => {
                return Err(ResolverError::AlreadyDeclared(name.clone()));
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(false);
            }
        }
        Ok(())
    }
    fn define(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }
        let scope = self.scopes.last_mut().unwrap();
        scope.insert(name.lexeme.clone(), true);
    }
    fn resolve_function(
        &mut self,
        _: &Token,
        params: Rc<[Token]>,
        body: Rc<[Stmt]>,
        ftype: FunctionType,
    ) -> VisitorResult<()> {
        let prev = self.cur_func;
        self.cur_func = ftype;
        self.begin_scope();
        for param in params.iter() {
            self.declare(param)?;
            self.define(param);
        }
        self.resolve(&body)?;
        self.end_scope();
        self.cur_func = prev;
        Ok(())
    }
}
impl StmtVisitor for Resolver {
    fn visit_block(&mut self, stmts: &[Stmt]) -> VisitorResult<()> {
        self.begin_scope();
        self.resolve(stmts)?;
        self.end_scope();
        Ok(())
    }
    fn visit_expression(&mut self, expr: &Expr) -> VisitorResult<()> {
        self.resolve_expr(expr)
    }
    fn visit_function(
        &mut self,
        name: &crate::syntax::token::Token,
        params: Rc<[crate::syntax::token::Token]>,
        body: Rc<[Stmt]>,
    ) -> VisitorResult<()> {
        self.declare(name)?;
        self.define(name);
        self.resolve_function(
            name,
            Rc::clone(&params),
            Rc::clone(&body),
            FunctionType::Function,
        )
    }
    fn visit_if(&mut self, cond: &Expr, body: &(Stmt, Option<Stmt>)) -> VisitorResult<()> {
        self.resolve_expr(cond)?;
        self.resolve_stmt(&body.0)?;
        if let Some(else_body) = &body.1 {
            self.resolve_stmt(else_body)?;
        }
        Ok(())
    }
    fn visit_print(&mut self, expr: &Expr) -> VisitorResult<()> {
        self.resolve_expr(expr)
    }
    fn visit_return(
        &mut self,
        ret: &crate::syntax::token::Token,
        expr: Option<&Expr>,
    ) -> VisitorResult<()> {
        if self.cur_func == FunctionType::None {
            return Err(ResolverError::ReturnFromTopLevel(ret.clone()).into());
        }
        if let Some(expr) = expr {
            if self.cur_func == FunctionType::Initializer {
                return Err(ResolverError::ReturnFromInitializer(ret.line).into());
            }
            self.resolve_expr(expr)?;
        }
        Ok(())
    }
    fn visit_var(
        &mut self,
        token: &crate::syntax::token::Token,
        expr: Option<&Expr>,
    ) -> VisitorResult<()> {
        self.declare(token)?;
        if let Some(expr) = expr {
            self.resolve_expr(expr)?;
        }
        self.define(token);
        Ok(())
    }
    fn visit_while(&mut self, cond: &Expr, body: &Stmt) -> VisitorResult<()> {
        self.resolve_expr(cond)?;
        self.resolve_stmt(body)?;
        Ok(())
    }
    fn visit_class(&mut self, class: &ClassStmt) -> VisitorResult<()> {
        let enclosing_class = self.cur_class;
        self.cur_class = ClassType::Class;
        self.declare(&class.name)?;
        self.define(&class.name);
        if let Some(superclass) = &class.superclass {
            let Expr::Variable(variable) = superclass else {
                unreachable!()
            };
            if variable.name.lexeme == class.name.lexeme {
                return Err(ResolverError::InheritFromSelf(class.name.clone()).into());
            }
            self.resolve_expr(superclass)?;
            self.begin_scope();
            self.scopes
                .last_mut()
                .unwrap()
                .insert("super".to_owned(), true);
        }
        self.begin_scope();
        self.scopes
            .last_mut()
            .unwrap()
            .insert("this".to_owned(), true);
        for method in class.methods.iter() {
            let ftype = if method.name.lexeme == "init" {
                FunctionType::Initializer
            } else {
                FunctionType::Method
            };
            self.resolve_function(
                &method.name,
                Rc::clone(&method.params),
                Rc::clone(&method.body),
                ftype,
            )?;
        }
        self.end_scope();
        if class.superclass.is_some() {
            self.end_scope();
        }
        self.cur_class = enclosing_class;
        Ok(())
    }
}
impl ExprVisitor for Resolver {
    fn visit_assign(&mut self, assign: &Assign) -> VisitorResult<Literal> {
        self.resolve_expr(&assign.value)?;
        self.resolve_local(assign)?;
        Ok(Literal::Nil)
    }
    fn visit_binary(&mut self, _: &Token, e1: &Expr, e2: &Expr) -> VisitorResult<Literal> {
        self.resolve_expr(e1)?;
        self.resolve_expr(e2)?;
        Ok(Literal::Nil)
    }
    fn visit_call(&mut self, callee: &Expr, _: &Token, args: &[Expr]) -> VisitorResult<Literal> {
        self.resolve_expr(callee)?;
        for arg in args {
            self.resolve_expr(arg)?;
        }
        Ok(Literal::Nil)
    }
    fn visit_grouping(&mut self, expr: &Expr) -> VisitorResult<Literal> {
        self.resolve_expr(expr)?;
        Ok(Literal::Nil)
    }
    fn visit_literal(&mut self, _literal: &Literal) -> VisitorResult<Literal> {
        Ok(Literal::Nil)
    }
    fn visit_logical(&mut self, left: &Expr, _: &Token, right: &Expr) -> VisitorResult<Literal> {
        self.resolve_expr(left)?;
        self.resolve_expr(right)?;
        Ok(Literal::Nil)
    }
    fn visit_unary(&mut self, _: &Token, expr: &Expr) -> VisitorResult<Literal> {
        self.resolve_expr(expr)?;
        Ok(Literal::Nil)
    }
    fn visit_variable(&mut self, variable: &Variable) -> VisitorResult<Literal> {
        if !self.scopes.is_empty()
            && self
                .scopes
                .last()
                .unwrap()
                .get(&variable.name.lexeme)
                .map_or(false, |v| !v)
        {
            return Err(ResolverError::NotInitialized(variable.name.clone()).into());
        }
        self.resolve_local(variable)?;
        Ok(Literal::Nil)
    }
    fn visit_get(&mut self, get: &Get) -> VisitorResult<Literal> {
        self.resolve_expr(&get.object)?;
        Ok(Literal::Nil)
    }
    fn visitor_set(&mut self, set: &Set) -> VisitorResult<Literal> {
        self.resolve_expr(&set.value)?;
        self.resolve_expr(&set.object)?;
        Ok(Literal::Nil)
    }
    fn visit_this(&mut self, this: &This) -> VisitorResult<Literal> {
        if self.cur_class == ClassType::None {
            return Err(ResolverError::InvalidThis(this.token.clone()).into());
        }
        self.resolve_local(this)?;
        Ok(Literal::Nil)
    }
    fn visit_super(&mut self, s: &Super) -> VisitorResult<Literal> {
        self.resolve_local(s)?;
        Ok(Literal::Nil)
    }
}

pub trait Resolvable {
    fn name(&self) -> &Token;
    fn set_dist(&self, dist: usize);
    fn get_dist(&self) -> Option<usize>;
}
