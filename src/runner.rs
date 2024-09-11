use super::interpreter::Interpreter;
use super::resolver::Resolver;
use super::syntax::{parser::Parser, tokenizer::Tokenizer};
use std::fs;
use std::io::{stdin, Write};
pub fn run(content: &str, interpreter: &mut Interpreter) {
    let mut scanner = Tokenizer::new(content.to_string());
    let tokens = scanner.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let stmts = parser.parse().unwrap();
    // let mut interpreter = Interpreter::default();
    let mut stmts: Option<Vec<_>> = stmts.into_iter().collect();
    if stmts.is_none() {
        eprintln!("Error parsing");
        return;
    }
    let mut resolver = Resolver::new();
    resolver.resolve(stmts.as_mut().unwrap()).unwrap();
    interpreter.interpret(stmts.as_mut().unwrap());
}
pub fn run_file(fname: &str) {
    let mut interpreter = Interpreter::default();
    let content = fs::read_to_string(fname).expect("File not found");
    run(&content, &mut interpreter);
}
pub fn run_prompt() {
    let mut input = String::new();
    let mut interpreter = Interpreter::default();
    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();
        match stdin().read_line(&mut input) {
            Ok(0) => {
                println!("EOF");
                break;
            }
            Ok(_) => {
                run(&input, &mut interpreter);
            }
            Err(_) => {
                println!("Error reading input");
                continue;
            }
        }

        input.clear();
    }
}
