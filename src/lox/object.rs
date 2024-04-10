use core::fmt::Display;

#[derive(Clone)]
pub struct StringLiteral(pub u8);

impl Display for StringLiteral {
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

    pub fn add_string(&mut self, string: &str) -> StringLiteral {
        let start = self.string.len();
        self.string.push_str(string);
        let end = self.string.len();

        let id = self.next_id;
        self.data.push(StringData { start, end });

        self.next_id += 1;

        StringLiteral(id)
    }

    pub fn get_string(&self, StringLiteral(id): &StringLiteral) -> &str {
        let l = &self.data[*id as usize];
        &self.string[l.start..l.end]
    }

    pub fn is_max_string(&self) -> bool {
        self.next_id as u8 == u8::MAX
    } 
}
