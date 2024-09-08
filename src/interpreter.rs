use rustc_hash::FxHashMap;

use super::environment::{Environment, EnvironmentRef, Envt};
use crate::syntax::ast::{Expr, ExprVisitor, Stmt, StmtVisitor};
use crate::syntax::token::{Func, Function, Literal, NativeFunc, Token, TokenType};
use std::{cell::RefCell, rc::Rc};
fn error(t: &Token, msg: &str) {
    println!("[Runtime Error]line {}: {} ** {msg}", t.line, t.lexeme);
}

pub struct Interpreter {
    global: EnvironmentRef,
    environment: EnvironmentRef,
    locals: FxHashMap<*const Token, usize>,
}
impl Default for Interpreter {
    fn default() -> Self {
        let env = Rc::new(RefCell::new(Environment::new(None)));
        let mut global = Rc::clone(&env);
        global.define(
            "clock".to_string(),
            Literal::Callable(Function::Native(NativeFunc {
                arity: 0,
                func: || {
                    let now = std::time::SystemTime::now();
                    let duration = now.duration_since(std::time::UNIX_EPOCH).unwrap();
                    Literal::Number(duration.as_secs_f64())
                },
                name: "clock".to_string(),
            })),
        );
        Self {
            environment: env,
            global,
            locals: FxHashMap::default(),
        }
    }
}
trait RloxCallable {
    fn call(self, interpreter: &mut Interpreter, args: Vec<Literal>) -> VisitorResult<Literal>;

    fn arity(&self) -> usize;
}
impl RloxCallable for Function {
    fn arity(&self) -> usize {
        match self {
            Function::Function(func) => func.params().len(),
            Function::Native(native) => native.arity,
        }
    }
    fn call(self, interpreter: &mut Interpreter, args: Vec<Literal>) -> VisitorResult<Literal> {
        match self {
            Function::Function(f) => {
                let mut func_env = Environment::new(Some(Rc::clone(&f.closure)));
                for (param, arg) in f.params().iter().zip(args.iter()) {
                    func_env.define(param.lexeme.clone(), arg.clone());
                }
                match interpreter.execute_block(f.body(), func_env) {
                    Ok(_) => Ok(Literal::Nil),
                    Err(VisitorError::ReturnValue(value)) => Ok(value),
                    Err(e) => Err(e),
                }
                // todo!()
            }
            Function::Native(native) => Ok((native.func)()),
        }
    }
}
use crate::syntax::ast::{VisitorError, VisitorResult};
impl Interpreter {
    pub fn interpret(&mut self, stmts: &[Option<Stmt>]) {
        for stmt in stmts {
            let Some(stmt) = stmt.as_ref() else {
                eprintln!("Error parsing statement");
                break;
            };
            if let Err(e) = self.execute(stmt) {
                eprintln!("[Runtime Error] {e:#}");
                break;
            }
        }
    }
    pub fn resolve(&mut self, token: &Token, depth: usize) {
        self.locals.insert(token as _, depth);
    }
    fn evaluate(&mut self, expr: &Expr) -> VisitorResult<Literal> {
        match expr {
            Expr::Literal(ltr) => self.visit_literal(ltr),
            Expr::Grouping(expr) => self.visit_grouping(expr),
            Expr::Unary(token, expr) => self.visit_unary(token, expr),
            Expr::Binary(e1, token, e2) => self.visit_binary(token, e1, e2),
            Expr::Variable(token) => self.visit_variable(token),
            Expr::Assign(token, expr) => self.visit_assign(token, expr),
            Expr::Logical(e1, token, e2) => self.visit_logical(e1, token, e2),
            Expr::Call(callee, paren, args) => self.visit_call(callee, paren, args),
        }
    }
    fn execute(&mut self, stmt: &Stmt) -> VisitorResult<()> {
        match stmt {
            Stmt::Block(stmts) => self.visit_block(stmts),
            Stmt::Expression(expr) => self.visit_expression(expr),
            Stmt::Print(expr) => self.visit_print(expr),
            Stmt::Var(token, expr) => self.visit_var(token, expr.as_ref()),
            Stmt::IfStmt(cond, body) => self.visit_if(cond, body),
            Stmt::WhileStmt(cond, body) => self.visit_while(cond, body),
            Stmt::Function(name, params, body) => self.visit_function(name, params, body),
            Stmt::Return(token, expr) => self.visit_return(token, expr.as_ref()),
            _ => Err(VisitorError::VistorError),
        }
    }
    fn execute_block(&mut self, stmts: &[Stmt], block_env: Environment) -> VisitorResult<()> {
        let prev = Rc::clone(&self.environment);

        self.environment = Rc::new(RefCell::new(block_env));
        for stmt in stmts {
            // if self.execute(stmt).is_err() {
            //     break;
            // }
            match self.execute(stmt) {
                Ok(_) => {}
                e @ Err(VisitorError::ReturnValue(_)) => {
                    self.environment = prev;
                    return e;
                }
                Err(e) => {
                    self.environment = prev;
                    return Err(e);
                }
            }
        }
        // self.environment = self.environment.enclosing.take().unwrap();
        self.environment = prev;
        Ok(())
    }
}
impl StmtVisitor for Interpreter {
    fn visit_while(&mut self, cond: &Expr, body: &Stmt) -> VisitorResult<()> {
        while self.evaluate(cond)?.is_truthy() {
            self.execute(body)?;
        }
        Ok(())
    }
    fn visit_if(&mut self, cond: &Expr, body: &(Stmt, Option<Stmt>)) -> VisitorResult<()> {
        if self.evaluate(cond)?.is_truthy() {
            self.execute(&body.0)?;
        } else if let Some(else_stmt) = &body.1 {
            self.execute(else_stmt)?;
        }
        Ok(())
    }
    fn visit_block(&mut self, stmts: &[Stmt]) -> VisitorResult<()> {
        let block_env = Environment::new(Some(Rc::clone(&self.environment)));
        self.execute_block(stmts, block_env)
    }
    fn visit_expression(&mut self, expr: &Expr) -> VisitorResult<()> {
        // if let Ok(literal) = self.evaluate(expr) {
        //     // println!("{}", literal);
        //     Ok(())
        // } else {
        //     Err(VisitorError::VistorError)
        // }
        self.evaluate(expr).map(|_| ())
    }
    fn visit_print(&mut self, expr: &Expr) -> VisitorResult<()> {
        // if let Ok(literal) = self.evaluate(expr) {
        //     println!("{}", literal);
        //     Ok(())
        // } else {
        //     Err(VisitorError::VistorError)
        // }
        self.evaluate(expr).map(|res| {
            println!("{}", res);
        })
    }
    fn visit_var(&mut self, token: &Token, expr: Option<&Expr>) -> VisitorResult<()> {
        let value = if let Some(expr) = expr {
            self.evaluate(expr)?
        } else {
            Literal::Nil
        };
        self.environment
            .borrow_mut()
            .define(token.lexeme.clone(), value);
        Ok(())
    }
    fn visit_function(
        &mut self,
        name: &Token,
        params: &[Token],
        body: &[Stmt],
    ) -> VisitorResult<()> {
        let new_func = Function::Function(Func {
            decl: Rc::new(Stmt::Function(name.clone(), params.to_vec(), body.to_vec())),
            closure: Rc::clone(&self.environment),
        });
        self.environment
            .define(name.lexeme.clone(), Literal::Callable(new_func));
        Ok(())
    }
    fn visit_return(&mut self, token: &Token, expr: Option<&Expr>) -> VisitorResult<()> {
        let value = if let Some(expr) = expr {
            self.evaluate(expr)?
        } else {
            Literal::Nil
        };
        Err(VisitorError::ReturnValue(value))
    }
}

impl ExprVisitor for Interpreter {
    fn visit_call(
        &mut self,
        callee: &Expr,
        paren: &Token,
        args: &[Expr],
    ) -> VisitorResult<Literal> {
        let callee = self.evaluate(callee)?;
        let mut arguments = Vec::new();
        for arg in args {
            arguments.push(self.evaluate(arg)?);
        }
        match callee {
            Literal::Callable(callable) => {
                if arguments.len() != callable.arity() {
                    return Err(VisitorError::ArityNotMatched(
                        callable.arity(),
                        arguments.len(),
                        paren.clone(),
                    ));
                }
                callable.call(self, arguments)
            }
            _ => Err(VisitorError::VistorError),
        }
    }
    fn visit_logical(
        &mut self,
        left: &Expr,
        token: &Token,
        right: &Expr,
    ) -> VisitorResult<Literal> {
        let l = self.evaluate(left)?;
        match token.token_type {
            TokenType::OR => {
                if l.is_truthy() {
                    return Ok(l);
                }
            }
            TokenType::AND => {
                if !l.is_truthy() {
                    return Ok(l);
                }
            }
            _ => {
                // error(token, "Unknown logical operator");
                return Err(VisitorError::UnknownOperator(token.clone(), "logical"));
            }
        }
        self.evaluate(right)
    }
    fn visit_literal(&mut self, ltr: &Literal) -> VisitorResult<Literal> {
        Ok(ltr.clone())
    }
    fn visit_variable(&mut self, token: &Token) -> VisitorResult<Literal> {
        self.environment.borrow().get(token)
    }
    fn visit_grouping(&mut self, expr: &Expr) -> VisitorResult<Literal> {
        self.evaluate(expr)
    }
    fn visit_unary(&mut self, token: &Token, expr: &Expr) -> VisitorResult<Literal> {
        let right = self.evaluate(expr)?;
        match token.token_type {
            TokenType::MINUS => match right {
                Literal::Number(n) => Ok(Literal::Number(-n)),
                _ => Err(VisitorError::VistorError),
            },
            TokenType::BANG => Ok(Literal::Boolean(!right.is_truthy())),
            _ => {
                // error(token, "Unknown unary operator");
                Err(VisitorError::UnknownOperator(token.clone(), "unary"))
            }
        }
    }
    fn visit_binary(&mut self, token: &Token, e1: &Expr, e2: &Expr) -> VisitorResult<Literal> {
        let l = self.evaluate(e1)?;
        let r = self.evaluate(e2)?;
        match token.token_type {
            TokenType::PLUS => match (l, r) {
                (Literal::Number(n1), Literal::Number(n2)) => Ok(Literal::Number(n1 + n2)),
                _ => {
                    // error(token, "Operands must be two numbers");
                    Err(VisitorError::ArithmeticError(token.clone()))
                }
            },
            TokenType::MINUS => match (l, r) {
                (Literal::Number(n1), Literal::Number(n2)) => Ok(Literal::Number(n1 - n2)),
                _ => {
                    // error(token, "Operands must be two numbers");
                    Err(VisitorError::ArithmeticError(token.clone()))
                }
            },
            TokenType::STAR => match (l, r) {
                (Literal::Number(n1), Literal::Number(n2)) => Ok(Literal::Number(n1 * n2)),
                _ => {
                    // error(token, "Operands must be two numbers");
                    Err(VisitorError::ArithmeticError(token.clone()))
                }
            },
            TokenType::SLASH => match (l, r) {
                (Literal::Number(n1), Literal::Number(n2)) => Ok(Literal::Number(n1 / n2)),
                _ => {
                    // error(token, "Operands must be two numbers");
                    Err(VisitorError::ArithmeticError(token.clone()))
                }
            },
            TokenType::GREATER => match (l, r) {
                (Literal::Number(n1), Literal::Number(n2)) => Ok(Literal::Boolean(n1 > n2)),
                _ => {
                    // error(token, "Operands must be two numbers");
                    Err(VisitorError::ArithmeticError(token.clone()))
                }
            },
            TokenType::GREATER_EQUAL => match (l, r) {
                (Literal::Number(n1), Literal::Number(n2)) => Ok(Literal::Boolean(n1 >= n2)),
                _ => {
                    // error(token, "Operands must be two numbers");
                    Err(VisitorError::ArithmeticError(token.clone()))
                }
            },
            TokenType::LESS => match (l, r) {
                (Literal::Number(n1), Literal::Number(n2)) => Ok(Literal::Boolean(n1 < n2)),
                _ => {
                    // error(token, "Operands must be two numbers");
                    Err(VisitorError::ArithmeticError(token.clone()))
                }
            },
            TokenType::LESS_EQUAL => match (l, r) {
                (Literal::Number(n1), Literal::Number(n2)) => Ok(Literal::Boolean(n1 <= n2)),
                _ => {
                    // error(token, "Operands must be two numbers");
                    Err(VisitorError::ArithmeticError(token.clone()))
                }
            },
            TokenType::BANG_EQUAL => Ok(Literal::Boolean(l != r)),
            TokenType::EQUAL_EQUAL => Ok(Literal::Boolean(l == r)),
            _ => {
                // error(token, "Unknown binary operator");
                Err(VisitorError::UnknownOperator(token.clone(), "binary"))
            }
        }
    }
    fn visit_assign(&mut self, token: &Token, expr: &Expr) -> VisitorResult<Literal> {
        let value = self.evaluate(expr)?;
        self.environment.borrow_mut().assign(token, value.clone())?;
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::run;
    #[test]
    fn test_block() {
        let mut interpreter = Interpreter::default();
        run(
            r#"var a = "global a";
        var b = "global b";
        var c = "global c";
        {
          var a = "outer a";
          var b = "outer b";
          {
            var a = "inner a";
            print a;
            print b;
            print c;
          }
          print a;
          print b;
          print c;
        }
        print a;
        print b;
        print c;"#,
            &mut interpreter,
        );
    }
    #[test]
    fn test_blcok2() {
        let mut interpreter = Interpreter::default();
        run(
            r"
            var a=1;
            {
                var a=a+3;
                print a;
            }
            print a;
        ",
            &mut interpreter,
        );
    }
    #[test]
    fn test_while() {
        let mut interpreter = Interpreter::default();
        run(
            r"
            var a=1;
            while(a<10){
                print a;
                a=a+1;
            }
        ",
            &mut interpreter,
        );
    }
    #[test]
    fn test_for() {
        let mut interpreter = Interpreter::default();
        run(
            r"
            var a = 0;
            var temp;

            for (var b = 1; a < 10000; b = temp + b) {
            print a;
            temp = a;
            a = b;
            }
        ",
            &mut interpreter,
        );
    }
    #[test]
    fn test_native() {
        let mut interpreter = Interpreter::default();
        run(
            r"
            var a=clock();
            while (clock() - a < 5) {}
            print clock() - a;
        ",
            &mut interpreter,
        );
    }
    #[test]
    fn test_fib() {
        let mut interpreter = Interpreter::default();
        run(
            r"
            var a=clock();
            for (var t=0;t<10;t=t+1){
                var a=0;
                var b=1;
                
                for (var i = 0; i < 30; i = i + 1) {
                    // print b;
                    var temp = a;
                    a = b;
                    b = b + temp;
                }
            }
            print clock()-a;
        ",
            &mut interpreter,
        );
    }
    #[test]
    fn local_fun() {
        let mut interpreter = Interpreter::default();
        run(
            r#"
            fun makeCounter() {
  var i = 0;
  fun count() {
    i = i + 1;
    print i;
  }

  return count;
}

var counter = makeCounter();
counter(); // "1".
counter(); // "2".
        "#,
            &mut interpreter,
        );
    }
    #[test]
    fn local_fun2() {
        let mut interpreter = Interpreter::default();
        run(
            r#"var a = "global";
        {
          fun showA() {
            print a;
          }
        
          showA();
          var a = "block";
          showA();
        }"#,
            &mut interpreter,
        );
    }
}
