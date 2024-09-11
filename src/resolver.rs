use super::syntax::{ast::*, token::*};
use rustc_hash::FxHashMap;
pub struct Resolver {
    scopes: Vec<FxHashMap<String, bool>>,
}
use thiserror::Error;
#[derive(Error, Debug)]
pub enum ResolverError {
    #[error("Variable {0} not initialized")]
    NotInitialized(Token),
    #[error("Already a variable with this name in this scope.")]
    AlreadyDeclared(Token),
}
impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}

impl Resolver {
    pub fn new() -> Self {
        Resolver { scopes: vec![] }
    }
    pub fn resolve(&mut self, stmts: &mut [Stmt]) -> VisitorResult<()> {
        for stmt in stmts {
            self.resolve_stmt(stmt)?;
        }
        Ok(())
    }
    fn resolve_stmt(&mut self, stmt: &mut Stmt) -> VisitorResult<()> {
        stmt.accept(self)
    }
    fn resolve_expr(&mut self, expr: &mut Expr) -> VisitorResult<()> {
        expr.accept(self).map(|_| ())
    }
    fn resolve_local(&mut self, token: &mut impl Resolvable) -> VisitorResult<()> {
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
        params: &[Token],
        body: &mut [Stmt],
    ) -> VisitorResult<()> {
        self.begin_scope();
        for param in params {
            self.declare(param)?;
            self.define(param);
        }
        self.resolve(body)?;
        self.end_scope();
        Ok(())
    }
}
impl StmtVisitor for Resolver {
    fn visit_block(&mut self, stmts: &mut [Stmt]) -> VisitorResult<()> {
        self.begin_scope();
        self.resolve(stmts)?;
        self.end_scope();
        Ok(())
    }
    fn visit_expression(&mut self, expr: &mut Expr) -> VisitorResult<()> {
        self.resolve_expr(expr)
    }
    fn visit_function(
        &mut self,
        name: &crate::syntax::token::Token,
        params: &[crate::syntax::token::Token],
        body: &mut [Stmt],
    ) -> VisitorResult<()> {
        self.declare(name)?;
        self.define(name);
        self.resolve_function(name, params, body)
    }
    fn visit_if(&mut self, cond: &mut Expr, body: &mut (Stmt, Option<Stmt>)) -> VisitorResult<()> {
        self.resolve_expr(cond)?;
        self.resolve_stmt(&mut body.0)?;
        if let Some(else_body) = &mut body.1 {
            self.resolve_stmt(else_body)?;
        }
        Ok(())
    }
    fn visit_print(&mut self, expr: &mut Expr) -> VisitorResult<()> {
        self.resolve_expr(expr)
    }
    fn visit_return(
        &mut self,
        _: &crate::syntax::token::Token,
        expr: Option<&mut Expr>,
    ) -> VisitorResult<()> {
        if let Some(expr) = expr {
            self.resolve_expr(expr)?;
        }
        Ok(())
    }
    fn visit_var(
        &mut self,
        token: &crate::syntax::token::Token,
        expr: Option<&mut Expr>,
    ) -> VisitorResult<()> {
        self.declare(token)?;
        if let Some(expr) = expr {
            self.resolve_expr(expr)?;
        }
        self.define(token);
        Ok(())
    }
    fn visit_while(&mut self, cond: &mut Expr, body: &mut Stmt) -> VisitorResult<()> {
        self.resolve_expr(cond)?;
        self.resolve_stmt(body)?;
        Ok(())
    }
}
impl ExprVisitor for Resolver {
    fn visit_assign(&mut self, assign: &mut Assign) -> VisitorResult<Literal> {
        self.resolve_expr(&mut assign.value)?;
        self.resolve_local(assign)?;
        Ok(Literal::Nil)
    }
    fn visit_binary(&mut self, _: &Token, e1: &mut Expr, e2: &mut Expr) -> VisitorResult<Literal> {
        self.resolve_expr(e1)?;
        self.resolve_expr(e2)?;
        Ok(Literal::Nil)
    }
    fn visit_call(
        &mut self,
        callee: &mut Expr,
        _: &Token,
        args: &mut [Expr],
    ) -> VisitorResult<Literal> {
        self.resolve_expr(callee)?;
        for arg in args {
            self.resolve_expr(arg)?;
        }
        Ok(Literal::Nil)
    }
    fn visit_grouping(&mut self, expr: &mut Expr) -> VisitorResult<Literal> {
        self.resolve_expr(expr)?;
        Ok(Literal::Nil)
    }
    fn visit_literal(&mut self, _literal: &Literal) -> VisitorResult<Literal> {
        Ok(Literal::Nil)
    }
    fn visit_logical(
        &mut self,
        left: &mut Expr,
        _: &Token,
        right: &mut Expr,
    ) -> VisitorResult<Literal> {
        self.resolve_expr(left)?;
        self.resolve_expr(right)?;
        Ok(Literal::Nil)
    }
    fn visit_unary(&mut self, _: &Token, expr: &mut Expr) -> VisitorResult<Literal> {
        self.resolve_expr(expr)?;
        Ok(Literal::Nil)
    }
    fn visit_variable(&mut self, variable: &mut Variable) -> VisitorResult<Literal> {
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
}

pub trait Resolvable {
    fn name(&self) -> &Token;
    fn set_dist(&mut self, dist: usize);
    fn get_dist(&self) -> Option<usize>;
}
