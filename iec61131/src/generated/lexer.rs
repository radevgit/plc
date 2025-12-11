//! Generated lexer for IEC 61131-3

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Bool,
    Byte,
    Word,
    Dword,
    Lword,
    Sint,
    Int,
    Dint,
    Lint,
    Usint,
    Uint,
    Udint,
    Ulint,
    Real,
    Lreal,
    String,
    Wstring,
    Char,
    Wchar,
    Time,
    Ltime,
    Date,
    Ldate,
    TimeOfDay,
    Tod,
    Ltod,
    LtimeOfDay,
    DateAndTime,
    Dt,
    Ldt,
    LdateAndTime,
    Function,
    EndFunction,
    FunctionBlock,
    EndFunctionBlock,
    Program,
    EndProgram,
    Method,
    EndMethod,
    Class,
    EndClass,
    Interface,
    EndInterface,
    Namespace,
    EndNamespace,
    Configuration,
    EndConfiguration,
    Resource,
    EndResource,
    Var,
    EndVar,
    VarInput,
    VarOutput,
    VarInOut,
    VarTemp,
    VarExternal,
    VarGlobal,
    VarAccess,
    VarConfig,
    Retain,
    NonRetain,
    Constant,
    At,
    Public,
    Protected,
    Private,
    Internal,
    Type,
    EndType,
    Struct,
    EndStruct,
    Array,
    Of,
    RefTo,
    Ref,
    Extends,
    Implements,
    Abstract,
    Final,
    Override,
    This,
    Super,
    If,
    Then,
    Elsif,
    Else,
    EndIf,
    Case,
    EndCase,
    For,
    To,
    By,
    Do,
    EndFor,
    While,
    EndWhile,
    Repeat,
    Until,
    EndRepeat,
    Exit,
    Continue,
    Return,
    Step,
    EndStep,
    InitialStep,
    Transition,
    EndTransition,
    From,
    Action,
    EndAction,
    Priority,
    Not,
    And,
    Or,
    Xor,
    Mod,
    Div,
    Task,
    With,
    Single,
    Interval,
    On,
    ReadWrite,
    ReadOnly,
    True,
    False,
    Null,
    Using,
    Overlap,

    // Literals
    Identifier(String),
    IntLiteral(String),
    RealLiteral(String),
    StringLiteral(String),
    TimeLiteral(String),
    DirectVariable(String),
    
    // Operators
    Plus,             // +
    Minus,            // -
    Star,             // *
    Slash,            // /
    Power,            // **
    Ampersand,        // &
    Equal,            // =
    NotEqual,         // <>
    Less,             // <
    LessEqual,        // <=
    Greater,          // >
    GreaterEqual,     // >=
    Assign,           // :=
    Arrow,            // =>
    DotDot,           // ..
    Percent,          // %
    Hash,             // #
    Caret,            // ^
    
    // Delimiters
    LParen,           // (
    RParen,           // )
    LBracket,         // [
    RBracket,         // ]
    Semicolon,        // ;
    Colon,            // :
    Comma,            // ,
    Dot,              // .
    
    // Special
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Identifier(s) => write!(f, "Identifier({})", s),
            Token::IntLiteral(s) => write!(f, "Int({})", s),
            Token::RealLiteral(s) => write!(f, "Real({})", s),
            Token::StringLiteral(s) => write!(f, "String(\"{}\")", s),
            _ => write!(f, "{:?}", self),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, position: 0 }
    }
    
    pub fn next_token(&mut self) -> SpannedToken {
        self.skip_whitespace_and_comments();
        
        let start = self.position;
        
        if self.position >= self.input.len() {
            return SpannedToken {
                token: Token::Eof,
                span: Span::new(start, start),
            };
        }
        
        let ch = self.current_char();
        
        // Identifiers and keywords
        if ch.is_alphabetic() || ch == '_' {
            return self.scan_identifier();
        }
        
        // Numbers
        if ch.is_numeric() {
            return self.scan_number();
        }
        
        // String literals
        if ch == '\'' {
            return self.scan_string_literal('\'');
        }
        if ch == '"' {
            return self.scan_string_literal('"');
        }
        
        // Direct variables %IX0.0
        if ch == '%' {
            return self.scan_direct_variable();
        }
        
        // Operators and delimiters
        let token = match ch {
            '+' => { self.advance(); Token::Plus }
            '-' => { self.advance(); Token::Minus }
            '/' => { self.advance(); Token::Slash }
            '&' => { self.advance(); Token::Ampersand }
            '%' => { self.advance(); Token::Percent }
            '#' => { self.advance(); Token::Hash }
            '^' => { self.advance(); Token::Caret }
            '(' => { self.advance(); Token::LParen }
            ')' => { self.advance(); Token::RParen }
            '[' => { self.advance(); Token::LBracket }
            ']' => { self.advance(); Token::RBracket }
            ';' => { self.advance(); Token::Semicolon }
            ',' => { self.advance(); Token::Comma }
            '*' => {
                self.advance();
                if self.current_char() == '*' {
                    self.advance();
                    Token::Power
                } else {
                    Token::Star
                }
            }
            ':' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    Token::Assign
                } else {
                    Token::Colon
                }
            }
            '=' => {
                self.advance();
                if self.current_char() == '>' {
                    self.advance();
                    Token::Arrow
                } else {
                    Token::Equal
                }
            }
            '<' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    Token::LessEqual
                } else if self.current_char() == '>' {
                    self.advance();
                    Token::NotEqual
                } else {
                    Token::Less
                }
            }
            '>' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    Token::GreaterEqual
                } else {
                    Token::Greater
                }
            }
            '.' => {
                self.advance();
                if self.position < self.input.len() && self.current_char() == '.' {
                    self.advance();
                    Token::DotDot
                } else {
                    Token::Dot
                }
            }
            _ => {
                self.advance();
                Token::Eof // Unknown character
            }
        };
        
        SpannedToken {
            token,
            span: Span::new(start, self.position),
        }
    }
    
    fn scan_identifier(&mut self) -> SpannedToken {
        let start = self.position;
        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch.is_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }
        
        let text = &self.input[start..self.position];
        let token = Self::keyword_or_identifier(text);
        
        SpannedToken {
            token,
            span: Span::new(start, self.position),
        }
    }
    
    fn keyword_or_identifier(text: &str) -> Token {
        match text.to_uppercase().as_str() {
            "BOOL" => Token::Bool,
            "BYTE" => Token::Byte,
            "WORD" => Token::Word,
            "DWORD" => Token::Dword,
            "LWORD" => Token::Lword,
            "SINT" => Token::Sint,
            "INT" => Token::Int,
            "DINT" => Token::Dint,
            "LINT" => Token::Lint,
            "USINT" => Token::Usint,
            "UINT" => Token::Uint,
            "UDINT" => Token::Udint,
            "ULINT" => Token::Ulint,
            "REAL" => Token::Real,
            "LREAL" => Token::Lreal,
            "STRING" => Token::String,
            "WSTRING" => Token::Wstring,
            "CHAR" => Token::Char,
            "WCHAR" => Token::Wchar,
            "TIME" => Token::Time,
            "LTIME" => Token::Ltime,
            "DATE" => Token::Date,
            "LDATE" => Token::Ldate,
            "TIME_OF_DAY" => Token::TimeOfDay,
            "TOD" => Token::Tod,
            "LTOD" => Token::Ltod,
            "LTIME_OF_DAY" => Token::LtimeOfDay,
            "DATE_AND_TIME" => Token::DateAndTime,
            "DT" => Token::Dt,
            "LDT" => Token::Ldt,
            "LDATE_AND_TIME" => Token::LdateAndTime,
            "FUNCTION" => Token::Function,
            "END_FUNCTION" => Token::EndFunction,
            "FUNCTION_BLOCK" => Token::FunctionBlock,
            "END_FUNCTION_BLOCK" => Token::EndFunctionBlock,
            "PROGRAM" => Token::Program,
            "END_PROGRAM" => Token::EndProgram,
            "METHOD" => Token::Method,
            "END_METHOD" => Token::EndMethod,
            "CLASS" => Token::Class,
            "END_CLASS" => Token::EndClass,
            "INTERFACE" => Token::Interface,
            "END_INTERFACE" => Token::EndInterface,
            "NAMESPACE" => Token::Namespace,
            "END_NAMESPACE" => Token::EndNamespace,
            "CONFIGURATION" => Token::Configuration,
            "END_CONFIGURATION" => Token::EndConfiguration,
            "RESOURCE" => Token::Resource,
            "END_RESOURCE" => Token::EndResource,
            "VAR" => Token::Var,
            "END_VAR" => Token::EndVar,
            "VAR_INPUT" => Token::VarInput,
            "VAR_OUTPUT" => Token::VarOutput,
            "VAR_IN_OUT" => Token::VarInOut,
            "VAR_TEMP" => Token::VarTemp,
            "VAR_EXTERNAL" => Token::VarExternal,
            "VAR_GLOBAL" => Token::VarGlobal,
            "VAR_ACCESS" => Token::VarAccess,
            "VAR_CONFIG" => Token::VarConfig,
            "RETAIN" => Token::Retain,
            "NON_RETAIN" => Token::NonRetain,
            "CONSTANT" => Token::Constant,
            "AT" => Token::At,
            "PUBLIC" => Token::Public,
            "PROTECTED" => Token::Protected,
            "PRIVATE" => Token::Private,
            "INTERNAL" => Token::Internal,
            "TYPE" => Token::Type,
            "END_TYPE" => Token::EndType,
            "STRUCT" => Token::Struct,
            "END_STRUCT" => Token::EndStruct,
            "ARRAY" => Token::Array,
            "OF" => Token::Of,
            "REF_TO" => Token::RefTo,
            "REF" => Token::Ref,
            "EXTENDS" => Token::Extends,
            "IMPLEMENTS" => Token::Implements,
            "ABSTRACT" => Token::Abstract,
            "FINAL" => Token::Final,
            "OVERRIDE" => Token::Override,
            "THIS" => Token::This,
            "SUPER" => Token::Super,
            "IF" => Token::If,
            "THEN" => Token::Then,
            "ELSIF" => Token::Elsif,
            "ELSE" => Token::Else,
            "END_IF" => Token::EndIf,
            "CASE" => Token::Case,
            "END_CASE" => Token::EndCase,
            "FOR" => Token::For,
            "TO" => Token::To,
            "BY" => Token::By,
            "DO" => Token::Do,
            "END_FOR" => Token::EndFor,
            "WHILE" => Token::While,
            "END_WHILE" => Token::EndWhile,
            "REPEAT" => Token::Repeat,
            "UNTIL" => Token::Until,
            "END_REPEAT" => Token::EndRepeat,
            "EXIT" => Token::Exit,
            "CONTINUE" => Token::Continue,
            "RETURN" => Token::Return,
            "STEP" => Token::Step,
            "END_STEP" => Token::EndStep,
            "INITIAL_STEP" => Token::InitialStep,
            "TRANSITION" => Token::Transition,
            "END_TRANSITION" => Token::EndTransition,
            "FROM" => Token::From,
            "ACTION" => Token::Action,
            "END_ACTION" => Token::EndAction,
            "PRIORITY" => Token::Priority,
            "NOT" => Token::Not,
            "AND" => Token::And,
            "OR" => Token::Or,
            "XOR" => Token::Xor,
            "MOD" => Token::Mod,
            "DIV" => Token::Div,
            "TASK" => Token::Task,
            "WITH" => Token::With,
            "SINGLE" => Token::Single,
            "INTERVAL" => Token::Interval,
            "ON" => Token::On,
            "READ_WRITE" => Token::ReadWrite,
            "READ_ONLY" => Token::ReadOnly,
            "TRUE" => Token::True,
            "FALSE" => Token::False,
            "NULL" => Token::Null,
            "USING" => Token::Using,
            "OVERLAP" => Token::Overlap,
            _ => Token::Identifier(text.to_string()),
        }
    }
    
    fn scan_number(&mut self) -> SpannedToken {
        let start = self.position;
        
        // Integer part
        while self.position < self.input.len() && self.current_char().is_numeric() {
            self.advance();
        }
        
        // Check for real literal (has decimal point)
        // But don't consume '..' (range operator)
        if self.position < self.input.len() && self.current_char() == '.' {
            // Look ahead to see if it's '..' (DotDot)
            let next_pos = self.position + 1;
            if next_pos < self.input.len() && self.input.as_bytes()[next_pos] as char == '.' {
                // It's a range operator, don't consume the dot
                return SpannedToken {
                    token: Token::IntLiteral(self.input[start..self.position].to_string()),
                    span: Span::new(start, self.position),
                };
            }
            
            self.advance();
            while self.position < self.input.len() && self.current_char().is_numeric() {
                self.advance();
            }
            
            // Exponent
            if self.position < self.input.len() && (self.current_char() == 'E' || self.current_char() == 'e') {
                self.advance();
                if self.position < self.input.len() && (self.current_char() == '+' || self.current_char() == '-') {
                    self.advance();
                }
                while self.position < self.input.len() && self.current_char().is_numeric() {
                    self.advance();
                }
            }
            
            SpannedToken {
                token: Token::RealLiteral(self.input[start..self.position].to_string()),
                span: Span::new(start, self.position),
            }
        } else {
            SpannedToken {
                token: Token::IntLiteral(self.input[start..self.position].to_string()),
                span: Span::new(start, self.position),
            }
        }
    }
    
    fn scan_string_literal(&mut self, quote: char) -> SpannedToken {
        let start = self.position;
        self.advance(); // Skip opening quote
        
        let mut value = String::new();
        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch == quote {
                self.advance(); // Skip closing quote
                break;
            }
            value.push(ch);
            self.advance();
        }
        
        SpannedToken {
            token: Token::StringLiteral(value),
            span: Span::new(start, self.position),
        }
    }
    
    fn scan_direct_variable(&mut self) -> SpannedToken {
        let start = self.position;
        self.advance(); // Skip %
        
        // Read %IX0.0 format
        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch.is_alphanumeric() || ch == '.' || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }
        
        SpannedToken {
            token: Token::DirectVariable(self.input[start..self.position].to_string()),
            span: Span::new(start, self.position),
        }
    }
    
    fn skip_whitespace_and_comments(&mut self) {
        loop {
            // Skip whitespace
            while self.position < self.input.len() {
                let ch = self.current_char();
                if ch.is_whitespace() {
                    self.advance();
                } else {
                    break;
                }
            }
            
            // Skip comments
            if self.position + 1 < self.input.len() {
                let ch1 = self.current_char();
                let ch2 = self.input.as_bytes()[self.position + 1] as char;
                
                // Line comment //
                if ch1 == '/' && ch2 == '/' {
                    while self.position < self.input.len() && self.current_char() != '\n' {
                        self.advance();
                    }
                    continue;
                }
                
                // Block comment (* ... *)
                if ch1 == '(' && ch2 == '*' {
                    self.advance(); // (
                    self.advance(); // *
                    while self.position + 1 < self.input.len() {
                        if self.current_char() == '*' && self.input.as_bytes()[self.position + 1] as char == ')' {
                            self.advance(); // *
                            self.advance(); // )
                            break;
                        }
                        self.advance();
                    }
                    continue;
                }
                
                // Block comment /* ... */
                if ch1 == '/' && ch2 == '*' {
                    self.advance(); // /
                    self.advance(); // *
                    while self.position + 1 < self.input.len() {
                        if self.current_char() == '*' && self.input.as_bytes()[self.position + 1] as char == '/' {
                            self.advance(); // *
                            self.advance(); // /
                            break;
                        }
                        self.advance();
                    }
                    continue;
                }
            }
            
            break;
        }
    }
    
    fn current_char(&self) -> char {
        if self.position < self.input.len() {
            self.input.as_bytes()[self.position] as char
        } else {
            '\0'
        }
    }
    
    fn advance(&mut self) {
        self.position += 1;
    }
}
