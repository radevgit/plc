//! Source span tracking for AST nodes.

use std::fmt;

/// A span in the source text (byte offsets).
#[derive(Clone, Copy, PartialEq, Eq, Default)]
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

    /// Create an empty/dummy span.
    pub fn empty() -> Self {
        Self { start: 0, end: 0 }
    }

    /// Length of the span in bytes.
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Is the span empty?
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    /// Merge two spans into one covering both.
    pub fn merge(&self, other: &Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    /// Extract the text covered by this span from source.
    pub fn text<'a>(&self, source: &'a str) -> &'a str {
        &source[self.start..self.end.min(source.len())]
    }
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
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

/// A value with an associated source span.
#[allow(dead_code)]
#[derive(Clone, PartialEq, Eq)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

#[allow(dead_code)]
impl<T> Spanned<T> {
    /// Create a new spanned value.
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }

    /// Map the inner value while preserving the span.
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Spanned<U> {
        Spanned {
            node: f(self.node),
            span: self.span,
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Spanned<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} @ {:?}", self.node, self.span)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_merge() {
        let a = Span::new(5, 10);
        let b = Span::new(15, 20);
        let merged = a.merge(&b);
        assert_eq!(merged.start, 5);
        assert_eq!(merged.end, 20);
    }

    #[test]
    fn test_span_text() {
        let source = "hello world";
        let span = Span::new(0, 5);
        assert_eq!(span.text(source), "hello");
    }
}
