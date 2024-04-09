use crate::lox::chunk::{Chunk, OpCode};
use crate::lox::compiler::compile;
use crate::lox::value::Value;

pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

struct Stack {
    values: Vec<Value>,
}

impl Stack {
    fn new() -> Stack {
        Stack {
            values: Vec::new(),
        }
    }

    fn push(&mut self, value: &Value) {
        self.values.push(value.clone())
    }

    fn pop(&mut self) -> Value {
        self.values.pop().unwrap()
    }

    fn reset(&mut self) {
        self.values.clear();
    }

    fn peek(&self, distance: usize) -> &Value {
        &self.values[self.values.len() - 1 - distance]
    }

    fn trace(&self) {
        print!("           ");
        if self.values.len() == 0 {
            println!("<empty>");
            return;
        }

        for i in 0..self.values.len() {
            print!("[ ");
            print!("{}", self.values[i]);
            print!(" ]");
        }
        println!("");
    }
}

macro_rules! dbg {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            println!($($arg)*);
        }
    };
}

macro_rules! dbg_if {
    ($cond: expr, $($arg:tt)*) => {
        if cfg!(debug_assertions) && $cond {
            println!($($arg)*);
        }
    };
}

macro_rules! stack_trace {
    ($stack: ident) => {
        if cfg!(debug_assertions) {
            $stack.trace();
        }
    }
}

macro_rules! disassemble_instruction {
    ($chunk: ident, $ip: ident) => {
        if cfg!(debug_assertions) {
            $chunk.disassemble_instruction($ip);
        }
    }
}

macro_rules! binary {
    ($stack: ident, $op: tt, $res_type: expr, $ip: tt, $debug: expr) => {
        {
            let b = $stack.pop();
            let a = $stack.pop();
            $stack.push(&$res_type(a.as_number() $op b.as_number()));
            dbg_if!($debug, "{} {} {}", stringify!($op), a, b);
            $ip += 1;
        }
    }
}

pub fn interpret(source: &String, debug: bool) -> InterpretResult {
    let mut stack = Stack::new();
    let res = compile(&source);
    match res {
        Ok(chunk) => {
            run(&chunk, &mut stack, debug)
        }
        Err(_) => InterpretResult::CompileError,
    }
}

fn run(chunk: &Chunk, stack: &mut Stack, debug: bool) -> InterpretResult {
    let mut ip = 0;
    loop {
        if cfg!(debug_assertions) {
            if debug {
                dbg!("");
                dbg!("Stack");
                stack_trace!(stack);
                dbg!("Instruction");
                disassemble_instruction!(chunk, ip);
            }
        }

        let instruction = chunk.byte(ip);
        let opcode = OpCode::from_u8(instruction);
        match opcode {
            OpCode::Constant => {
                let constant = chunk.read_constant(ip + 1);
                stack.push(constant);
                dbg_if!(debug, "Read {}", constant);
                ip += 2;
            },
            OpCode::Nil => {
                stack.push(&Value::Nil);
                dbg_if!(debug, "Push Nil");
                ip += 1;
            },
            OpCode::True => {
                stack.push(&Value::Bool(true));
                dbg_if!(debug, "Push True");
                ip += 1;
            },
            OpCode::False => {
                stack.push(&Value::Bool(false));
                dbg_if!(debug, "Push False");
                ip += 1;
            },
            OpCode::Equal => {
                let b = stack.pop();
                let a = stack.pop();
                stack.push(&Value::Bool(values_equal(&a, &b)));
                dbg_if!(debug, "Equal {} {}", a, b);
                ip += 1;
            },
            OpCode::Greater => binary!(stack, >, Value::Bool, ip, debug),
            OpCode::Less => binary!(stack, <, Value::Bool, ip, debug),
            OpCode::Add => binary!(stack, +, Value::Number, ip, debug),
            OpCode::Subtract => binary!(stack, -, Value::Number, ip, debug),
            OpCode::Multiply => binary!(stack, *, Value::Number, ip, debug),
            OpCode::Divide => binary!(stack, /, Value::Number, ip, debug),
            OpCode::Not => {
                let value = stack.pop();
                stack.push(&Value::Bool(is_falsy(&value)));
                dbg_if!(debug, "Not {}", value);
                ip += 1;
            },
            OpCode::Negate => {
                if !stack.peek(0).is_number() {
                    runtime_error(stack, opcode, chunk.get_line(ip), "Operand must be a number");
                    return InterpretResult::RuntimeError;
                }
                let value = stack.pop();
                stack.push(&Value::Number(-value.as_number()));
                dbg_if!(debug, "Negate {}", value);
                ip += 1;
            },
            OpCode::Return => {
                let value = stack.pop();
                dbg_if!(debug, "Return {}", value);
                return InterpretResult::Ok
            },
        }
    }
}

fn is_falsy(value: &Value) -> bool {
    match value {
        Value::Nil => true,
        Value::Bool(false) => true,
        _ => false,
    }
}

fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Nil, Value::Nil) => true,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Number(a), Value::Number(b)) => a == b,
        _ => false,
    }
}

fn runtime_error(stack: &mut Stack, opcode: OpCode, line: u32, message: &str) {
    eprintln!("[line {}] Runtime Error: {} {}", line, opcode, message);
    stack.reset();
}
