// Generated SCL lexer

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    At,
    EndRepeat,
    Public,
    EndStruct,
    EndWhile,
    True,
    Do,
    Extends,
    AnyString,
    Program,
    Exit,
    VarInput,
    Retain,
    RefTo,
    Return,
    Struct,
    EndType,
    AnyNum,
    OrganizationBlock,
    VarOutput,
    EndOrganizationBlock,
    VarAccess,
    EndProgram,
    Else,
    By,
    AnyReal,
    EndFunction,
    EndFor,
    For,
    Goto,
    VarTemp,
    Method,
    Variant,
    Repeat,
    Internal,
    Attribute,
    Then,
    EndInterface,
    Implements,
    EndMethod,
    EndVar,
    Type,
    Of,
    AnyDate,
    NonRetain,
    EndDataBlock,
    Constant,
    Array,
    If,
    EndIf,
    Continue,
    Any,
    Xor,
    While,
    Function,
    Private,
    EndClass,
    Protected,
    Elsif,
    Case,
    To,
    Pointer,
    EndFunctionBlock,
    FunctionBlock,
    Begin,
    Class,
    Var,
    Or,
    And,
    Until,
    AnyBit,
    Mod,
    Not,
    False,
    AnyInt,
    VarInOut,
    Interface,
    VarExternal,
    DataBlock,
    EndCase,
    TimeOfDay,
    String,
    Time,
    LReal,
    EndRegion,
    Title,
    Dtl,
    Date,
    LInt,
    UDInt,
    WString,
    Author,
    Family,
    UInt,
    LWord,
    Byte,
    DateTime,
    USInt,
    ULInt,
    Region,
    Version,
    WChar,
    Char,
    Identifier(String),
    IntLit(String),
    FloatLit(String),
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
        let mut lexer = Lexer {
            input: input.chars().collect(),
            pos: 0,
        };
        // Skip UTF-8 BOM if present
        if lexer.input.len() > 0 && lexer.input[0] == '\u{feff}' {
            lexer.pos = 1;
        }
        lexer
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
        
        // Match quoted identifier (TIA Portal style: "name")
        if ch == '"' {
            return self.read_quoted_identifier(start);
        }
        
        // Match # prefix for TIA Portal local variables
        if ch == '#' && self.pos + 1 < self.input.len() {
            let next_ch = self.input[self.pos + 1];
            if next_ch.is_alphabetic() || next_ch == '_' {
                self.pos += 1; // Skip the #
                return self.read_identifier(start);
            }
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
        let remaining_upper = remaining.to_uppercase();
        if remaining_upper.starts_with("END_ORGANIZATION_BLOCK") {
            let end = self.pos + 22;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::EndOrganizationBlock, start, end));
            }
        }
        if remaining_upper.starts_with("ORGANIZATION_BLOCK") {
            let end = self.pos + 18;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::OrganizationBlock, start, end));
            }
        }
        if remaining_upper.starts_with("END_FUNCTION_BLOCK") {
            let end = self.pos + 18;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::EndFunctionBlock, start, end));
            }
        }
        if remaining_upper.starts_with("END_DATA_BLOCK") {
            let end = self.pos + 14;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::EndDataBlock, start, end));
            }
        }
        if remaining_upper.starts_with("FUNCTION_BLOCK") {
            let end = self.pos + 14;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::FunctionBlock, start, end));
            }
        }
        if remaining_upper.starts_with("END_INTERFACE") {
            let end = self.pos + 13;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::EndInterface, start, end));
            }
        }
        if remaining_upper.starts_with("END_FUNCTION") {
            let end = self.pos + 12;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::EndFunction, start, end));
            }
        }
        if remaining_upper.starts_with("VAR_EXTERNAL") {
            let end = self.pos + 12;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::VarExternal, start, end));
            }
        }
        if remaining_upper.starts_with("END_PROGRAM") {
            let end = self.pos + 11;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::EndProgram, start, end));
            }
        }
        if remaining_upper.starts_with("END_REPEAT") {
            let end = self.pos + 10;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::EndRepeat, start, end));
            }
        }
        if remaining_upper.starts_with("END_STRUCT") {
            let end = self.pos + 10;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::EndStruct, start, end));
            }
        }
        if remaining_upper.starts_with("ANY_STRING") {
            let end = self.pos + 10;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::AnyString, start, end));
            }
        }
        if remaining_upper.starts_with("VAR_OUTPUT") {
            let end = self.pos + 10;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::VarOutput, start, end));
            }
        }
        if remaining_upper.starts_with("VAR_ACCESS") {
            let end = self.pos + 10;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::VarAccess, start, end));
            }
        }
        if remaining_upper.starts_with("IMPLEMENTS") {
            let end = self.pos + 10;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::Implements, start, end));
            }
        }
        if remaining_upper.starts_with("END_METHOD") {
            let end = self.pos + 10;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::EndMethod, start, end));
            }
        }
        if remaining_upper.starts_with("NON_RETAIN") {
            let end = self.pos + 10;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::NonRetain, start, end));
            }
        }
        if remaining_upper.starts_with("VAR_IN_OUT") {
            let end = self.pos + 10;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::VarInOut, start, end));
            }
        }
        if remaining_upper.starts_with("DATA_BLOCK") {
            let end = self.pos + 10;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::DataBlock, start, end));
            }
        }
        if remaining_upper.starts_with("END_WHILE") {
            let end = self.pos + 9;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::EndWhile, start, end));
            }
        }
        if remaining_upper.starts_with("VAR_INPUT") {
            let end = self.pos + 9;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::VarInput, start, end));
            }
        }
        if remaining_upper.starts_with("attribute") {
            let end = self.pos + 9;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::Attribute, start, end));
            }
        }
        if remaining_upper.starts_with("END_CLASS") {
            let end = self.pos + 9;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::EndClass, start, end));
            }
        }
        if remaining_upper.starts_with("PROTECTED") {
            let end = self.pos + 9;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::Protected, start, end));
            }
        }
        if remaining_upper.starts_with("INTERFACE") {
            let end = self.pos + 9;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::Interface, start, end));
            }
        }
        if remaining_upper.starts_with("END_TYPE") {
            let end = self.pos + 8;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::EndType, start, end));
            }
        }
        if remaining_upper.starts_with("ANY_REAL") {
            let end = self.pos + 8;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::AnyReal, start, end));
            }
        }
        if remaining_upper.starts_with("VAR_TEMP") {
            let end = self.pos + 8;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::VarTemp, start, end));
            }
        }
        if remaining_upper.starts_with("INTERNAL") {
            let end = self.pos + 8;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::Internal, start, end));
            }
        }
        if remaining_upper.starts_with("ANY_DATE") {
            let end = self.pos + 8;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::AnyDate, start, end));
            }
        }
        if remaining_upper.starts_with("CONSTANT") {
            let end = self.pos + 8;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::Constant, start, end));
            }
        }
        if remaining_upper.starts_with("CONTINUE") {
            let end = self.pos + 8;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::Continue, start, end));
            }
        }
        if remaining_upper.starts_with("FUNCTION") {
            let end = self.pos + 8;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::Function, start, end));
            }
        }
        if remaining_upper.starts_with("END_CASE") {
            let end = self.pos + 8;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::EndCase, start, end));
            }
        }
        if remaining_upper.starts_with("EXTENDS") {
            let end = self.pos + 7;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::Extends, start, end));
            }
        }
        if remaining_upper.starts_with("PROGRAM") {
            let end = self.pos + 7;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::Program, start, end));
            }
        }
        if remaining_upper.starts_with("ANY_NUM") {
            let end = self.pos + 7;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::AnyNum, start, end));
            }
        }
        if remaining_upper.starts_with("END_FOR") {
            let end = self.pos + 7;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::EndFor, start, end));
            }
        }
        if remaining_upper.starts_with("VARIANT") {
            let end = self.pos + 7;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::Variant, start, end));
            }
        }
        if remaining_upper.starts_with("END_VAR") {
            let end = self.pos + 7;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::EndVar, start, end));
            }
        }
        if remaining_upper.starts_with("PRIVATE") {
            let end = self.pos + 7;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::Private, start, end));
            }
        }
        if remaining_upper.starts_with("POINTER") {
            let end = self.pos + 7;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::Pointer, start, end));
            }
        }
        if remaining_upper.starts_with("ANY_BIT") {
            let end = self.pos + 7;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::AnyBit, start, end));
            }
        }
        if remaining_upper.starts_with("ANY_INT") {
            let end = self.pos + 7;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::AnyInt, start, end));
            }
        }
        if remaining_upper.starts_with("PUBLIC") {
            let end = self.pos + 6;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::Public, start, end));
            }
        }
        if remaining_upper.starts_with("RETAIN") {
            let end = self.pos + 6;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::Retain, start, end));
            }
        }
        if remaining_upper.starts_with("REF_TO") {
            let end = self.pos + 6;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::RefTo, start, end));
            }
        }
        if remaining_upper.starts_with("RETURN") {
            let end = self.pos + 6;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::Return, start, end));
            }
        }
        if remaining_upper.starts_with("STRUCT") {
            let end = self.pos + 6;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::Struct, start, end));
            }
        }
        if remaining_upper.starts_with("METHOD") {
            let end = self.pos + 6;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::Method, start, end));
            }
        }
        if remaining_upper.starts_with("REPEAT") {
            let end = self.pos + 6;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::Repeat, start, end));
            }
        }
        if remaining_upper.starts_with("END_IF") {
            let end = self.pos + 6;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::EndIf, start, end));
            }
        }
        if remaining_upper.starts_with("ARRAY") {
            let end = self.pos + 5;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::Array, start, end));
            }
        }
        if remaining_upper.starts_with("WHILE") {
            let end = self.pos + 5;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::While, start, end));
            }
        }
        if remaining_upper.starts_with("ELSIF") {
            let end = self.pos + 5;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::Elsif, start, end));
            }
        }
        if remaining_upper.starts_with("BEGIN") {
            let end = self.pos + 5;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::Begin, start, end));
            }
        }
        if remaining_upper.starts_with("CLASS") {
            let end = self.pos + 5;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::Class, start, end));
            }
        }
        if remaining_upper.starts_with("UNTIL") {
            let end = self.pos + 5;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::Until, start, end));
            }
        }
        if remaining_upper.starts_with("FALSE") {
            let end = self.pos + 5;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::False, start, end));
            }
        }
        if remaining_upper.starts_with("TRUE") {
            let end = self.pos + 4;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::True, start, end));
            }
        }
        if remaining_upper.starts_with("EXIT") {
            let end = self.pos + 4;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::Exit, start, end));
            }
        }
        if remaining_upper.starts_with("ELSE") {
            let end = self.pos + 4;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::Else, start, end));
            }
        }
        if remaining_upper.starts_with("GOTO") {
            let end = self.pos + 4;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::Goto, start, end));
            }
        }
        if remaining_upper.starts_with("THEN") {
            let end = self.pos + 4;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::Then, start, end));
            }
        }
        if remaining_upper.starts_with("TYPE") {
            let end = self.pos + 4;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::Type, start, end));
            }
        }
        if remaining_upper.starts_with("CASE") {
            let end = self.pos + 4;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::Case, start, end));
            }
        }
        if remaining_upper.starts_with("FOR") {
            let end = self.pos + 3;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::For, start, end));
            }
        }
        if remaining_upper.starts_with("ANY") {
            let end = self.pos + 3;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::Any, start, end));
            }
        }
        if remaining_upper.starts_with("XOR") {
            let end = self.pos + 3;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::Xor, start, end));
            }
        }
        if remaining_upper.starts_with("VAR") {
            let end = self.pos + 3;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::Var, start, end));
            }
        }
        if remaining_upper.starts_with("AND") {
            let end = self.pos + 3;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::And, start, end));
            }
        }
        if remaining_upper.starts_with("MOD") {
            let end = self.pos + 3;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::Mod, start, end));
            }
        }
        if remaining_upper.starts_with("NOT") {
            let end = self.pos + 3;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::Not, start, end));
            }
        }
        if remaining_upper.starts_with("AT") {
            let end = self.pos + 2;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::At, start, end));
            }
        }
        if remaining_upper.starts_with("DO") {
            let end = self.pos + 2;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::Do, start, end));
            }
        }
        if remaining_upper.starts_with("BY") {
            let end = self.pos + 2;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::By, start, end));
            }
        }
        if remaining_upper.starts_with("OF") {
            let end = self.pos + 2;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::Of, start, end));
            }
        }
        if remaining_upper.starts_with("IF") {
            let end = self.pos + 2;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::If, start, end));
            }
        }
        if remaining_upper.starts_with("TO") {
            let end = self.pos + 2;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::To, start, end));
            }
        }
        if remaining_upper.starts_with("OR") {
            let end = self.pos + 2;
            if end >= self.input.len() || !(self.input[end].is_alphanumeric() || self.input[end] == '_') {
                self.pos = end;
                return Some(Token::new(TokenKind::Or, start, end));
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
    
    fn read_quoted_identifier(&mut self, start: usize) -> Token {
        self.pos += 1; // Skip opening "
        let name_start = self.pos;
        while self.pos < self.input.len() {
            let ch = self.input[self.pos];
            if ch == '"' {
                break;
            }
            self.pos += 1;
        }
        let text: String = self.input[name_start..self.pos].iter().collect();
        if self.pos < self.input.len() {
            self.pos += 1; // Skip closing "
        }
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
            Token::new(TokenKind::FloatLit(text), start, self.pos)
        } else {
            Token::new(TokenKind::IntLit(text), start, self.pos)
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
        // Try multi-character operators first
        if self.pos + 2 <= self.input.len() {
            let op_chars: String = self.input[self.pos..self.pos+2].iter().collect();
            if op_chars == "<>" {
                self.pos += 2;
                return Token::new(TokenKind::Operator("<>".to_string()), start, self.pos);
            }
        }
        if self.pos + 2 <= self.input.len() {
            let op_chars: String = self.input[self.pos..self.pos+2].iter().collect();
            if op_chars == ".." {
                self.pos += 2;
                return Token::new(TokenKind::Operator("..".to_string()), start, self.pos);
            }
        }
        if self.pos + 2 <= self.input.len() {
            let op_chars: String = self.input[self.pos..self.pos+2].iter().collect();
            if op_chars == ">=" {
                self.pos += 2;
                return Token::new(TokenKind::Operator(">=".to_string()), start, self.pos);
            }
        }
        if self.pos + 2 <= self.input.len() {
            let op_chars: String = self.input[self.pos..self.pos+2].iter().collect();
            if op_chars == ":=" {
                self.pos += 2;
                return Token::new(TokenKind::Operator(":=".to_string()), start, self.pos);
            }
        }
        if self.pos + 2 <= self.input.len() {
            let op_chars: String = self.input[self.pos..self.pos+2].iter().collect();
            if op_chars == "<=" {
                self.pos += 2;
                return Token::new(TokenKind::Operator("<=".to_string()), start, self.pos);
            }
        }
        if self.pos + 2 <= self.input.len() {
            let op_chars: String = self.input[self.pos..self.pos+2].iter().collect();
            if op_chars == "**" {
                self.pos += 2;
                return Token::new(TokenKind::Operator("**".to_string()), start, self.pos);
            }
        }
        // Single character operators
        let ch = self.input[self.pos];
        self.pos += 1;
        Token::new(TokenKind::Operator(ch.to_string()), start, self.pos)
    }
}
