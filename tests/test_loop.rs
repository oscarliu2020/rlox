use rlox::runner::run_file;
#[test]
// #[should_panic]
fn test_while() {
    run_file("test_data/while/var_in_body.lox");
}
