use crate::lox::value::{Value, ValueArray};
use super::object::{StringId, StringLiteralStorage};

use std::fmt::Display;

#[repr(u8)]
pub enum OpCode {
    Constant,
    StringLiteral,
    Nil,
    True,
    False,
    Pop,
    GetGlobal,
    DefineGlobal,
    SetGlobal,
    Equal,
    Greater,
    Less,
    Add,
    Subtract,
    Multiply,
    Divide,
    Not,
    Negate,
    Print,
    Return,
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            OpCode::Constant => write!(f, "OP_CONSTANT"),
            OpCode::StringLiteral => write!(f, "OP_STRING_LITERAL"),
            OpCode::Nil => write!(f, "OP_NIL"),
            OpCode::True => write!(f, "OP_TRUE"),
            OpCode::False => write!(f, "OP_FALSE"),
            OpCode::Pop => write!(f, "OP_POP"),
            OpCode::GetGlobal => write!(f, "OP_GET_GLOBAL"),
            OpCode::DefineGlobal => write!(f, "OP_DEFINE_GLOBAL"),
            OpCode::SetGlobal => write!(f, "OP_SET_GLOBAL"),
            OpCode::Equal => write!(f, "OP_EQUAL"),
            OpCode::Greater => write!(f, "OP_GREATER"),
            OpCode::Less => write!(f, "OP_LESS"),
            OpCode::Add => write!(f, "OP_ADD"),
            OpCode::Subtract => write!(f, "OP_SUBTRACT"),
            OpCode::Multiply => write!(f, "OP_MULTIPLY"),
            OpCode::Divide => write!(f, "OP_DIVIDE"),
            OpCode::Not => write!(f, "OP_NOT"),
            OpCode::Negate => write!(f, "OP_NEGATE"),
            OpCode::Print => write!(f, "OP_PRINT"),
            OpCode::Return => write!(f, "OP_RETURN"),
        }
    }
}

impl OpCode {
    pub fn from_u8(value: u8) -> OpCode {
        match value {
            0 => OpCode::Constant,
            1 => OpCode::StringLiteral,
            2 => OpCode::Nil,
            3 => OpCode::True,
            4 => OpCode::False,
            5 => OpCode::Pop,
            6 => OpCode::GetGlobal,
            7 => OpCode::DefineGlobal,
            8 => OpCode::SetGlobal,
            9 => OpCode::Equal,
            10 => OpCode::Greater,
            11 => OpCode::Less,
            12 => OpCode::Add,
            13 => OpCode::Subtract,
            14 => OpCode::Multiply,
            15 => OpCode::Divide,
            16 => OpCode::Not,
            17 => OpCode::Negate,
            18 => OpCode::Print,
            19 => OpCode::Return,
            _ => panic!("Invalid opcode"),
        }
    }
}

pub struct Chunk {
    code: Vec<u8>,
    lines: Vec<u32>,
    constants: ValueArray,
    string_literals: StringLiteralStorage,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: Vec::new(),
            lines: Vec::new(),
            constants: ValueArray::new(),
            string_literals: StringLiteralStorage::new(),
        }
    }

    pub fn write(&mut self, opcode: OpCode, line: u32) {
        self.code.push(opcode as u8);
        self.lines.push(line);
    }

    pub fn write_u8(&mut self, v: u8, line: u32) {
        self.code.push(v);
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, value: Value) -> Result<u8, String> {
        if self.constants.values.len() >= u8::MAX as usize {
            return Err(String::from("Too many constants in one chunk"));
        }
        self.constants.write(value);
        
        Ok((self.constants.values.len() - 1) as u8)
    }

    pub fn write_string_literal_id(&mut self, id: &StringId, line: u32) -> Result<(), String> {
        if id.is_literal() {
            let id = id.0 as u8;
            self.code.push(id);
            self.lines.push(line);

            Ok(())
        } else {
            Err(String::from("Invalid string literal id"))
        }
    }

    pub fn add_or_retrieve_string_literal(&mut self, string: &str) -> Result<StringId, String> {
        match self.string_literals.exist_string(string) {
            Some(id) => Ok(id),
            None => self.string_literals.add_string(string),
        }
    }

    pub fn byte(&self, offset: usize) -> u8 {
        self.code[offset]
    }

    pub fn read_constant(&self, offset: usize) -> &Value {
        self.constants.read(self.code[offset] as usize)
    }

    pub fn read_string_literal(&self, literal: &StringId) -> &str {
        self.string_literals.get_string(literal)
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
            OpCode::StringLiteral => self.string_literal_instruction("OP_STRING_LITERAL", offset),
            OpCode::Nil => self.simple_instruction("OP_NIL", offset),
            OpCode::True => self.simple_instruction("OP_TRUE", offset),
            OpCode::False => self.simple_instruction("OP_FALSE", offset),
            OpCode::Pop => self.simple_instruction("OP_POP", offset),
            OpCode::GetGlobal => self.global_instruction("OP_GET_GLOBAL", offset),
            OpCode::DefineGlobal => self.global_instruction("OP_DEFINE_GLOBAL", offset),
            OpCode::SetGlobal => self.global_instruction("OP_SET_GLOBAL", offset),
            OpCode::Equal => self.simple_instruction("OP_EQUAL", offset),
            OpCode::Greater => self.simple_instruction("OP_GREATER", offset),
            OpCode::Less => self.simple_instruction("OP_LESS", offset),
            OpCode::Add => self.simple_instruction("OP_ADD", offset),
            OpCode::Subtract => self.simple_instruction("OP_SUBTRACT", offset),
            OpCode::Multiply => self.simple_instruction("OP_MULTIPLY", offset),
            OpCode::Divide => self.simple_instruction("OP_DIVIDE", offset),
            OpCode::Not => self.simple_instruction("OP_NOT", offset),
            OpCode::Negate => self.simple_instruction("OP_NEGATE", offset),
            OpCode::Print => self.simple_instruction("OP_PRINT", offset),
            OpCode::Return => self.simple_instruction("OP_RETURN", offset),
        }
    }

    pub fn print_codes(&self) {
        for (_, code) in self.code.iter().enumerate() {
            let code = OpCode::from_u8(*code);
            println!("{}", code);
        }
    }

    fn constant_instruction(&self, name: &str, offset: usize) -> usize {
        let value_idx = self.code[offset + 1];
        println!("{:16} {:4} '{}'", name, value_idx, self.constants.read(value_idx as usize));
        offset + 2
    }

    fn global_instruction(&self, name: &str, offset: usize) -> usize {
        let literal_idx = self.code[offset + 1];
        println!("{:16} {:4} '{}'", name, literal_idx, self.string_literals.get_string(&StringId::new_literal_id(literal_idx)));
        offset + 2
    }

    fn string_literal_instruction(&self, name: &str, offset: usize) -> usize {
        let literal_idx = self.code[offset + 1];
        println!("{:16} {:4} '{}'", name, literal_idx, self.string_literals.get_string(&StringId::new_literal_id(literal_idx)));
        offset + 2
    }

    fn simple_instruction(&self, name: &str, offset: usize) -> usize {
        println!("{}", name);
        offset + 1
    }
}
