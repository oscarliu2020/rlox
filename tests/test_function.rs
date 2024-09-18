use rlox::runner::run_file;
#[test]
fn body_must_be_block() {
    run_file("test_data/function/body_must_be_block.lox");
}
#[test]
fn empty_body() {
    run_file("test_data/function/empty_body.lox");
}
#[test]
fn extra_arguments() {
    run_file("test_data/function/extra_arguments.lox");
}
#[test]
fn local_mutual_recursion() {
    run_file("test_data/function/local_mutual_recursion.lox");
}
#[test]
fn local_recursion() {
    run_file("test_data/function/local_recursion.lox");
}
#[test]
fn missing_arguments() {
    run_file("test_data/function/missing_arguments.lox");
}
#[test]
fn missing_comma_in_parameters() {
    run_file("test_data/function/missing_comma_in_parameters.lox");
}
#[test]
fn mutual_recursion() {
    run_file("test_data/function/mutual_recursion.lox");
}
#[test]
fn nested_call_with_arguments() {
    run_file("test_data/function/nested_call_with_arguments.lox");
}
#[test]
fn parameters() {
    run_file("test_data/function/parameters.lox");
}
#[test]
fn print() {
    run_file("test_data/function/print.lox");
}
#[test]
fn recursion() {
    run_file("test_data/function/recursion.lox");
}
#[test]
#[should_panic]
fn too_many_arguments() {
    run_file("test_data/function/too_many_arguments.lox");
}
#[test]
fn too_many_parameters() {
    run_file("test_data/function/too_many_parameters.lox");
}
