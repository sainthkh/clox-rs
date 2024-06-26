use crate::lox::scanner::{TokenType, Token, ScannerPointer, scan_token};
use crate::lox::chunk::{OpCode, Chunk};
use crate::lox::value::Value;
use crate::lox::object::StringId;

#[derive(PartialEq, PartialOrd)]
enum Precedence {
    None,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! - +
    Call,       // . ()
    Primary,    // literals
}

impl Precedence {
    fn next_higher_precedence(precedence: &Precedence) -> Precedence {
        match precedence {
            Precedence::None => Precedence::Assignment,
            Precedence::Assignment => Precedence::Or,
            Precedence::Or => Precedence::And,
            Precedence::And => Precedence::Equality,
            Precedence::Equality => Precedence::Comparison,
            Precedence::Comparison => Precedence::Term,
            Precedence::Term => Precedence::Factor,
            Precedence::Factor => Precedence::Unary,
            Precedence::Unary => Precedence::Call,
            Precedence::Call => Precedence::Primary,
            Precedence::Primary => panic!("No higher precedence than Primary"),
        }
    }
}

type ParseFn = fn(&mut Chunk, &String, &mut CompilerContext);

struct ParseRule {
    prefix: Option<ParseFn>,
    infix: Option<ParseFn>,
    precedence: Precedence,
}

impl ParseRule {
    fn new(prefix: Option<ParseFn>, infix: Option<ParseFn>, precedence: Precedence) -> ParseRule {
        ParseRule {
            prefix,
            infix,
            precedence,
        }
    }

    fn query(token_type: TokenType) -> ParseRule {
        match token_type {
            TokenType::LeftParen => ParseRule::new(Some(grouping), None, Precedence::None),
            TokenType::RightParen => ParseRule::new(None, None, Precedence::None),
            TokenType::LeftBrace => ParseRule::new(None, None, Precedence::None),
            TokenType::RightBrace => ParseRule::new(None, None, Precedence::None),
            TokenType::Comma => ParseRule::new(None, None, Precedence::None),
            TokenType::Dot => ParseRule::new(None, None, Precedence::None),
            TokenType::Minus => ParseRule::new(Some(unary), Some(binary), Precedence::Term),
            TokenType::Plus => ParseRule::new(None, Some(binary), Precedence::Term),
            TokenType::Semicolon => ParseRule::new(None, None, Precedence::None),
            TokenType::Slash => ParseRule::new(None, Some(binary), Precedence::Factor),
            TokenType::Star => ParseRule::new(None, Some(binary), Precedence::Factor),
            TokenType::Bang => ParseRule::new(Some(unary), None, Precedence::None),
            TokenType::BangEqual => ParseRule::new(None, Some(binary), Precedence::Equality),
            TokenType::Equal => ParseRule::new(None, None, Precedence::None),
            TokenType::EqualEqual => ParseRule::new(None, Some(binary), Precedence::Equality),
            TokenType::Greater => ParseRule::new(None, Some(binary), Precedence::Comparison),
            TokenType::GreaterEqual => ParseRule::new(None, Some(binary), Precedence::Comparison),
            TokenType::Less => ParseRule::new(None, Some(binary), Precedence::Comparison),
            TokenType::LessEqual => ParseRule::new(None, Some(binary), Precedence::Comparison),
            TokenType::Identifier => ParseRule::new(Some(variable), None, Precedence::None),
            TokenType::String => ParseRule::new(Some(string), None, Precedence::None),
            TokenType::Number => ParseRule::new(Some(number), None, Precedence::None),
            TokenType::And => ParseRule::new(None, None, Precedence::None),
            TokenType::Class => ParseRule::new(None, None, Precedence::None),
            TokenType::Else => ParseRule::new(None, None, Precedence::None),
            TokenType::False => ParseRule::new(Some(literal), None, Precedence::None),
            TokenType::Fun => ParseRule::new(None, None, Precedence::None),
            TokenType::For => ParseRule::new(None, None, Precedence::None),
            TokenType::If => ParseRule::new(None, None, Precedence::None),
            TokenType::Nil => ParseRule::new(Some(literal), None, Precedence::None),
            TokenType::Or => ParseRule::new(None, None, Precedence::None),
            TokenType::Print => ParseRule::new(None, None, Precedence::None),
            TokenType::Return => ParseRule::new(None, None, Precedence::None),
            TokenType::Super => ParseRule::new(None, None, Precedence::None),
            TokenType::This => ParseRule::new(None, None, Precedence::None),
            TokenType::True => ParseRule::new(Some(literal), None, Precedence::None),
            TokenType::Var => ParseRule::new(None, None, Precedence::None),
            TokenType::While => ParseRule::new(None, None, Precedence::None),
            TokenType::Error => ParseRule::new(None, None, Precedence::None),
            TokenType::EOF => ParseRule::new(None, None, Precedence::None),
        }
    }
}

struct CompilerContext {
    sp: ScannerPointer,
    pp: ParserPointer,
    ps: ParserState,
    can_assign: bool,
    line: u32,
}

struct ParserPointer {
    current: Token,
    previous: Token,
}

struct ParserState {
    panic_mode: bool,
    had_error: bool,
}

pub fn compile(source: &String) -> Result<Chunk, ()> {
    let mut chunk = Chunk::new();
    let mut ctx = CompilerContext {
        sp: ScannerPointer::new(),
        pp: ParserPointer {
            current: Token::new(TokenType::EOF, 0, 0, 0),
            previous: Token::new(TokenType::EOF, 0, 0, 0),
        },
        ps: ParserState {
            panic_mode: false,
            had_error: false,
        },
        can_assign: false,
        line: 1,
    };
    advance(source, &mut ctx);

    loop {
        if match_token(TokenType::EOF, source, &mut ctx) {
            break;
        }

        declaration(&mut chunk, source, &mut ctx);
    }

    // expression(&mut chunk, source, &mut ctx);
    consume(TokenType::EOF, "Expect end of expression.", source, &mut ctx);
    chunk.write(OpCode::Return, ctx.line);

    Ok(chunk)
}

fn match_token(token_type: TokenType, source: &String, ctx: &mut CompilerContext) -> bool {
    if !check(token_type, &ctx.pp) {
        return false;
    }

    advance(source, ctx);

    true
} 

fn check(token_type: TokenType, pp: &ParserPointer) -> bool {
    pp.current.token_type == token_type
}

fn declaration(chunk: &mut Chunk, source: &String, ctx: &mut CompilerContext) {
    match ctx.pp.current.token_type {
        TokenType::Var => var_declaration(chunk, source, ctx),
        _ => statement(chunk, source, ctx),
    }

    if ctx.ps.panic_mode {
        synchronize(source, ctx);
    }
}

fn var_declaration(chunk: &mut Chunk, source: &String, ctx: &mut CompilerContext) {
    advance(source, ctx);

    let global = parse_variable("Expect variable name.", chunk, source, ctx);

    let global = match global {
        Ok(global) => global,
        Err(msg) => {
            error_at(ctx.pp.previous.line, &msg, &mut ctx.ps);
            return;
        },
    };

    if match_token(TokenType::Equal, source, ctx) {
        expression(chunk, source, ctx);
    } else {
        chunk.write(OpCode::Nil, ctx.pp.previous.line);
    }

    consume(TokenType::Semicolon, "Expect ';' after variable declaration.", source, ctx);

    define_variable(&global, chunk, ctx);
}

fn parse_variable(error_msg: &str, chunk: &mut Chunk, source: &String, ctx: &mut CompilerContext) -> Result<StringId, String>{
    consume(TokenType::Identifier, error_msg, source, ctx);
    identifier_constant(chunk, source, ctx)
}

fn identifier_constant(chunk: &mut Chunk, source: &String, ctx: &mut CompilerContext) -> Result<StringId, String> {
    let name = &source[ctx.pp.previous.start..ctx.pp.previous.start + ctx.pp.previous.length];
    
    chunk.write(OpCode::StringLiteral, ctx.pp.previous.line);
    let idx = chunk.add_or_retrieve_string_literal(name);

    match idx {
        Ok(idx) => {
            chunk
                .write_string_literal_id(&idx, ctx.pp.previous.line)
                .expect("Failed to write variable as string literal id");
            
            return Ok(idx);
        },
        Err(msg) => {
            let msg = format!("Failed to add string literal: {}", msg);

            return Err(msg);
        },
    }
}

fn define_variable(global: &StringId, chunk: &mut Chunk, ctx: &mut CompilerContext) {
    chunk.write(OpCode::DefineGlobal, ctx.pp.previous.line);
    chunk.write_u8(global.0 as u8, ctx.pp.previous.line);
}

fn synchronize(source: &String, ctx: &mut CompilerContext) {
    ctx.ps.panic_mode = false;

    while ctx.pp.current.token_type != TokenType::EOF {
        if ctx.pp.previous.token_type == TokenType::Semicolon {
            return;
        }

        match ctx.pp.current.token_type {
            TokenType::Class | 
            TokenType::Fun | 
            TokenType::Var | 
            TokenType::For | 
            TokenType::If | 
            TokenType::While | 
            TokenType::Print | 
            TokenType::Return => return,
            _ => (),
        }

        advance(source, ctx);
    }
}

fn statement(chunk: &mut Chunk, source: &String, ctx: &mut CompilerContext) {
    if match_token(TokenType::Print, source, ctx) {
        print_statement(chunk, source, ctx);
    } else {
        expression_statement(chunk, source, ctx);
    }
}

fn print_statement(chunk: &mut Chunk, source: &String, ctx: &mut CompilerContext) {
    expression(chunk, source, ctx);
    consume(TokenType::Semicolon, "Expect ';' after value.", source, ctx);
    chunk.write(OpCode::Print, ctx.pp.previous.line);
}

fn expression_statement(chunk: &mut Chunk, source: &String, ctx: &mut CompilerContext) {
    expression(chunk, source, ctx);
    consume(TokenType::Semicolon, "Expect ';' after expression.", source, ctx);
    chunk.write(OpCode::Pop, ctx.pp.previous.line);
}

fn expression(
    chunk: &mut Chunk, 
    source: &String, 
    ctx: &mut CompilerContext
) {
    parse_precedence(Precedence::Assignment, chunk, source, ctx);
}

fn variable(
    chunk: &mut Chunk, 
    source: &String, 
    ctx: &mut CompilerContext
) {
    named_variable(chunk, source, ctx);
}

fn named_variable(
    chunk: &mut Chunk,
    source: &String,
    ctx: &mut CompilerContext
) {
    let arg = identifier_constant(chunk, source, ctx);

    match arg {
        Ok(arg) => {
            let arg = arg.0 as u8;
            if ctx.can_assign && match_token(TokenType::Equal, source, ctx) {
                expression(chunk, source, ctx);
                chunk.write(OpCode::SetGlobal, ctx.pp.previous.line);
                chunk.write_u8(arg, ctx.pp.previous.line);
            } else {
                chunk.write(OpCode::GetGlobal, ctx.pp.previous.line);
                chunk.write_u8(arg, ctx.pp.previous.line);
            }
        },
        Err(msg) => error_at(ctx.pp.previous.line, &msg, &mut ctx.ps),
    }
}

fn string(
    chunk: &mut Chunk, 
    source: &String, 
    ctx: &mut CompilerContext
) {
    let string = &source[(ctx.pp.previous.start + 1)..(ctx.pp.previous.start + ctx.pp.previous.length - 1)];
    
    chunk.write(OpCode::StringLiteral, ctx.pp.previous.line);
    let idx = chunk.add_or_retrieve_string_literal(string);

    match idx {
        Ok(idx) => 
            chunk
                .write_string_literal_id(&idx, ctx.pp.previous.line)
                .expect("Failed to write string literal id"),
        Err(msg) => error_at(ctx.pp.previous.line, &msg, &mut ctx.ps),
    }
}

fn number(
    chunk: &mut Chunk, 
    source: &String, 
    ctx: &mut CompilerContext
) {
    let number = &source[ctx.pp.previous.start..ctx.pp.previous.start + ctx.pp.previous.length];
    let number = number.parse::<f64>().unwrap();
    
    chunk.write(OpCode::Constant, ctx.pp.previous.line);
    let idx = chunk.add_constant(Value::Number(number));
    match idx {
        Ok(idx) => chunk.write_u8(idx, ctx.pp.previous.line),
        Err(msg) => error_at(ctx.pp.previous.line, &msg, &mut ctx.ps),
    }
}

fn grouping(
    chunk: &mut Chunk,
    source: &String, 
    ctx: &mut CompilerContext
) {
    expression(chunk, source, ctx);
    consume(TokenType::RightParen, "Expect ')' after expression.", source, ctx);
}

fn unary(
    chunk: &mut Chunk,
    source: &String, 
    ctx: &mut CompilerContext
) {
    let operator_type = ctx.pp.previous.token_type;

    parse_precedence(Precedence::Unary, chunk, source, ctx);

    match operator_type {
        TokenType::Bang => chunk.write(OpCode::Not, ctx.pp.previous.line),
        TokenType::Minus => chunk.write(OpCode::Negate, ctx.pp.previous.line),
        _ => panic!("Unknown unary operator: {:?}", operator_type),
    }
}

fn binary(
    chunk: &mut Chunk,
    source: &String,
    ctx: &mut CompilerContext
) {
    let operator_type = ctx.pp.previous.token_type;
    let rule = ParseRule::query(operator_type);
    let precedence = Precedence::next_higher_precedence(&rule.precedence);

    parse_precedence(precedence, chunk, source, ctx);

    match operator_type {
        TokenType::BangEqual => {
            chunk.write(OpCode::Equal, ctx.pp.previous.line);
            chunk.write(OpCode::Not, ctx.pp.previous.line);
        },
        TokenType::EqualEqual => chunk.write(OpCode::Equal, ctx.pp.previous.line),
        TokenType::Greater => chunk.write(OpCode::Greater, ctx.pp.previous.line),
        TokenType::GreaterEqual => {
            chunk.write(OpCode::Less, ctx.pp.previous.line);
            chunk.write(OpCode::Not, ctx.pp.previous.line);
        },
        TokenType::Less => chunk.write(OpCode::Less, ctx.pp.previous.line),
        TokenType::LessEqual => {
            chunk.write(OpCode::Greater, ctx.pp.previous.line);
            chunk.write(OpCode::Not, ctx.pp.previous.line);
        },
        TokenType::Plus => chunk.write(OpCode::Add, ctx.pp.previous.line),
        TokenType::Minus => chunk.write(OpCode::Subtract, ctx.pp.previous.line),
        TokenType::Star => chunk.write(OpCode::Multiply, ctx.pp.previous.line),
        TokenType::Slash => chunk.write(OpCode::Divide, ctx.pp.previous.line),
        _ => panic!("Unknown binary operator: {:?}", operator_type),
    }
}

fn literal(
    chunk: &mut Chunk,
    _: &String, 
    ctx: &mut CompilerContext
) {
    match ctx.pp.previous.token_type {
        TokenType::True => chunk.write(OpCode::True, ctx.pp.previous.line),
        TokenType::False => chunk.write(OpCode::False, ctx.pp.previous.line),
        TokenType::Nil => chunk.write(OpCode::Nil, ctx.pp.previous.line),
        _ => panic!("Unknown literal type: {:?}", ctx.pp.previous.token_type),
    }
}

fn parse_precedence(
    precedence: Precedence,
    chunk: &mut Chunk,
    source: &String, 
    ctx: &mut CompilerContext
) {
    advance(source, ctx);
    let prefix_rule = ParseRule::query(ctx.pp.previous.token_type).prefix;
    let prefix_rule = match prefix_rule {
        Some(rule) => rule,
        None => {
            error_at(ctx.pp.previous.line, "Expect expression.", &mut ctx.ps);
            return;
        }
    };

    ctx.can_assign = precedence <= Precedence::Assignment;
    prefix_rule(chunk, source, ctx);

    while precedence <= ParseRule::query(ctx.pp.current.token_type).precedence {
        advance(source, ctx);
        let infix_rule = ParseRule::query(ctx.pp.previous.token_type).infix.unwrap();
        infix_rule(chunk, source, ctx);
    }

    if ctx.can_assign && match_token(TokenType::Equal, source, ctx) {
        error_at(ctx.pp.previous.line, "Invalid assignment target.", &mut ctx.ps);
    }
}

fn advance(
    source: &String, 
    ctx: &mut CompilerContext
) {
    ctx.pp.previous = ctx.pp.current.clone();
    
    loop {
        let result = scan_token(source, &mut ctx.sp, &mut ctx.line);

        match result {
            Ok(token) => {
                ctx.pp.current = token;
                break;
            }
            Err(err) => {
                error_at(err.line, &err.message, &mut ctx.ps);
            }
        }
    }
}

fn consume(
    token_type: TokenType,
    message: &str,
    source: &String, 
    ctx: &mut CompilerContext
) {
    if ctx.pp.current.token_type == token_type {
        advance(source, ctx);
        return;
    }

    error_at(ctx.line, message, &mut ctx.ps);
}

fn error_at(line: u32, message: &str, ps: &mut ParserState) {
    if ps.panic_mode {
        return;
    }

    ps.panic_mode = true;
    ps.had_error = true;

    eprintln!("[line {}] Error: {}", line, message);
}
