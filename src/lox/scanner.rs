#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen, RightParen, LeftBrace, RightBrace,
    Comma, Dot, Minus, Plus, Semicolon, Slash, Star,

    // One or two character tokens.
    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,

    // Literals.
    Identifier, String, Number,

    // Keywords.
    And, Class, Else, False, Fun, For, If, Nil, Or,
    Print, Return, Super, This, True, Var, While,

    Error, EOF,
}

#[derive(Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub start: usize,
    pub length: usize,
    pub line: u32,
}

pub struct ErrorToken {
    pub message: String,
    pub line: u32,
}

impl Token {
    pub fn new(token_type: TokenType, start: usize, length: usize, line: u32) -> Token {
        Token {
            token_type,
            start,
            length,
            line,
        }
    }
}

fn make_token(token_type: TokenType, pointer: &ScannerPointer, line: &u32) -> Token {
    Token::new(token_type, pointer.start, pointer.current - pointer.start, *line)
}

fn make_error_token(message: &str, line: &u32) -> ErrorToken {
    ErrorToken {
        message: message.to_string(),
        line: *line,
    }
}

pub struct ScannerPointer {
    start: usize,
    current: usize,
}

impl ScannerPointer {
    pub fn new() -> ScannerPointer {
        ScannerPointer {
            start: 0,
            current: 0,
        }
    }
}

pub fn scan_token(source: &String, pointer: &mut ScannerPointer, line: &mut u32) -> Result<Token, ErrorToken> {
    skip_whitespace(source, pointer, line);
    pointer.start = pointer.current;

    if is_at_end(source, pointer) {
        return Ok(make_token(TokenType::EOF, pointer, line));
    }

    let c = advance(source, pointer);

    if is_alpha(c) {
        return Ok(identifier(source, pointer, line));
    }

    if is_digit(c) {
        return Ok(number(source, pointer, line));
    }

    match c {
        '(' => Ok(make_token(TokenType::LeftParen, pointer, line)),
        ')' => Ok(make_token(TokenType::RightParen, pointer, line)),
        '{' => Ok(make_token(TokenType::LeftBrace, pointer, line)),
        '}' => Ok(make_token(TokenType::RightBrace, pointer, line)),
        ';' => Ok(make_token(TokenType::Semicolon, pointer, line)),
        ',' => Ok(make_token(TokenType::Comma, pointer, line)),
        '.' => Ok(make_token(TokenType::Dot, pointer, line)),
        '-' => Ok(make_token(TokenType::Minus, pointer, line)),
        '+' => Ok(make_token(TokenType::Plus, pointer, line)),
        '/' => Ok(make_token(TokenType::Slash, pointer, line)),
        '*' => Ok(make_token(TokenType::Star, pointer, line)),
        '!' => {
            if match_char(source, pointer, '=') {
                Ok(make_token(TokenType::BangEqual, pointer, line))
            } else {
                Ok(make_token(TokenType::Bang, pointer, line))
            }
        },
        '=' => {
            if match_char(source, pointer, '=') {
                Ok(make_token(TokenType::EqualEqual, pointer, line))
            } else {
                Ok(make_token(TokenType::Equal, pointer, line))
            }
        },
        '<' => {
            if match_char(source, pointer, '=') {
                Ok(make_token(TokenType::LessEqual, pointer, line))
            } else {
                Ok(make_token(TokenType::Less, pointer, line))
            }
        },
        '>' => {
            if match_char(source, pointer, '=') {
                Ok(make_token(TokenType::GreaterEqual, pointer, line))
            } else {
                Ok(make_token(TokenType::Greater, pointer, line))
            }
        },
        '"' => string(source, pointer, line),
        _ => Err(make_error_token("Unexpected character.", line)),
    }
}

fn identifier(source: &String, pointer: &mut ScannerPointer, line: &mut u32) -> Token {
    while is_alphanumeric(peek(source, pointer)) {
        advance(source, pointer);
    }

    let text = &source[pointer.start..pointer.current];
    let token_type = match text {
        "and" => TokenType::And,
        "class" => TokenType::Class,
        "else" => TokenType::Else,
        "false" => TokenType::False,
        "for" => TokenType::For,
        "fun" => TokenType::Fun,
        "if" => TokenType::If,
        "nil" => TokenType::Nil,
        "or" => TokenType::Or,
        "print" => TokenType::Print,
        "return" => TokenType::Return,
        "super" => TokenType::Super,
        "this" => TokenType::This,
        "true" => TokenType::True,
        "var" => TokenType::Var,
        "while" => TokenType::While,
        _ => TokenType::Identifier,
    };

    make_token(token_type, pointer, line)
}

fn number(source: &String, pointer: &mut ScannerPointer, line: &mut u32) -> Token {
    while is_digit(peek(source, pointer)) {
        advance(source, pointer);
    }

    if peek(source, pointer) == '.' && is_digit(peek_next(source, pointer)) {
        advance(source, pointer);
        while is_digit(peek(source, pointer)) {
            advance(source, pointer);
        }
    }

    make_token(TokenType::Number, pointer, line)
}

fn string(source: &String, pointer: &mut ScannerPointer, line: &mut u32) -> Result<Token, ErrorToken> {
    while peek(source, pointer) != '"' && !is_at_end(source, pointer) {
        if peek(source, pointer) == '\n' {
            *line += 1;
        }
        advance(source, pointer);
    }

    if is_at_end(source, pointer) {
        return Err(make_error_token("Unterminated string.", line));
    }

    advance(source, pointer);
    
    Ok(make_token(TokenType::String, pointer, line))
}

fn match_char(source: &String, pointer: &mut ScannerPointer, expected: char) -> bool {
    if is_at_end(source, pointer) {
        return false;
    }
    if source.chars().nth(pointer.current).unwrap() != expected {
        return false;
    }

    pointer.current += 1;
    true
}

fn skip_whitespace(source: &String, pointer: &mut ScannerPointer, line: &mut u32) {
    loop {
        let c = peek(source, pointer);

        match c {
            ' ' | '\r' | '\t' => {
                advance(source, pointer);
            },
            '\n' => {
                *line += 1;
                advance(source, pointer);
            },
            '/' => {
                if peek_next(source, pointer) == '/' {
                    while peek(source, pointer) != '\n' && !is_at_end(source, pointer) {
                        advance(source, pointer);
                    }
                } else {
                    return;
                }
            },
            _ => return,
        }
    }
}

fn advance (source: &String, pointer: &mut ScannerPointer) -> char {
    pointer.current += 1;
    source.chars().nth(pointer.current - 1).unwrap()
}

fn peek(source: &String, pointer: &ScannerPointer) -> char {
    if is_at_end(source, pointer) {
        return '\0';
    }
    source.chars().nth(pointer.current).unwrap()
}

fn peek_next(source: &String, pointer: &ScannerPointer) -> char {
    if pointer.current + 1 >= source.len() {
        return '\0';
    }
    source.chars().nth(pointer.current + 1).unwrap()
}

fn is_digit(c: char) -> bool {
    c >= '0' && c <= '9'
}

fn is_alpha(c: char) -> bool {
    c >= 'a' && c <= 'z' || c >= 'A' && c <= 'Z' || c == '_'
}

fn is_alphanumeric(c: char) -> bool {
    is_alpha(c) || is_digit(c)
}

fn is_at_end(source: &String, pointer: &ScannerPointer) -> bool {
    pointer.current >= source.len()
}

