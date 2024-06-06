use crate::syntax::ast::{Expr, ExprVisitor, Stmt, StmtVisitor};
use crate::syntax::token::{Literal, Token, TokenType};
fn error(t: &Token, msg: &str) {
    println!("[Runtime Error]line {}: {} ** {msg}", t.line, t.lexeme);
}

#[derive(Default)]
pub struct Interpreter {
    environment: Environment,
}
use crate::syntax::ast::{VisitorError, VisitorResult};
impl Interpreter {
    pub fn interpret(&mut self, stmts: &[Option<Stmt>]) {
        for stmt in stmts {
            if self.execute(stmt.as_ref().unwrap()).is_err() {
                break;
            }
        }
    }
    fn evaluate(&mut self, expr: &Expr) -> VisitorResult<Literal> {
        match expr {
            Expr::Literal(ltr) => self.visit_literal(ltr),
            Expr::Grouping(expr) => self.visit_grouping(expr),
            Expr::Unary(token, expr) => self.visit_unary(token, expr),
            Expr::Binary(e1, token, e2) => self.visit_binary(token, e1, e2),
            Expr::Variable(token) => self.visit_variable(token),
            Expr::Assign(token, expr) => self.visit_assign(token, expr),
        }
    }
    fn execute(&mut self, stmt: &Stmt) -> VisitorResult<()> {
        match stmt {
            Stmt::Block(stmts) => self.visit_block(stmts),
            Stmt::Expression(expr) => self.visit_expression(expr),
            Stmt::Print(expr) => self.visit_print(expr),
            Stmt::Var(token, expr) => self.visit_var(token, expr.as_ref()),
        }
    }
    fn execute_block(&mut self, stmts: &[Stmt]) -> VisitorResult<()> {
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
    fn visit_block(&mut self, stmts: &[Stmt]) -> VisitorResult<()> {
        self.execute_block(stmts)
    }
    fn visit_expression(&mut self, expr: &Expr) -> VisitorResult<()> {
        if let Ok(literal) = self.evaluate(expr) {
            println!("{}", literal);
            Ok(())
        } else {
            Err(VisitorError::VistorError)
        }
    }
    fn visit_print(&mut self, expr: &Expr) -> VisitorResult<()> {
        if let Ok(literal) = self.evaluate(expr) {
            println!("{}", literal);
            Ok(())
        } else {
            Err(VisitorError::VistorError)
        }
    }
    fn visit_var(&mut self, token: &Token, expr: Option<&Expr>) -> VisitorResult<()> {
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
    fn visit_literal(&mut self, ltr: &Literal) -> VisitorResult<Literal> {
        Ok(ltr.clone())
    }
    fn visit_variable(&mut self, token: &Token) -> VisitorResult<Literal> {
        self.environment.get(token)
    }
    fn visit_grouping(&mut self, expr: &Expr) -> VisitorResult<Literal> {
        self.evaluate(expr)
    }
    fn visit_unary(&mut self, token: &Token, expr: &Expr) -> VisitorResult<Literal> {
        let right = self.evaluate(expr)?;
        match token.token_type {
            TokenType::MINUS => match right {
                Literal::Number(n) => Ok(Literal::Number(-n)),
                _ => {
                    error(token, "Unary - must be used with a number");
                    Err(VisitorError::VistorError)
                }
            },
            TokenType::BANG => Ok(Literal::Boolean(!right.is_truthy())),
            _ => {
                error(token, "Unknown unary operator");
                Err(VisitorError::VistorError)
            }
        }
    }
    fn visit_binary(&mut self, token: &Token, e1: &Expr, e2: &Expr) -> VisitorResult<Literal> {
        let l = self.evaluate(e1)?;
        let r = self.evaluate(e2)?;
        match token.token_type {
            TokenType::PLUS => match (l, r) {
                (Literal::Number(n1), Literal::Number(n2)) => Ok(Literal::Number(n1 + n2)),
                _ => {
                    error(token, "Operands must be two numbers");
                    Err(VisitorError::VistorError)
                }
            },
            TokenType::MINUS => match (l, r) {
                (Literal::Number(n1), Literal::Number(n2)) => Ok(Literal::Number(n1 - n2)),
                _ => {
                    error(token, "Operands must be two numbers");
                    Err(VisitorError::VistorError)
                }
            },
            TokenType::STAR => match (l, r) {
                (Literal::Number(n1), Literal::Number(n2)) => Ok(Literal::Number(n1 * n2)),
                _ => {
                    error(token, "Operands must be two numbers");
                    Err(VisitorError::VistorError)
                }
            },
            TokenType::SLASH => match (l, r) {
                (Literal::Number(n1), Literal::Number(n2)) => Ok(Literal::Number(n1 / n2)),
                _ => {
                    error(token, "Operands must be two numbers");
                    Err(VisitorError::VistorError)
                }
            },
            TokenType::GREATER => match (l, r) {
                (Literal::Number(n1), Literal::Number(n2)) => Ok(Literal::Boolean(n1 > n2)),
                _ => {
                    error(token, "Operands must be two numbers");
                    Err(VisitorError::VistorError)
                }
            },
            TokenType::GREATER_EQUAL => match (l, r) {
                (Literal::Number(n1), Literal::Number(n2)) => Ok(Literal::Boolean(n1 >= n2)),
                _ => {
                    error(token, "Operands must be two numbers");
                    Err(VisitorError::VistorError)
                }
            },
            TokenType::LESS => match (l, r) {
                (Literal::Number(n1), Literal::Number(n2)) => Ok(Literal::Boolean(n1 < n2)),
                _ => {
                    error(token, "Operands must be two numbers");
                    Err(VisitorError::VistorError)
                }
            },
            TokenType::LESS_EQUAL => match (l, r) {
                (Literal::Number(n1), Literal::Number(n2)) => Ok(Literal::Boolean(n1 <= n2)),
                _ => {
                    error(token, "Operands must be two numbers");
                    Err(VisitorError::VistorError)
                }
            },
            TokenType::BANG_EQUAL => Ok(Literal::Boolean(l != r)),
            TokenType::EQUAL_EQUAL => Ok(Literal::Boolean(l == r)),
            _ => {
                error(token, "Unknown binary operator");
                Err(VisitorError::VistorError)
            }
        }
    }
    fn visit_assign(&mut self, token: &Token, expr: &Expr) -> VisitorResult<Literal> {
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
    pub fn get(&self, name: &Token) -> Result<Literal, VisitorError> {
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
                VisitorError::EnvironmentError
            })
    }
    pub fn assign(&mut self, name: &Token, value: Literal) -> Result<(), VisitorError> {
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
                VisitorError::EnvironmentError
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
    #[test]
    fn test_blcok2() {
        let mut interpreter = Interpreter::default();
        run(
            r"
            var a=1;
            {
                var a=a+3;
                print a;
            }
            print a;
        ",
            &mut interpreter,
        );
    }
}
