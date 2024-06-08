use super::ast;
use super::token::{Literal, Token, TokenType};
pub struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
}
use thiserror::Error;
#[derive(Debug, Error)]
#[error("ParserError")]
#[repr(transparent)]
pub struct ParserError();
macro_rules! match_token {
    ($self:ident, [$($token:pat_param),*]) => {
        match_token!($self, $($token),*)
    };
    ($self:ident, $($token:pat_param),*) => {
        {
            if $self.is_at_end() {
                false
            } else {
                match $self.peek().token_type {
                    $($token)|* => {
                        $self.advance();
                        true
                    }
                    _ => false
                }
            }

        }
    };
}
impl<'a> Parser<'a> {
    #[inline]
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, current: 0 }
    }
    #[inline]
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }
    #[inline]
    fn is_at_end(&self) -> bool {
        return self.peek().token_type == TokenType::EOF;
    }
    #[inline]
    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }
    #[inline]
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }
    #[inline]
    fn check(&self, ty: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        return self.peek().token_type == *ty;
    }
    #[inline]
    fn error(&self, t: &Token, msg: &str) {
        if t.token_type == TokenType::EOF {
            eprintln!("[line {}] Error at end: {}", t.line, msg);
        } else {
            eprintln!("[line {}] Error at '{}': {}", t.line, t.lexeme, msg);
        }
    }
    #[inline]
    fn consume(&mut self, ty: TokenType, msg: &str) -> Result<Token, ParserError> {
        if self.check(&ty) {
            return Ok(self.advance().clone());
        }
        self.error(self.peek(), msg);
        Err(ParserError())
    }
    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().token_type == TokenType::SEMICOLON {
                return;
            }
            match self.peek().token_type {
                TokenType::CLASS
                | TokenType::FUN
                | TokenType::VAR
                | TokenType::FOR
                | TokenType::IF
                | TokenType::WHILE
                | TokenType::PRINT
                | TokenType::RETURN => {
                    return;
                }
                _ => {}
            }
            self.advance();
        }
    }
    fn primary(&mut self) -> Result<ast::Expr, ParserError> {
        if match_token!(self, [TokenType::FALSE]) {
            return Ok(ast::Expr::Literal(Literal::Boolean(false)));
        }
        if match_token!(self, [TokenType::TRUE]) {
            return Ok(ast::Expr::Literal(Literal::Boolean(true)));
        }
        if match_token!(self, [TokenType::NIL]) {
            return Ok(ast::Expr::Literal(Literal::Nil));
        }
        if match_token!(self, [TokenType::NUMBER, TokenType::STRING]) {
            return Ok(ast::Expr::Literal(self.previous().literal.clone().unwrap()));
        }
        if match_token!(self, [TokenType::LEFT_PAREN]) {
            let expr = self.expression()?;
            self.consume(TokenType::RIGHT_PAREN, "ecpected ')' after expression")?;
            return Ok(ast::Expr::Grouping(Box::new(expr)));
        }
        if match_token!(self, [TokenType::IDENTIFIER]) {
            return Ok(ast::Expr::Variable(self.previous().clone()));
        }
        self.error(self.peek(), "expected expression");
        Err(ParserError())
    }
    fn unary(&mut self) -> Result<ast::Expr, ParserError> {
        if match_token!(self, [TokenType::BANG, TokenType::MINUS]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            return Ok(ast::Expr::Unary(operator, Box::new(right)));
        }
        self.primary()
    }
    fn factor(&mut self) -> Result<ast::Expr, ParserError> {
        let mut expr = self.unary()?;
        while match_token!(self, [TokenType::SLASH, TokenType::STAR]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = ast::Expr::Binary(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }
    fn term(&mut self) -> Result<ast::Expr, ParserError> {
        let mut expr = self.factor()?;
        while match_token!(self, [TokenType::PLUS, TokenType::MINUS]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = ast::Expr::Binary(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }
    fn comparison(&mut self) -> Result<ast::Expr, ParserError> {
        let mut expr = self.term()?;
        while match_token!(
            self,
            [
                TokenType::GREATER,
                TokenType::GREATER_EQUAL,
                TokenType::LESS,
                TokenType::LESS_EQUAL
            ]
        ) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = ast::Expr::Binary(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }
    fn equality(&mut self) -> Result<ast::Expr, ParserError> {
        let mut expr = self.comparison()?;
        while match_token!(self, [TokenType::BANG_EQUAL, TokenType::EQUAL_EQUAL]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = ast::Expr::Binary(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }
    fn assignment(&mut self) -> Result<ast::Expr, ParserError> {
        let expr = self.equality()?;
        if match_token!(self, [TokenType::EQUAL]) {
            let equals = self.previous().clone();
            let value = self.assignment()?;
            match expr {
                ast::Expr::Variable(name) => {
                    return Ok(ast::Expr::Assign(name, Box::new(value)));
                }
                _ => {
                    self.error(&equals, "Invalid assignment target");
                    return Err(ParserError());
                }
            }
        }
        Ok(expr)
    }
    fn expression(&mut self) -> Result<ast::Expr, ParserError> {
        self.assignment()
    }
    fn expression_statement(&mut self) -> Result<ast::Stmt, ParserError> {
        let value = self.expression()?;
        self.consume(TokenType::SEMICOLON, "expected ';' after value")?;
        Ok(ast::Stmt::Expression(value))
    }
    fn print_statement(&mut self) -> Result<ast::Stmt, ParserError> {
        let value = self.expression()?;
        self.consume(TokenType::SEMICOLON, "expected ';' after value")?;
        Ok(ast::Stmt::Print(value))
    }
    fn if_statement(&mut self) -> Result<ast::Stmt, ParserError> {
        self.consume(TokenType::LEFT_PAREN, "expected '(' after 'if'")?;
        let cond = self.expression()?;
        self.consume(TokenType::RIGHT_PAREN, "expected ')' after condition")?;
        let then_stmt = self.statement()?;
        if match_token!(self, [TokenType::ELSE]) {
            let else_stmt = self.statement()?;
            Ok(ast::Stmt::IfStmt(
                cond,
                Box::new((then_stmt, Some(else_stmt))),
            ))
        } else {
            Ok(ast::Stmt::IfStmt(cond, Box::new((then_stmt, None))))
        }
    }
    fn statement(&mut self) -> Result<ast::Stmt, ParserError> {
        if match_token!(self, [TokenType::PRINT]) {
            return self.print_statement();
        }
        if match_token!(self, TokenType::LEFT_BRACE) {
            return self.block().map(ast::Stmt::Block);
        }
        if match_token!(self, [TokenType::IF]) {
            return self.if_statement();
        }
        self.expression_statement()
    }
    fn var_declaration(&mut self) -> Result<ast::Stmt, ParserError> {
        let name = self.consume(TokenType::IDENTIFIER, "expected variable name")?;
        let initializer = if match_token!(self, [TokenType::EQUAL]) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(
            TokenType::SEMICOLON,
            "expected ';' after variable declaration",
        )?;
        Ok(ast::Stmt::Var(name.clone(), initializer))
    }
    fn declaration(&mut self) -> Option<ast::Stmt> {
        let res = if match_token!(self, [TokenType::VAR]) {
            self.var_declaration()
        } else {
            self.statement()
        };
        match res {
            Ok(stmt) => Some(stmt),
            Err(_) => {
                self.synchronize();
                None
            }
        }
    }
    pub fn block(&mut self) -> Result<Vec<ast::Stmt>, ParserError> {
        let mut stmts = vec![];
        while !self.check(&TokenType::RIGHT_BRACE) && !self.is_at_end() {
            stmts.push(self.declaration().unwrap());
        }
        self.consume(TokenType::RIGHT_BRACE, "expected '}' after block")?;
        Ok(stmts)
    }
    pub fn parse(&mut self) -> Result<Vec<Option<ast::Stmt>>, ParserError> {
        let mut stmts = vec![];
        while !self.is_at_end() {
            stmts.push(self.declaration());
        }
        Ok(stmts)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::Interpreter;
    #[test]
    fn test_parse() {
        let content = "print true;";
        let mut scanner = super::super::tokenizer::Tokenizer::new(content.to_string());
        let tokens = scanner.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let stmts = parser.parse().unwrap();
        let mut interpreter = Interpreter::default();
        interpreter.interpret(&stmts);
    }
}
