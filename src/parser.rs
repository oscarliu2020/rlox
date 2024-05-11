use crate::scanner::{token::TokenType, Literal, Token};

pub mod ast;
pub mod interpreter;
pub struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
}
impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, current: 0 }
    }
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }
    fn is_at_end(&self) -> bool {
        return self.peek().token_type == TokenType::EOF;
    }
    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }
    fn check(&self, ty: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        return self.peek().token_type == *ty;
    }
    fn matches<const N: usize>(&mut self, types: [TokenType; N]) -> bool {
        for ty in types.iter() {
            if self.check(ty) {
                self.advance();
                return true;
            }
        }
        false
    }
    fn error(&self, t: &Token, msg: &str) {
        if t.token_type == TokenType::EOF {
            eprintln!("[line {}] Error at end: {}", t.line, msg);
        } else {
            eprintln!("[line {}] Error at '{}': {}", t.line, t.lexeme, msg);
        }
    }
    fn consume(&mut self, ty: TokenType, msg: &str) -> Result<(), ()> {
        if self.check(&ty) {
            self.advance();
            return Ok(());
        }
        self.error(self.peek(), msg);
        Err(())
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
        }
    }
    fn primary(&mut self) -> Result<ast::Expr, ()> {
        if self.matches([TokenType::FALSE]) {
            return Ok(ast::Expr::Literal(Literal::Boolean(false)));
        }
        if self.matches([TokenType::TRUE]) {
            return Ok(ast::Expr::Literal(Literal::Boolean(true)));
        }
        if self.matches([TokenType::NIL]) {
            return Ok(ast::Expr::Literal(Literal::Nil));
        }
        if self.matches([TokenType::NUMBER, TokenType::STRING]) {
            return Ok(ast::Expr::Literal(self.previous().literal.clone().unwrap()));
        }
        if self.matches([TokenType::LEFT_PAREN]) {
            let expr = self.expression()?;
            self.consume(TokenType::RIGHT_PAREN, "ecpected ')' after expression")?;
            return Ok(ast::Expr::Grouping(Box::new(expr)));
        }
        if self.matches([TokenType::IDENTIFIER]) {
            todo!()
        }
        self.error(self.peek(), "expected expression");
        Err(())
    }
    fn unary(&mut self) -> Result<ast::Expr, ()> {
        if self.matches([TokenType::BANG, TokenType::MINUS]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            return Ok(ast::Expr::Unary(operator, Box::new(right)));
        }
        self.primary()
    }
    fn factor(&mut self) -> Result<ast::Expr, ()> {
        let mut expr = self.unary()?;
        while self.matches([TokenType::SLASH, TokenType::STAR]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = ast::Expr::Binary(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }
    fn term(&mut self) -> Result<ast::Expr, ()> {
        let mut expr = self.factor()?;
        while self.matches([TokenType::PLUS, TokenType::MINUS]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = ast::Expr::Binary(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }
    fn comparison(&mut self) -> Result<ast::Expr, ()> {
        let mut expr = self.term()?;
        while self.matches([
            TokenType::GREATER,
            TokenType::GREATER_EQUAL,
            TokenType::LESS,
            TokenType::LESS_EQUAL,
        ]) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = ast::Expr::Binary(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }
    fn equality(&mut self) -> Result<ast::Expr, ()> {
        let mut expr = self.comparison()?;
        while self.matches([TokenType::BANG_EQUAL, TokenType::EQUAL_EQUAL]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = ast::Expr::Binary(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }
    fn expression(&mut self) -> Result<ast::Expr, ()> {
        self.equality()
    }
    fn expression_statement(&mut self) -> Result<ast::Stmt, ()> {
        let value = self.expression()?;
        self.consume(TokenType::SEMICOLON, "expected ';' after value")?;
        Ok(ast::Stmt::Expression(value))
    }
    fn print_statement(&mut self) -> Result<ast::Stmt, ()> {
        let value = self.expression()?;
        self.consume(TokenType::SEMICOLON, "expected ';' after value")?;
        Ok(ast::Stmt::Print(value))
    }
    fn statement(&mut self) -> Result<ast::Stmt, ()> {
        if self.matches([TokenType::PRINT]) {
            return self.print_statement();
        }
        self.expression_statement()
    }
    pub fn parse(&mut self) -> Result<Vec<ast::Stmt>, ()> {
        let mut stmts = vec![];
        while !self.is_at_end() {
            stmts.push(self.statement()?);
        }
        Ok(stmts)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse() {
        let content = "print true;";
        let mut scanner = crate::scanner::Scanner::new(content.to_string());
        let tokens = scanner.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let stmts = parser.parse().unwrap();
        let interpreter = interpreter::Interpreter();
        interpreter.interpret(&stmts);
    }
}
