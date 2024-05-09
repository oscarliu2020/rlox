use rlox::runner::run_file;
#[test]
#[should_panic]
fn test_parse() {
    run_file("test_data/unexpected_char.lox");
}
