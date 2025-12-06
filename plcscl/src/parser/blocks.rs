//! Block parsing methods.

use crate::ast::*;
use crate::error::{Error, Result};
use crate::lexer::TokenKind;
use crate::span::Span;

use super::Parser;

impl Parser {
    /// Parse a block (FUNCTION, FUNCTION_BLOCK, etc.).
    pub(super) fn parse_block(&mut self) -> Result<Block> {
        let start = self.current.span.start;
        
        // Collect pragmas before block
        let mut pragmas = Vec::new();
        while let TokenKind::Pragma(ref content) = self.current.kind {
            pragmas.push(Pragma {
                content: content.clone(),
                span: self.current.span,
            });
            self.advance()?;
        }

        // Parse block kind and name
        let kind = match &self.current.kind {
            TokenKind::Function => BlockKind::Function,
            TokenKind::FunctionBlock => BlockKind::FunctionBlock,
            TokenKind::DataBlock => BlockKind::DataBlock,
            TokenKind::Organization => BlockKind::OrganizationBlock,
            TokenKind::Type => BlockKind::Type,
            _ => {
                return Err(Error::UnexpectedToken {
                    expected: "block declaration".to_string(),
                    found: format!("{:?}", self.current.kind),
                    span: self.current.span,
                })
            }
        };
        self.advance()?;

        // Parse block name (can be quoted)
        let name = self.parse_identifier()?;
        
        // Optional block number
        let number = None; // TODO: Parse block number if present

        // Skip block attributes (TITLE, AUTHOR, FAMILY, VERSION, etc.)
        while matches!(
            self.current.kind,
            TokenKind::Identifier(_) | TokenKind::Pragma(_)
        ) {
            // Check if this looks like an attribute (identifier followed by = or :=)
            if let TokenKind::Identifier(ref attr) = self.current.kind {
                let is_attr = matches!(
                    attr.to_uppercase().as_str(),
                    "TITLE" | "AUTHOR" | "FAMILY" | "VERSION" | "KNOW_HOW_PROTECT"
                );
                
                if is_attr {
                    self.advance()?; // attribute name
                    if self.consume(&TokenKind::Colon) || self.consume(&TokenKind::Equal) || self.consume(&TokenKind::Assign) {
                        // Skip attribute value (could be string, identifier, number)
                        self.advance()?;
                    }
                    continue;
                }
            }
            
            // Pragma
            if matches!(self.current.kind, TokenKind::Pragma(_)) {
                if let TokenKind::Pragma(ref content) = self.current.kind {
                    pragmas.push(Pragma {
                        content: content.clone(),
                        span: self.current.span,
                    });
                }
                self.advance()?;
                continue;
            }
            
            break;
        }
        
        // Skip any comments before variable sections
        self.skip_trivia();

        // Parse variable sections
        let mut variables = Vec::new();
        while self.is_var_section_start() {
            variables.push(self.parse_var_section()?);
        }

        // Parse return type for FUNCTION
        let return_type = if kind == BlockKind::Function && self.consume(&TokenKind::Colon) {
            Some(self.parse_data_type()?)
        } else {
            None
        };

        // Parse body
        let mut body = Vec::new();
        if self.consume(&TokenKind::Begin) {
            while !self.check(&self.end_token_for_block(kind)) && !self.is_at_end() {
                if let Some(stmt) = self.parse_statement_recovering() {
                    body.push(stmt);
                }
                self.skip_trivia();
            }
            self.expect(self.end_token_for_block(kind))?;
        }

        let end = self.current.span.end;

        Ok(Block {
            kind,
            name,
            number,
            pragmas,
            variables,
            body,
            return_type,
            span: Span::new(start, end),
        })
    }

    /// Check if current position starts a variable section.
    fn is_var_section_start(&self) -> bool {
        matches!(
            self.current.kind,
            TokenKind::Var
                | TokenKind::VarInput
                | TokenKind::VarOutput
                | TokenKind::VarInOut
                | TokenKind::VarTemp
                | TokenKind::VarGlobal
                | TokenKind::VarExternal
                | TokenKind::Constant
        )
    }

    /// Get the END token for a block kind.
    fn end_token_for_block(&self, kind: BlockKind) -> TokenKind {
        match kind {
            BlockKind::Function => TokenKind::EndFunction,
            BlockKind::FunctionBlock => TokenKind::EndFunctionBlock,
            BlockKind::DataBlock => TokenKind::EndDataBlock,
            BlockKind::OrganizationBlock => TokenKind::EndOrganization,
            BlockKind::Type => TokenKind::EndType,
        }
    }

    /// Parse identifier (quoted or unquoted).
    pub(super) fn parse_identifier(&mut self) -> Result<String> {
        match &self.current.kind {
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance()?;
                Ok(name)
            }
            _ => Err(Error::UnexpectedToken {
                expected: "identifier".to_string(),
                found: format!("{:?}", self.current.kind),
                span: self.current.span,
            }),
        }
    }
}
