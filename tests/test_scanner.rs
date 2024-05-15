use rlox::runner::run_file;

#[test]
#[should_panic]
fn run_error() {
    run_file("test_data/unexpected_char.lox");
}
