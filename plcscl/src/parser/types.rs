//! Data type parsing.

use crate::ast::*;
use crate::error::{Error, Result};
use crate::lexer::TokenKind;

use super::Parser;

impl Parser {
    /// Parse a data type.
    pub(super) fn parse_data_type(&mut self) -> Result<DataType> {
        let data_type = match &self.current.kind {
            // Elementary types
            TokenKind::Bool => { self.advance()?; DataType::Bool }
            TokenKind::Byte => { self.advance()?; DataType::Byte }
            TokenKind::Word => { self.advance()?; DataType::Word }
            TokenKind::Dword => { self.advance()?; DataType::Dword }
            TokenKind::Lword => { self.advance()?; DataType::Lword }
            TokenKind::Sint => { self.advance()?; DataType::Sint }
            TokenKind::Int => { self.advance()?; DataType::Int }
            TokenKind::Dint => { self.advance()?; DataType::Dint }
            TokenKind::Lint => { self.advance()?; DataType::Lint }
            TokenKind::Usint => { self.advance()?; DataType::Usint }
            TokenKind::Uint => { self.advance()?; DataType::Uint }
            TokenKind::Udint => { self.advance()?; DataType::Udint }
            TokenKind::Ulint => { self.advance()?; DataType::Ulint }
            TokenKind::Real => { self.advance()?; DataType::Real }
            TokenKind::Lreal => { self.advance()?; DataType::Lreal }
            TokenKind::Char => { self.advance()?; DataType::Char }
            TokenKind::Wchar => { self.advance()?; DataType::Wchar }
            
            // String types
            TokenKind::String => {
                self.advance()?;
                let size = if self.consume(&TokenKind::LeftBracket) {
                    let s = self.parse_integer_literal()?;
                    self.expect(TokenKind::RightBracket)?;
                    Some(s as u32)
                } else {
                    None
                };
                DataType::String(size)
            }
            TokenKind::Wstring => {
                self.advance()?;
                let size = if self.consume(&TokenKind::LeftBracket) {
                    let s = self.parse_integer_literal()?;
                    self.expect(TokenKind::RightBracket)?;
                    Some(s as u32)
                } else {
                    None
                };
                DataType::WString(size)
            }
            
            // Time types
            TokenKind::Time => { self.advance()?; DataType::Time }
            TokenKind::LTime => { self.advance()?; DataType::LTime }
            TokenKind::Date => { self.advance()?; DataType::Date }
            TokenKind::TimeOfDay => { self.advance()?; DataType::TimeOfDay }
            TokenKind::DateAndTime => { self.advance()?; DataType::DateAndTime }
            
            // Array
            TokenKind::Array => {
                self.advance()?;
                self.expect(TokenKind::LeftBracket)?;
                let lower = Box::new(self.parse_expression()?);
                self.expect(TokenKind::DotDot)?;
                let upper = Box::new(self.parse_expression()?);
                self.expect(TokenKind::RightBracket)?;
                self.expect(TokenKind::Of)?;
                let element_type = Box::new(self.parse_data_type()?);
                DataType::Array { lower, upper, element_type }
            }
            
            // Struct
            TokenKind::Struct => {
                self.advance()?;
                let mut members = Vec::new();
                while !self.check(&TokenKind::EndStruct) && !self.is_at_end() {
                    self.skip_trivia();
                    if self.check(&TokenKind::EndStruct) {
                        break;
                    }
                    members.push(self.parse_var_declaration()?);
                }
                self.expect(TokenKind::EndStruct)?;
                DataType::Struct { members }
            }
            
            // Pointer
            TokenKind::Pointer => {
                self.advance()?;
                self.expect(TokenKind::To)?;
                let target = Box::new(self.parse_data_type()?);
                DataType::Pointer(target)
            }
            
            // Reference
            TokenKind::Ref | TokenKind::RefTo => {
                self.advance()?;
                if self.consume(&TokenKind::To) {
                    // REF_TO variant
                }
                let target = Box::new(self.parse_data_type()?);
                DataType::Ref(target)
            }
            
            // Special types
            TokenKind::Any => { self.advance()?; DataType::Any }
            TokenKind::Void => { self.advance()?; DataType::Void }
            
            // User-defined type
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance()?;
                DataType::UserDefined(name)
            }
            
            _ => {
                return Err(Error::UnexpectedToken {
                    expected: "data type".to_string(),
                    found: format!("{:?}", self.current.kind),
                    span: self.current.span,
                })
            }
        };

        Ok(data_type)
    }

    /// Parse integer literal.
    fn parse_integer_literal(&mut self) -> Result<i64> {
        match self.current.kind {
            TokenKind::IntegerLiteral(n) => {
                self.advance()?;
                Ok(n)
            }
            _ => Err(Error::UnexpectedToken {
                expected: "integer literal".to_string(),
                found: format!("{:?}", self.current.kind),
                span: self.current.span,
            }),
        }
    }
}
