pub mod parser;
pub mod runner;
pub mod scanner;

type Result<T> = std::result::Result<T, (usize, &'static str)>;
