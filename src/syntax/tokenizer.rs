use super::token;
// use crate::Result;
use thiserror::Error;
use token::{Literal, Token};
#[derive(Error, Debug)]
pub enum TokenizerError {
    #[error("Error at line {0}")]
    UnterminatedString(usize),
    #[error("Unexpected character at line {0}")]
    UnexpectedCharacter(usize),
}
pub struct Tokenizer {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}
impl Tokenizer {
    pub fn new(source: String) -> Self {
        Self {
            source: source.chars().collect(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }
    fn _add_token(&mut self, ty: token::TokenType, literal: Option<Literal>) {
        let text = self.source[self.start..self.current].iter().collect();
        self.tokens.push(Token {
            token_type: ty,
            lexeme: text,
            literal,
            line: self.line,
        });
    }
    fn add_token(&mut self, ty: token::TokenType) {
        self._add_token(ty, None);
    }
    fn peek_match(&mut self, expected: char) -> bool {
        let f = self
            .source
            .get(self.current)
            .copied()
            .unwrap_or(b'\0' as char)
            == expected;
        if f {
            self.current += 1;
        }
        f
    }
    fn peek(&self) -> char {
        self.source
            .get(self.current)
            .copied()
            .unwrap_or(b'\0' as char)
    }
    fn string(&mut self) -> Result<(), TokenizerError> {
        while (self.peek() != '"') && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        if self.is_at_end() {
            return Err(TokenizerError::UnterminatedString(self.line));
        }
        // closing
        self.advance();
        let value = self.source[self.start + 1..self.current - 1]
            .iter()
            .collect();
        self._add_token(token::TokenType::STRING, Some(Literal::String(value)));
        Ok(())
    }
    fn peek_next(&self) -> char {
        self.source
            .get(self.current + 1)
            .copied()
            .unwrap_or(b'\0' as char)
    }
    fn number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }
        self._add_token(
            token::TokenType::NUMBER,
            Some(Literal::Number(
                self.source[self.start..self.current]
                    .iter()
                    .collect::<String>()
                    .parse()
                    .unwrap(),
            )),
        );
    }
    fn identifier(&mut self) {
        while self.peek().is_ascii_alphanumeric() || self.peek() == '_' {
            self.advance();
        }
        let tt = self.source[self.start..self.current]
            .iter()
            .collect::<String>();
        let ty = token::get_keywords(tt).unwrap_or(token::TokenType::IDENTIFIER);
        self.add_token(ty)
    }
    pub fn scan_token(&mut self) -> Result<(), TokenizerError> {
        let c = self.advance();
        match c {
            '(' => self.add_token(token::TokenType::LEFT_PAREN),
            ')' => self.add_token(token::TokenType::RIGHT_PAREN),
            '{' => self.add_token(token::TokenType::LEFT_BRACE),
            '}' => self.add_token(token::TokenType::RIGHT_BRACE),
            ',' => self.add_token(token::TokenType::COMMA),
            '.' => self.add_token(token::TokenType::DOT),
            '-' => self.add_token(token::TokenType::MINUS),
            '+' => self.add_token(token::TokenType::PLUS),
            ';' => self.add_token(token::TokenType::SEMICOLON),
            '*' => self.add_token(token::TokenType::STAR),
            '!' => {
                let tt = if self.peek_match('=') {
                    token::TokenType::BANG_EQUAL
                } else {
                    token::TokenType::BANG
                };
                self.add_token(tt)
            }
            '=' => {
                let tt = if self.peek_match('=') {
                    token::TokenType::EQUAL_EQUAL
                } else {
                    token::TokenType::EQUAL
                };
                self.add_token(tt);
            }
            '<' => {
                let tt = if self.peek_match('=') {
                    token::TokenType::LESS_EQUAL
                } else {
                    token::TokenType::LESS
                };
                self.add_token(tt);
            }
            '>' => {
                let tt = if self.peek_match('=') {
                    token::TokenType::GREATER_EQUAL
                } else {
                    token::TokenType::GREATER
                };
                self.add_token(tt);
            }
            '/' => {
                if self.peek_match('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(token::TokenType::SLASH);
                }
            }
            ' ' | '\r' | '\t' => {
                // ignore whitespace
            }
            '\n' => {
                self.line += 1;
            }
            '"' => {
                self.string()?;
            }
            '0'..='9' => {
                self.number();
            }
            _ if c.is_ascii_alphabetic() || c == '_' => {
                self.identifier();
            }
            _ => {
                return Err(TokenizerError::UnexpectedCharacter(self.line));
            }
        }
        Ok(())
    }
    fn advance(&mut self) -> char {
        self.current += 1;
        self.source[self.current - 1]
    }
    pub fn scan_tokens(&mut self) -> Result<&[Token], TokenizerError> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
            // self.scan_token()?;
        }
        self.tokens.push(Token {
            token_type: token::TokenType::EOF,
            lexeme: "".to_string(),
            literal: None,
            line: self.line,
        });
        Ok(&self.tokens)
    }
    pub fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_scanner() {
        let mut scanner = Tokenizer::new("(abc=a+b)".to_string());
        println!("{:?}", scanner.scan_tokens().unwrap());
    }
    #[test]
    fn test_string() {
        let mut scanner = Tokenizer::new("\"abc\"".to_string());
        println!("{:?}", scanner.scan_tokens().unwrap());
    }
    #[test]
    fn test_number() {
        let mut scanner = Tokenizer::new(".1234".to_string());
        println!("{:?}", scanner.scan_tokens());
    }
    #[test]
    fn test_ident_and_keyw() {
        let mut scanner = Tokenizer::new("andand_ //abcde_\na".to_string());
        println!("{:?}", scanner.scan_tokens());
    }
    #[test]
    fn test_paren() {
        let mut scanner = Tokenizer::new(r#"print("Hello, World")"#.to_string());

        println!("{:#?}", scanner.scan_tokens());
    }
    #[test]
    fn test_fail() {
        let mut scanner = Tokenizer::new("1+1=2\n\"abc".to_string());
        println!("{:?}", scanner.scan_tokens().unwrap_err());
    }
}
