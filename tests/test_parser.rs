use rlox::runner::run_file;
#[test]
#[should_panic]
fn test_parse() {
    run_file("test_data/unexpected_char.lox");
}
#[test]
fn test_assignment() {
    run_file("test_data/assignment/global.lox");
}
#[test]
fn test_assignment_associative() {
    run_file("test_data/assignment/associative.lox");
}
#[test]
#[should_panic]
fn test_assignment_grouping_error() {
    run_file("test_data/assignment/grouping.lox");
}
#[test]
#[should_panic]
fn test_infix_error() {
    run_file("test_data/assignment/infix.lox");
}
#[test]
fn test_syntax() {
    run_file("test_data/assignment/syntax.lox");
}
#[test]
// #[should_panic]
fn test_undefined() {
    run_file("test_data/assignment/undefined.lox");
}
