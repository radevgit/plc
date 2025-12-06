//! Lexer for SCL source code.

use crate::error::{Error, Result};
use crate::span::Span;

/// Token types in SCL.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords - Block declarations
    Function,
    FunctionBlock,
    DataBlock,
    Organization,
    Type,
    Struct,
    
    // Keywords - Variable sections
    Var,
    VarInput,
    VarOutput,
    VarInOut,
    VarTemp,
    VarGlobal,
    VarExternal,
    Constant,
    Retain,
    NonRetain,
    
    // Keywords - Control flow
    If,
    Then,
    Elsif,
    Else,
    Case,
    Of,
    For,
    To,
    By,
    Do,
    While,
    Repeat,
    Until,
    Continue,
    Exit,
    Return,
    Goto,
    Label,
    
    // Keywords - Operators (word forms)
    And,
    Or,
    Xor,
    Not,
    Mod,
    
    // Keywords - Data types
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
    Char,
    Wchar,
    String,
    Wstring,
    Time,
    LTime,
    Date,
    TimeOfDay,
    DateAndTime,
    Array,
    Pointer,
    Any,
    Void,
    
    // Keywords - Other
    Begin,
    End,
    EndVar,
    EndType,
    EndStruct,
    EndFunction,
    EndFunctionBlock,
    EndDataBlock,
    EndOrganization,
    EndCase,
    EndFor,
    EndIf,
    EndWhile,
    EndRepeat,
    EndRegion,
    Region,
    At,
    Ref,
    RefTo,
    
    // Literals
    IntegerLiteral(i64),
    RealLiteral(f64),
    StringLiteral(String),
    BoolLiteral(bool),
    TimeLiteral(String), // Parsed later
    DateLiteral(String), // Parsed later
    BinaryLiteral(String), // 2#1010
    OctalLiteral(String),  // 8#177
    HexLiteral(String),    // 16#FF
    
    // Identifiers
    Identifier(String),
    
    // Operators - Arithmetic
    Plus,           // +
    Minus,          // -
    Star,           // *
    Slash,          // /
    DoubleStar,     // ** (exponentiation)
    
    // Operators - Comparison
    Equal,          // =
    NotEqual,       // <>
    Less,           // <
    LessEqual,      // <=
    Greater,        // >
    GreaterEqual,   // >=
    
    // Operators - Assignment
    Assign,         // :=
    PlusAssign,     // +=
    MinusAssign,    // -=
    StarAssign,     // *=
    SlashAssign,    // /=
    
    // Operators - Bitwise
    BitwiseAnd,     // &
    
    // Delimiters
    LeftParen,      // (
    RightParen,     // )
    LeftBracket,    // [
    RightBracket,   // ]
    LeftBrace,      // {
    RightBrace,     // }
    Semicolon,      // ;
    Colon,          // :
    Comma,          // ,
    Dot,            // .
    DotDot,         // ..
    Hash,           // #
    Percent,        // %
    
    // Special
    Pragma(String), // {S7_...}
    Comment(String),
    
    // End of file
    Eof,
}

/// A token with its kind, span, and source text.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub text: String,
}

/// Lexer for SCL source code.
pub struct Lexer {
    source: String,
    pos: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    /// Create a new lexer for the given source.
    pub fn new(source: impl Into<String>) -> Self {
        let source: String = source.into();
        // Strip BOM if present
        let source = source.strip_prefix('\u{feff}').unwrap_or(&source).to_string();
        
        Self {
            source,
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    /// Get the next token.
    pub fn next_token(&mut self) -> Result<Token> {
        self.skip_whitespace();
        
        if self.is_eof() {
            return Ok(Token {
                kind: TokenKind::Eof,
                span: Span::new(self.pos, self.pos),
                text: String::new(),
            });
        }

        let start = self.pos;
        let ch = self.current_char();

        // Comments
        if ch == '/' && self.peek_char() == Some('/') {
            return self.read_line_comment(start);
        }
        if ch == '(' && self.peek_char() == Some('*') {
            return self.read_block_comment(start);
        }
        if ch == '{' {
            // Could be pragma or just left brace
            if self.is_pragma_start() {
                return self.read_pragma(start);
            }
        }

        // String literals
        if ch == '\'' {
            return self.read_string_literal(start);
        }

        // Numbers
        if ch.is_ascii_digit() {
            return self.read_number(start);
        }

        // Identifiers and keywords
        if ch.is_alphabetic() || ch == '_' || ch == '"' {
            return self.read_identifier_or_keyword(start);
        }

        // Operators and delimiters
        self.read_operator_or_delimiter(start)
    }

    /// Check if we're at end of file.
    fn is_eof(&self) -> bool {
        self.pos >= self.source.len()
    }

    /// Get current character without advancing.
    fn current_char(&self) -> char {
        self.source[self.pos..].chars().next().unwrap_or('\0')
    }

    /// Peek at next character.
    fn peek_char(&self) -> Option<char> {
        self.source[self.pos..].chars().nth(1)
    }

    /// Advance to next character.
    fn advance(&mut self) -> char {
        let ch = self.current_char();
        self.pos += ch.len_utf8();
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        ch
    }

    /// Skip whitespace.
    fn skip_whitespace(&mut self) {
        while !self.is_eof() && self.current_char().is_whitespace() {
            self.advance();
        }
    }

    /// Check if current position starts a pragma.
    fn is_pragma_start(&self) -> bool {
        // Pragma: { identifier := value }
        // Must have { followed by whitespace or identifier starting with letter/S
        if self.current_char() == '{' {
            if let Some(next) = self.peek_char() {
                return next.is_whitespace() || next.is_alphabetic() || next == 'S';
            }
        }
        false
    }

    /// Read a line comment (// ...).
    fn read_line_comment(&mut self, start: usize) -> Result<Token> {
        self.advance(); // /
        self.advance(); // /
        
        let mut comment = String::new();
        while !self.is_eof() && self.current_char() != '\n' {
            comment.push(self.advance());
        }
        
        Ok(Token {
            kind: TokenKind::Comment(comment),
            span: Span::new(start, self.pos),
            text: self.source[start..self.pos].to_string(),
        })
    }

    /// Read a block comment (* ... *).
    fn read_block_comment(&mut self, start: usize) -> Result<Token> {
        self.advance(); // (
        self.advance(); // *
        
        let mut comment = String::new();
        let mut depth = 1;
        
        while !self.is_eof() && depth > 0 {
            let ch = self.current_char();
            if ch == '(' && self.peek_char() == Some('*') {
                depth += 1;
                comment.push(self.advance());
                comment.push(self.advance());
            } else if ch == '*' && self.peek_char() == Some(')') {
                depth -= 1;
                self.advance();
                self.advance();
            } else {
                comment.push(self.advance());
            }
        }
        
        if depth > 0 {
            return Err(Error::UnterminatedComment {
                span: Span::new(start, self.pos),
            });
        }
        
        Ok(Token {
            kind: TokenKind::Comment(comment),
            span: Span::new(start, self.pos),
            text: self.source[start..self.pos].to_string(),
        })
    }

    /// Read a pragma { ... }.
    fn read_pragma(&mut self, start: usize) -> Result<Token> {
        self.advance(); // {
        
        let mut pragma = String::new();
        while !self.is_eof() && self.current_char() != '}' {
            pragma.push(self.advance());
        }
        
        if self.is_eof() {
            return Err(Error::InvalidPragma {
                text: pragma,
                span: Span::new(start, self.pos),
            });
        }
        
        self.advance(); // }
        
        Ok(Token {
            kind: TokenKind::Pragma(pragma.trim().to_string()),
            span: Span::new(start, self.pos),
            text: self.source[start..self.pos].to_string(),
        })
    }

    /// Read a string literal.
    fn read_string_literal(&mut self, start: usize) -> Result<Token> {
        self.advance(); // '
        
        let mut s = String::new();
        while !self.is_eof() && self.current_char() != '\'' {
            let ch = self.advance();
            if ch == '$' && !self.is_eof() {
                // Escape sequence $' $$ $L $N $P $R $T
                let escape = self.advance();
                match escape {
                    '\'' => s.push('\''),
                    '$' => s.push('$'),
                    'L' | 'l' => s.push('\n'),
                    'N' | 'n' => s.push('\n'),
                    'P' | 'p' => s.push('\x0C'), // form feed
                    'R' | 'r' => s.push('\r'),
                    'T' | 't' => s.push('\t'),
                    _ => {
                        s.push('$');
                        s.push(escape);
                    }
                }
            } else {
                s.push(ch);
            }
        }
        
        if self.is_eof() {
            return Err(Error::UnterminatedString {
                span: Span::new(start, self.pos),
            });
        }
        
        self.advance(); // '
        
        Ok(Token {
            kind: TokenKind::StringLiteral(s),
            span: Span::new(start, self.pos),
            text: self.source[start..self.pos].to_string(),
        })
    }

    /// Read a number literal.
    fn read_number(&mut self, start: usize) -> Result<Token> {
        // Check for based literals: 2#, 8#, 16#
        if self.current_char().is_ascii_digit() {
            let mut num_str = String::new();
            while !self.is_eof() && self.current_char().is_ascii_digit() {
                num_str.push(self.advance());
            }
            
            if !self.is_eof() && self.current_char() == '#' {
                self.advance(); // #
                let base = num_str.parse::<u32>().unwrap_or(10);
                let value = self.read_based_number(base)?;
                
                return Ok(Token {
                    kind: match base {
                        2 => TokenKind::BinaryLiteral(value),
                        8 => TokenKind::OctalLiteral(value),
                        16 => TokenKind::HexLiteral(value),
                        _ => TokenKind::IntegerLiteral(i64::from_str_radix(&value, base).unwrap_or(0)),
                    },
                    span: Span::new(start, self.pos),
                    text: self.source[start..self.pos].to_string(),
                });
            }
            
            // Regular integer or real
            if !self.is_eof() && self.current_char() == '.' && self.peek_char() != Some('.') {
                num_str.push('.'); // Add the decimal point to the string
                self.advance(); // .
                while !self.is_eof() && self.current_char().is_ascii_digit() {
                    num_str.push(self.advance());
                }
                
                // Exponent?
                if !self.is_eof() && matches!(self.current_char(), 'e' | 'E') {
                    num_str.push(self.advance());
                    if !self.is_eof() && matches!(self.current_char(), '+' | '-') {
                        num_str.push(self.advance());
                    }
                    while !self.is_eof() && self.current_char().is_ascii_digit() {
                        num_str.push(self.advance());
                    }
                }
                
                let value = num_str.parse::<f64>().map_err(|_| Error::InvalidNumber {
                    text: num_str.clone(),
                    span: Span::new(start, self.pos),
                })?;
                
                return Ok(Token {
                    kind: TokenKind::RealLiteral(value),
                    span: Span::new(start, self.pos),
                    text: self.source[start..self.pos].to_string(),
                });
            }
            
            let value = num_str.parse::<i64>().map_err(|_| Error::InvalidNumber {
                text: num_str.clone(),
                span: Span::new(start, self.pos),
            })?;
            
            return Ok(Token {
                kind: TokenKind::IntegerLiteral(value),
                span: Span::new(start, self.pos),
                text: self.source[start..self.pos].to_string(),
            });
        }
        
        Err(Error::InvalidNumber {
            text: String::new(),
            span: Span::new(start, self.pos),
        })
    }

    /// Read based number value.
    fn read_based_number(&mut self, base: u32) -> Result<String> {
        let mut value = String::new();
        
        while !self.is_eof() {
            let ch = self.current_char();
            let valid = match base {
                2 => ch == '0' || ch == '1' || ch == '_',
                8 => ch.is_ascii_digit() && ch < '8' || ch == '_',
                10 => ch.is_ascii_digit() || ch == '_',
                16 => ch.is_ascii_hexdigit() || ch == '_',
                _ => false,
            };
            
            if valid {
                if ch != '_' {
                    value.push(ch);
                }
                self.advance();
            } else {
                break;
            }
        }
        
        Ok(value)
    }

    /// Read identifier or keyword.
    fn read_identifier_or_keyword(&mut self, start: usize) -> Result<Token> {
        let mut ident = String::new();
        
        // Quoted identifier: "MyBlock"
        if self.current_char() == '"' {
            self.advance(); // "
            while !self.is_eof() && self.current_char() != '"' {
                ident.push(self.advance());
            }
            if !self.is_eof() {
                self.advance(); // "
            }
            
            return Ok(Token {
                kind: TokenKind::Identifier(ident),
                span: Span::new(start, self.pos),
                text: self.source[start..self.pos].to_string(),
            });
        }
        
        // Regular identifier
        while !self.is_eof() {
            let ch = self.current_char();
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(self.advance());
            } else {
                break;
            }
        }
        
        let kind = Self::keyword_or_identifier(&ident);
        
        Ok(Token {
            kind,
            span: Span::new(start, self.pos),
            text: self.source[start..self.pos].to_string(),
        })
    }

    /// Check if identifier is a keyword.
    fn keyword_or_identifier(ident: &str) -> TokenKind {
        match ident.to_uppercase().as_str() {
            // Block declarations
            "FUNCTION" => TokenKind::Function,
            "FUNCTION_BLOCK" => TokenKind::FunctionBlock,
            "DATA_BLOCK" => TokenKind::DataBlock,
            "ORGANIZATION_BLOCK" => TokenKind::Organization,
            "TYPE" => TokenKind::Type,
            "STRUCT" => TokenKind::Struct,
            
            // Variable sections
            "VAR" => TokenKind::Var,
            "VAR_INPUT" => TokenKind::VarInput,
            "VAR_OUTPUT" => TokenKind::VarOutput,
            "VAR_IN_OUT" => TokenKind::VarInOut,
            "VAR_TEMP" => TokenKind::VarTemp,
            "VAR_GLOBAL" => TokenKind::VarGlobal,
            "VAR_EXTERNAL" => TokenKind::VarExternal,
            "CONSTANT" => TokenKind::Constant,
            "RETAIN" => TokenKind::Retain,
            "NON_RETAIN" => TokenKind::NonRetain,
            
            // Control flow
            "IF" => TokenKind::If,
            "THEN" => TokenKind::Then,
            "ELSIF" => TokenKind::Elsif,
            "ELSE" => TokenKind::Else,
            "CASE" => TokenKind::Case,
            "OF" => TokenKind::Of,
            "FOR" => TokenKind::For,
            "TO" => TokenKind::To,
            "BY" => TokenKind::By,
            "DO" => TokenKind::Do,
            "WHILE" => TokenKind::While,
            "REPEAT" => TokenKind::Repeat,
            "UNTIL" => TokenKind::Until,
            "CONTINUE" => TokenKind::Continue,
            "EXIT" => TokenKind::Exit,
            "RETURN" => TokenKind::Return,
            "GOTO" => TokenKind::Goto,
            
            // Operators
            "AND" => TokenKind::And,
            "OR" => TokenKind::Or,
            "XOR" => TokenKind::Xor,
            "NOT" => TokenKind::Not,
            "MOD" => TokenKind::Mod,
            
            // Data types
            "BOOL" => TokenKind::Bool,
            "BYTE" => TokenKind::Byte,
            "WORD" => TokenKind::Word,
            "DWORD" => TokenKind::Dword,
            "LWORD" => TokenKind::Lword,
            "SINT" => TokenKind::Sint,
            "INT" => TokenKind::Int,
            "DINT" => TokenKind::Dint,
            "LINT" => TokenKind::Lint,
            "USINT" => TokenKind::Usint,
            "UINT" => TokenKind::Uint,
            "UDINT" => TokenKind::Udint,
            "ULINT" => TokenKind::Ulint,
            "REAL" => TokenKind::Real,
            "LREAL" => TokenKind::Lreal,
            "CHAR" => TokenKind::Char,
            "WCHAR" => TokenKind::Wchar,
            "STRING" => TokenKind::String,
            "WSTRING" => TokenKind::Wstring,
            "TIME" => TokenKind::Time,
            "LTIME" => TokenKind::LTime,
            "DATE" => TokenKind::Date,
            "TIME_OF_DAY" | "TOD" => TokenKind::TimeOfDay,
            "DATE_AND_TIME" | "DT" => TokenKind::DateAndTime,
            "ARRAY" => TokenKind::Array,
            "POINTER" => TokenKind::Pointer,
            "ANY" => TokenKind::Any,
            "VOID" => TokenKind::Void,
            
            // Other keywords
            "BEGIN" => TokenKind::Begin,
            "END" => TokenKind::End,
            "END_VAR" => TokenKind::EndVar,
            "END_TYPE" => TokenKind::EndType,
            "END_STRUCT" => TokenKind::EndStruct,
            "END_FUNCTION" => TokenKind::EndFunction,
            "END_FUNCTION_BLOCK" => TokenKind::EndFunctionBlock,
            "END_DATA_BLOCK" => TokenKind::EndDataBlock,
            "END_ORGANIZATION_BLOCK" => TokenKind::EndOrganization,
            "END_CASE" => TokenKind::EndCase,
            "END_FOR" => TokenKind::EndFor,
            "END_IF" => TokenKind::EndIf,
            "END_WHILE" => TokenKind::EndWhile,
            "END_REPEAT" => TokenKind::EndRepeat,
            "END_REGION" => TokenKind::EndRegion,
            "REGION" => TokenKind::Region,
            "AT" => TokenKind::At,
            "REF" => TokenKind::Ref,
            "REF_TO" => TokenKind::RefTo,
            
            // Bool literals
            "TRUE" => TokenKind::BoolLiteral(true),
            "FALSE" => TokenKind::BoolLiteral(false),
            
            _ => TokenKind::Identifier(ident.to_string()),
        }
    }

    /// Read operator or delimiter.
    fn read_operator_or_delimiter(&mut self, start: usize) -> Result<Token> {
        let ch = self.advance();
        
        let kind = match ch {
            '+' => {
                if !self.is_eof() && self.current_char() == '=' {
                    self.advance();
                    TokenKind::PlusAssign
                } else {
                    TokenKind::Plus
                }
            }
            '-' => {
                if !self.is_eof() && self.current_char() == '=' {
                    self.advance();
                    TokenKind::MinusAssign
                } else {
                    TokenKind::Minus
                }
            }
            '*' => {
                if !self.is_eof() && self.current_char() == '*' {
                    self.advance();
                    TokenKind::DoubleStar
                } else if !self.is_eof() && self.current_char() == '=' {
                    self.advance();
                    TokenKind::StarAssign
                } else {
                    TokenKind::Star
                }
            }
            '/' => {
                if !self.is_eof() && self.current_char() == '=' {
                    self.advance();
                    TokenKind::SlashAssign
                } else {
                    TokenKind::Slash
                }
            }
            '=' => TokenKind::Equal,
            '<' => {
                if !self.is_eof() && self.current_char() == '=' {
                    self.advance();
                    TokenKind::LessEqual
                } else if !self.is_eof() && self.current_char() == '>' {
                    self.advance();
                    TokenKind::NotEqual
                } else {
                    TokenKind::Less
                }
            }
            '>' => {
                if !self.is_eof() && self.current_char() == '=' {
                    self.advance();
                    TokenKind::GreaterEqual
                } else {
                    TokenKind::Greater
                }
            }
            ':' => {
                if !self.is_eof() && self.current_char() == '=' {
                    self.advance();
                    TokenKind::Assign
                } else {
                    TokenKind::Colon
                }
            }
            '&' => TokenKind::BitwiseAnd,
            '(' => TokenKind::LeftParen,
            ')' => TokenKind::RightParen,
            '[' => TokenKind::LeftBracket,
            ']' => TokenKind::RightBracket,
            '{' => TokenKind::LeftBrace,
            '}' => TokenKind::RightBrace,
            ';' => TokenKind::Semicolon,
            ',' => TokenKind::Comma,
            '.' => {
                if !self.is_eof() && self.current_char() == '.' {
                    self.advance();
                    TokenKind::DotDot
                } else {
                    TokenKind::Dot
                }
            }
            '#' => TokenKind::Hash,
            '%' => TokenKind::Percent,
            
            _ => {
                return Err(Error::UnexpectedChar {
                    ch,
                    span: Span::new(start, self.pos),
                })
            }
        };
        
        Ok(Token {
            kind,
            span: Span::new(start, self.pos),
            text: self.source[start..self.pos].to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords() {
        let mut lexer = Lexer::new("FUNCTION_BLOCK BEGIN END VAR_INPUT");
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::FunctionBlock);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Begin);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::End);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::VarInput);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Eof);
    }

    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new(":= += <= <> ** AND OR");
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Assign);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::PlusAssign);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::LessEqual);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::NotEqual);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::DoubleStar);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::And);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Or);
    }

    #[test]
    fn test_numbers() {
        let mut lexer = Lexer::new("42 3.14 2#1010 16#FF");
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::IntegerLiteral(42));
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::RealLiteral(3.14));
        assert!(matches!(lexer.next_token().unwrap().kind, TokenKind::BinaryLiteral(_)));
        assert!(matches!(lexer.next_token().unwrap().kind, TokenKind::HexLiteral(_)));
    }

    #[test]
    fn test_string_literal() {
        let mut lexer = Lexer::new("'Hello World'");
        match lexer.next_token().unwrap().kind {
            TokenKind::StringLiteral(s) => assert_eq!(s, "Hello World"),
            _ => panic!("Expected string literal"),
        }
    }

    #[test]
    fn test_comments() {
        let mut lexer = Lexer::new("// line comment\n(* block comment *)");
        assert!(matches!(lexer.next_token().unwrap().kind, TokenKind::Comment(_)));
        assert!(matches!(lexer.next_token().unwrap().kind, TokenKind::Comment(_)));
    }
}
