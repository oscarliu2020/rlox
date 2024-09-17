use rlox::runner::run_file;
#[test]
fn empty(){
    run_file("test_data/class/empty.lox");
}
#[test]
fn local_reference_self() {
    run_file("test_data/class/local_reference_self.lox");
}
#[test]
fn reference_self() {
    run_file("test_data/class/reference_self.lox");
}