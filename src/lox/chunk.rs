use crate::lox::value::{Value, ValueArray};

use std::fmt::Display;

#[repr(u8)]
pub enum OpCode {
    Constant,
    Nil,
    True,
    False,
    Equal,
    Greater,
    Less,
    Add,
    Subtract,
    Multiply,
    Divide,
    Not,
    Negate,
    Return,
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            OpCode::Constant => write!(f, "OP_CONSTANT"),
            OpCode::Nil => write!(f, "OP_NIL"),
            OpCode::True => write!(f, "OP_TRUE"),
            OpCode::False => write!(f, "OP_FALSE"),
            OpCode::Equal => write!(f, "OP_EQUAL"),
            OpCode::Greater => write!(f, "OP_GREATER"),
            OpCode::Less => write!(f, "OP_LESS"),
            OpCode::Add => write!(f, "OP_ADD"),
            OpCode::Subtract => write!(f, "OP_SUBTRACT"),
            OpCode::Multiply => write!(f, "OP_MULTIPLY"),
            OpCode::Divide => write!(f, "OP_DIVIDE"),
            OpCode::Not => write!(f, "OP_NOT"),
            OpCode::Negate => write!(f, "OP_NEGATE"),
            OpCode::Return => write!(f, "OP_RETURN"),
        }
    }
}

impl OpCode {
    pub fn from_u8(value: u8) -> OpCode {
        match value {
            0 => OpCode::Constant,
            1 => OpCode::Nil,
            2 => OpCode::True,
            3 => OpCode::False,
            4 => OpCode::Equal,
            5 => OpCode::Greater,
            6 => OpCode::Less,
            7 => OpCode::Add,
            8 => OpCode::Subtract,
            9 => OpCode::Multiply,
            10 => OpCode::Divide,
            11 => OpCode::Not,
            12 => OpCode::Negate,
            13 => OpCode::Return,
            _ => panic!("Invalid opcode"),
        }
    }
}

pub struct Chunk {
    code: Vec<u8>,
    lines: Vec<u32>,
    constants: ValueArray,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: Vec::new(),
            lines: Vec::new(),
            constants: ValueArray::new(),
        }
    }

    pub fn write(&mut self, opcode: OpCode, line: u32) {
        self.code.push(opcode as u8);
        self.lines.push(line);
    }

    pub fn write_constant_index(&mut self, constant_index: u8, line: u32) {
        self.code.push(constant_index);
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, value: Value) -> Result<u8, String> {
        if self.constants.values.len() >= u8::MAX as usize {
            return Err(String::from("Too many constants in one chunk"));
        }
        self.constants.write(value);
        
        Ok((self.constants.values.len() - 1) as u8)
    }

    pub fn byte(&self, offset: usize) -> u8 {
        self.code[offset]
    }

    pub fn read_constant(&self, offset: usize) -> &Value {
        self.constants.read(self.code[offset] as usize)
    }

    pub fn get_line(&self, offset: usize) -> u32 {
        self.lines[offset]
    }

    pub fn disassemble(&self, name: &str) {
        println!("== {} ==", name);

        let mut offset = 0;

        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset);
        }
    }

    pub fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{:04} ", offset);

        if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            print!("   | ");
        } else {
            print!("{:4} ", self.lines[offset]);
        }

        let code = OpCode::from_u8(self.code[offset]);

        match code {
            OpCode::Constant => self.constant_instruction("OP_CONSTANT", offset),
            OpCode::Nil => self.simple_instruction("OP_NIL", offset),
            OpCode::True => self.simple_instruction("OP_TRUE", offset),
            OpCode::False => self.simple_instruction("OP_FALSE", offset),
            OpCode::Equal => self.simple_instruction("OP_EQUAL", offset),
            OpCode::Greater => self.simple_instruction("OP_GREATER", offset),
            OpCode::Less => self.simple_instruction("OP_LESS", offset),
            OpCode::Add => self.simple_instruction("OP_ADD", offset),
            OpCode::Subtract => self.simple_instruction("OP_SUBTRACT", offset),
            OpCode::Multiply => self.simple_instruction("OP_MULTIPLY", offset),
            OpCode::Divide => self.simple_instruction("OP_DIVIDE", offset),
            OpCode::Not => self.simple_instruction("OP_NOT", offset),
            OpCode::Negate => self.simple_instruction("OP_NEGATE", offset),
            OpCode::Return => self.simple_instruction("OP_RETURN", offset),
        }
    }

    fn constant_instruction(&self, name: &str, offset: usize) -> usize {
        let value_idx = self.code[offset + 1];
        println!("{:16} {:4} '{}'", name, value_idx, self.constants.read(value_idx as usize));
        offset + 2
    }

    fn simple_instruction(&self, name: &str, offset: usize) -> usize {
        println!("{}", name);
        offset + 1
    }
}
