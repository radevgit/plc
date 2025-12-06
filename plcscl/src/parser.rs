//! Parser for SCL source code.

use crate::ast::*;
use crate::error::{Error, Result};
use crate::lexer::{Lexer, Token, TokenKind};
use crate::span::Span;

mod blocks;
mod expressions;
mod statements;
mod types;
mod variables;

/// Parser for SCL programs with error recovery.
pub struct Parser {
    lexer: Lexer,
    current: Token,
    peeked: Option<Token>,
    /// Errors encountered during parsing (for error recovery)
    pub errors: Vec<Error>,
    /// Whether to panic on first error or collect errors
    panic_mode: bool,
}

impl Parser {
    /// Create a new parser from a lexer.
    pub fn new(mut lexer: Lexer) -> Result<Self> {
        let current = lexer.next_token()?;
        Ok(Self {
            lexer,
            current,
            peeked: None,
            errors: Vec::new(),
            panic_mode: false,
        })
    }

    /// Enable error recovery mode (collect errors instead of failing fast).
    pub fn with_error_recovery(mut self) -> Self {
        self.panic_mode = false;
        self
    }

    /// Parse a complete compilation unit.
    pub fn parse(&mut self) -> Result<CompilationUnit> {
        let start = self.current.span.start;
        let mut blocks = Vec::new();

        // Skip any leading comments or pragmas
        self.skip_trivia();

        while !self.is_at_end() {
            match self.parse_block() {
                Ok(block) => blocks.push(block),
                Err(e) => {
                    self.errors.push(e.clone());
                    // Try to recover by skipping to next block or END_*
                    if !self.synchronize_to_next_block() {
                        break;
                    }
                }
            }
            self.skip_trivia();
        }

        let end = if let Some(last) = blocks.last() {
            last.span.end
        } else {
            self.current.span.end
        };

        let result = CompilationUnit {
            blocks,
            span: Span::new(start, end),
        };

        // If we collected errors, return the first one but keep the partial parse
        if !self.errors.is_empty() && self.panic_mode {
            return Err(self.errors[0].clone());
        }

        Ok(result)
    }

    /// Check if at end of input.
    fn is_at_end(&self) -> bool {
        matches!(self.current.kind, TokenKind::Eof)
    }

    /// Skip comments and standalone pragmas.
    fn skip_trivia(&mut self) {
        while matches!(
            self.current.kind,
            TokenKind::Comment(_) | TokenKind::Pragma(_)
        ) {
            if self.advance().is_err() {
                break;
            }
        }
    }

    /// Synchronize parser to next valid block start after error.
    fn synchronize_to_next_block(&mut self) -> bool {
        // Skip tokens until we find a block keyword or EOF
        while !self.is_at_end() {
            // Skip to next statement boundary
            if matches!(
                self.current.kind,
                TokenKind::Function
                    | TokenKind::FunctionBlock
                    | TokenKind::DataBlock
                    | TokenKind::Organization
                    | TokenKind::Type
            ) {
                return true;
            }

            // Also stop at END_* tokens that might indicate end of malformed block
            if matches!(
                self.current.kind,
                TokenKind::EndFunction
                    | TokenKind::EndFunctionBlock
                    | TokenKind::EndDataBlock
                    | TokenKind::EndOrganization
                    | TokenKind::EndType
            ) {
                if self.advance().is_ok() {
                    return !self.is_at_end();
                }
                return false;
            }

            if self.advance().is_err() {
                return false;
            }
        }
        false
    }

    /// Report error and optionally continue parsing.
    pub(super) fn report_error(&mut self, error: Error) -> Result<()> {
        if self.panic_mode {
            Err(error)
        } else {
            self.errors.push(error);
            Ok(())
        }
    }

    /// Advance to next token.
    fn advance(&mut self) -> Result<Token> {
        let prev = std::mem::replace(
            &mut self.current,
            if let Some(peeked) = self.peeked.take() {
                peeked
            } else {
                self.lexer.next_token()?
            },
        );
        Ok(prev)
    }

    /// Peek at next token without consuming.
    fn peek(&mut self) -> Result<&Token> {
        if self.peeked.is_none() {
            self.peeked = Some(self.lexer.next_token()?);
        }
        Ok(self.peeked.as_ref().unwrap())
    }

    /// Check if current token matches kind.
    fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.current.kind) == std::mem::discriminant(kind)
    }

    /// Consume token if it matches, error otherwise.
    fn expect(&mut self, kind: TokenKind) -> Result<Token> {
        if self.check(&kind) {
            self.advance()
        } else {
            Err(Error::UnexpectedToken {
                expected: format!("{:?}", kind),
                found: format!("{:?}", self.current.kind),
                span: self.current.span,
            })
        }
    }

    /// Consume token if it matches.
    fn consume(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance().ok();
            true
        } else {
            false
        }
    }
}
