use crate::scanner::Literal;
use crate::scanner::Token;
use std::fmt::{Display, Formatter};
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
}

pub enum Stmt {
    Expression(Expr),
    Print(Expr),
}
