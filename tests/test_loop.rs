use rlox::runner::run_file;
#[test]
// #[should_panic]
fn test_while() {
    run_file("test_data/while/var_in_body.lox");
}
#[test]
fn test_while2() {
    run_file("test_data/while/syntax.lox");
}
#[test]
fn test_return_inside() {
    run_file("test_data/while/return_inside.lox");
}
#[test]
fn test_fun_in_body() {
    run_file("test_data/while/fun_in_body.lox");
}
//for loop
#[test]
fn test_for_fun_in_body() {
    run_file("test_data/for/fun_in_body.lox");
}
#[test]
fn test_for_return_inside() {
    run_file("test_data/for/return_inside.lox");
}
#[test]
fn test_scope() {
    run_file("test_data/for/scope.lox");
}
#[test]
fn test_statement_cond() {
    run_file("test_data/for/statement_cond.lox");
}
#[test]
fn test_statement_inc() {
    run_file("test_data/for/statement_inc.lox");
}
#[test]
fn test_statement_init() {
    run_file("test_data/for/statement_init.lox");
}
#[test]
fn test_for_syntax() {
    run_file("test_data/for/syntax.lox");
}
#[test]
fn test_for_var_in_body() {
    run_file("test_data/for/var_in_body.lox");
}
