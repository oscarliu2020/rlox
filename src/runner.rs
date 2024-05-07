use super::scanner;
use std::fs;
use std::io::{stdin, Write};
pub fn run(content: &str) {
    let mut scanner = scanner::Scanner::new(content.to_string());
    let tokens = scanner.scan_tokens().unwrap();
    for token in tokens {
        println!("{}", token);
    }
}
pub fn run_file(fname: &str) {
    let content = fs::read_to_string(fname).expect("File not found");
    run(&content);
    // todo!();
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
