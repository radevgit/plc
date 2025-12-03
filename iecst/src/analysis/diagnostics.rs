//! Diagnostic types for analysis errors and warnings.

use crate::Span;
use std::fmt;

/// Severity level of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Informational hint
    Hint,
    /// Warning (code may work but has issues)
    Warning,
    /// Error (code will not work correctly)
    Error,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Hint => write!(f, "hint"),
            Severity::Warning => write!(f, "warning"),
            Severity::Error => write!(f, "error"),
        }
    }
}

/// Kind of diagnostic.
#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticKind {
    // Symbol errors
    /// Undefined identifier
    UndefinedIdentifier { name: String },
    /// Duplicate definition
    DuplicateDefinition { name: String, original: Span },
    /// Unused variable
    UnusedVariable { name: String },
    /// Uninitialized variable
    UninitializedVariable { name: String },
    /// Assignment to constant
    AssignmentToConstant { name: String },
    /// Assignment to input parameter
    AssignmentToInput { name: String },

    // Type errors
    /// Type mismatch
    TypeMismatch { expected: String, found: String },
    /// Incompatible types for operation
    IncompatibleTypes { left: String, right: String, op: String },
    /// Wrong number of arguments
    WrongArgumentCount { expected: usize, found: usize },
    /// Wrong argument type
    WrongArgumentType { param: String, expected: String, found: String },
    /// Cannot apply operator to type
    InvalidOperator { op: String, operand_type: String },
    /// Array index type must be integer
    NonIntegerArrayIndex,
    /// Array dimension mismatch
    ArrayDimensionMismatch { expected: usize, found: usize },

    // Code smells
    /// Empty statement block
    EmptyBlock { block_type: String },
    /// Deeply nested code
    DeepNesting { depth: usize, max_recommended: usize },
    /// Long function
    LongFunction { lines: usize, max_recommended: usize },
    /// Complex condition
    ComplexCondition { complexity: usize, max_recommended: usize },
    /// Magic number (literal that should be constant)
    MagicNumber { value: String },
    /// Redundant condition (always true/false)
    RedundantCondition { always: bool },
    /// Duplicate code pattern
    DuplicateCode { description: String },
    /// Dead code (unreachable)
    DeadCode { reason: String },
    /// Shadowed variable
    ShadowedVariable { name: String, original: Span },
    /// Empty CASE branch
    EmptyCaseBranch,
    /// Missing ELSE in CASE
    MissingCaseElse,
    /// Comparison with assignment (= vs :=)
    PossibleAssignmentInCondition,
}

impl fmt::Display for DiagnosticKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiagnosticKind::UndefinedIdentifier { name } => {
                write!(f, "undefined identifier '{}'", name)
            }
            DiagnosticKind::DuplicateDefinition { name, .. } => {
                write!(f, "duplicate definition of '{}'", name)
            }
            DiagnosticKind::UnusedVariable { name } => {
                write!(f, "unused variable '{}'", name)
            }
            DiagnosticKind::UninitializedVariable { name } => {
                write!(f, "variable '{}' may be uninitialized", name)
            }
            DiagnosticKind::AssignmentToConstant { name } => {
                write!(f, "cannot assign to constant '{}'", name)
            }
            DiagnosticKind::AssignmentToInput { name } => {
                write!(f, "cannot assign to input parameter '{}'", name)
            }
            DiagnosticKind::TypeMismatch { expected, found } => {
                write!(f, "type mismatch: expected '{}', found '{}'", expected, found)
            }
            DiagnosticKind::IncompatibleTypes { left, right, op } => {
                write!(f, "incompatible types '{}' and '{}' for operator '{}'", left, right, op)
            }
            DiagnosticKind::WrongArgumentCount { expected, found } => {
                write!(f, "wrong number of arguments: expected {}, found {}", expected, found)
            }
            DiagnosticKind::WrongArgumentType { param, expected, found } => {
                write!(f, "wrong type for parameter '{}': expected '{}', found '{}'", param, expected, found)
            }
            DiagnosticKind::InvalidOperator { op, operand_type } => {
                write!(f, "operator '{}' cannot be applied to type '{}'", op, operand_type)
            }
            DiagnosticKind::NonIntegerArrayIndex => {
                write!(f, "array index must be an integer type")
            }
            DiagnosticKind::ArrayDimensionMismatch { expected, found } => {
                write!(f, "array dimension mismatch: expected {} indices, found {}", expected, found)
            }
            DiagnosticKind::EmptyBlock { block_type } => {
                write!(f, "empty {} block", block_type)
            }
            DiagnosticKind::DeepNesting { depth, max_recommended } => {
                write!(f, "deeply nested code (depth {}, recommended max {})", depth, max_recommended)
            }
            DiagnosticKind::LongFunction { lines, max_recommended } => {
                write!(f, "function is too long ({} lines, recommended max {})", lines, max_recommended)
            }
            DiagnosticKind::ComplexCondition { complexity, max_recommended } => {
                write!(f, "complex condition (complexity {}, recommended max {})", complexity, max_recommended)
            }
            DiagnosticKind::MagicNumber { value } => {
                write!(f, "magic number '{}' should be a named constant", value)
            }
            DiagnosticKind::RedundantCondition { always } => {
                write!(f, "condition is always {}", if *always { "true" } else { "false" })
            }
            DiagnosticKind::DuplicateCode { description } => {
                write!(f, "duplicate code: {}", description)
            }
            DiagnosticKind::DeadCode { reason } => {
                write!(f, "unreachable code: {}", reason)
            }
            DiagnosticKind::ShadowedVariable { name, .. } => {
                write!(f, "variable '{}' shadows an outer variable", name)
            }
            DiagnosticKind::EmptyCaseBranch => {
                write!(f, "empty CASE branch")
            }
            DiagnosticKind::MissingCaseElse => {
                write!(f, "CASE statement has no ELSE clause")
            }
            DiagnosticKind::PossibleAssignmentInCondition => {
                write!(f, "possible assignment in condition (did you mean '=' for comparison?)")
            }
        }
    }
}

/// A diagnostic message with location and severity.
#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    /// What kind of issue
    pub kind: DiagnosticKind,
    /// Where in the source
    pub span: Span,
    /// How severe
    pub severity: Severity,
}

impl Diagnostic {
    /// Create a new error diagnostic.
    pub fn error(kind: DiagnosticKind, span: Span) -> Self {
        Self {
            kind,
            span,
            severity: Severity::Error,
        }
    }

    /// Create a new warning diagnostic.
    pub fn warning(kind: DiagnosticKind, span: Span) -> Self {
        Self {
            kind,
            span,
            severity: Severity::Warning,
        }
    }

    /// Create a new hint diagnostic.
    pub fn hint(kind: DiagnosticKind, span: Span) -> Self {
        Self {
            kind,
            span,
            severity: Severity::Hint,
        }
    }

    /// Format the diagnostic with source context.
    pub fn format_with_source(&self, source: &str) -> String {
        let mut result = String::new();
        
        // Find line info
        let (line_num, col, line_start, line_end) = self.find_line_info(source);
        let line_text = &source[line_start..line_end];
        
        // Header
        result.push_str(&format!("{}: {}\n", self.severity, self.kind));
        result.push_str(&format!(" --> {}:{}\n", line_num, col));
        
        // Source line
        let gutter = format!("{} | ", line_num);
        result.push_str(&gutter);
        result.push_str(line_text);
        if !line_text.ends_with('\n') {
            result.push('\n');
        }
        
        // Underline
        let padding = " ".repeat(gutter.len() + col.saturating_sub(1));
        let underline_len = (self.span.end - self.span.start).min(line_end - self.span.start);
        let underline = "^".repeat(underline_len.max(1));
        result.push_str(&format!("{}{}\n", padding, underline));
        
        result
    }

    fn find_line_info(&self, source: &str) -> (usize, usize, usize, usize) {
        let mut line_num = 1;
        let mut line_start = 0;
        
        for (i, c) in source.char_indices() {
            if i >= self.span.start {
                break;
            }
            if c == '\n' {
                line_num += 1;
                line_start = i + 1;
            }
        }
        
        let col = self.span.start - line_start + 1;
        let line_end = source[self.span.start..]
            .find('\n')
            .map(|i| self.span.start + i)
            .unwrap_or(source.len());
        
        (line_num, col, line_start, line_end)
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {} at {:?}", self.severity, self.kind, self.span)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_display() {
        let diag = Diagnostic::error(
            DiagnosticKind::UndefinedIdentifier { name: "foo".to_string() },
            Span::new(0, 3),
        );
        let s = format!("{}", diag);
        assert!(s.contains("error"));
        assert!(s.contains("undefined"));
        assert!(s.contains("foo"));
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Hint < Severity::Warning);
        assert!(Severity::Warning < Severity::Error);
    }
}
