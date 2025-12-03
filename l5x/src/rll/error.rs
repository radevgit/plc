//! Error types for RLL parsing.

use std::fmt;

/// A span in the source text (byte offsets).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// Start byte offset (inclusive)
    pub start: usize,
    /// End byte offset (exclusive)
    pub end: usize,
}

impl Span {
    /// Create a new span.
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Create a span at a single position.
    pub fn at(position: usize) -> Self {
        Self { start: position, end: position + 1 }
    }

    /// Length of the span.
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Is the span empty?
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.len() <= 1 {
            write!(f, "{}", self.start)
        } else {
            write!(f, "{}..{}", self.start, self.end)
        }
    }
}

/// Errors that can occur during RLL parsing.
#[derive(Debug, Clone, PartialEq)]
pub enum RllError {
    /// Unexpected character at position
    UnexpectedChar { char: char, position: usize },
    /// Expected a specific token
    Expected { expected: &'static str, position: usize },
    /// Unclosed bracket
    UnclosedBracket { position: usize },
    /// Unclosed parenthesis
    UnclosedParen { position: usize },
    /// Empty input
    EmptyInput,
    /// Missing rung terminator (;)
    MissingTerminator,
    /// Invalid instruction (empty mnemonic)
    InvalidInstruction { position: usize },
    /// Unexpected end of input
    UnexpectedEof,
}

impl RllError {
    /// Get the position where the error occurred, if available.
    pub fn position(&self) -> Option<usize> {
        match self {
            RllError::UnexpectedChar { position, .. } => Some(*position),
            RllError::Expected { position, .. } => Some(*position),
            RllError::UnclosedBracket { position } => Some(*position),
            RllError::UnclosedParen { position } => Some(*position),
            RllError::InvalidInstruction { position } => Some(*position),
            RllError::EmptyInput | RllError::MissingTerminator | RllError::UnexpectedEof => None,
        }
    }

    /// Get the span of the error in the source text.
    pub fn span(&self) -> Option<Span> {
        self.position().map(Span::at)
    }

    /// Format the error with source context.
    ///
    /// Returns a multi-line string showing:
    /// - The error message
    /// - The relevant portion of source text
    /// - A caret (^) pointing to the error location
    pub fn format_with_context(&self, source: &str) -> String {
        let mut result = String::new();
        
        // Error message
        result.push_str(&format!("error: {}\n", self));
        
        // If we have a position, show context
        if let Some(pos) = self.position() {
            if pos < source.len() {
                // Find the line containing the error
                let (line_start, line_num) = find_line_start(source, pos);
                let line_end = source[line_start..].find('\n')
                    .map(|i| line_start + i)
                    .unwrap_or(source.len());
                let line = &source[line_start..line_end];
                
                // Column within the line
                let col = pos - line_start;
                
                // Format: " --> position"
                result.push_str(&format!(" --> position {}:{}\n", line_num, col));
                
                // Show the line (truncate if too long)
                let (display_line, display_col) = if line.len() > 80 {
                    // Show a window around the error
                    let window_start = col.saturating_sub(30);
                    let window_end = (col + 50).min(line.len());
                    let prefix = if window_start > 0 { "..." } else { "" };
                    let suffix = if window_end < line.len() { "..." } else { "" };
                    let window = &line[window_start..window_end];
                    (format!("{}{}{}", prefix, window, suffix), col - window_start + prefix.len())
                } else {
                    (line.to_string(), col)
                };
                
                // Line number gutter
                let gutter = format!("{} | ", line_num);
                result.push_str(&gutter);
                result.push_str(&display_line);
                result.push('\n');
                
                // Caret pointing to error
                let caret_padding: String = " ".repeat(gutter.len() + display_col);
                result.push_str(&caret_padding);
                result.push_str("^ here");
            }
        }
        
        result
    }
}

/// Find the start of the line containing the given position.
/// Returns (byte offset of line start, 1-based line number).
fn find_line_start(source: &str, pos: usize) -> (usize, usize) {
    let mut line_start = 0;
    let mut line_num = 1;
    
    for (i, c) in source.char_indices() {
        if i >= pos {
            break;
        }
        if c == '\n' {
            line_start = i + 1;
            line_num += 1;
        }
    }
    
    (line_start, line_num)
}

impl fmt::Display for RllError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RllError::UnexpectedChar { char, position } => {
                write!(f, "unexpected character '{}' at position {}", char, position)
            }
            RllError::Expected { expected, position } => {
                write!(f, "expected {} at position {}", expected, position)
            }
            RllError::UnclosedBracket { position } => {
                write!(f, "unclosed bracket '[' at position {}", position)
            }
            RllError::UnclosedParen { position } => {
                write!(f, "unclosed parenthesis '(' at position {}", position)
            }
            RllError::EmptyInput => {
                write!(f, "empty input")
            }
            RllError::MissingTerminator => {
                write!(f, "missing rung terminator ';'")
            }
            RllError::InvalidInstruction { position } => {
                write!(f, "invalid instruction at position {}", position)
            }
            RllError::UnexpectedEof => {
                write!(f, "unexpected end of input")
            }
        }
    }
}

impl std::error::Error for RllError {}

/// Result type for RLL parsing operations.
pub type RllResult<T> = Result<T, RllError>;

/// A parse error bundled with its source text for rich error reporting.
#[derive(Debug, Clone)]
pub struct ParseError {
    /// The error that occurred
    pub error: RllError,
    /// The source text that was being parsed
    pub source: String,
    /// Optional file/location context
    pub context: Option<ErrorContext>,
}

/// Additional context for where an error occurred.
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Program name
    pub program: String,
    /// Routine name
    pub routine: String,
    /// Rung number
    pub rung_number: u32,
}

impl ErrorContext {
    /// Create a new error context.
    pub fn new(program: impl Into<String>, routine: impl Into<String>, rung_number: u32) -> Self {
        Self {
            program: program.into(),
            routine: routine.into(),
            rung_number,
        }
    }

    /// Format as a path string.
    pub fn path(&self) -> String {
        format!("{}/{}/Rung#{}", self.program, self.routine, self.rung_number)
    }
}

impl ParseError {
    /// Create a new parse error.
    pub fn new(error: RllError, source: impl Into<String>) -> Self {
        Self {
            error,
            source: source.into(),
            context: None,
        }
    }

    /// Add context to this error.
    pub fn with_context(mut self, context: ErrorContext) -> Self {
        self.context = Some(context);
        self
    }

    /// Format the error with full source context.
    pub fn format(&self) -> String {
        let mut result = String::new();
        
        // Location context if available
        if let Some(ctx) = &self.context {
            result.push_str(&format!("in {}\n", ctx.path()));
        }
        
        // Delegate to RllError's context formatting
        result.push_str(&self.error.format_with_context(&self.source));
        
        result
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format())
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_display() {
        let span = Span::new(10, 15);
        assert_eq!(format!("{}", span), "10..15");
        
        let single = Span::at(42);
        assert_eq!(format!("{}", single), "42");
    }

    #[test]
    fn test_error_position() {
        let err = RllError::UnexpectedChar { char: 'x', position: 5 };
        assert_eq!(err.position(), Some(5));
        
        let err = RllError::EmptyInput;
        assert_eq!(err.position(), None);
    }

    #[test]
    fn test_format_with_context() {
        let err = RllError::UnclosedBracket { position: 4 };
        let source = "XIC([Incomplete;";
        let formatted = err.format_with_context(source);
        
        assert!(formatted.contains("unclosed bracket"));
        assert!(formatted.contains("XIC([Incomplete;"));
        assert!(formatted.contains("^")); // caret indicator
    }

    #[test]
    fn test_format_with_context_multiline() {
        let err = RllError::UnexpectedChar { char: '!', position: 15 };
        let source = "XIC(Tag1)\nOTE(!)MOV(A,B);";
        let formatted = err.format_with_context(source);
        
        assert!(formatted.contains("unexpected character '!'"));
        assert!(formatted.contains("OTE(!)MOV")); // shows the line with the error
        assert!(formatted.contains("2:")); // line 2
    }

    #[test]
    fn test_parse_error_with_context() {
        let err = RllError::MissingTerminator;
        let parse_err = ParseError::new(err, "XIC(Tag)OTE(Out)")
            .with_context(ErrorContext::new("MainProgram", "MainRoutine", 5));
        
        let formatted = parse_err.format();
        assert!(formatted.contains("MainProgram/MainRoutine/Rung#5"));
        assert!(formatted.contains("missing rung terminator"));
    }

    #[test]
    fn test_long_line_truncation() {
        // Create a long line with error near the middle
        let long_line = format!("{}XIC(BadTag!Here){}", "A".repeat(50), "B".repeat(50));
        let err = RllError::UnexpectedChar { char: '!', position: 60 };
        let formatted = err.format_with_context(&long_line);
        
        // Should contain ellipsis for truncation
        assert!(formatted.contains("...") || formatted.len() < long_line.len() + 100);
        // Should still show the error location
        assert!(formatted.contains("^"));
    }
}