use super::interpreter::Interpreter;
use super::syntax::{ast::*, token::*};
use rustc_hash::FxHashMap;
pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<FxHashMap<String, bool>>,
}
impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Resolver {
            interpreter,
            scopes: vec![],
        }
    }
    pub fn resolve(&mut self, stmts: &[Stmt]) -> VisitorResult<()> {
        for stmt in stmts {
            self.resolve_stmt(stmt)?;
        }
        Ok(())
    }
    fn resolve_stmt(&mut self, stmt: &Stmt) -> VisitorResult<()> {
        match stmt {
            Stmt::Block(stmts) => self.visit_block(stmts),
            Stmt::Expression(expr) => self.visit_expression(expr),
            Stmt::Function(name, params, body) => self.visit_function(name, params, body),
            Stmt::IfStmt(cond, body) => self.visit_if(cond, body),
            Stmt::Print(expr) => self.visit_print(expr),
            Stmt::Return(token, expr) => self.visit_return(token, expr.as_ref()),
            Stmt::Var(token, expr) => self.visit_var(token, expr.as_ref()),
            Stmt::WhileStmt(cond, body) => self.visit_while(cond, body),
        }
    }
    fn resolve_expr(&mut self, expr: &Expr) -> VisitorResult<()> {
        match expr {
            Expr::Assign(token, expr) => self.visit_assign(token, expr),
            Expr::Binary(e1, token, e2) => self.visit_binary(token, e1, e2),
            Expr::Call(callee, paren, args) => self.visit_call(callee, paren, args),
            Expr::Grouping(expr) => self.visit_grouping(expr),
            Expr::Literal(literal) => self.visit_literal(literal),
            Expr::Logical(left, token, right) => self.visit_logical(left, token, right),
            Expr::Unary(token, expr) => self.visit_unary(token, expr),
            Expr::Variable(token) => self.visit_variable(token),
        }
        .map(|_| ())
    }
    fn resolve_local(&mut self, token: &Token) -> VisitorResult<()> {
        for (i, scope) in self.scopes.iter().enumerate().rev() {
            if scope.contains_key(&token.lexeme) {
                self.interpreter.resolve(token, self.scopes.len() - 1 - i);
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
    fn declare(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }
        let scope = self.scopes.last_mut().unwrap();
        scope.insert(name.lexeme.clone(), false);
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
        body: &[Stmt],
    ) -> VisitorResult<()> {
        self.begin_scope();
        for param in params {
            self.declare(param);
            self.define(param);
        }
        self.resolve(body)?;
        self.end_scope();
        Ok(())
    }
}
impl StmtVisitor for Resolver<'_> {
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
        params: &[crate::syntax::token::Token],
        body: &[Stmt],
    ) -> VisitorResult<()> {
        self.declare(name);
        self.define(name);
        self.resolve_function(name, params, body)
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
        _: &crate::syntax::token::Token,
        expr: Option<&Expr>,
    ) -> VisitorResult<()> {
        if let Some(expr) = expr {
            self.resolve_expr(expr)?;
        }
        Ok(())
    }
    fn visit_var(
        &mut self,
        token: &crate::syntax::token::Token,
        expr: Option<&Expr>,
    ) -> VisitorResult<()> {
        self.declare(token);
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
}
impl ExprVisitor for Resolver<'_> {
    fn visit_assign(&mut self, token: &Token, expr: &Expr) -> VisitorResult<Literal> {
        self.resolve_expr(expr)?;
        self.resolve_local(token)?;
        Ok(Literal::Nil)
    }
    fn visit_binary(&mut self, token: &Token, e1: &Expr, e2: &Expr) -> VisitorResult<Literal> {
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
    fn visit_literal(&mut self, literal: &Literal) -> VisitorResult<Literal> {
        Ok(Literal::Nil)
    }
    fn visit_logical(
        &mut self,
        left: &Expr,
        token: &Token,
        right: &Expr,
    ) -> VisitorResult<Literal> {
        self.resolve_expr(left)?;
        self.resolve_expr(right)?;
        Ok(Literal::Nil)
    }
    fn visit_unary(&mut self, token: &Token, expr: &Expr) -> VisitorResult<Literal> {
        self.resolve_expr(expr)?;
        Ok(Literal::Nil)
    }
    fn visit_variable(&mut self, token: &Token) -> VisitorResult<Literal> {
        if !self.scopes.is_empty() && self.scopes.last().unwrap().get(&token.lexeme).is_none() {
            return Err(VisitorError::NotInitialized(token.clone()));
        }
        self.resolve_local(token)?;
        Ok(Literal::Nil)
    }
}
