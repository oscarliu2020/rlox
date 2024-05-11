use super::ast::{Expr, Stmt};
use crate::scanner::{token::TokenType, Literal, Token};
fn error(t: &Token, msg: &str) {
    println!("[Runtime Error]line {}: {} ** {msg}", t.line, t.lexeme);
}
trait ExprVisitor {
    fn visit_binary(&self, token: &Token, e1: &Expr, e2: &Expr) -> Result<Literal, ()>;
    fn visit_grouping(&self, expr: &Expr) -> Result<Literal, ()>;
    fn visit_literal(&self, ltr: &Literal) -> Result<Literal, ()>;
    fn visit_unary(&self, token: &Token, expr: &Expr) -> Result<Literal, ()>;
}
trait StmtVisitor {
    fn visit_expression(&self, expr: &Expr) -> Result<(), ()>;
    fn visit_print(&self, expr: &Expr) -> Result<(), ()>;
}
pub struct Interpreter();
impl Interpreter {
    pub fn interpret(&self, stmts: &[Stmt]) {
        for stmt in stmts {
            if let Err(_) = self.execute(stmt) {
                break;
            }
        }
    }
    fn evaluate(&self, expr: &Expr) -> Result<Literal, ()> {
        match expr {
            Expr::Literal(ltr) => self.visit_literal(ltr),
            Expr::Grouping(expr) => self.visit_grouping(expr),
            Expr::Unary(token, expr) => self.visit_unary(token, expr),
            Expr::Binary(e1, token, e2) => self.visit_binary(token, e1, e2),
        }
    }
    fn execute(&self, stmt: &Stmt) -> Result<(), ()> {
        match stmt {
            Stmt::Expression(expr) => self.visit_expression(expr),
            Stmt::Print(expr) => self.visit_print(expr),
        }
    }
}
impl StmtVisitor for Interpreter {
    fn visit_expression(&self, expr: &Expr) -> Result<(), ()> {
        if let Ok(literal) = self.evaluate(expr) {
            println!("{}", literal);
            Ok(())
        } else {
            Err(())
        }
    }
    fn visit_print(&self, expr: &Expr) -> Result<(), ()> {
        if let Ok(literal) = self.evaluate(expr) {
            println!("{}", literal);
            Ok(())
        } else {
            Err(())
        }
    }
}
impl ExprVisitor for Interpreter {
    fn visit_literal(&self, ltr: &Literal) -> Result<Literal, ()> {
        Ok(ltr.clone())
    }
    fn visit_grouping(&self, expr: &Expr) -> Result<Literal, ()> {
        self.evaluate(expr)
    }
    fn visit_unary(&self, token: &Token, expr: &Expr) -> Result<Literal, ()> {
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
    fn visit_binary(&self, token: &Token, e1: &Expr, e2: &Expr) -> Result<Literal, ()> {
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
}
