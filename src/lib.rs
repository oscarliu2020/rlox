pub mod runner;
pub mod scanner;
#[derive(Debug)]
pub enum Error {
    UnexpectedToken,
    UnterminatedString,
}
