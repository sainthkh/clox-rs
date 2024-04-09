use crate::lox::vm::interpret;

use std::fs::read_to_string;

pub mod lox {
    pub mod chunk;
    pub mod compiler;
    pub mod scanner;
    pub mod value;
    pub mod vm;
}

fn main() {
    let file = read_to_string("src/scripts/main.lox").unwrap();

    interpret(&file, true);
}
