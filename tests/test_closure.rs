use rlox::runner::run_file;
#[test]
fn assign_to_closure() {
    run_file("test_data/closure/assign_to_closure.lox");
}
#[test]
fn assign_to_shadowed_later() {
    run_file("test_data/closure/assign_to_shadowed_later.lox");
}
#[test]
fn close_over_function_parameter() {
    run_file("test_data/closure/close_over_function_parameter.lox");
}
#[test]
fn close_over_later_variable() {
    run_file("test_data/closure/close_over_later_variable.lox");
}
#[test]
fn close_over_method_parameter() {
    run_file("test_data/closure/close_over_method_parameter.lox");
}
#[test]
fn closed_closure_in_function() {
    run_file("test_data/closure/closed_closure_in_function.lox");
}
#[test]
fn nested_closure() {
    run_file("test_data/closure/nested_closure.lox");
}
#[test]
fn open_closure_in_function() {
    run_file("test_data/closure/open_closure_in_function.lox");
}
#[test]
fn reference_closure_multiple_times() {
    run_file("test_data/closure/reference_closure_multiple_times.lox");
}
#[test]
fn reuse_closure_slot() {
    run_file("test_data/closure/reuse_closure_slot.lox");
}
#[test]
fn shadow_closure_with_local() {
    run_file("test_data/closure/shadow_closure_with_local.lox");
}
#[test]
fn unused_closure() {
    run_file("test_data/closure/unused_closure.lox");
}
#[test]
fn unused_later_closure() {
    run_file("test_data/closure/unused_later_closure.lox");
}