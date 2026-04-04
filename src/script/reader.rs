use crate::script::value::LispError;
use crate::script::value::LispVal;

/// Parses all top-level expressions from `input` and returns them in order.
pub fn read_all(input: &str) -> Result<Vec<LispVal>, LispError> {
    let tokens: Vec<Token> = tokenize(input)?;
    let mut position: usize = 0;
    let mut expressions: Vec<LispVal> = Vec::new();
    while position < tokens.len() {
        let (expression, next_position) = parse(&tokens, position)?;
        expressions.push(expression);
        position = next_position;
    }
    Ok(expressions)
}


#[derive(Debug)]
enum Token {
    Open,
    Close,
    Atom(String),
}

fn tokenize(input: &str) -> Result<Vec<Token>, LispError> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' | '\n' | '\r' => { chars.next(); }
            ';' => {
                // Line comment: consume through end of line
                while let Some(&next_ch) = chars.peek() {
                    chars.next();
                    if next_ch == '\n' { break; }
                }
            }
            '(' => { tokens.push(Token::Open); chars.next(); }
            ')' => { tokens.push(Token::Close); chars.next(); }
            '"' => {
                chars.next(); // consume opening `"`
                let mut string: String = String::new();
                let mut escaped: bool = false;
                loop {
                    match chars.next() {
                        None => return Err(LispError::Parse("unterminated string literal".to_string())),
                        Some('\\') if !escaped => { escaped = true; }
                        Some('"')  if !escaped => break,
                        Some('n')  if  escaped => { string.push('\n'); escaped = false; }
                        Some('t')  if  escaped => { string.push('\t'); escaped = false; }
                        Some('\\') if  escaped => { string.push('\\'); escaped = false; }
                        Some('"')  if  escaped => { string.push('"');  escaped = false; }
                        Some(other) if escaped  => { string.push('\\'); string.push(other); escaped = false; }
                        Some(other) => { string.push(other); }
                    }
                }
                tokens.push(Token::Atom(format!("\"{string}\"")));
            }
            _ => {
                let mut atom: String = String::new();
                while let Some(&next_ch) = chars.peek() {
                    if next_ch == '(' || next_ch == ')' || next_ch == '"' || next_ch.is_whitespace() {
                        break;
                    }
                    atom.push(next_ch);
                    chars.next();
                }
                if !atom.is_empty() {
                    tokens.push(Token::Atom(atom));
                }
            }
        }
    }
    Ok(tokens)
}

fn parse(tokens: &[Token], position: usize) -> Result<(LispVal, usize), LispError> {
    if position >= tokens.len() {
        return Err(LispError::Parse("unexpected end of input".to_string()));
    }
    match &tokens[position] {
        Token::Open  => parse_list(tokens, position + 1),
        Token::Close => Err(LispError::Parse("unexpected `)`".to_string())),
        Token::Atom(text) => Ok((parse_atom(text), position + 1)),
    }
}

fn parse_list(tokens: &[Token], start: usize) -> Result<(LispVal, usize), LispError> {
    let mut elements: Vec<LispVal> = Vec::new();
    let mut position: usize = start;
    loop {
        if position >= tokens.len() {
            return Err(LispError::Parse("unterminated list — missing `)`".to_string()));
        }
        match &tokens[position] {
            Token::Close => return Ok((LispVal::List(elements), position + 1)),
            _ => {
                let (element, next_position) = parse(tokens, position)?;
                elements.push(element);
                position = next_position;
            }
        }
    }
}

fn parse_atom(text: &str) -> LispVal {
    // String literal (stored in the token with surrounding quotes)
    if text.starts_with('"') && text.ends_with('"') && text.len() >= 2 {
        return LispVal::Str(text[1..text.len() - 1].to_string());
    }
    // Booleans
    if text == "true"  { return LispVal::Bool(true);  }
    if text == "false" { return LispVal::Bool(false); }
    // Nil
    if text == "nil" { return LispVal::Nil; }
    // Integer (try before float so `1` stays a `Num`)
    if let Ok(number) = text.parse::<i64>() {
        return LispVal::Num(number);
    }
    // Float
    if let Ok(number) = text.parse::<f64>() {
        return LispVal::Float(number);
    }
    // Symbol
    LispVal::Symbol(text.to_string())
}
