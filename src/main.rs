use rlox::runner;
use std::env;
fn main() {
    let len = env::args().len();
    match len {
        1 => runner::run_prompt(),
        2 => runner::run_file(&env::args().nth(1).unwrap()),
        _ => {
            let prog = env::args().next().unwrap();
            println!("{prog} [script]");
        }
    }
}
