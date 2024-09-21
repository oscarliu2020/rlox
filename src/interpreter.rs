use rustc_hash::FxHashMap;

use super::environment::{Environment, EnvironmentRef, Envt};
use crate::environment::EnvironmentError;
use crate::resolver::Resolvable;
use crate::syntax::ast::*;
use crate::syntax::token::*;
use std::cell::RefCell;
use std::rc::Rc;
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
            Function::Class(class) => class.get_method("init").map_or(0, |e| {
                let Literal::Callable(ref f) = e else {
                    unreachable!()
                };
                f.arity()
            }),
        }
    }
    fn call(self, interpreter: &mut Interpreter, args: Vec<Literal>) -> VisitorResult<Literal> {
        match self {
            Function::Function(mut f) => {
                let mut func_env = Environment::new(Some(Rc::clone(&f.closure)));
                for (param, arg) in f.params().iter().zip(args.iter()) {
                    func_env.define(param.lexeme.clone(), arg.clone());
                }
                match interpreter.execute_block(f.body(), func_env) {
                    Ok(_) => {
                        if f.is_initializer {
                            return f
                                .closure
                                .get_at(
                                    0,
                                    &Token {
                                        token_type: TokenType::THIS,
                                        lexeme: "this".to_owned(),
                                        literal: None,
                                        line: f.decl.name.line,
                                    },
                                )
                                .map_err(|e| e.into());
                        }
                        Ok(Literal::Nil)
                    }
                    Err(VisitorError::ReturnValue(value)) => {
                        if f.is_initializer {
                            return f
                                .closure
                                .get_at(
                                    0,
                                    &Token {
                                        token_type: TokenType::THIS,
                                        lexeme: "this".to_owned(),
                                        literal: None,
                                        line: f.decl.name.line,
                                    },
                                )
                                .map_err(|e| e.into());
                        }
                        Ok(value)
                    }
                    Err(e) => Err(e),
                }
            }
            Function::Native(native) => Ok((native.func)()),
            Function::Class(class) => {
                let inner = Rc::new(RefCell::new(Instance::new(class)));
                let instance = Literal::Instance(Rc::clone(&inner));
                let ff = inner.borrow().class.get_method("init");
                if let Some(Literal::Callable(Function::Function(mut init))) = ff {
                    Function::Function(init.bind(Rc::clone(&inner))).call(interpreter, args)?;
                }
                Ok(instance)
            }
        }
    }
}
use crate::syntax::ast::{VisitorError, VisitorResult};
impl Interpreter {
    pub fn interpret(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
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
        expr.accept(self)
    }
    fn execute(&mut self, stmt: &Stmt) -> VisitorResult<()> {
        stmt.accept(self)
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
        self.environment = prev;
        Ok(())
    }
    fn look_up_variable(&self, variable: &impl Resolvable) -> VisitorResult<Literal> {
        variable.get_dist().map_or(
            self.global.get(variable.name()).map_err(|e| e.into()),
            |dist| {
                self.environment
                    .get_at(dist, variable.name())
                    .map_err(|e| e.into())
            },
        )
    }
}

impl StmtVisitor for Interpreter {
    fn visit_while(&mut self, cond: &Expr, body: &Stmt) -> VisitorResult<()> {
        while self.evaluate(cond)?.is_truthy() {
            self.execute(body)?;
        }
        Ok(())
    }
    fn visit_expression(&mut self, expr: &Expr) -> VisitorResult<()> {
        self.evaluate(expr).map(|_| ())
    }
    fn visit_print(&mut self, expr: &Expr) -> VisitorResult<()> {
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
        self.environment.define(token.lexeme.clone(), value);
        Ok(())
    }
    fn visit_block(&mut self, stmts: &[Stmt]) -> VisitorResult<()> {
        let block_env = Environment::new(Some(Rc::clone(&self.environment)));
        self.execute_block(stmts, block_env)
    }
    fn visit_if(&mut self, cond: &Expr, body: &(Stmt, Option<Stmt>)) -> VisitorResult<()> {
        if self.evaluate(cond)?.is_truthy() {
            self.execute(&body.0)?;
        } else if let Some(else_stmt) = &body.1 {
            self.execute(else_stmt)?;
        }
        Ok(())
    }
    fn visit_function(
        &mut self,
        name: &Token,
        params: Rc<[Token]>,
        body: Rc<[Stmt]>,
    ) -> VisitorResult<()> {
        let new_func = Function::Function(Func {
            decl: Rc::new(FnStmt {
                name: name.clone(),
                params,
                body,
            }),
            closure: Rc::clone(&self.environment),
            is_initializer: false,
        });
        self.environment
            .define(name.lexeme.clone(), Literal::Callable(new_func));
        Ok(())
    }
    fn visit_return(&mut self, _token: &Token, expr: Option<&Expr>) -> VisitorResult<()> {
        let value = if let Some(expr) = expr {
            self.evaluate(expr)?
        } else {
            Literal::Nil
        };
        Err(VisitorError::ReturnValue(value))
    }
    fn visit_class(&mut self, class: &crate::syntax::ast::ClassStmt) -> VisitorResult<()> {
        /*
        TODO: extract to a function
        */
        let superclass = match &class.superclass {
            Some(expr) => {
                let Literal::Callable(Function::Class(s)) = self.evaluate(expr)? else {
                    return Err(VisitorError::SuperclassMustBeAClass(class.name.line));
                };
                Some(Rc::new(s))
            }
            None => None,
        };
        self.environment
            .define(class.name.lexeme.clone(), Literal::Nil);
        if superclass.is_some() {
            self.environment = Rc::new(RefCell::new(Environment::new(Some(Rc::clone(
                &self.environment,
            )))));
        }
        let mut method_table = FxHashMap::default();
        for method in class.methods.iter() {
            let is_initializer = method.name.lexeme == "init";
            let func = Function::Function(Func {
                decl: Rc::new(method.clone()),
                closure: Rc::clone(&self.environment),
                is_initializer,
            });
            method_table.insert(method.name.lexeme.clone(), Literal::Callable(func));
        }
        let klass = Literal::Callable(Function::Class(Class::new(
            class.name.lexeme.clone(),
            method_table,
            superclass.clone(),
        )));
        if superclass.is_some() {
            let parent = self.environment.borrow().enclosing.clone();
            self.environment = parent.unwrap();
        }
        self.environment.assign(&class.name, klass.clone())?;
        Ok(())
    }
}

impl ExprVisitor for Interpreter {
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
    fn visit_grouping(&mut self, expr: &Expr) -> VisitorResult<Literal> {
        self.evaluate(expr)
    }
    fn visit_literal(&mut self, ltr: &Literal) -> VisitorResult<Literal> {
        Ok(ltr.clone())
    }
    fn visit_unary(&mut self, token: &Token, expr: &Expr) -> VisitorResult<Literal> {
        let right = self.evaluate(expr)?;
        match token.token_type {
            TokenType::MINUS => match right {
                Literal::Number(n) => Ok(Literal::Number(-n)),
                _ => Err(VisitorError::VistorError),
            },
            TokenType::BANG => Ok(Literal::Boolean(!right.is_truthy())),
            _ => Err(VisitorError::UnknownOperator(token.clone(), "unary")),
        }
    }
    fn visit_variable(&mut self, variable: &Variable) -> VisitorResult<Literal> {
        self.look_up_variable(variable)
    }
    fn visit_assign(&mut self, assign: &Assign) -> VisitorResult<Literal> {
        let value = self.evaluate(&assign.value)?;
        // self.environment
        //     .borrow_mut()
        //     .assign(assign.name(), value.clone())?;
        // Ok(value)
        assign
            .get_dist()
            .map_or_else(
                || {
                    self.global
                        .assign(assign.name(), value.clone())
                        .map_err(|e| e.into())
                },
                |dist| {
                    self.environment
                        .assign_at(dist, assign.name(), value.clone())
                        .map_err(|e| e.into())
                },
            )
            .map(|_| value)
        // todo!()
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
    fn visit_get(&mut self, get: &Get) -> VisitorResult<Literal> {
        let x = self.evaluate(&get.object)?;
        if let Literal::Instance(instance) = x {
            Instance::get(&get.name, &instance).ok_or_else(|| {
                VisitorError::UndefinedProperty(get.name.clone(), get.name.lexeme.clone())
            })
        } else {
            Err(VisitorError::VistorError)
        }
    }
    fn visitor_set(&mut self, set: &Set) -> VisitorResult<Literal> {
        let obj = self.evaluate(&set.object)?;
        if let Literal::Instance(instance) = obj {
            let value = self.evaluate(&set.value)?;
            instance.borrow_mut().set(&set.name.lexeme, value.clone());
            Ok(value)
        } else {
            Err(VisitorError::VistorError)
        }
    }
    fn visit_this(&mut self, token: &This) -> VisitorResult<Literal> {
        self.look_up_variable(token)
    }
    fn visit_super(&mut self, s: &Super) -> VisitorResult<Literal> {
        // self.look_up_variable(s)
        let superclass = s.get_dist().map_or_else(
            || Err(EnvironmentError::InvalidEnvironmentDistance),
            |dist| self.environment.get_at(dist, s.name()),
        )?;
        let Literal::Callable(Function::Class(superclass)) = superclass else {
            unreachable!()
        };
        let dist = s.get_dist().unwrap(); //safe to unwrap
        let obj = self.environment.get_at(
            dist - 1,
            &Token {
                token_type: TokenType::THIS,
                lexeme: "this".to_owned(),
                literal: None,
                line: s.name().line,
            },
        )?;
        let Literal::Instance(instance) = obj else {
            unreachable!()
        };
        let Literal::Callable(Function::Function(mut method)) = superclass
            .get_method(s.method.lexeme.as_str())
            .ok_or_else(|| {
                VisitorError::UndefinedProperty(s.method.clone(), s.method.lexeme.clone())
            })?
        else {
            unreachable!()
        };
        Ok(Literal::Callable(Function::Function(method.bind(instance))))
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
    #[should_panic]
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
    #[test]
    fn test_rec() {
        let mut interpreter = Interpreter::default();
        run(
            r#"
            fun fib(n) {
                if (n <= 1) return n;
                return fib(n - 1) + fib(n - 2);
            }
            for (var i = 0; i < 20; i = i + 1) {
                print fib(i);
            }
        "#,
            &mut interpreter,
        );
    }
    #[test]
    fn test_fn() {
        let mut interpreter = Interpreter::default();
        run(
            r#"
            fun foo() {
                print "foo";
            }
            foo();
        "#,
            &mut interpreter,
        );
    }

    #[test]
    #[should_panic]
    fn test_decl() {
        let mut interpreter = Interpreter::default();
        run(
            r#"
            fun foo() {
                var a=1;
                var a=2;
            }
        "#,
            &mut interpreter,
        );
    }
    #[test]
    #[should_panic]
    fn test_invalid_ret() {
        let mut interpreter = Interpreter::default();
        run(
            r#"
            fun foo() {
                return 1;
            }
            foo();
            return 2;
        "#,
            &mut interpreter,
        );
    }
    #[test]
    fn test_class() {
        let mut interpreter = Interpreter::default();
        run(
            r#"
            class Foo {
                bar() {
                    print "bar";
                }
            }
            fun out() {
                print "out";
            }
            var foo = Foo();
            foo.bar();
            foo.a=1;
            print foo.a;
            foo.out=out;
            foo.out();
            class Bacon {
                eat() {
                    print "Crunch crunch crunch!";
                }
            }

            Bacon().eat(); // Prints "Crunch crunch crunch!".
            "#,
            &mut interpreter,
        );
    }
    #[test]
    fn test_this() {
        let mut interpreter = Interpreter::default();
        run(
            r#"
            class Foo {
                bar() {
                    print this.x;
                }
            }
            var foo = Foo();
            foo.x=1;
            foo.bar();
            "#,
            &mut interpreter,
        );
    }
    #[test]
    fn test_init() {
        let mut interpreter = Interpreter::default();
        run(
            r#"
            class Foo {
                init(x) {
                    this.x = x;
                    return;
                }
                bar() {
                    print this.x;
                }
            }
            var foo = Foo(1);
            foo.bar();
            "#,
            &mut interpreter,
        );
    }
    #[test]
    #[should_panic]
    fn test_init2() {
        let mut interpreter = Interpreter::default();
        run(
            r#"
            class Foo {
                init(x) {
                    this.x = x;
                    return 1;
                }
                bar() {
                    print this.x;
                }
            }
            var foo = Foo(1);
            foo.bar();
            "#,
            &mut interpreter,
        );
    }
    #[test]
    fn test_init3() {
        let mut interpreter = Interpreter::default();
        run(
            r#"
            class Foo {
                init(x) {
                    this.x = x;
                }
                bar() {
                    print this.x;
                }
            }
            var foo = Foo(2);
            var bar=foo.init(1);
            print bar.x;
            print foo.x;
            foo.x=3;
            print bar.x;
            "#,
            &mut interpreter,
        );
    }
    #[test]
    fn test_inherit() {
        let mut interpreter = Interpreter::default();
        run(
            r#"
            class B{
                bar() {
                    print "bar";
                }
            }
            class A <B {
                init() {
                    this.x = 1;
                }
                foo() {
                    print this.x;
                }
            }
            A().bar();
            "#,
            &mut interpreter,
        );
    }
}
