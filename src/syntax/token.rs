use std::fmt::{self};
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
use TokenType::*;
pub fn get_keywords(s: impl AsRef<str>) -> Option<TokenType> {
    get_keyword_impl(s.as_ref())
}
pub use super::literal::*;
#[derive(Debug, Clone, PartialEq)]
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

macro_rules! define_keywords {
    ($($x:expr => $y:ident),* $(,)?) => {
        fn get_keyword_impl(keyword:&str)->Option<TokenType>{
            match keyword {
                $($x=>Some($y),)*
                _=>None
            }
        }
    };
}
define_keywords!(
    "and"=>AND,
    "class"=>CLASS,
    "else"=>ELSE,
    "false"=>FALSE,
    "for"=>FOR,
    "fun"=>FUN,
    "if"=>IF,
    "nil"=>NIL,
    "or"=>OR,
    "print"=>PRINT,
    "return"=>RETURN,
    "super"=>SUPER,
    "this"=>THIS,
    "true"=>TRUE,
    "var"=>VAR,
    "while"=>WHILE
);
