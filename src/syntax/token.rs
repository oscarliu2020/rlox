use std::fmt::{self, Display};
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    // Single-character tokens.
    LEFT_PAREN,
    RIGHT_PAREN,
    LEFT_BRACE,
    RIGHT_BRACE,
    COMMA,
    DOT,
    MINUS,
    PLUS,
    SEMICOLON,
    SLASH,
    STAR,

    // One or two character tokens.
    BANG,
    BANG_EQUAL,
    EQUAL,
    EQUAL_EQUAL,
    GREATER,
    GREATER_EQUAL,
    LESS,
    LESS_EQUAL,

    // Literals.
    IDENTIFIER,
    STRING,
    NUMBER,

    // Keywords.
    AND,
    CLASS,
    ELSE,
    FALSE,
    FUN,
    FOR,
    IF,
    NIL,
    OR,
    PRINT,
    RETURN,
    SUPER,
    THIS,
    TRUE,
    VAR,
    WHILE,

    EOF,
}

use lazy_static::lazy_static;
use rustc_hash::FxHashMap;
use TokenType::*;
lazy_static! {
    pub(super) static ref KEYWORDS: FxHashMap<&'static str, TokenType> = {
        let mut keywords = FxHashMap::default();
        keywords.insert("and", AND);
        keywords.insert("class", CLASS);
        keywords.insert("else", ELSE);
        keywords.insert("false", FALSE);
        keywords.insert("for", FOR);
        keywords.insert("fun", FUN);
        keywords.insert("if", IF);
        keywords.insert("nil", NIL);
        keywords.insert("or", OR);
        keywords.insert("print", PRINT);
        keywords.insert("return", RETURN);
        keywords.insert("super", SUPER);
        keywords.insert("this", THIS);
        keywords.insert("true", TRUE);
        keywords.insert("var", VAR);
        keywords.insert("while", WHILE);
        keywords
    };
}
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}
impl Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Nil => write!(f, "nil"),
            Literal::Boolean(b) => write!(f, "{}", b),
            Literal::Number(n) => write!(f, "{:.}", *n),
            Literal::String(s) => write!(f, "\"{}\"", s),
        }
    }
}
impl Literal {
    pub fn is_truthy(&self) -> bool {
        match self {
            Literal::Nil => false,
            Literal::Boolean(b) => *b,
            _ => true,
        }
    }
}
#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<Literal>,
    pub line: usize,
}
impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?} {} {:?}",
            self.token_type, self.lexeme, self.literal
        )
    }
}
