use rlox::runner::run_file;
#[test]
fn closure() {
    run_file("test_data/this/closure.lox");
}
#[test]
fn nested_closure() {
    run_file("test_data/this/nested_closure.lox");
}
#[test]
fn nested_class() {
    run_file("test_data/this/nested_class.lox");
}
#[test]
#[should_panic]
fn this_at_top_level() {
    run_file("test_data/this/this_at_top_level.lox");
}
#[test]
fn this_in_method() {
    run_file("test_data/this/this_in_method.lox");
}
#[test]
#[should_panic]
fn this_in_top_level_function() {
    run_file("test_data/this/this_in_top_level_function.lox");
}