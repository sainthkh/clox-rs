use crate::lox::chunk::{Chunk, OpCode};
use crate::lox::compiler::compile;
use crate::lox::value::Value;
use crate::lox::object::{StringId, DynamicStringStorage};

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

    fn is_empty(&self) -> bool {
        self.values.len() == 0
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
    ($env: ident) => {
        if cfg!(debug_assertions) {
            $env.stack.trace();
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
    ($env: ident, $op: tt, $res_type: expr, $ip: tt, $debug: expr) => {
        {
            let b = $env.stack.pop();
            let a = $env.stack.pop();
            $env.stack.push(&$res_type(a.as_number() $op b.as_number()));
            dbg_if!($debug, "{} {} {}", stringify!($op), a, b);
            $ip += 1;
        }
    }
}

struct Env {
    stack: Stack,
    dynamic_strings: DynamicStringStorage,
}

impl Env {
    fn new() -> Env {
        Env {
            stack: Stack::new(),
            dynamic_strings: DynamicStringStorage::new(),
        }
    }
}

pub fn interpret(source: &String, debug: bool) -> InterpretResult {
    let mut env = Env::new();
    let res = compile(&source);
    match res {
        Ok(chunk) => {
            run(&chunk, &mut env, debug)
        }
        Err(_) => InterpretResult::CompileError,
    }
}

fn run(chunk: &Chunk, env: &mut Env, debug: bool) -> InterpretResult {
    chunk.print_codes();

    let mut ip = 0;
    loop {
        if cfg!(debug_assertions) {
            if debug {
                dbg!("");
                dbg!("Stack");
                stack_trace!(env);
                dbg!("Instruction");
                disassemble_instruction!(chunk, ip);
            }
        }

        let instruction = chunk.byte(ip);
        let opcode = OpCode::from_u8(instruction);
        match opcode {
            OpCode::Constant => {
                let constant = chunk.read_constant(ip + 1);
                env.stack.push(constant);
                dbg_if!(debug, "Read {}", constant);
                ip += 2;
            },
            OpCode::StringLiteral => {
                let string_idx = chunk.byte(ip + 1);
                env.stack.push(&Value::String(StringId::new_literal_id(string_idx)));
                dbg_if!(debug, "Push StringLiteral {}", string_idx);
                ip += 2;
            }
            OpCode::Nil => {
                env.stack.push(&Value::Nil);
                dbg_if!(debug, "Push Nil");
                ip += 1;
            },
            OpCode::True => {
                env.stack.push(&Value::Bool(true));
                dbg_if!(debug, "Push True");
                ip += 1;
            },
            OpCode::False => {
                env.stack.push(&Value::Bool(false));
                dbg_if!(debug, "Push False");
                ip += 1;
            },
            OpCode::Equal => {
                let b = env.stack.pop();
                let a = env.stack.pop();
                env.stack.push(&Value::Bool(values_equal(&a, &b, &chunk)));
                dbg_if!(debug, "Equal {} {}", a, b);
                ip += 1;
            },
            OpCode::Greater => binary!(env, >, Value::Bool, ip, debug),
            OpCode::Less => binary!(env, <, Value::Bool, ip, debug),
            OpCode::Add => {
                let b = env.stack.pop();
                let a = env.stack.pop();
                match (a, b) {
                    (Value::Number(a), Value::Number(b)) => {
                        env.stack.push(&Value::Number(a + b));
                        dbg!("Add numbers {} {}", a, b);
                    },
                    (Value::String(a), Value::String(b)) => {
                        let a_str = chunk.read_string_literal(&a);
                        let b_str = chunk.read_string_literal(&b);
                        let mut new_string = String::new();
                        new_string.push_str(&a_str);
                        new_string.push_str(&b_str);
                        dbg!("Add strings {} {} {}", a_str, b_str, new_string);

                        let new_dynamic_string = env.dynamic_strings.add_string(&new_string).expect("Too many dynamic strings");
                        env.stack.push(&Value::String(new_dynamic_string));
                    },
                    _ => {
                        runtime_error(&mut env.stack, opcode, chunk.get_line(ip), "Operands must be two numbers or two strings");
                        return InterpretResult::RuntimeError;
                    }
                }
                ip += 1;
            }
            OpCode::Subtract => binary!(env, -, Value::Number, ip, debug),
            OpCode::Multiply => binary!(env, *, Value::Number, ip, debug),
            OpCode::Divide => binary!(env, /, Value::Number, ip, debug),
            OpCode::Not => {
                let value = env.stack.pop();
                env.stack.push(&Value::Bool(is_falsy(&value)));
                dbg_if!(debug, "Not {}", value);
                ip += 1;
            },
            OpCode::Negate => {
                if !env.stack.peek(0).is_number() {
                    runtime_error(&mut env.stack, opcode, chunk.get_line(ip), "Operand must be a number");
                    return InterpretResult::RuntimeError;
                }
                let value = env.stack.pop();
                env.stack.push(&Value::Number(-value.as_number()));
                dbg_if!(debug, "Negate {}", value);
                ip += 1;
            },
            OpCode::Print => {
                let value = env.stack.pop();
                dbg_if!(debug, "Print {}", value);
                print_value(&value, &chunk, env);
                ip += 1;
            },
            OpCode::Return => {
                if env.stack.is_empty() {
                    dbg_if!(debug, "Stack Empty. Return Nothing")
                } else {
                    let value = env.stack.pop();
                    dbg_if!(debug, "Return {}", value);
                };
                return InterpretResult::Ok
            },
        }
    }
}

fn print_value(value: &Value, chunk: &Chunk, env: &Env) {
    match value {
        Value::Nil => println!("nil"),
        Value::Bool(b) => println!("{}", b),
        Value::Number(n) => println!("{}", n),
        Value::String(id) => {
            let string = if id.is_literal() {
                chunk.read_string_literal(id)
            } else {
                env.dynamic_strings.get_string(id)
            };

            println!("{}", string);
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

fn values_equal(a: &Value, b: &Value, chunk: &Chunk) -> bool {
    match (a, b) {
        (Value::Nil, Value::Nil) => true,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Number(a), Value::Number(b)) => a == b,
        (Value::String(a), Value::String(b)) => {
            let a_str = chunk.read_string_literal(a);
            let b_str = chunk.read_string_literal(b);
            a_str == b_str
        }
        _ => false,
    }
}

fn runtime_error(stack: &mut Stack, opcode: OpCode, line: u32, message: &str) {
    eprintln!("[line {}] Runtime Error: {} {}", line, opcode, message);
    stack.reset();
}
