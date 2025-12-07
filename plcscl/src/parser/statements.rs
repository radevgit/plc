//! Statement parsing.

use crate::ast::*;
use crate::error::{Error, Result};
use crate::lexer::TokenKind;
use crate::span::Span;

use super::Parser;

impl Parser {
    /// Parse a statement (returns None for empty statements).
    pub(super) fn parse_statement(&mut self) -> Result<Option<Statement>> {
        // Skip all leading comments
        while matches!(self.current.kind, TokenKind::Comment(_)) {
            self.advance()?;
        }
        
        // Return None for block-ending tokens (caller should handle these)
        match &self.current.kind {
            TokenKind::EndRegion | TokenKind::EndIf | TokenKind::EndCase |
            TokenKind::EndFor | TokenKind::EndWhile | TokenKind::EndRepeat |
            TokenKind::EndFunction | TokenKind::EndFunctionBlock | 
            TokenKind::EndDataBlock | TokenKind::EndOrganization |
            TokenKind::Elsif | TokenKind::Else | TokenKind::Until |
            TokenKind::Eof => return Ok(None),
            _ => {}
        }

        let start = self.current.span.start;

        let stmt = match &self.current.kind {
            // Control flow
            TokenKind::If => self.parse_if_statement(start)?,
            TokenKind::Case => self.parse_case_statement(start)?,
            TokenKind::For => self.parse_for_statement(start)?,
            TokenKind::While => self.parse_while_statement(start)?,
            TokenKind::Repeat => self.parse_repeat_statement(start)?,
            
            // Jump statements
            TokenKind::Continue => {
                self.advance()?;
                self.expect(TokenKind::Semicolon)?;
                Statement::Continue { span: Span::new(start, self.current.span.end) }
            }
            TokenKind::Exit => {
                self.advance()?;
                self.expect(TokenKind::Semicolon)?;
                Statement::Exit { span: Span::new(start, self.current.span.end) }
            }
            TokenKind::Return => {
                self.advance()?;
                let value = if !self.check(&TokenKind::Semicolon) {
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                self.expect(TokenKind::Semicolon)?;
                Statement::Return {
                    value,
                    span: Span::new(start, self.current.span.end),
                }
            }
            TokenKind::Goto => {
                self.advance()?;
                let label = self.parse_identifier()?;
                self.expect(TokenKind::Semicolon)?;
                Statement::Goto {
                    label,
                    span: Span::new(start, self.current.span.end),
                }
            }
            
            // Region
            TokenKind::Region => {
                self.advance()?;
                
                // Region name can be multiple words/tokens until end of line
                // Collect all tokens except structural ones (semicolons, BEGIN, etc.)
                let mut name_parts = Vec::new();
                while !self.is_at_end() && !self.check(&TokenKind::EndRegion) {
                    match &self.current.kind {
                        // Identifiers
                        TokenKind::Identifier(id) => {
                            name_parts.push(id.clone());
                            self.advance()?;
                        }
                        // Comments mark end of line
                        TokenKind::Comment(_) => {
                            self.advance()?;
                            break;
                        }
                        // Structural tokens mark end of name - start of body statements
                        TokenKind::Semicolon | TokenKind::Begin | TokenKind::Hash => break,
                        
                        // Allow operators and punctuation in names (e.g., "Anti-integral")
                        TokenKind::Minus => { name_parts.push("-".to_string()); self.advance()?; }
                        TokenKind::Plus => { name_parts.push("+".to_string()); self.advance()?; }
                        TokenKind::Slash => { name_parts.push("/".to_string()); self.advance()?; }
                        TokenKind::Star => { name_parts.push("*".to_string()); self.advance()?; }
                        
                        // Allow common keywords in region names (ALL keywords that might appear)
                        TokenKind::And => { name_parts.push("and".to_string()); self.advance()?; }
                        TokenKind::Or => { name_parts.push("or".to_string()); self.advance()?; }
                        TokenKind::Of => { name_parts.push("of".to_string()); self.advance()?; }
                        TokenKind::To => { name_parts.push("to".to_string()); self.advance()?; }
                        TokenKind::For => { name_parts.push("for".to_string()); self.advance()?; }
                        TokenKind::If => { name_parts.push("if".to_string()); self.advance()?; }
                        TokenKind::Not => { name_parts.push("not".to_string()); self.advance()?; }
                        
                        // Data type keywords can appear in region names too
                        TokenKind::Time => { name_parts.push("time".to_string()); self.advance()?; }
                        TokenKind::LTime => { name_parts.push("ltime".to_string()); self.advance()?; }
                        TokenKind::Date => { name_parts.push("date".to_string()); self.advance()?; }
                        TokenKind::Int => { name_parts.push("int".to_string()); self.advance()?; }
                        TokenKind::Real => { name_parts.push("real".to_string()); self.advance()?; }
                        TokenKind::Bool => { name_parts.push("bool".to_string()); self.advance()?; }
                        TokenKind::String => { name_parts.push("string".to_string()); self.advance()?; }
                        TokenKind::Array => { name_parts.push("array".to_string()); self.advance()?; }
                        TokenKind::Struct => { name_parts.push("struct".to_string()); self.advance()?; }
                        
                        // Everything else ends the name
                        _ => break,
                    }
                }
                let name = name_parts.join(" ");
                
                let mut body = Vec::new();
                while !self.check(&TokenKind::EndRegion) && !self.is_at_end() {
                    if let Some(stmt) = self.parse_statement_recovering() {
                        body.push(stmt);
                    }
                    // Skip trivia (comments/pragmas) to avoid infinite loop
                    // when parse_statement returns None
                    self.skip_trivia();
                }
                self.expect(TokenKind::EndRegion)?;
                
                Statement::Region {
                    name,
                    body,
                    span: Span::new(start, self.current.span.end),
                }
            }
            
            // Semicolon alone
            TokenKind::Semicolon => {
                self.advance()?;
                Statement::Empty { span: Span::new(start, self.current.span.end) }
            }
            
            // Assignment or call
            _ => {
                let expr = self.parse_expression()?;
                
                // Check for assignment operators
                let assign_op = match &self.current.kind {
                    TokenKind::Assign => Some(AssignOp::Assign),
                    TokenKind::PlusAssign => Some(AssignOp::AddAssign),
                    TokenKind::MinusAssign => Some(AssignOp::SubAssign),
                    TokenKind::StarAssign => Some(AssignOp::MulAssign),
                    TokenKind::SlashAssign => Some(AssignOp::DivAssign),
                    _ => None,
                };
                
                if let Some(op) = assign_op {
                    self.advance()?;
                    let value = self.parse_expression()?;
                    self.expect(TokenKind::Semicolon)?;
                    
                    Statement::Assignment {
                        target: expr,
                        operator: op,
                        value,
                        span: Span::new(start, self.current.span.end),
                    }
                } else {
                    // Must be a call statement
                    self.expect(TokenKind::Semicolon)?;
                    
                    match expr {
                        Expression::Call { function, arguments, .. } => {
                            Statement::Call {
                                target: *function,
                                arguments,
                                span: Span::new(start, self.current.span.end),
                            }
                        }
                        _ => {
                            return Err(Error::SyntaxError {
                                message: "Expected assignment or call".to_string(),
                                span: Span::new(start, self.current.span.end),
                            })
                        }
                    }
                }
            }
        };

        Ok(Some(stmt))
    }

    /// Parse a statement with error recovery.
    pub(super) fn parse_statement_recovering(&mut self) -> Option<Statement> {
        match self.parse_statement() {
            Ok(stmt) => stmt,
            Err(e) => {
                // Record error
                self.errors.push(e.clone());
                
                // Try to recover by skipping to statement boundary
                self.synchronize_statement();
                
                None // Continue parsing
            }
        }
    }

    /// Synchronize to next statement after error.
    fn synchronize_statement(&mut self) {
        // Skip to next semicolon, or statement keyword, or block end
        while !self.is_at_end() {
            if matches!(self.current.kind, TokenKind::Semicolon) {
                let _ = self.advance();
                return;
            }

            // Stop at statement keywords
            if matches!(
                self.current.kind,
                TokenKind::If
                    | TokenKind::Case
                    | TokenKind::For
                    | TokenKind::While
                    | TokenKind::Repeat
                    | TokenKind::Return
                    | TokenKind::Continue
                    | TokenKind::Exit
                    | TokenKind::Region
                    | TokenKind::EndRegion
                    | TokenKind::EndIf
                    | TokenKind::EndCase
                    | TokenKind::EndFor
                    | TokenKind::EndWhile
                    | TokenKind::EndRepeat
                    | TokenKind::EndFunction
                    | TokenKind::EndFunctionBlock
            ) {
                return;
            }

            if self.advance().is_err() {
                return;
            }
        }
    }

    /// Parse IF statement.
    fn parse_if_statement(&mut self, start: usize) -> Result<Statement> {
        self.expect(TokenKind::If)?;
        let condition = self.parse_expression()?;
        self.expect(TokenKind::Then)?;

        let mut then_branch = Vec::new();
        while !self.check(&TokenKind::Elsif)
            && !self.check(&TokenKind::Else)
            && !self.check(&TokenKind::EndIf)
            && !self.is_at_end()
        {
            if let Some(stmt) = self.parse_statement_recovering() {
                then_branch.push(stmt);
            }
            self.skip_trivia();
        }

        let mut elsif_branches = Vec::new();
        while self.consume(&TokenKind::Elsif) {
            let elsif_cond = self.parse_expression()?;
            self.expect(TokenKind::Then)?;
            
            let mut elsif_body = Vec::new();
            while !self.check(&TokenKind::Elsif)
                && !self.check(&TokenKind::Else)
                && !self.check(&TokenKind::EndIf)
                && !self.is_at_end()
            {
                if let Some(stmt) = self.parse_statement_recovering() {
                    elsif_body.push(stmt);
                }
                self.skip_trivia();
            }
            elsif_branches.push((elsif_cond, elsif_body));
        }

        let else_branch = if self.consume(&TokenKind::Else) {
            let mut else_body = Vec::new();
            while !self.check(&TokenKind::EndIf) && !self.is_at_end() {
                if let Some(stmt) = self.parse_statement_recovering() {
                    else_body.push(stmt);
                }
                self.skip_trivia();
            }
            Some(else_body)
        } else {
            None
        };

        self.expect(TokenKind::EndIf)?;
        self.expect(TokenKind::Semicolon)?;

        Ok(Statement::If {
            condition,
            then_branch,
            elsif_branches,
            else_branch,
            span: Span::new(start, self.current.span.end),
        })
    }

    /// Parse CASE statement.
    fn parse_case_statement(&mut self, start: usize) -> Result<Statement> {
        self.expect(TokenKind::Case)?;
        let selector = self.parse_expression()?;
        self.expect(TokenKind::Of)?;

        let mut branches = Vec::new();
        while !self.check(&TokenKind::Else) && !self.check(&TokenKind::EndCase) && !self.is_at_end() {
            let branch_start = self.current.span.start;
            
            // Parse case values
            let mut values = Vec::new();
            loop {
                let val = self.parse_expression()?;
                
                // Check for range
                if self.consume(&TokenKind::DotDot) {
                    let end = self.parse_expression()?;
                    values.push(CaseValue::Range(val, end));
                } else {
                    values.push(CaseValue::Single(val));
                }
                
                if !self.consume(&TokenKind::Comma) {
                    break;
                }
            }
            
            self.expect(TokenKind::Colon)?;
            
            // Parse branch body
            let mut body = Vec::new();
            while !self.is_case_branch_end() && !self.is_at_end() {
                if let Some(stmt) = self.parse_statement_recovering() {
                    body.push(stmt);
                }
                self.skip_trivia();
            }
            
            branches.push(CaseBranch {
                values,
                body,
                span: Span::new(branch_start, self.current.span.end),
            });
        }

        let else_branch = if self.consume(&TokenKind::Else) {
            let mut else_body = Vec::new();
            while !self.check(&TokenKind::EndCase) && !self.is_at_end() {
                if let Some(stmt) = self.parse_statement_recovering() {
                    else_body.push(stmt);
                }
                self.skip_trivia();
            }
            Some(else_body)
        } else {
            None
        };

        self.expect(TokenKind::EndCase)?;
        self.expect(TokenKind::Semicolon)?;

        Ok(Statement::Case {
            selector,
            branches,
            else_branch,
            span: Span::new(start, self.current.span.end),
        })
    }

    /// Check if at end of case branch.
    fn is_case_branch_end(&self) -> bool {
        // Check if next token could start a new case value
        matches!(
            self.current.kind,
            TokenKind::IntegerLiteral(_)
                | TokenKind::Identifier(_)
                | TokenKind::Else
                | TokenKind::EndCase
        )
    }

    /// Parse FOR statement.
    fn parse_for_statement(&mut self, start: usize) -> Result<Statement> {
        self.expect(TokenKind::For)?;
        
        // Variable can have # prefix
        let variable = if self.consume(&TokenKind::Hash) {
            self.parse_identifier()?
        } else {
            self.parse_identifier()?
        };
        
        self.expect(TokenKind::Assign)?;
        let start_expr = self.parse_expression()?;
        self.expect(TokenKind::To)?;
        let end_expr = self.parse_expression()?;
        
        let step = if self.consume(&TokenKind::By) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        self.expect(TokenKind::Do)?;

        let mut body = Vec::new();
        while !self.check(&TokenKind::EndFor) && !self.is_at_end() {
            if let Some(stmt) = self.parse_statement_recovering() {
                body.push(stmt);
            }
            self.skip_trivia();
        }

        self.expect(TokenKind::EndFor)?;
        self.expect(TokenKind::Semicolon)?;

        Ok(Statement::For {
            variable,
            start: start_expr,
            end: end_expr,
            step,
            body,
            span: Span::new(start, self.current.span.end),
        })
    }

    /// Parse WHILE statement.
    fn parse_while_statement(&mut self, start: usize) -> Result<Statement> {
        self.expect(TokenKind::While)?;
        let condition = self.parse_expression()?;
        self.expect(TokenKind::Do)?;

        let mut body = Vec::new();
        while !self.check(&TokenKind::EndWhile) && !self.is_at_end() {
            if let Some(stmt) = self.parse_statement_recovering() {
                body.push(stmt);
            }
            self.skip_trivia();
        }

        self.expect(TokenKind::EndWhile)?;
        self.expect(TokenKind::Semicolon)?;

        Ok(Statement::While {
            condition,
            body,
            span: Span::new(start, self.current.span.end),
        })
    }

    /// Parse REPEAT statement.
    fn parse_repeat_statement(&mut self, start: usize) -> Result<Statement> {
        self.expect(TokenKind::Repeat)?;

        let mut body = Vec::new();
        while !self.check(&TokenKind::Until) && !self.is_at_end() {
            if let Some(stmt) = self.parse_statement_recovering() {
                body.push(stmt);
            }
            self.skip_trivia();
        }

        self.expect(TokenKind::Until)?;
        let condition = self.parse_expression()?;
        self.expect(TokenKind::Semicolon)?;

        Ok(Statement::Repeat {
            body,
            condition,
            span: Span::new(start, self.current.span.end),
        })
    }
}
