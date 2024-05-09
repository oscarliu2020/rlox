use rlox::runner::run_file;
use std::fs;
#[test]
fn run() {
    for f in fs::read_dir("test_data/scanning").unwrap() {
        let f = f.unwrap();
        let path = f.path();
        let path = path.to_str().unwrap();
        println!("{}", path);
        if path.ends_with(".lox") {
            run_file(path);
        }
        println!("-------------------");
    }
}
#[test]
fn test_evaluate() {
    run_file("test_data/parsing/evaluate.lox");
}
#[test]
#[should_panic]
fn run_error() {
    run_file("test_data/unexpected_char.lox");
}
