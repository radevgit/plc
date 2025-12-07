// Generated SCL lexer

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    And,
    Array,
    Begin,
    By,
    Case,
    DataBlock,
    Do,
    Else,
    Elsif,
    EndCase,
    EndDataBlock,
    EndFor,
    EndFunction,
    EndFunctionBlock,
    EndIf,
    EndRepeat,
    EndStruct,
    EndType,
    EndVar,
    EndWhile,
    False,
    For,
    Function,
    FunctionBlock,
    If,
    Mod,
    Not,
    Of,
    Or,
    Pointer,
    Repeat,
    Return,
    Struct,
    Then,
    To,
    True,
    Type,
    Until,
    Var,
    VarInput,
    VarInOut,
    VarOutput,
    VarTemp,
    While,
    Xor,
    Identifier(String),
    Integer(String),
    Real(String),
    StringLit(String),
    Operator(String),
    Eof,
}


#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: (usize, usize),
}

impl Token {
    pub fn new(kind: TokenKind, start: usize, end: usize) -> Self {
        Token { kind, span: (start, end) }
    }
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            pos: 0,
        }
    }
}

impl Lexer {
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();
        
        let start = self.pos;
        
        if self.pos >= self.input.len() {
            return Token::new(TokenKind::Eof, start, start);
        }
        
        let ch = self.input[self.pos];
        
        // Try to match keywords first
        if let Some(token) = self.try_keyword(start) {
            return token;
        }
        
        // Match identifier
        if ch.is_alphabetic() || ch == '_' {
            return self.read_identifier(start);
        }
        
        // Match number
        if ch.is_numeric() {
            return self.read_number(start);
        }
        
        // Match string
        if ch == '\'' {
            return self.read_string(start);
        }
        
        // Match operators
        self.read_operator(start)
    }
    
    fn try_keyword(&mut self, start: usize) -> Option<Token> {
        let remaining: String = self.input[self.pos..].iter().collect();
        if remaining.starts_with("END_FUNCTION_BLOCK") {
            let end = self.pos + 18;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::EndFunctionBlock, start, end));
            }
        }
        if remaining.starts_with("END_DATA_BLOCK") {
            let end = self.pos + 14;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::EndDataBlock, start, end));
            }
        }
        if remaining.starts_with("FUNCTION_BLOCK") {
            let end = self.pos + 14;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::FunctionBlock, start, end));
            }
        }
        if remaining.starts_with("END_FUNCTION") {
            let end = self.pos + 12;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::EndFunction, start, end));
            }
        }
        if remaining.starts_with("VAR_OUTPUT") {
            let end = self.pos + 10;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::VarOutput, start, end));
            }
        }
        if remaining.starts_with("DATA_BLOCK") {
            let end = self.pos + 10;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::DataBlock, start, end));
            }
        }
        if remaining.starts_with("VAR_IN_OUT") {
            let end = self.pos + 10;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::VarInOut, start, end));
            }
        }
        if remaining.starts_with("END_STRUCT") {
            let end = self.pos + 10;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::EndStruct, start, end));
            }
        }
        if remaining.starts_with("END_REPEAT") {
            let end = self.pos + 10;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::EndRepeat, start, end));
            }
        }
        if remaining.starts_with("END_WHILE") {
            let end = self.pos + 9;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::EndWhile, start, end));
            }
        }
        if remaining.starts_with("VAR_INPUT") {
            let end = self.pos + 9;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::VarInput, start, end));
            }
        }
        if remaining.starts_with("END_CASE") {
            let end = self.pos + 8;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::EndCase, start, end));
            }
        }
        if remaining.starts_with("END_TYPE") {
            let end = self.pos + 8;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::EndType, start, end));
            }
        }
        if remaining.starts_with("VAR_TEMP") {
            let end = self.pos + 8;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::VarTemp, start, end));
            }
        }
        if remaining.starts_with("FUNCTION") {
            let end = self.pos + 8;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::Function, start, end));
            }
        }
        if remaining.starts_with("END_FOR") {
            let end = self.pos + 7;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::EndFor, start, end));
            }
        }
        if remaining.starts_with("END_VAR") {
            let end = self.pos + 7;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::EndVar, start, end));
            }
        }
        if remaining.starts_with("POINTER") {
            let end = self.pos + 7;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::Pointer, start, end));
            }
        }
        if remaining.starts_with("END_IF") {
            let end = self.pos + 6;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::EndIf, start, end));
            }
        }
        if remaining.starts_with("REPEAT") {
            let end = self.pos + 6;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::Repeat, start, end));
            }
        }
        if remaining.starts_with("RETURN") {
            let end = self.pos + 6;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::Return, start, end));
            }
        }
        if remaining.starts_with("STRUCT") {
            let end = self.pos + 6;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::Struct, start, end));
            }
        }
        if remaining.starts_with("WHILE") {
            let end = self.pos + 5;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::While, start, end));
            }
        }
        if remaining.starts_with("FALSE") {
            let end = self.pos + 5;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::False, start, end));
            }
        }
        if remaining.starts_with("ELSIF") {
            let end = self.pos + 5;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::Elsif, start, end));
            }
        }
        if remaining.starts_with("ARRAY") {
            let end = self.pos + 5;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::Array, start, end));
            }
        }
        if remaining.starts_with("BEGIN") {
            let end = self.pos + 5;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::Begin, start, end));
            }
        }
        if remaining.starts_with("UNTIL") {
            let end = self.pos + 5;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::Until, start, end));
            }
        }
        if remaining.starts_with("TYPE") {
            let end = self.pos + 4;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::Type, start, end));
            }
        }
        if remaining.starts_with("THEN") {
            let end = self.pos + 4;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::Then, start, end));
            }
        }
        if remaining.starts_with("ELSE") {
            let end = self.pos + 4;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::Else, start, end));
            }
        }
        if remaining.starts_with("CASE") {
            let end = self.pos + 4;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::Case, start, end));
            }
        }
        if remaining.starts_with("TRUE") {
            let end = self.pos + 4;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::True, start, end));
            }
        }
        if remaining.starts_with("XOR") {
            let end = self.pos + 3;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::Xor, start, end));
            }
        }
        if remaining.starts_with("NOT") {
            let end = self.pos + 3;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::Not, start, end));
            }
        }
        if remaining.starts_with("MOD") {
            let end = self.pos + 3;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::Mod, start, end));
            }
        }
        if remaining.starts_with("FOR") {
            let end = self.pos + 3;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::For, start, end));
            }
        }
        if remaining.starts_with("AND") {
            let end = self.pos + 3;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::And, start, end));
            }
        }
        if remaining.starts_with("VAR") {
            let end = self.pos + 3;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::Var, start, end));
            }
        }
        if remaining.starts_with("DO") {
            let end = self.pos + 2;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::Do, start, end));
            }
        }
        if remaining.starts_with("OF") {
            let end = self.pos + 2;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::Of, start, end));
            }
        }
        if remaining.starts_with("TO") {
            let end = self.pos + 2;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::To, start, end));
            }
        }
        if remaining.starts_with("BY") {
            let end = self.pos + 2;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::By, start, end));
            }
        }
        if remaining.starts_with("OR") {
            let end = self.pos + 2;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::Or, start, end));
            }
        }
        if remaining.starts_with("IF") {
            let end = self.pos + 2;
            if end >= self.input.len() || !self.input[end].is_alphanumeric() {
                self.pos = end;
                return Some(Token::new(TokenKind::If, start, end));
            }
        }
        None
    }
    
    fn skip_whitespace_and_comments(&mut self) {
        while self.pos < self.input.len() {
            let ch = self.input[self.pos];
            if ch.is_whitespace() {
                self.pos += 1;
            } else if ch == '/' && self.pos + 1 < self.input.len() && self.input[self.pos + 1] == '/' {
                // Line comment
                self.pos += 2;
                while self.pos < self.input.len() && self.input[self.pos] != '\n' {
                    self.pos += 1;
                }
            } else if ch == '(' && self.pos + 1 < self.input.len() && self.input[self.pos + 1] == '*' {
                // Block comment
                self.pos += 2;
                while self.pos + 1 < self.input.len() {
                    if self.input[self.pos] == '*' && self.input[self.pos + 1] == ')' {
                        self.pos += 2;
                        break;
                    }
                    self.pos += 1;
                }
            } else {
                break;
            }
        }
    }
    
    fn read_identifier(&mut self, start: usize) -> Token {
        while self.pos < self.input.len() {
            let ch = self.input[self.pos];
            if ch.is_alphanumeric() || ch == '_' {
                self.pos += 1;
            } else {
                break;
            }
        }
        let text: String = self.input[start..self.pos].iter().collect();
        Token::new(TokenKind::Identifier(text), start, self.pos)
    }
    
    fn read_number(&mut self, start: usize) -> Token {
        let mut has_dot = false;
        while self.pos < self.input.len() {
            let ch = self.input[self.pos];
            if ch.is_numeric() {
                self.pos += 1;
            } else if ch == '.' && !has_dot && self.pos + 1 < self.input.len() && self.input[self.pos + 1].is_numeric() {
                has_dot = true;
                self.pos += 1;
            } else {
                break;
            }
        }
        let text: String = self.input[start..self.pos].iter().collect();
        if has_dot {
            Token::new(TokenKind::Real(text), start, self.pos)
        } else {
            Token::new(TokenKind::Integer(text), start, self.pos)
        }
    }
    
    fn read_string(&mut self, start: usize) -> Token {
        self.pos += 1; // Skip opening '
        while self.pos < self.input.len() {
            let ch = self.input[self.pos];
            self.pos += 1;
            if ch == '\'' {
                break;
            }
        }
        let text: String = self.input[start..self.pos].iter().collect();
        Token::new(TokenKind::StringLit(text), start, self.pos)
    }
    
    fn read_operator(&mut self, start: usize) -> Token {
        // Try two-character operators first
        if self.pos + 1 < self.input.len() {
            let two_char: String = self.input[self.pos..self.pos+2].iter().collect();
            match two_char.as_str() {
                ":=" | "<>" | "<=" | ">=" | ".." => {
                    self.pos += 2;
                    return Token::new(TokenKind::Operator(two_char), start, self.pos);
                }
                _ => {}
            }
        }
        
        // Single character operators
        let ch = self.input[self.pos];
        self.pos += 1;
        Token::new(TokenKind::Operator(ch.to_string()), start, self.pos)
    }
}
