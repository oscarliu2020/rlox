use rlox::runner;
use std::env;
fn main() {
    let len = env::args().len();
    if len > 2 {
        let prog = env::args().next().unwrap();
        println!("{prog} [script]");
    } else if len == 2 {
        runner::run_file(&env::args().nth(1).unwrap());
    } else {
        runner::run_prompt();
    }
}
