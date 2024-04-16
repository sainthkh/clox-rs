use core::fmt::Display;
use std::collections::HashMap;

const MAX_STRING_LITERAL: u8 = u8::MAX;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct StringId(pub u64);

impl StringId {
    pub fn is_literal(&self) -> bool {
        self.0 < MAX_STRING_LITERAL as u64
    }

    pub fn new_literal_id(id: u8) -> StringId {
        StringId(id as u64)
    }

    pub fn new_dynamic_id(id: u64) -> StringId {
        StringId(id)
    }
}

impl Display for StringId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "string literal: {}", self.0)
    }
}

struct StringData {
    start: usize,
    end: usize,
}

pub struct StringLiteralStorage {
    string: String,
    data: Vec<StringData>,
    next_id: u8,
}

impl StringLiteralStorage {
    pub fn new() -> StringLiteralStorage {
        StringLiteralStorage {
            string: String::new(),
            data: Vec::new(),
            next_id: 0,
        }
    }

    pub fn exist_string(&self, string: &str) -> Option<StringId> {
        for (i, l) in self.data.iter().enumerate() {
            if &self.string[l.start..l.end] == string {
                return Some(StringId(i as u64));
            }
        }

        None
    }

    pub fn add_string(&mut self, string: &str) -> Result<StringId, String> {
        if self.is_max_string() {
            return Err(String::from("Too many string literals"));
        }

        let start = self.string.len();
        self.string.push_str(string);
        let end = self.string.len();

        let id = self.next_id;
        self.data.push(StringData { start, end });

        self.next_id += 1;

        Ok(StringId(id as u64))
    }

    pub fn get_string(&self, StringId(id): &StringId) -> &str {
        let l = &self.data[*id as usize];
        &self.string[l.start..l.end]
    }

    pub fn is_max_string(&self) -> bool {
        self.next_id as u8 == MAX_STRING_LITERAL
    } 
}

pub struct DynamicStringStorage {
    string: String,
    data: HashMap<u64, StringData>,
    next_id: u64,
}

impl DynamicStringStorage {
    pub fn new() -> DynamicStringStorage {
        DynamicStringStorage {
            string: String::new(),
            data: HashMap::new(),
            next_id: MAX_STRING_LITERAL as u64,
        }
    }

    pub fn add_string(&mut self, string: &str) -> Result<StringId, String> {
        if self.next_id == u64::MAX {
            return Err(String::from("Too many string literals"));
        }

        let start = self.string.len();
        self.string.push_str(string);
        let end = self.string.len();

        let id = self.next_id;
        self.data.insert(id, StringData { start, end });

        self.next_id += 1;

        Ok(StringId(id))
    }

    pub fn get_string(&self, StringId(id): &StringId) -> &str {
        let l = self.data.get(&id).unwrap();
        &self.string[l.start..l.end]
    }
}
