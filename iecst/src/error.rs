//! Error types for ST parsing.

use crate::Span;
use std::fmt;

/// Kinds of parse errors that can occur.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseErrorKind {
    /// Unexpected token encountered
    UnexpectedToken { expected: &'static str },
    /// Unexpected end of input
    UnexpectedEof,
    /// Invalid number literal
    InvalidNumber,
    /// Invalid string literal (unclosed, invalid escape)
    InvalidString,
    /// Invalid time literal
    InvalidTimeLiteral,
    /// Unknown keyword or identifier
    UnknownKeyword,
    /// Missing semicolon
    MissingSemicolon,
    /// Missing END_* keyword
    MissingEndKeyword { keyword: &'static str },
    /// Unclosed parenthesis
    UnclosedParen,
    /// Unclosed bracket
    UnclosedBracket,
    /// Invalid operator
    InvalidOperator,
    /// Invalid expression
    InvalidExpression,
    /// Invalid statement
    InvalidStatement,
    /// Invalid declaration
    InvalidDeclaration,
    /// Invalid type specification
    InvalidType,
    /// Duplicate identifier
    DuplicateIdentifier,
    /// Invalid direct address format
    InvalidDirectAddress,
    /// Security limit exceeded (e.g., max depth, max statements)
    Security(String),
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseErrorKind::UnexpectedToken { expected } => {
                write!(f, "unexpected token, expected {}", expected)
            }
            ParseErrorKind::UnexpectedEof => write!(f, "unexpected end of input"),
            ParseErrorKind::InvalidNumber => write!(f, "invalid number literal"),
            ParseErrorKind::InvalidString => write!(f, "invalid string literal"),
            ParseErrorKind::InvalidTimeLiteral => write!(f, "invalid time literal"),
            ParseErrorKind::UnknownKeyword => write!(f, "unknown keyword"),
            ParseErrorKind::MissingSemicolon => write!(f, "missing semicolon"),
            ParseErrorKind::MissingEndKeyword { keyword } => {
                write!(f, "missing {}", keyword)
            }
            ParseErrorKind::UnclosedParen => write!(f, "unclosed parenthesis"),
            ParseErrorKind::UnclosedBracket => write!(f, "unclosed bracket"),
            ParseErrorKind::InvalidOperator => write!(f, "invalid operator"),
            ParseErrorKind::InvalidExpression => write!(f, "invalid expression"),
            ParseErrorKind::InvalidStatement => write!(f, "invalid statement"),
            ParseErrorKind::InvalidDeclaration => write!(f, "invalid declaration"),
            ParseErrorKind::InvalidType => write!(f, "invalid type specification"),
            ParseErrorKind::DuplicateIdentifier => write!(f, "duplicate identifier"),
            ParseErrorKind::InvalidDirectAddress => write!(f, "invalid direct address format"),
            ParseErrorKind::Security(msg) => write!(f, "security limit exceeded: {}", msg),
        }
    }
}

/// Parse error with location information.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    /// The kind of error
    pub kind: ParseErrorKind,
    /// Location in source where error occurred
    pub span: Span,
}

impl ParseError {
    /// Create a new parse error.
    pub fn new(kind: ParseErrorKind, span: Span) -> Self {
        Self { kind, span }
    }

    /// Format the error with source context.
    pub fn format_with_source(&self, source: &str) -> String {
        let mut result = String::new();

        // Find line and column
        let (line_num, col, line_start, line_end) = self.find_line_info(source);
        let line_text = &source[line_start..line_end];

        // Error header
        result.push_str(&format!("error: {}\n", self.kind));
        result.push_str(&format!(" --> {}:{}\n", line_num, col));

        // Source line
        let gutter = format!("{} | ", line_num);
        result.push_str(&gutter);
        result.push_str(line_text);
        if !line_text.ends_with('\n') {
            result.push('\n');
        }

        // Error indicator
        let indicator_padding = " ".repeat(gutter.len() + col.saturating_sub(1));
        let indicator_len = (self.span.end - self.span.start).max(1).min(line_text.len() - col + 1);
        result.push_str(&indicator_padding);
        result.push_str(&"^".repeat(indicator_len));
        result.push('\n');

        result
    }

    /// Find line number, column, and line boundaries for the error position.
    fn find_line_info(&self, source: &str) -> (usize, usize, usize, usize) {
        let pos = self.span.start.min(source.len());
        let mut line_num = 1;
        let mut line_start = 0;

        for (i, c) in source.char_indices() {
            if i >= pos {
                break;
            }
            if c == '\n' {
                line_num += 1;
                line_start = i + 1;
            }
        }

        let line_end = source[line_start..]
            .find('\n')
            .map(|i| line_start + i)
            .unwrap_or(source.len());

        let col = pos - line_start + 1;

        (line_num, col, line_start, line_end)
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {}", self.kind, self.span)
    }
}

impl std::error::Error for ParseError {}

/// Result type for parsing operations.
pub type ParseResult<T> = Result<T, ParseError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_format() {
        let source = "x := 1 + ;\ny := 2;";
        let err = ParseError::new(
            ParseErrorKind::UnexpectedToken { expected: "expression" },
            Span::new(9, 10),
        );
        
        let formatted = err.format_with_source(source);
        assert!(formatted.contains("unexpected token"));
        assert!(formatted.contains("expected expression"));
        assert!(formatted.contains("1:10")); // line 1, col 10
        assert!(formatted.contains("^"));
    }

    #[test]
    fn test_error_kinds() {
        let err = ParseErrorKind::MissingEndKeyword { keyword: "END_IF" };
        assert_eq!(format!("{}", err), "missing END_IF");
        
        let err = ParseErrorKind::UnclosedParen;
        assert_eq!(format!("{}", err), "unclosed parenthesis");
    }
}

