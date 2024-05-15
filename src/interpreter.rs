use crate::syntax::ast::{Expr, ExprVisitor, Stmt, StmtVisitor};
use crate::syntax::token::{Literal, Token, TokenType};
fn error(t: &Token, msg: &str) {
    println!("[Runtime Error]line {}: {} ** {msg}", t.line, t.lexeme);
}

#[derive(Default)]
pub struct Interpreter {
    environment: Environment,
}
impl Interpreter {
    pub fn interpret(&mut self, stmts: &[Option<Stmt>]) {
        for stmt in stmts {
            if self.execute(stmt.as_ref().unwrap()).is_err() {
                break;
            }
        }
    }
    fn evaluate(&mut self, expr: &Expr) -> Result<Literal, ()> {
        match expr {
            Expr::Literal(ltr) => self.visit_literal(ltr),
            Expr::Grouping(expr) => self.visit_grouping(expr),
            Expr::Unary(token, expr) => self.visit_unary(token, expr),
            Expr::Binary(e1, token, e2) => self.visit_binary(token, e1, e2),
            Expr::Variable(token) => self.visit_variable(token),
            Expr::Assign(token, expr) => self.visit_assign(token, expr),
        }
    }
    fn execute(&mut self, stmt: &Stmt) -> Result<(), ()> {
        match stmt {
            Stmt::Block(stmts) => self.visit_block(stmts),
            Stmt::Expression(expr) => self.visit_expression(expr),
            Stmt::Print(expr) => self.visit_print(expr),
            Stmt::Var(token, expr) => self.visit_var(token, expr.as_ref()),
        }
    }
    fn execute_block(&mut self, stmts: &[Stmt]) -> Result<(), ()> {
        let prev = self.environment.clone();
        self.environment = Environment::new(Some(Box::new(prev)));
        for stmt in stmts {
            if self.execute(stmt).is_err() {
                break;
            }
        }
        self.environment = *self.environment.enclosing.take().unwrap();
        Ok(())
    }
}
impl StmtVisitor for Interpreter {
    fn visit_block(&mut self, stmts: &[Stmt]) -> Result<(), ()> {
        self.execute_block(stmts)
    }
    fn visit_expression(&mut self, expr: &Expr) -> Result<(), ()> {
        if let Ok(literal) = self.evaluate(expr) {
            println!("{}", literal);
            Ok(())
        } else {
            Err(())
        }
    }
    fn visit_print(&mut self, expr: &Expr) -> Result<(), ()> {
        if let Ok(literal) = self.evaluate(expr) {
            println!("{}", literal);
            Ok(())
        } else {
            Err(())
        }
    }
    fn visit_var(&mut self, token: &Token, expr: Option<&Expr>) -> Result<(), ()> {
        let value = if let Some(expr) = expr {
            self.evaluate(expr)?
        } else {
            Literal::Nil
        };
        self.environment.define(token.lexeme.clone(), value);
        Ok(())
    }
}
impl ExprVisitor for Interpreter {
    fn visit_literal(&mut self, ltr: &Literal) -> Result<Literal, ()> {
        Ok(ltr.clone())
    }
    fn visit_variable(&mut self, token: &Token) -> Result<Literal, ()> {
        self.environment.get(token)
    }
    fn visit_grouping(&mut self, expr: &Expr) -> Result<Literal, ()> {
        self.evaluate(expr)
    }
    fn visit_unary(&mut self, token: &Token, expr: &Expr) -> Result<Literal, ()> {
        let right = self.evaluate(expr)?;
        match token.token_type {
            TokenType::MINUS => match right {
                Literal::Number(n) => Ok(Literal::Number(-n)),
                _ => {
                    error(token, "Unary - must be used with a number");
                    Err(())
                }
            },
            TokenType::BANG => Ok(Literal::Boolean(!right.is_truthy())),
            _ => {
                error(token, "Unknown unary operator");
                Err(())
            }
        }
    }
    fn visit_binary(&mut self, token: &Token, e1: &Expr, e2: &Expr) -> Result<Literal, ()> {
        let l = self.evaluate(e1)?;
        let r = self.evaluate(e2)?;
        match token.token_type {
            TokenType::PLUS => match (l, r) {
                (Literal::Number(n1), Literal::Number(n2)) => Ok(Literal::Number(n1 + n2)),
                _ => {
                    error(token, "Operands must be two numbers");
                    Err(())
                }
            },
            TokenType::MINUS => match (l, r) {
                (Literal::Number(n1), Literal::Number(n2)) => Ok(Literal::Number(n1 - n2)),
                _ => {
                    error(token, "Operands must be two numbers");
                    Err(())
                }
            },
            TokenType::STAR => match (l, r) {
                (Literal::Number(n1), Literal::Number(n2)) => Ok(Literal::Number(n1 * n2)),
                _ => {
                    error(token, "Operands must be two numbers");
                    Err(())
                }
            },
            TokenType::SLASH => match (l, r) {
                (Literal::Number(n1), Literal::Number(n2)) => Ok(Literal::Number(n1 / n2)),
                _ => {
                    error(token, "Operands must be two numbers");
                    Err(())
                }
            },
            TokenType::GREATER => match (l, r) {
                (Literal::Number(n1), Literal::Number(n2)) => Ok(Literal::Boolean(n1 > n2)),
                _ => {
                    error(token, "Operands must be two numbers");
                    Err(())
                }
            },
            TokenType::GREATER_EQUAL => match (l, r) {
                (Literal::Number(n1), Literal::Number(n2)) => Ok(Literal::Boolean(n1 >= n2)),
                _ => {
                    error(token, "Operands must be two numbers");
                    Err(())
                }
            },
            TokenType::LESS => match (l, r) {
                (Literal::Number(n1), Literal::Number(n2)) => Ok(Literal::Boolean(n1 < n2)),
                _ => {
                    error(token, "Operands must be two numbers");
                    Err(())
                }
            },
            TokenType::LESS_EQUAL => match (l, r) {
                (Literal::Number(n1), Literal::Number(n2)) => Ok(Literal::Boolean(n1 <= n2)),
                _ => {
                    error(token, "Operands must be two numbers");
                    Err(())
                }
            },
            TokenType::BANG_EQUAL => Ok(Literal::Boolean(l != r)),
            TokenType::EQUAL_EQUAL => Ok(Literal::Boolean(l == r)),
            _ => {
                error(token, "Unknown binary operator");
                Err(())
            }
        }
    }
    fn visit_assign(&mut self, token: &Token, expr: &Expr) -> Result<Literal, ()> {
        let value = self.evaluate(expr)?;
        self.environment.assign(token, value.clone())?;
        Ok(value)
    }
}
use rustc_hash::FxHashMap;
#[derive(Default, Clone)]
pub struct Environment {
    values: FxHashMap<String, Literal>,
    enclosing: Option<Box<Environment>>,
}
impl Environment {
    pub fn new(enclosing: Option<Box<Environment>>) -> Self {
        Self {
            values: FxHashMap::default(),
            enclosing,
        }
    }
    pub fn define(&mut self, name: String, value: Literal) {
        self.values.insert(name, value);
    }
    pub fn get(&self, name: &Token) -> Result<Literal, ()> {
        self.values
            .get(&name.lexeme)
            .cloned()
            .or_else(|| {
                self.enclosing
                    .as_ref()
                    .and_then(|enclosing| enclosing.get(name).ok())
            })
            .ok_or_else(|| {
                error(name, "Undefined variable");
            })
    }
    pub fn assign(&mut self, name: &Token, value: Literal) -> Result<(), ()> {
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
                error(name, "Undefined variable");
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::run;
    #[test]
    fn test_block() {
        let mut interpreter = Interpreter::default();
        run(
            r#"var a = "global a";
        var b = "global b";
        var c = "global c";
        {
          var a = "outer a";
          var b = "outer b";
          {
            var a = "inner a";
            print a;
            print b;
            print c;
          }
          print a;
          print b;
          print c;
        }
        print a;
        print b;
        print c;"#,
            &mut interpreter,
        );
    }
}
