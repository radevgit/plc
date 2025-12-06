//! Variable declaration parsing.

use crate::ast::*;
use crate::error::{Error, Result};
use crate::lexer::TokenKind;
use crate::span::Span;

use super::Parser;

impl Parser {
    /// Parse a variable section.
    pub(super) fn parse_var_section(&mut self) -> Result<VarSection> {
        let start = self.current.span.start;

        let kind = match &self.current.kind {
            TokenKind::VarInput => VarSectionKind::Input,
            TokenKind::VarOutput => VarSectionKind::Output,
            TokenKind::VarInOut => VarSectionKind::InOut,
            TokenKind::VarTemp => VarSectionKind::Temp,
            TokenKind::Var => VarSectionKind::Stat,
            TokenKind::VarGlobal => VarSectionKind::Global,
            TokenKind::VarExternal => VarSectionKind::External,
            TokenKind::Constant => VarSectionKind::Constant,
            _ => {
                return Err(Error::UnexpectedToken {
                    expected: "variable section".to_string(),
                    found: format!("{:?}", self.current.kind),
                    span: self.current.span,
                })
            }
        };
        self.advance()?;
        
        // Optional RETAIN or CONSTANT modifier
        if self.consume(&TokenKind::Retain) {
            // Section-level RETAIN
        } else if self.consume(&TokenKind::Constant) {
            // VAR CONSTANT
        }

        let mut variables = Vec::new();
        while !self.check(&TokenKind::EndVar) && !self.is_at_end() {
            // Skip comments and pragmas
            self.skip_trivia();
            
            if self.check(&TokenKind::EndVar) {
                break;
            }

            match self.parse_var_declaration() {
                Ok(var) => variables.push(var),
                Err(e) => {
                    // Record error and try to recover
                    self.errors.push(e.clone());
                    if self.panic_mode {
                        return Err(e);
                    }
                    // Try to skip to next variable or END_VAR
                    self.synchronize_var_declaration();
                }
            }
        }

        self.expect(TokenKind::EndVar)?;
        let end = self.current.span.end;

        Ok(VarSection {
            kind,
            variables,
            span: Span::new(start, end),
        })
    }

    /// Parse a single variable declaration.
    pub(super) fn parse_var_declaration(&mut self) -> Result<VarDeclaration> {
        let start = self.current.span.start;

        // Collect pragmas
        let mut pragmas = Vec::new();
        while let TokenKind::Pragma(ref content) = self.current.kind {
            pragmas.push(Pragma {
                content: content.clone(),
                span: self.current.span,
            });
            self.advance()?;
        }

        // Variable name
        let name = self.parse_identifier()?;

        // Collect more pragmas after name
        while let TokenKind::Pragma(ref content) = self.current.kind {
            pragmas.push(Pragma {
                content: content.clone(),
                span: self.current.span,
            });
            self.advance()?;
        }

        // Check for AT address
        let address = if self.consume(&TokenKind::At) {
            Some(self.parse_address()?)
        } else {
            None
        };

        // Data type
        self.expect(TokenKind::Colon)?;
        let data_type = self.parse_data_type()?;

        // Check for RETAIN
        let retain = self.consume(&TokenKind::Retain);

        // Initial value
        let initial_value = if self.consume(&TokenKind::Assign) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        // Semicolon
        self.expect(TokenKind::Semicolon)?;

        let end = self.current.span.end;

        Ok(VarDeclaration {
            name,
            data_type,
            initial_value,
            retain,
            address,
            pragmas,
            span: Span::new(start, end),
        })
    }

    /// Parse an address (absolute or DB).
    pub(super) fn parse_address(&mut self) -> Result<Address> {
        self.expect(TokenKind::Percent)?;

        // Parse area prefix
        let area_char = self.parse_identifier()?;
        let area = match area_char.to_uppercase().as_str() {
            "I" => MemoryArea::Input,
            "Q" => MemoryArea::Output,
            "M" => MemoryArea::Memory,
            "P" => MemoryArea::Peripheral,
            _ => {
                return Err(Error::SyntaxError {
                    message: format!("Invalid memory area: {}", area_char),
                    span: self.current.span,
                })
            }
        };

        // Parse size and offset
        let size_char = if !self.check(&TokenKind::IntegerLiteral(0)) {
            let s = self.parse_identifier()?;
            Some(match s.to_uppercase().as_str() {
                "X" => AddressSize::Bit,
                "B" => AddressSize::Byte,
                "W" => AddressSize::Word,
                "D" => AddressSize::Dword,
                "L" => AddressSize::Lword,
                _ => {
                    return Err(Error::SyntaxError {
                        message: format!("Invalid address size: {}", s),
                        span: self.current.span,
                    })
                }
            })
        } else {
            None
        };

        // Byte offset
        let byte_offset = if let TokenKind::IntegerLiteral(n) = self.current.kind {
            self.advance()?;
            n as u32
        } else {
            0
        };

        // Bit offset
        let bit_offset = if self.consume(&TokenKind::Dot) {
            if let TokenKind::IntegerLiteral(n) = self.current.kind {
                self.advance()?;
                Some(n as u8)
            } else {
                None
            }
        } else {
            None
        };

        Ok(Address::Absolute {
            area,
            size: size_char,
            byte_offset,
            bit_offset,
        })
    }

    /// Synchronize to next variable declaration after error.
    fn synchronize_var_declaration(&mut self) {
        // Skip to next semicolon or END_VAR
        while !self.is_at_end() {
            if matches!(
                self.current.kind,
                TokenKind::Semicolon | TokenKind::EndVar
            ) {
                if matches!(self.current.kind, TokenKind::Semicolon) {
                    let _ = self.advance();
                }
                return;
            }

            if self.advance().is_err() {
                return;
            }
        }
    }
}
