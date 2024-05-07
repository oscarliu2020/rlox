#[derive(Debug, Clone, Copy)]
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
