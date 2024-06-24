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
fn test_assignment_grouping_error() {
    run_file("test_data/assignment/grouping.lox");
}
#[test]
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
#[test]
fn test_block_empty() {
    run_file("test_data/block/empty.lox");
}
#[test]
fn test_block_scope() {
    run_file("test_data/block/scope.lox");
}
#[test]
fn test_if_dangling_else() {
    run_file("test_data/if/dangling_else.lox");
}
#[test]
fn test_if_stmt() {
    run_file("test_data/if/if.lox");
}
#[test]
fn test_if_truth() {
    run_file("test_data/if/truth.lox");
}
#[test]
fn test_else() {
    run_file("test_data/if/else.lox");
}
#[test]
fn test_var_in_else() {
    run_file("test_data/if/var_in_else.lox");
}
#[test]
fn test_var_in_else_branch() {
    run_file("test_data/if/var_in_then.lox");
}
#[test]
fn test_fn_in_else() {
    run_file("test_data/if/fun_in_else.lox");
}
#[test]
fn test_fn_in_then() {
    run_file("test_data/if/fun_in_then.lox");
}
#[test]
fn test_precedence() {
    run_file("test_data/precedence.lox");
}
#[test]
fn test_empty() {
    run_file("test_data/empty.lox");
}
