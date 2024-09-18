use super::ast::{self, Assign, FnStmt, Get, Set, This, Variable};
use super::token::{Literal, Token, TokenType};
use std::rc::Rc;
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
            return Ok(ast::Expr::Grouping(Rc::new(expr)));
        }
        if match_token!(self, [TokenType::IDENTIFIER]) {
            return Ok(ast::Expr::Variable(Variable::new(self.previous().clone())));
        }
        if match_token!(self, [TokenType::THIS]) {
            return Ok(ast::Expr::This(This::new(self.previous().clone())));
        }
        self.error(self.peek(), "expected expression");
        Err(ParserError())
    }
    #[inline]
    fn finish_call(&mut self, callee: ast::Expr) -> Result<ast::Expr, ParserError> {
        let mut args = vec![];
        if !self.check(&TokenType::RIGHT_PAREN) {
            loop {
                if args.len() >= 255 {
                    self.error(self.peek(), "Cannot have more than 255 arguments");
                    return Err(ParserError());
                }
                args.push(self.expression()?);
                if !match_token!(self, [TokenType::COMMA]) {
                    break;
                }
            }
        }
        let paren = self.consume(TokenType::RIGHT_PAREN, "expected ')' after arguments")?;
        Ok(ast::Expr::Call(Rc::new(callee), paren, args.into()))
    }
    fn call(&mut self) -> Result<ast::Expr, ParserError> {
        let mut expr = self.primary()?;
        loop {
            if match_token!(self, [TokenType::LEFT_PAREN]) {
                expr = self.finish_call(expr)?;
            } else if match_token!(self, TokenType::DOT) {
                let name =
                    self.consume(TokenType::IDENTIFIER, "expected property name after '.'")?;
                expr = ast::Expr::Get(Get::new(Rc::new(expr), name.clone()));
            } else {
                break;
            }
        }
        Ok(expr)
    }
    fn unary(&mut self) -> Result<ast::Expr, ParserError> {
        if match_token!(self, [TokenType::BANG, TokenType::MINUS]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            return Ok(ast::Expr::Unary(operator, Rc::new(right)));
        }
        self.call()
    }
    fn factor(&mut self) -> Result<ast::Expr, ParserError> {
        let mut expr = self.unary()?;
        while match_token!(self, [TokenType::SLASH, TokenType::STAR]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = ast::Expr::Binary(Rc::new(expr), operator, Rc::new(right));
        }
        Ok(expr)
    }
    fn term(&mut self) -> Result<ast::Expr, ParserError> {
        let mut expr = self.factor()?;
        while match_token!(self, [TokenType::PLUS, TokenType::MINUS]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = ast::Expr::Binary(Rc::new(expr), operator, Rc::new(right));
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
            expr = ast::Expr::Binary(Rc::new(expr), operator, Rc::new(right));
        }
        Ok(expr)
    }
    fn equality(&mut self) -> Result<ast::Expr, ParserError> {
        let mut expr = self.comparison()?;
        while match_token!(self, [TokenType::BANG_EQUAL, TokenType::EQUAL_EQUAL]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = ast::Expr::Binary(Rc::new(expr), operator, Rc::new(right));
        }
        Ok(expr)
    }
    fn and(&mut self) -> Result<ast::Expr, ParserError> {
        let mut expr = self.equality()?;
        while match_token!(self, [TokenType::AND]) {
            let operator = self.previous().clone();
            let right = self.equality()?;
            expr = ast::Expr::Logical(Rc::new(expr), operator, Rc::new(right));
        }
        Ok(expr)
    }
    fn or(&mut self) -> Result<ast::Expr, ParserError> {
        let mut expr = self.and()?;
        while match_token!(self, [TokenType::OR]) {
            let operator = self.previous().clone();
            let right = self.and()?;
            expr = ast::Expr::Logical(Rc::new(expr), operator, Rc::new(right));
        }
        Ok(expr)
    }
    fn assignment(&mut self) -> Result<ast::Expr, ParserError> {
        let expr = self.or()?;
        if match_token!(self, [TokenType::EQUAL]) {
            let equals = self.previous().clone();
            let value = self.assignment()?;
            match expr {
                ast::Expr::Variable(name) => {
                    return Ok(ast::Expr::Assign(Assign::new(name.name, Rc::new(value))));
                }
                ast::Expr::Get(get) => {
                    return Ok(ast::Expr::Set(Set::from_get(get, Rc::new(value))));
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
                Rc::new((then_stmt, Some(else_stmt))),
            ))
        } else {
            Ok(ast::Stmt::IfStmt(cond, Rc::new((then_stmt, None))))
        }
    }
    fn while_statement(&mut self) -> Result<ast::Stmt, ParserError> {
        self.consume(TokenType::LEFT_PAREN, "expected '(' after 'while'")?;
        let cond = self.expression()?;
        self.consume(TokenType::RIGHT_PAREN, "expected ')' after condition")?;
        let body = self.statement()?;
        Ok(ast::Stmt::WhileStmt(cond, Rc::new(body)))
    }
    fn for_statement(&mut self) -> Result<ast::Stmt, ParserError> {
        self.consume(TokenType::LEFT_PAREN, "expected '(' after 'for'")?;
        let initializer = if match_token!(self, [TokenType::SEMICOLON]) {
            None
        } else if match_token!(self, [TokenType::VAR]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };
        let cond = if !self.check(&TokenType::SEMICOLON) {
            self.expression()?
        } else {
            ast::Expr::Literal(Literal::Boolean(true))
        };
        self.consume(TokenType::SEMICOLON, "expected ';' after loop condition")?;
        let increment = if !self.check(&TokenType::RIGHT_PAREN) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenType::RIGHT_PAREN, "expected ')' after for clauses")?;
        let body = self.statement()?;
        let mut block = if let Some(increment) = increment {
            ast::Stmt::Block(vec![body, ast::Stmt::Expression(increment)])
        } else {
            ast::Stmt::Block(vec![body])
        };

        block = ast::Stmt::WhileStmt(cond, Rc::new(block));

        block = if let Some(initializer) = initializer {
            ast::Stmt::Block(vec![initializer, block])
        } else {
            block
        };
        Ok(block)
    }
    fn return_statement(&mut self) -> Result<ast::Stmt, ParserError> {
        let keyword = self.previous().clone();
        let value = if !self.check(&TokenType::SEMICOLON) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenType::SEMICOLON, "expected ';' after return value")?;
        Ok(ast::Stmt::Return(keyword, value))
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
        if match_token!(self, [TokenType::WHILE]) {
            return self.while_statement();
        }
        if match_token!(self, [TokenType::FOR]) {
            return self.for_statement();
        }
        if match_token!(self, [TokenType::RETURN]) {
            return self.return_statement();
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
    fn function(&mut self, kind: &str) -> Result<ast::Stmt, ParserError> {
        let name = self.consume(TokenType::IDENTIFIER, &format!("expected {} name", kind))?;
        self.consume(
            TokenType::LEFT_PAREN,
            &format!("expected '(' after {}", kind),
        )?;
        let mut params = vec![];
        if !self.check(&TokenType::RIGHT_PAREN) {
            loop {
                if params.len() >= 255 {
                    self.error(self.peek(), "Cannot have more than 255 parameters");
                    return Err(ParserError());
                }
                params.push(
                    self.consume(TokenType::IDENTIFIER, "expected parameter name")?
                        .clone(),
                );
                if !match_token!(self, [TokenType::COMMA]) {
                    break;
                }
            }
        }
        self.consume(TokenType::RIGHT_PAREN, "expected ')' after parameters")?;
        self.consume(
            TokenType::LEFT_BRACE,
            &format!("expected '{{' before {} body", kind),
        )?;
        let body = self.block()?;
        Ok(ast::Stmt::Function(FnStmt::new(
            name.clone(),
            params.into(),
            body.into(),
        )))
    }
    fn class_declaration(&mut self) -> Result<ast::Stmt, ParserError> {
        let name = self.consume(TokenType::IDENTIFIER, "Expect class name")?;
        let mut superclass = None;
        if match_token!(self, TokenType::LESS) {
            self.consume(TokenType::IDENTIFIER, "Expect superclass name.")?;
            superclass = Some(ast::Expr::Variable(ast::Variable::new(
                self.previous().clone(),
            )));
        }
        self.consume(TokenType::LEFT_BRACE, "Expect '{' before class body")?;

        let mut methods = vec![];
        while !self.check(&TokenType::RIGHT_BRACE) && !self.is_at_end() {
            let ast::Stmt::Function(func) = self.function("method")? else {
                unreachable!()
            };
            methods.push(func);
        }
        self.consume(TokenType::RIGHT_BRACE, "Expect '}' after class body")?;
        Ok(ast::Stmt::Class(ast::ClassStmt::new(
            name.clone(),
            methods.into(),
            superclass,
        )))
    }
    fn declaration(&mut self) -> Option<ast::Stmt> {
        // let res = if match_token!(self, [TokenType::VAR]) {
        //     self.var_declaration()
        // } else {
        //     self.statement()
        // };
        let res = match self.peek().token_type {
            TokenType::VAR => {
                self.advance();
                self.var_declaration()
            }
            TokenType::FUN => {
                self.advance();
                self.function("function")
            }
            TokenType::CLASS => {
                self.advance();
                self.class_declaration()
            }
            _ => self.statement(),
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
        let mut stmts: Option<Vec<_>> = stmts.into_iter().collect();
        interpreter.interpret(stmts.as_mut().unwrap());
    }
}
