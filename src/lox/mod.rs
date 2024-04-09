use std::fs::read_to_string;
use std::process::exit;

use crate::lox::vm::{VM, InterpretResult};

pub fn run_file(vm: &VM, path: &str) {
    let source = read_to_string(path).expect("Failed to read file");
    let result = vm.interpret(&source);

    match result {
        InterpretResult::Ok => {}
        InterpretResult::CompileError => {
            println!("Compile error");
            exit(65);
        }
        InterpretResult::RuntimeError => {
            println!("Runtime error");
            exit(70);
        }
    }
}
