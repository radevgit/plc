//! Expression parsing.

use crate::ast::*;
use crate::error::{Error, Result};
use crate::lexer::TokenKind;
use crate::span::Span;

use super::Parser;

impl Parser {
    /// Parse an expression.
    pub(super) fn parse_expression(&mut self) -> Result<Expression> {
        self.parse_binary_expression(0)
    }

    /// Parse binary expression with precedence climbing.
    fn parse_binary_expression(&mut self, min_precedence: u8) -> Result<Expression> {
        let mut left = self.parse_unary_expression()?;

        loop {
            let op = match self.current.kind {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Sub,
                TokenKind::Star => BinaryOp::Mul,
                TokenKind::Slash => BinaryOp::Div,
                TokenKind::Mod => BinaryOp::Mod,
                TokenKind::DoubleStar => BinaryOp::Exp,
                TokenKind::Equal => BinaryOp::Eq,
                TokenKind::NotEqual => BinaryOp::Ne,
                TokenKind::Less => BinaryOp::Lt,
                TokenKind::LessEqual => BinaryOp::Le,
                TokenKind::Greater => BinaryOp::Gt,
                TokenKind::GreaterEqual => BinaryOp::Ge,
                TokenKind::And => BinaryOp::And,
                TokenKind::Or => BinaryOp::Or,
                TokenKind::Xor => BinaryOp::Xor,
                TokenKind::BitwiseAnd => BinaryOp::BitAnd,
                _ => break,
            };

            let precedence = op.precedence();
            if precedence < min_precedence {
                break;
            }

            self.advance()?;
            let right = self.parse_binary_expression(precedence + 1)?;
            
            let span = left.span().merge(right.span());
            left = Expression::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    /// Parse unary expression.
    fn parse_unary_expression(&mut self) -> Result<Expression> {
        let start = self.current.span.start;

        let op = match &self.current.kind {
            TokenKind::Minus => Some(UnaryOp::Neg),
            TokenKind::Plus => Some(UnaryOp::Pos),
            TokenKind::Not => Some(UnaryOp::Not),
            _ => None,
        };

        if let Some(op) = op {
            self.advance()?;
            let operand = Box::new(self.parse_unary_expression()?);
            let span = Span::new(start, operand.span().end);
            
            Ok(Expression::Unary {
                op,
                operand,
                span,
            })
        } else {
            self.parse_postfix_expression()
        }
    }

    /// Parse postfix expression (call, member, index).
    fn parse_postfix_expression(&mut self) -> Result<Expression> {
        let mut expr = self.parse_primary_expression()?;

        loop {
            match &self.current.kind {
                // Function call
                TokenKind::LeftParen => {
                    self.advance()?;
                    let arguments = self.parse_arguments()?;
                    self.expect(TokenKind::RightParen)?;
                    
                    let span = expr.span().merge(self.current.span);
                    expr = Expression::Call {
                        function: Box::new(expr),
                        arguments,
                        span,
                    };
                }
                
                // Member access
                TokenKind::Dot => {
                    self.advance()?;
                    let member = self.parse_identifier()?;
                    let span = expr.span().merge(self.current.span);
                    
                    expr = Expression::Member {
                        object: Box::new(expr),
                        member,
                        span,
                    };
                }
                
                // Array indexing
                TokenKind::LeftBracket => {
                    self.advance()?;
                    let index = Box::new(self.parse_expression()?);
                    self.expect(TokenKind::RightBracket)?;
                    
                    let span = expr.span().merge(self.current.span);
                    expr = Expression::Index {
                        array: Box::new(expr),
                        index,
                        span,
                    };
                }
                
                _ => break,
            }
        }

        Ok(expr)
    }

    /// Parse primary expression.
    fn parse_primary_expression(&mut self) -> Result<Expression> {
        let start = self.current.span.start;

        match &self.current.kind.clone() {
            // Literals
            TokenKind::IntegerLiteral(n) => {
                let n = *n;
                let span = self.current.span;
                self.advance()?;
                Ok(Expression::Literal(Literal {
                    kind: LiteralKind::Integer(n),
                    span,
                }))
            }
            TokenKind::RealLiteral(f) => {
                let f = *f;
                let span = self.current.span;
                self.advance()?;
                Ok(Expression::Literal(Literal {
                    kind: LiteralKind::Real(f),
                    span,
                }))
            }
            TokenKind::StringLiteral(s) => {
                let s = s.clone();
                let span = self.current.span;
                self.advance()?;
                Ok(Expression::Literal(Literal {
                    kind: LiteralKind::String(s),
                    span,
                }))
            }
            TokenKind::BoolLiteral(b) => {
                let b = *b;
                let span = self.current.span;
                self.advance()?;
                Ok(Expression::Literal(Literal {
                    kind: LiteralKind::Bool(b),
                    span,
                }))
            }
            TokenKind::BinaryLiteral(s) => {
                let s = s.clone();
                let span = self.current.span;
                self.advance()?;
                Ok(Expression::Literal(Literal {
                    kind: LiteralKind::Binary(s),
                    span,
                }))
            }
            TokenKind::OctalLiteral(s) => {
                let s = s.clone();
                let span = self.current.span;
                self.advance()?;
                Ok(Expression::Literal(Literal {
                    kind: LiteralKind::Octal(s),
                    span,
                }))
            }
            TokenKind::HexLiteral(s) => {
                let s = s.clone();
                let span = self.current.span;
                self.advance()?;
                Ok(Expression::Literal(Literal {
                    kind: LiteralKind::Hex(s),
                    span,
                }))
            }
            
            // Identifier
            TokenKind::Identifier(name) => {
                let name = name.clone();
                let span = self.current.span;
                self.advance()?;
                Ok(Expression::Identifier { name, span })
            }
            
            // Parenthesized expression
            TokenKind::LeftParen => {
                self.advance()?;
                let inner = Box::new(self.parse_expression()?);
                self.expect(TokenKind::RightParen)?;
                let span = Span::new(start, self.current.span.end);
                Ok(Expression::Paren { inner, span })
            }
            
            // Absolute address
            TokenKind::Percent => {
                let address = self.parse_address()?;
                Ok(Expression::Address(address))
            }
            
            // Local variable reference: #varname
            TokenKind::Hash => {
                self.advance()?;
                let name = self.parse_identifier()?;
                let span = Span::new(start, self.current.span.end);
                Ok(Expression::Identifier { name, span })
            }
            
            // Type cast (data type name followed by #)
            _ if self.is_data_type_start() => {
                let target_type = self.parse_data_type()?;
                self.expect(TokenKind::Hash)?;
                let value = Box::new(self.parse_primary_expression()?);
                let span = Span::new(start, value.span().end);
                
                Ok(Expression::TypeCast {
                    target_type,
                    value,
                    span,
                })
            }
            
            _ => Err(Error::UnexpectedToken {
                expected: "expression".to_string(),
                found: format!("{:?}", self.current.kind),
                span: self.current.span,
            }),
        }
    }

    /// Parse function arguments.
    fn parse_arguments(&mut self) -> Result<Vec<Argument>> {
        let mut arguments = Vec::new();

        while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
            let arg_start = self.current.span.start;
            
            // Check for named argument with := or =>
            let (name, is_output) = if matches!(self.current.kind, TokenKind::Identifier(_)) {
                let peek_ahead = self.peek()?.kind.clone();
                if matches!(peek_ahead, TokenKind::Assign) {
                    // Input parameter: name := value
                    let n = self.parse_identifier()?;
                    self.expect(TokenKind::Assign)?;
                    (Some(n), false)
                } else if matches!(peek_ahead, TokenKind::Arrow) {
                    // Output parameter: name => variable
                    let n = self.parse_identifier()?;
                    self.expect(TokenKind::Arrow)?;
                    (Some(n), true)
                } else {
                    (None, false)
                }
            } else {
                (None, false)
            };

            let value = self.parse_expression()?;

            arguments.push(Argument {
                name,
                value,
                is_output,
                span: Span::new(arg_start, self.current.span.end),
            });

            if !self.consume(&TokenKind::Comma) {
                break;
            }
        }

        Ok(arguments)
    }

    /// Check if current token could start a data type.
    fn is_data_type_start(&self) -> bool {
        matches!(
            self.current.kind,
            TokenKind::Bool
                | TokenKind::Byte
                | TokenKind::Word
                | TokenKind::Dword
                | TokenKind::Lword
                | TokenKind::Sint
                | TokenKind::Int
                | TokenKind::Dint
                | TokenKind::Lint
                | TokenKind::Usint
                | TokenKind::Uint
                | TokenKind::Udint
                | TokenKind::Ulint
                | TokenKind::Real
                | TokenKind::Lreal
                | TokenKind::Char
                | TokenKind::Wchar
                | TokenKind::String
                | TokenKind::Wstring
                | TokenKind::Time
                | TokenKind::LTime
                | TokenKind::Date
                | TokenKind::TimeOfDay
                | TokenKind::DateAndTime
                | TokenKind::Array
                | TokenKind::Struct
                | TokenKind::Pointer
                | TokenKind::Ref
                | TokenKind::RefTo
                | TokenKind::Any
                | TokenKind::Void
        )
    }
}
