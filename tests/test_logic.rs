use rlox::runner::run_file;
#[test]
fn test_and() {
    run_file("test_data/logic/and.lox");
}
#[test]
fn test_and_truth() {
    run_file("test_data/logic/and_truth.lox");
}
#[test]
fn test_or() {
    run_file("test_data/logic/or.lox");
}
#[test]
fn test_or_truth() {
    run_file("test_data/logic/or_truth.lox");
}
