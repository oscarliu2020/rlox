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
