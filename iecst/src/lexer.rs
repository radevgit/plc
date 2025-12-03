//! Lexer for IEC 61131-3 Structured Text.
//!
//! Tokenizes ST source code into a stream of tokens.

use crate::Span;

/// Token types for Structured Text.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    /// Integer literal: 123, 16#FF, 2#1010, 8#77
    IntLiteral(i64),
    /// Real literal: 1.5, 1.5E10, 1.5e-3
    RealLiteral(f64),
    /// String literal: 'hello', "world"
    StringLiteral(String),
    /// Wide string literal: "hello"
    WStringLiteral(String),
    /// Boolean TRUE
    True,
    /// Boolean FALSE
    False,

    // Time literals
    /// Time duration: T#1h2m3s, TIME#500ms
    TimeLiteral(String),
    /// Date: D#2024-01-15
    DateLiteral(String),
    /// Time of day: TOD#14:30:00
    TodLiteral(String),
    /// Date and time: DT#2024-01-15-14:30:00
    DateTimeLiteral(String),

    // Identifiers and keywords
    /// Identifier: myVar, _count, Sensor1
    Ident(String),

    // Keywords - Program Organization Units
    Program,
    EndProgram,
    Function,
    EndFunction,
    FunctionBlock,
    EndFunctionBlock,

    // Keywords - Variable declarations
    Var,
    VarInput,
    VarOutput,
    VarInOut,
    VarTemp,
    VarGlobal,
    VarExternal,
    EndVar,
    Constant,
    Retain,
    NonRetain,
    At,

    // Keywords - Type declarations
    Type,
    EndType,
    Struct,
    EndStruct,
    Array,
    Of,
    String_,  // STRING keyword (avoiding conflict with String type)
    WString,

    // Keywords - Control flow
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
    Return,
    Continue,

    // Operators - Arithmetic
    Plus,       // +
    Minus,      // -
    Star,       // *
    Slash,      // /
    Mod,        // MOD
    Power,      // **

    // Operators - Comparison
    Eq,         // =
    Ne,         // <>
    Lt,         // <
    Le,         // <=
    Gt,         // >
    Ge,         // >=

    // Operators - Logical
    And,        // AND, &
    Or,         // OR
    Xor,        // XOR
    Not,        // NOT

    // Operators - Assignment
    Assign,     // :=
    OutputAssign, // =>

    // Delimiters
    LParen,     // (
    RParen,     // )
    LBracket,   // [
    RBracket,   // ]
    Comma,      // ,
    Semicolon,  // ;
    Colon,      // :
    Dot,        // .
    DotDot,     // ..
    Hash,       // #
    Percent,    // %

    /// Direct address: %IX0.0, %MW100, etc.
    DirectAddress(String),

    // Special
    /// End of file
    Eof,
    /// Unknown/invalid token
    Unknown(char),
}

impl Token {
    /// Check if this token is a keyword.
    #[allow(dead_code)]
    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            Token::Program
                | Token::EndProgram
                | Token::Function
                | Token::EndFunction
                | Token::FunctionBlock
                | Token::EndFunctionBlock
                | Token::Var
                | Token::VarInput
                | Token::VarOutput
                | Token::VarInOut
                | Token::VarTemp
                | Token::VarGlobal
                | Token::VarExternal
                | Token::EndVar
                | Token::Constant
                | Token::Retain
                | Token::NonRetain
                | Token::Type
                | Token::EndType
                | Token::Struct
                | Token::EndStruct
                | Token::Array
                | Token::Of
                | Token::If
                | Token::Then
                | Token::Elsif
                | Token::Else
                | Token::EndIf
                | Token::Case
                | Token::EndCase
                | Token::For
                | Token::To
                | Token::By
                | Token::Do
                | Token::EndFor
                | Token::While
                | Token::EndWhile
                | Token::Repeat
                | Token::Until
                | Token::EndRepeat
                | Token::Exit
                | Token::Return
                | Token::Continue
                | Token::And
                | Token::Or
                | Token::Xor
                | Token::Not
                | Token::Mod
                | Token::True
                | Token::False
        )
    }
}

/// A token with its source span.
#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
}

impl SpannedToken {
    pub fn new(token: Token, span: Span) -> Self {
        Self { token, span }
    }
}

/// Lexer for Structured Text.
pub struct Lexer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given input.
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    /// Get remaining input.
    fn remaining(&self) -> &'a str {
        &self.input[self.pos..]
    }

    /// Peek at the current character.
    fn peek(&self) -> Option<char> {
        self.remaining().chars().next()
    }

    /// Peek at the next character (after current).
    fn peek_next(&self) -> Option<char> {
        self.remaining().chars().nth(1)
    }

    /// Advance by one character.
    fn advance(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    /// Skip whitespace and comments.
    fn skip_whitespace_and_comments(&mut self) {
        loop {
            // Skip whitespace
            while self.peek().map(|c| c.is_whitespace()).unwrap_or(false) {
                self.advance();
            }

            // Check for comments
            let remaining = self.remaining();
            if remaining.starts_with("//") {
                // Line comment
                while self.peek().map(|c| c != '\n').unwrap_or(false) {
                    self.advance();
                }
            } else if remaining.starts_with("(*") {
                // Block comment (* ... *)
                self.advance(); // (
                self.advance(); // *
                let mut depth = 1;
                while depth > 0 && self.peek().is_some() {
                    if self.remaining().starts_with("(*") {
                        depth += 1;
                        self.advance();
                        self.advance();
                    } else if self.remaining().starts_with("*)") {
                        depth -= 1;
                        self.advance();
                        self.advance();
                    } else {
                        self.advance();
                    }
                }
            } else if remaining.starts_with("/*") {
                // C-style block comment /* ... */
                self.advance(); // /
                self.advance(); // *
                while self.peek().is_some() && !self.remaining().starts_with("*/") {
                    self.advance();
                }
                if self.remaining().starts_with("*/") {
                    self.advance();
                    self.advance();
                }
            } else {
                break;
            }
        }
    }

    /// Tokenize an identifier or keyword.
    fn lex_ident(&mut self) -> SpannedToken {
        let start = self.pos;
        while self
            .peek()
            .map(|c| c.is_alphanumeric() || c == '_')
            .unwrap_or(false)
        {
            self.advance();
        }
        let text = &self.input[start..self.pos];
        let span = Span::new(start, self.pos);

        // Check for keywords (case-insensitive)
        let token = match text.to_uppercase().as_str() {
            "PROGRAM" => Token::Program,
            "END_PROGRAM" => Token::EndProgram,
            "FUNCTION" => Token::Function,
            "END_FUNCTION" => Token::EndFunction,
            "FUNCTION_BLOCK" => Token::FunctionBlock,
            "END_FUNCTION_BLOCK" => Token::EndFunctionBlock,
            "VAR" => Token::Var,
            "VAR_INPUT" => Token::VarInput,
            "VAR_OUTPUT" => Token::VarOutput,
            "VAR_IN_OUT" => Token::VarInOut,
            "VAR_TEMP" => Token::VarTemp,
            "VAR_GLOBAL" => Token::VarGlobal,
            "VAR_EXTERNAL" => Token::VarExternal,
            "END_VAR" => Token::EndVar,
            "CONSTANT" => Token::Constant,
            "RETAIN" => Token::Retain,
            "NON_RETAIN" => Token::NonRetain,
            "AT" => Token::At,
            "TYPE" => Token::Type,
            "END_TYPE" => Token::EndType,
            "STRUCT" => Token::Struct,
            "END_STRUCT" => Token::EndStruct,
            "ARRAY" => Token::Array,
            "OF" => Token::Of,
            "STRING" => Token::String_,
            "WSTRING" => Token::WString,
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
            "RETURN" => Token::Return,
            "CONTINUE" => Token::Continue,
            "AND" => Token::And,
            "OR" => Token::Or,
            "XOR" => Token::Xor,
            "NOT" => Token::Not,
            "MOD" => Token::Mod,
            "TRUE" => Token::True,
            "FALSE" => Token::False,
            _ => Token::Ident(text.to_string()),
        };

        SpannedToken::new(token, span)
    }

    /// Tokenize a number (integer or real).
    fn lex_number(&mut self) -> SpannedToken {
        let start = self.pos;

        // Check for base prefix: 2#, 8#, 16#
        let base = if self.remaining().len() >= 2 {
            let prefix = &self.remaining()[..2];
            if prefix.chars().next().unwrap().is_ascii_digit()
                && prefix.chars().nth(1) == Some('#')
            {
                let base_char = prefix.chars().next().unwrap();
                match base_char {
                    '2' => {
                        self.advance();
                        self.advance();
                        Some(2)
                    }
                    '8' => {
                        self.advance();
                        self.advance();
                        Some(8)
                    }
                    _ => None,
                }
            } else if self.remaining().starts_with("16#") {
                self.advance();
                self.advance();
                self.advance();
                Some(16)
            } else {
                None
            }
        } else {
            None
        };

        // Parse digits
        let digit_start = self.pos;
        while self
            .peek()
            .map(|c| c.is_ascii_alphanumeric() || c == '_')
            .unwrap_or(false)
        {
            self.advance();
        }

        // Check for decimal point (real number)
        let is_real = if base.is_none() && self.peek() == Some('.') && self.peek_next() != Some('.') {
            self.advance(); // consume '.'
            while self
                .peek()
                .map(|c| c.is_ascii_digit() || c == '_')
                .unwrap_or(false)
            {
                self.advance();
            }
            true
        } else {
            false
        };

        // Check for exponent
        let has_exponent = if self.peek() == Some('E') || self.peek() == Some('e') {
            self.advance();
            if self.peek() == Some('+') || self.peek() == Some('-') {
                self.advance();
            }
            while self
                .peek()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
            {
                self.advance();
            }
            true
        } else {
            false
        };

        let text = &self.input[start..self.pos];
        let span = Span::new(start, self.pos);

        // Parse the number
        let token = if is_real || has_exponent {
            // Remove underscores and parse as float
            let clean: String = text.chars().filter(|c| *c != '_').collect();
            match clean.parse::<f64>() {
                Ok(v) => Token::RealLiteral(v),
                Err(_) => Token::Unknown(text.chars().next().unwrap_or('?')),
            }
        } else if let Some(base) = base {
            // Parse with base
            let digits = &self.input[digit_start..self.pos];
            let clean: String = digits.chars().filter(|c| *c != '_').collect();
            match i64::from_str_radix(&clean, base) {
                Ok(v) => Token::IntLiteral(v),
                Err(_) => Token::Unknown(text.chars().next().unwrap_or('?')),
            }
        } else {
            // Decimal integer
            let clean: String = text.chars().filter(|c| *c != '_').collect();
            match clean.parse::<i64>() {
                Ok(v) => Token::IntLiteral(v),
                Err(_) => Token::Unknown(text.chars().next().unwrap_or('?')),
            }
        };

        SpannedToken::new(token, span)
    }

    /// Tokenize a string literal.
    fn lex_string(&mut self, quote: char) -> SpannedToken {
        let start = self.pos;
        self.advance(); // consume opening quote

        let mut value = String::new();
        while let Some(c) = self.peek() {
            if c == quote {
                // Check for escaped quote (doubled)
                if self.peek_next() == Some(quote) {
                    value.push(quote);
                    self.advance();
                    self.advance();
                } else {
                    break;
                }
            } else if c == '$' {
                // IEC escape sequence
                self.advance();
                if let Some(esc) = self.peek() {
                    match esc {
                        'L' | 'l' => value.push('\n'),
                        'N' | 'n' => value.push('\n'),
                        'P' | 'p' => value.push('\x0C'), // form feed
                        'R' | 'r' => value.push('\r'),
                        'T' | 't' => value.push('\t'),
                        '$' => value.push('$'),
                        '\'' => value.push('\''),
                        '"' => value.push('"'),
                        _ => {
                            value.push('$');
                            value.push(esc);
                        }
                    }
                    self.advance();
                }
            } else {
                value.push(c);
                self.advance();
            }
        }

        if self.peek() == Some(quote) {
            self.advance(); // consume closing quote
        }

        let span = Span::new(start, self.pos);
        let token = if quote == '"' {
            Token::WStringLiteral(value)
        } else {
            Token::StringLiteral(value)
        };

        SpannedToken::new(token, span)
    }

    /// Tokenize a time/date literal.
    fn lex_time_literal(&mut self) -> SpannedToken {
        let start = self.pos;

        // Read the prefix (T, TIME, D, DATE, TOD, DT, etc.)
        while self
            .peek()
            .map(|c| c.is_alphabetic() || c == '_')
            .unwrap_or(false)
        {
            self.advance();
        }

        // Expect #
        if self.peek() != Some('#') {
            // Not a time literal, back up and treat as identifier
            self.pos = start;
            return self.lex_ident();
        }
        self.advance(); // consume #

        // Read the value
        while self
            .peek()
            .map(|c| c.is_alphanumeric() || c == '_' || c == ':' || c == '.' || c == '-')
            .unwrap_or(false)
        {
            self.advance();
        }

        let text = &self.input[start..self.pos];
        let span = Span::new(start, self.pos);

        // Determine the type based on prefix
        let upper = text.to_uppercase();
        let token = if upper.starts_with("TIME#") || upper.starts_with("T#") {
            Token::TimeLiteral(text.to_string())
        } else if upper.starts_with("DATE#") || upper.starts_with("D#") {
            Token::DateLiteral(text.to_string())
        } else if upper.starts_with("TIME_OF_DAY#") || upper.starts_with("TOD#") {
            Token::TodLiteral(text.to_string())
        } else if upper.starts_with("DATE_AND_TIME#") || upper.starts_with("DT#") {
            Token::DateTimeLiteral(text.to_string())
        } else {
            Token::TimeLiteral(text.to_string())
        };

        SpannedToken::new(token, span)
    }

    /// Get the next token.
    pub fn next_token(&mut self) -> SpannedToken {
        self.skip_whitespace_and_comments();

        let start = self.pos;

        let Some(c) = self.peek() else {
            return SpannedToken::new(Token::Eof, Span::new(start, start));
        };

        // Check for time/date literals first (before identifiers)
        if c.is_alphabetic() || c == '_' {
            let remaining = self.remaining().to_uppercase();
            if c.is_alphabetic() && (remaining.starts_with("TIME#")
                || remaining.starts_with("T#")
                || remaining.starts_with("DATE#")
                || remaining.starts_with("D#")
                || remaining.starts_with("TIME_OF_DAY#")
                || remaining.starts_with("TOD#")
                || remaining.starts_with("DATE_AND_TIME#")
                || remaining.starts_with("DT#"))
            {
                return self.lex_time_literal();
            }
            return self.lex_ident();
        }

        if c.is_ascii_digit() {
            return self.lex_number();
        }

        if c == '\'' || c == '"' {
            return self.lex_string(c);
        }

        // Multi-character operators
        self.advance();
        let span_one = Span::new(start, self.pos);

        match c {
            ':' => {
                if self.peek() == Some('=') {
                    self.advance();
                    SpannedToken::new(Token::Assign, Span::new(start, self.pos))
                } else {
                    SpannedToken::new(Token::Colon, span_one)
                }
            }
            '=' => {
                if self.peek() == Some('>') {
                    self.advance();
                    SpannedToken::new(Token::OutputAssign, Span::new(start, self.pos))
                } else {
                    SpannedToken::new(Token::Eq, span_one)
                }
            }
            '<' => {
                if self.peek() == Some('>') {
                    self.advance();
                    SpannedToken::new(Token::Ne, Span::new(start, self.pos))
                } else if self.peek() == Some('=') {
                    self.advance();
                    SpannedToken::new(Token::Le, Span::new(start, self.pos))
                } else {
                    SpannedToken::new(Token::Lt, span_one)
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    self.advance();
                    SpannedToken::new(Token::Ge, Span::new(start, self.pos))
                } else {
                    SpannedToken::new(Token::Gt, span_one)
                }
            }
            '*' => {
                if self.peek() == Some('*') {
                    self.advance();
                    SpannedToken::new(Token::Power, Span::new(start, self.pos))
                } else {
                    SpannedToken::new(Token::Star, span_one)
                }
            }
            '.' => {
                if self.peek() == Some('.') {
                    self.advance();
                    SpannedToken::new(Token::DotDot, Span::new(start, self.pos))
                } else {
                    SpannedToken::new(Token::Dot, span_one)
                }
            }
            '&' => SpannedToken::new(Token::And, span_one),
            '+' => SpannedToken::new(Token::Plus, span_one),
            '-' => SpannedToken::new(Token::Minus, span_one),
            '/' => SpannedToken::new(Token::Slash, span_one),
            '(' => SpannedToken::new(Token::LParen, span_one),
            ')' => SpannedToken::new(Token::RParen, span_one),
            '[' => SpannedToken::new(Token::LBracket, span_one),
            ']' => SpannedToken::new(Token::RBracket, span_one),
            ',' => SpannedToken::new(Token::Comma, span_one),
            ';' => SpannedToken::new(Token::Semicolon, span_one),
            '#' => SpannedToken::new(Token::Hash, span_one),
            '%' => {
                // Direct address: %IX0.0, %MW100, %QD5, etc.
                // Format: % [I|Q|M] [X|B|W|D|L]? address
                if self.peek().map(|c| matches!(c, 'I' | 'Q' | 'M' | 'i' | 'q' | 'm')).unwrap_or(false) {
                    // Read the rest of the direct address
                    while self.peek().map(|c| c.is_alphanumeric() || c == '.' || c == '_').unwrap_or(false) {
                        self.advance();
                    }
                    let text = &self.input[start..self.pos];
                    SpannedToken::new(Token::DirectAddress(text.to_string()), Span::new(start, self.pos))
                } else {
                    SpannedToken::new(Token::Percent, span_one)
                }
            }
            _ => SpannedToken::new(Token::Unknown(c), span_one),
        }
    }

    /// Tokenize the entire input.
    #[allow(dead_code)]
    pub fn tokenize(&mut self) -> Vec<SpannedToken> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            let is_eof = token.token == Token::Eof;
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenize(input: &str) -> Vec<Token> {
        Lexer::new(input)
            .tokenize()
            .into_iter()
            .map(|t| t.token)
            .collect()
    }

    #[test]
    fn test_simple_tokens() {
        let tokens = tokenize("x := 1 + 2;");
        assert_eq!(
            tokens,
            vec![
                Token::Ident("x".to_string()),
                Token::Assign,
                Token::IntLiteral(1),
                Token::Plus,
                Token::IntLiteral(2),
                Token::Semicolon,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_keywords() {
        let tokens = tokenize("IF x THEN END_IF");
        assert_eq!(
            tokens,
            vec![
                Token::If,
                Token::Ident("x".to_string()),
                Token::Then,
                Token::EndIf,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_numbers() {
        assert_eq!(tokenize("123")[0], Token::IntLiteral(123));
        assert_eq!(tokenize("16#FF")[0], Token::IntLiteral(255));
        assert_eq!(tokenize("2#1010")[0], Token::IntLiteral(10));
        assert_eq!(tokenize("1.5")[0], Token::RealLiteral(1.5));
        assert_eq!(tokenize("1.5E10")[0], Token::RealLiteral(1.5e10));
    }

    #[test]
    fn test_strings() {
        let tokens = tokenize("'hello'");
        assert_eq!(tokens[0], Token::StringLiteral("hello".to_string()));

        let tokens = tokenize("\"world\"");
        assert_eq!(tokens[0], Token::WStringLiteral("world".to_string()));
    }

    #[test]
    fn test_time_literals() {
        let tokens = tokenize("T#1h2m3s");
        assert!(matches!(tokens[0], Token::TimeLiteral(_)));

        let tokens = tokenize("D#2024-01-15");
        assert!(matches!(tokens[0], Token::DateLiteral(_)));
    }

    #[test]
    fn test_operators() {
        let tokens = tokenize(":= <> <= >= ** ..");
        assert_eq!(tokens[0], Token::Assign);
        assert_eq!(tokens[1], Token::Ne);
        assert_eq!(tokens[2], Token::Le);
        assert_eq!(tokens[3], Token::Ge);
        assert_eq!(tokens[4], Token::Power);
        assert_eq!(tokens[5], Token::DotDot);
    }

    #[test]
    fn test_comments() {
        let tokens = tokenize("x // comment\ny");
        assert_eq!(tokens[0], Token::Ident("x".to_string()));
        assert_eq!(tokens[1], Token::Ident("y".to_string()));

        let tokens = tokenize("x (* block *) y");
        assert_eq!(tokens[0], Token::Ident("x".to_string()));
        assert_eq!(tokens[1], Token::Ident("y".to_string()));
    }

    #[test]
    fn test_case_insensitive_keywords() {
        let tokens = tokenize("if IF If");
        assert_eq!(tokens[0], Token::If);
        assert_eq!(tokens[1], Token::If);
        assert_eq!(tokens[2], Token::If);
    }
}
