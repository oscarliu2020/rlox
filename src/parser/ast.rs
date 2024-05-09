use crate::scanner::token::TokenType;
use crate::scanner::Literal;
use crate::scanner::Token;
use std::fmt::{Display, Formatter};
use std::mem;
pub trait Interpreter {
    fn interpret(&self);
}
pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Literal),
    Unary(Token, Box<Expr>),
}
impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Literal(ltr) => {
                write!(f, "{}", ltr)
            }
            Expr::Grouping(expr) => {
                write!(f, "( group {})", expr)
            }
            Expr::Unary(tok, expr) => {
                write!(f, "({} {})", tok.lexeme, expr)
            }
            Expr::Binary(left, tok, right) => {
                write!(f, "({} {} {})", tok.lexeme, left, right)
            }
        }
    }
}
fn error(t: &Token, msg: &str) {
    println!("[Runtime Error]line {}: {} ** {msg}", t.line, t.lexeme);
    todo!()
}
impl Expr {
    fn evaluate(&self) -> Result<Literal, ()> {
        match self {
            Expr::Literal(ltr) => Ok(ltr.clone()),
            Expr::Grouping(expr) => expr.evaluate(),
            Expr::Unary(t, expr) => {
                let right = expr.evaluate()?;
                match t.token_type {
                    TokenType::MINUS => match right {
                        Literal::Number(n) => Ok(Literal::Number(-n)),
                        _ => {
                            error(t, "Unary - must be used with a number");
                            Err(())
                        }
                    },
                    TokenType::BANG => Ok(Literal::Boolean(!right.is_truthy())),
                    _ => {
                        error(t, "Unknown unary operator");
                        Err(())
                    }
                }
            }
            Expr::Binary(e1, t, e2) => {
                let l = e1.evaluate()?;
                let r = e2.evaluate()?;
                match t.token_type {
                    TokenType::PLUS => match (l, r) {
                        (Literal::Number(n1), Literal::Number(n2)) => Ok(Literal::Number(n1 + n2)),
                        _ => {
                            error(t, "+ must be used with two numbers");
                            Err(())
                        }
                    },
                    TokenType::MINUS => match (l, r) {
                        (Literal::Number(n1), Literal::Number(n2)) => Ok(Literal::Number(n1 - n2)),
                        _ => {
                            error(t, "- must be used with two numbers");
                            Err(())
                        }
                    },
                    TokenType::STAR => match (l, r) {
                        (Literal::Number(n1), Literal::Number(n2)) => Ok(Literal::Number(n1 * n2)),
                        _ => {
                            error(t, "* must be used with two numbers");
                            Err(())
                        }
                    },
                    TokenType::SLASH => match (l, r) {
                        (Literal::Number(n1), Literal::Number(n2)) => Ok(Literal::Number(n1 / n2)),
                        _ => {
                            error(t, "/ must be used with two numbers");
                            Err(())
                        }
                    },
                    TokenType::GREATER => match (l, r) {
                        (Literal::Number(n1), Literal::Number(n2)) => Ok(Literal::Boolean(n1 > n2)),
                        _ => {
                            error(t, "> must be used with two numbers");
                            Err(())
                        }
                    },
                    TokenType::GREATER_EQUAL => match (l, r) {
                        (Literal::Number(n1), Literal::Number(n2)) => {
                            Ok(Literal::Boolean(n1 >= n2))
                        }
                        _ => {
                            error(t, ">= must be used with two numbers");
                            Err(())
                        }
                    },
                    TokenType::LESS => match (l, r) {
                        (Literal::Number(n1), Literal::Number(n2)) => Ok(Literal::Boolean(n1 < n2)),
                        _ => {
                            error(t, "< must be used with two numbers");
                            Err(())
                        }
                    },
                    TokenType::LESS_EQUAL => match (l, r) {
                        (Literal::Number(n1), Literal::Number(n2)) => {
                            Ok(Literal::Boolean(n1 <= n2))
                        }
                        _ => {
                            error(t, "<= must be used with two numbers");
                            Err(())
                        }
                    },
                    TokenType::BANG_EQUAL => Ok(Literal::Boolean(l != r)),
                    TokenType::EQUAL_EQUAL => Ok(Literal::Boolean(l == r)),
                    _ => {
                        error(t, "Unknown binary operator");
                        Err(())
                    }
                }
            }
        }
    }
}
impl Interpreter for Expr {
    fn interpret(&self) {
        match self.evaluate() {
            Ok(literal) => println!("{}", literal),
            Err(_) => {}
        }
    }
}
pub enum Stmt {
    Expression(Expr),
    Print(Expr),
}
impl Stmt {
    pub fn execute(&self) -> Result<(), ()> {
        match self {
            Stmt::Expression(expr) => expr.evaluate().map(|_| ()),
            Stmt::Print(expr) => match expr.evaluate() {
                Ok(literal) => {
                    println!("{}", literal.to_string());
                    Ok(())
                }
                Err(_) => Err(()),
            },
        }
    }
}
impl Interpreter for &[Stmt] {
    fn interpret(&self) {
        for stmt in *self {
            match stmt.execute() {
                Ok(_) => {}
                Err(_) => {}
            }
        }
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn test_print() {
        use super::*;
        let expr = Expr::Binary(
            Box::new(Expr::Unary(
                Token {
                    token_type: crate::scanner::token::TokenType::MINUS,
                    lexeme: "-".to_string(),
                    literal: None,
                    line: 1,
                },
                Box::new(Expr::Literal(Literal::Number(123.0))),
            )),
            Token {
                token_type: crate::scanner::token::TokenType::STAR,
                lexeme: "*".to_string(),
                literal: None,
                line: 1,
            },
            Box::new(Expr::Grouping(Box::new(Expr::Literal(Literal::Number(
                45.67,
            ))))),
        );
        println!("{}", expr);
        expr.interpret();
        let expr = Expr::Binary(
            Box::new(Expr::Literal(Literal::Number(1.0111))),
            Token {
                token_type: crate::scanner::token::TokenType::PLUS,
                lexeme: "+".to_string(),
                literal: None,
                line: 1,
            },
            Box::new(Expr::Literal(Literal::Boolean(true))),
        );
        println!("{}", expr);
        expr.interpret();
    }
    #[test]
    fn test_float() {
        println!("{}", 1.23.to_string());
    }
}
