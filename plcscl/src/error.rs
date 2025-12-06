//! Error types for SCL parsing.

use crate::span::Span;
use std::fmt;

/// Result type for SCL operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Error types that can occur during lexing or parsing.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// Unexpected character in input
    UnexpectedChar { ch: char, span: Span },
    
    /// Unexpected token during parsing
    UnexpectedToken { expected: String, found: String, span: Span },
    
    /// Unexpected end of file
    UnexpectedEof { expected: String, span: Span },
    
    /// Invalid number literal
    InvalidNumber { text: String, span: Span },
    
    /// Invalid time literal
    InvalidTime { text: String, span: Span },
    
    /// Unterminated string literal
    UnterminatedString { span: Span },
    
    /// Unterminated comment
    UnterminatedComment { span: Span },
    
    /// Invalid pragma syntax
    InvalidPragma { text: String, span: Span },
    
    /// Invalid identifier (e.g., reserved word used as name)
    InvalidIdentifier { text: String, span: Span },
    
    /// Type mismatch
    TypeMismatch { expected: String, found: String, span: Span },
    
    /// Generic syntax error
    SyntaxError { message: String, span: Span },
}

impl Error {
    /// Get the span where the error occurred.
    pub fn span(&self) -> Span {
        match self {
            Error::UnexpectedChar { span, .. }
            | Error::UnexpectedToken { span, .. }
            | Error::UnexpectedEof { span, .. }
            | Error::InvalidNumber { span, .. }
            | Error::InvalidTime { span, .. }
            | Error::UnterminatedString { span }
            | Error::UnterminatedComment { span }
            | Error::InvalidPragma { span, .. }
            | Error::InvalidIdentifier { span, .. }
            | Error::TypeMismatch { span, .. }
            | Error::SyntaxError { span, .. } => *span,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnexpectedChar { ch, span } => {
                write!(f, "Unexpected character '{}' at {}", ch, span)
            }
            Error::UnexpectedToken { expected, found, span } => {
                write!(f, "Expected {} but found {} at {}", expected, found, span)
            }
            Error::UnexpectedEof { expected, span } => {
                write!(f, "Unexpected end of file, expected {} at {}", expected, span)
            }
            Error::InvalidNumber { text, span } => {
                write!(f, "Invalid number literal '{}' at {}", text, span)
            }
            Error::InvalidTime { text, span } => {
                write!(f, "Invalid time literal '{}' at {}", text, span)
            }
            Error::UnterminatedString { span } => {
                write!(f, "Unterminated string literal at {}", span)
            }
            Error::UnterminatedComment { span } => {
                write!(f, "Unterminated comment at {}", span)
            }
            Error::InvalidPragma { text, span } => {
                write!(f, "Invalid pragma '{}' at {}", text, span)
            }
            Error::InvalidIdentifier { text, span } => {
                write!(f, "Invalid identifier '{}' at {}", text, span)
            }
            Error::TypeMismatch { expected, found, span } => {
                write!(f, "Type mismatch: expected {} but found {} at {}", expected, found, span)
            }
            Error::SyntaxError { message, span } => {
                write!(f, "Syntax error: {} at {}", message, span)
            }
        }
    }
}

impl std::error::Error for Error {}
