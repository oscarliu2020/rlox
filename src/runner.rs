use super::parser::{interpreter::Interpreter, Parser};
use super::scanner;
use std::fs;
use std::io::{stdin, Write};
pub fn run(content: &str) {
    let mut scanner = scanner::Scanner::new(content.to_string());
    let tokens = scanner.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let stmts = parser.parse().unwrap();
    let interpreter = Interpreter();
    interpreter.interpret(&stmts);
}
pub fn run_file(fname: &str) {
    let content = fs::read_to_string(fname).expect("File not found");
    run(&content);
}
pub fn run_prompt() {
    let mut input = String::new();
    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();
        match stdin().read_line(&mut input) {
            Ok(0) => {
                println!("EOF");
                break;
            }
            Ok(_) => {
                run(&input);
            }
            Err(_) => {
                println!("Error reading input");
                continue;
            }
        }

        input.clear();
    }
}
