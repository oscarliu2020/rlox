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
#[test]
fn local_inherit_other() {
    run_file("test_data/class/local_inherit_other.lox");
}
#[test]
#[should_panic]
fn inherit_self() {
    run_file("test_data/class/inherit_self.lox");
}
#[test]
fn inherited_method() {
    run_file("test_data/class/inherited_method.lox");
}
#[test]
fn local_inherit_self() {
    run_file("test_data/class/local_inherit_self.lox");
}