//! Parser for IEC 61131-3 Structured Text.
//!
//! Uses a Pratt parser for expressions and recursive descent for statements.

use crate::ast::*;
use crate::error::{ParseError, ParseErrorKind, ParseResult};
use crate::lexer::{Lexer, SpannedToken, Token};
use crate::security::{ParserLimits, ParserState};

/// Parser state.
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current: SpannedToken,
    previous: SpannedToken,
    security: ParserState,
    depth: usize,
}

impl<'a> Parser<'a> {
    /// Create a new parser for the given input with default security limits.
    pub fn new(input: &'a str) -> Self {
        Self::with_limits(input, ParserLimits::default())
    }

    /// Create a new parser with custom security limits.
    pub fn with_limits(input: &'a str, limits: ParserLimits) -> Self {
        // Check input size
        if input.len() > limits.max_input_size {
            panic!(
                "Input size {} exceeds maximum {}",
                input.len(),
                limits.max_input_size
            );
        }

        let mut lexer = Lexer::new(input);
        let current = lexer.next_token();
        Self {
            lexer,
            current: current.clone(),
            previous: current,
            security: ParserState::new(limits),
            depth: 0,
        }
    }

    /// Advance to the next token.
    fn advance(&mut self) {
        self.previous = self.current.clone();
        self.current = self.lexer.next_token();
    }

    /// Check if current token matches the expected type.
    fn check(&self, token: &Token) -> bool {
        std::mem::discriminant(&self.current.token) == std::mem::discriminant(token)
    }

    /// Check if current token is the given token exactly.
    #[allow(dead_code)]
    fn check_exact(&self, token: &Token) -> bool {
        &self.current.token == token
    }

    /// Consume the current token if it matches, otherwise error.
    fn expect(&mut self, expected: &Token, msg: &'static str) -> ParseResult<SpannedToken> {
        if self.check(expected) {
            let tok = self.current.clone();
            self.advance();
            Ok(tok)
        } else {
            Err(ParseError::new(
                ParseErrorKind::UnexpectedToken { expected: msg },
                self.current.span,
            ))
        }
    }

    /// Consume the current token if it matches.
    fn eat(&mut self, token: &Token) -> bool {
        if self.check(token) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Check if at end of input.
    fn at_end(&self) -> bool {
        self.current.token == Token::Eof
    }

    // ========================================================================
    // Expression Parsing (Pratt Parser)
    // ========================================================================

    /// Parse an expression.
    pub fn parse_expression(&mut self) -> ParseResult<Expr> {
        self.parse_expr_prec(0)
    }

    /// Parse expression with given minimum precedence.
    fn parse_expr_prec(&mut self, min_prec: u8) -> ParseResult<Expr> {
        let mut left = self.parse_unary()?;

        loop {
            let Some(op) = self.current_binary_op() else {
                break;
            };

            let prec = op.precedence();
            if prec < min_prec {
                break;
            }

            self.advance(); // consume operator
            let _op_span = self.previous.span;

            // For right-associative operators, use same precedence
            // For left-associative, use precedence + 1
            let next_prec = if op.is_right_assoc() { prec } else { prec + 1 };

            let right = self.parse_expr_prec(next_prec)?;
            let span = left.span.merge(&right.span);

            left = Expr::new(
                ExprKind::BinaryOp {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                },
                span,
            );
        }

        Ok(left)
    }

    /// Get the binary operator for the current token, if any.
    fn current_binary_op(&self) -> Option<BinaryOp> {
        match &self.current.token {
            Token::Plus => Some(BinaryOp::Add),
            Token::Minus => Some(BinaryOp::Sub),
            Token::Star => Some(BinaryOp::Mul),
            Token::Slash => Some(BinaryOp::Div),
            Token::Mod => Some(BinaryOp::Mod),
            Token::Power => Some(BinaryOp::Power),
            Token::Eq => Some(BinaryOp::Eq),
            Token::Ne => Some(BinaryOp::Ne),
            Token::Lt => Some(BinaryOp::Lt),
            Token::Le => Some(BinaryOp::Le),
            Token::Gt => Some(BinaryOp::Gt),
            Token::Ge => Some(BinaryOp::Ge),
            Token::And => Some(BinaryOp::And),
            Token::Or => Some(BinaryOp::Or),
            Token::Xor => Some(BinaryOp::Xor),
            _ => None,
        }
    }

    /// Parse unary expression.
    fn parse_unary(&mut self) -> ParseResult<Expr> {
        let start = self.current.span;

        if self.eat(&Token::Minus) {
            let expr = self.parse_unary()?;
            let span = start.merge(&expr.span);
            return Ok(Expr::new(
                ExprKind::UnaryOp {
                    op: UnaryOp::Neg,
                    expr: Box::new(expr),
                },
                span,
            ));
        }

        if self.eat(&Token::Not) {
            let expr = self.parse_unary()?;
            let span = start.merge(&expr.span);
            return Ok(Expr::new(
                ExprKind::UnaryOp {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                },
                span,
            ));
        }

        self.parse_postfix()
    }

    /// Parse postfix expressions (function calls, array indexing, member access).
    fn parse_postfix(&mut self) -> ParseResult<Expr> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.eat(&Token::LParen) {
                // Function call
                let name = match &expr.kind {
                    ExprKind::Ident(name) => name.clone(),
                    _ => {
                        return Err(ParseError::new(
                            ParseErrorKind::InvalidExpression,
                            expr.span,
                        ))
                    }
                };

                let args = self.parse_arg_list()?;
                let end = self.expect(&Token::RParen, ")")?.span;
                let span = expr.span.merge(&end);

                expr = Expr::new(ExprKind::FunctionCall { name, args }, span);
            } else if self.eat(&Token::LBracket) {
                // Array indexing
                let mut indices = vec![self.parse_expression()?];
                while self.eat(&Token::Comma) {
                    indices.push(self.parse_expression()?);
                }
                let end = self.expect(&Token::RBracket, "]")?.span;
                let span = expr.span.merge(&end);

                expr = Expr::new(
                    ExprKind::ArrayIndex {
                        array: Box::new(expr),
                        indices,
                    },
                    span,
                );
            } else if self.eat(&Token::Dot) {
                // Member access
                let member_tok = self.expect(&Token::Ident(String::new()), "identifier")?;
                let member = match member_tok.token {
                    Token::Ident(name) => name,
                    _ => unreachable!(),
                };
                let span = expr.span.merge(&member_tok.span);

                expr = Expr::new(
                    ExprKind::MemberAccess {
                        expr: Box::new(expr),
                        member,
                    },
                    span,
                );
            } else {
                break;
            }
        }

        Ok(expr)
    }

    /// Parse primary expression.
    fn parse_primary(&mut self) -> ParseResult<Expr> {
        let tok = self.current.clone();

        match &tok.token {
            Token::IntLiteral(v) => {
                let v = *v;
                self.advance();
                Ok(Expr::new(ExprKind::IntLiteral(v), tok.span))
            }
            Token::RealLiteral(v) => {
                let v = *v;
                self.advance();
                Ok(Expr::new(ExprKind::RealLiteral(v), tok.span))
            }
            Token::StringLiteral(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::new(ExprKind::StringLiteral(s), tok.span))
            }
            Token::WStringLiteral(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::new(ExprKind::WStringLiteral(s), tok.span))
            }
            Token::True => {
                self.advance();
                Ok(Expr::new(ExprKind::BoolLiteral(true), tok.span))
            }
            Token::False => {
                self.advance();
                Ok(Expr::new(ExprKind::BoolLiteral(false), tok.span))
            }
            Token::TimeLiteral(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::new(ExprKind::TimeLiteral(s), tok.span))
            }
            Token::DateLiteral(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::new(ExprKind::DateLiteral(s), tok.span))
            }
            Token::TodLiteral(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::new(ExprKind::TodLiteral(s), tok.span))
            }
            Token::DateTimeLiteral(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::new(ExprKind::DateTimeLiteral(s), tok.span))
            }
            Token::Ident(name) => {
                let name = name.clone();
                self.advance();
                Ok(Expr::new(ExprKind::Ident(name), tok.span))
            }
            Token::DirectAddress(addr) => {
                let addr = addr.clone();
                self.advance();
                let direct_addr = self.parse_direct_address(&addr)?;
                Ok(Expr::new(ExprKind::DirectAddress(direct_addr), tok.span))
            }
            Token::LParen => {
                self.advance();
                let inner = self.parse_expression()?;
                let end = self.expect(&Token::RParen, ")")?.span;
                let span = tok.span.merge(&end);
                Ok(Expr::new(ExprKind::Paren(Box::new(inner)), span))
            }
            _ => Err(ParseError::new(
                ParseErrorKind::InvalidExpression,
                tok.span,
            )),
        }
    }

    /// Parse a direct address string like "%IX0.0" or "%MW100".
    fn parse_direct_address(&self, addr: &str) -> ParseResult<DirectAddress> {
        let mut chars = addr.chars().peekable();
        
        // Skip the '%'
        if chars.next() != Some('%') {
            return Err(ParseError::new(
                ParseErrorKind::InvalidDirectAddress,
                self.previous.span,
            ));
        }
        
        // Parse location prefix (I, Q, M)
        let location = match chars.next() {
            Some('I') | Some('i') => LocationPrefix::Input,
            Some('Q') | Some('q') => LocationPrefix::Output,
            Some('M') | Some('m') => LocationPrefix::Memory,
            _ => return Err(ParseError::new(
                ParseErrorKind::InvalidDirectAddress,
                self.previous.span,
            )),
        };
        
        // Parse optional size prefix (X, B, W, D, L)
        let (size, address_start) = match chars.peek() {
            Some('X') | Some('x') => { chars.next(); (SizePrefix::Bit, true) }
            Some('B') | Some('b') => { chars.next(); (SizePrefix::Byte, true) }
            Some('W') | Some('w') => { chars.next(); (SizePrefix::Word, true) }
            Some('D') | Some('d') => { chars.next(); (SizePrefix::DoubleWord, true) }
            Some('L') | Some('l') => { chars.next(); (SizePrefix::LongWord, true) }
            _ => (SizePrefix::Bit, false), // Default to bit if no size prefix
        };
        let _ = address_start; // silence warning
        
        // Rest is the address
        let address: String = chars.collect();
        
        Ok(DirectAddress {
            location,
            size,
            address,
        })
    }

    /// Parse comma-separated argument list with optional named parameters.
    /// Supports Rockwell-specific empty arguments like `GSV(a, , b)`.
    fn parse_arg_list(&mut self) -> ParseResult<Vec<FunctionArg>> {
        let mut args = Vec::new();

        if !self.check(&Token::RParen) {
            args.push(self.parse_function_arg()?);
            while self.eat(&Token::Comma) {
                // Check for empty argument (comma followed by comma or rparen)
                if self.check(&Token::Comma) || self.check(&Token::RParen) {
                    // Empty argument - Rockwell extension
                    let span = self.current.span;
                    args.push(FunctionArg {
                        name: None,
                        value: None,
                        span,
                    });
                } else {
                    args.push(self.parse_function_arg()?);
                }
            }
        }

        Ok(args)
    }

    /// Parse a single function argument (may be named).
    fn parse_function_arg(&mut self) -> ParseResult<FunctionArg> {
        let start = self.current.span;
        
        // Check for named argument: ident := expr
        if let Token::Ident(name) = &self.current.token {
            let name = name.clone();
            let name_span = self.current.span;
            
            // Peek ahead to see if this is a named argument
            self.advance();
            if self.eat(&Token::Assign) {
                // Named argument
                let value = self.parse_expression()?;
                let span = start.merge(&value.span);
                return Ok(FunctionArg {
                    name: Some(name),
                    value: Some(value),
                    span,
                });
            } else {
                // Not a named argument, backtrack
                // We already consumed the identifier, so create an Ident expression
                let ident_expr = Expr::new(ExprKind::Ident(name), name_span);
                
                // Continue parsing as expression from this point
                let value = self.continue_expression_from(ident_expr)?;
                let span = start.merge(&value.span);
                return Ok(FunctionArg {
                    name: None,
                    value: Some(value),
                    span,
                });
            }
        }
        
        // Regular positional argument
        let value = self.parse_expression()?;
        let span = start.merge(&value.span);
        Ok(FunctionArg {
            name: None,
            value: Some(value),
            span,
        })
    }

    /// Continue parsing an expression from a given left-hand side.
    fn continue_expression_from(&mut self, left: Expr) -> ParseResult<Expr> {
        // Continue with postfix operations
        let mut expr = left;
        
        loop {
            if self.eat(&Token::LParen) {
                // Function call
                let name = match &expr.kind {
                    ExprKind::Ident(name) => name.clone(),
                    _ => {
                        return Err(ParseError::new(
                            ParseErrorKind::InvalidExpression,
                            expr.span,
                        ))
                    }
                };

                let args = self.parse_arg_list()?;
                let end = self.expect(&Token::RParen, ")")?.span;
                let span = expr.span.merge(&end);

                expr = Expr::new(ExprKind::FunctionCall { name, args }, span);
            } else if self.eat(&Token::LBracket) {
                // Array indexing
                let mut indices = vec![self.parse_expression()?];
                while self.eat(&Token::Comma) {
                    indices.push(self.parse_expression()?);
                }
                let end = self.expect(&Token::RBracket, "]")?.span;
                let span = expr.span.merge(&end);

                expr = Expr::new(
                    ExprKind::ArrayIndex {
                        array: Box::new(expr),
                        indices,
                    },
                    span,
                );
            } else if self.eat(&Token::Dot) {
                // Member access
                let member_tok = self.expect(&Token::Ident(String::new()), "identifier")?;
                let member = match member_tok.token {
                    Token::Ident(name) => name,
                    _ => unreachable!(),
                };
                let span = expr.span.merge(&member_tok.span);

                expr = Expr::new(
                    ExprKind::MemberAccess {
                        expr: Box::new(expr),
                        member,
                    },
                    span,
                );
            } else {
                break;
            }
        }

        // Now handle binary operators
        self.continue_binary_expr(expr, 0)
    }

    /// Continue parsing binary expression from a given left side.
    fn continue_binary_expr(&mut self, mut left: Expr, min_prec: u8) -> ParseResult<Expr> {
        loop {
            let Some(op) = self.current_binary_op() else {
                break;
            };

            let prec = op.precedence();
            if prec < min_prec {
                break;
            }

            self.advance(); // consume operator

            let next_prec = if op.is_right_assoc() { prec } else { prec + 1 };

            let right = self.parse_expr_prec(next_prec)?;
            let span = left.span.merge(&right.span);

            left = Expr::new(
                ExprKind::BinaryOp {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                },
                span,
            );
        }

        Ok(left)
    }

    // ========================================================================
    // Statement Parsing
    // ========================================================================

    /// Parse a single statement.
    pub fn parse_statement(&mut self) -> ParseResult<Stmt> {
        // Track statement count for security
        if let Err(e) = self.security.record_statement() {
            return Err(ParseError::new(
                ParseErrorKind::Security(e.to_string()),
                self.current.span,
            ));
        }

        let start = self.current.span;

        // Empty statement
        if self.eat(&Token::Semicolon) {
            return Ok(Stmt::new(StmtKind::Empty, start));
        }

        // Control flow statements
        if self.check(&Token::If) {
            return self.parse_if();
        }
        if self.check(&Token::Case) {
            return self.parse_case();
        }
        if self.check(&Token::For) {
            return self.parse_for();
        }
        if self.check(&Token::While) {
            return self.parse_while();
        }
        if self.check(&Token::Repeat) {
            return self.parse_repeat();
        }

        // Simple statements
        if self.eat(&Token::Exit) {
            self.expect(&Token::Semicolon, ";")?;
            return Ok(Stmt::new(StmtKind::Exit, start.merge(&self.previous.span)));
        }
        if self.eat(&Token::Continue) {
            self.expect(&Token::Semicolon, ";")?;
            return Ok(Stmt::new(
                StmtKind::Continue,
                start.merge(&self.previous.span),
            ));
        }
        if self.eat(&Token::Return) {
            let value = if !self.check(&Token::Semicolon) {
                Some(self.parse_expression()?)
            } else {
                None
            };
            self.expect(&Token::Semicolon, ";")?;
            return Ok(Stmt::new(
                StmtKind::Return { value },
                start.merge(&self.previous.span),
            ));
        }

        // Assignment or function call
        let target = self.parse_expression()?;

        if self.eat(&Token::Assign) {
            let value = self.parse_expression()?;
            self.expect(&Token::Semicolon, ";")?;
            let span = start.merge(&self.previous.span);
            return Ok(Stmt::new(StmtKind::Assignment { target, value }, span));
        }

        // Function call as statement (expression was the call)
        if let ExprKind::FunctionCall { name, args } = target.kind {
            self.expect(&Token::Semicolon, ";")?;
            let span = start.merge(&self.previous.span);
            let call_args = args
                .into_iter()
                .map(|arg| CallArg {
                    name: arg.name,
                    span: arg.span,
                    value: arg.value,
                })
                .collect();
            return Ok(Stmt::new(StmtKind::Call { name, args: call_args }, span));
        }

        Err(ParseError::new(
            ParseErrorKind::InvalidStatement,
            self.current.span,
        ))
    }

    /// Parse multiple statements until an end token.
    pub fn parse_statements(&mut self) -> ParseResult<Vec<Stmt>> {
        let mut stmts = Vec::new();
        while !self.at_end() {
            stmts.push(self.parse_statement()?);
        }
        Ok(stmts)
    }

    /// Parse statements until one of the given end tokens.
    fn parse_statements_until(&mut self, end_tokens: &[Token]) -> ParseResult<Vec<Stmt>> {
        let mut stmts = Vec::new();
        while !self.at_end() && !end_tokens.iter().any(|t| self.check(t)) {
            stmts.push(self.parse_statement()?);
        }
        Ok(stmts)
    }

    /// Parse IF statement.
    fn parse_if(&mut self) -> ParseResult<Stmt> {
        let start = self.current.span;
        self.expect(&Token::If, "IF")?;

        // Track nesting depth for security
        self.depth += 1;
        if let Err(e) = self.security.enter_depth() {
            return Err(ParseError::new(
                ParseErrorKind::Security(e.to_string()),
                self.current.span,
            ));
        }

        let condition = self.parse_expression()?;
        self.expect(&Token::Then, "THEN")?;

        let then_body = self.parse_statements_until(&[
            Token::Elsif,
            Token::Else,
            Token::EndIf,
        ])?;

        let mut elsif_branches = Vec::new();
        while self.eat(&Token::Elsif) {
            let cond = self.parse_expression()?;
            self.expect(&Token::Then, "THEN")?;
            let body = self.parse_statements_until(&[
                Token::Elsif,
                Token::Else,
                Token::EndIf,
            ])?;
            elsif_branches.push((cond, body));
        }

        let else_body = if self.eat(&Token::Else) {
            Some(self.parse_statements_until(&[Token::EndIf])?)
        } else {
            None
        };

        let end = self.expect(&Token::EndIf, "END_IF")?.span;
        self.expect(&Token::Semicolon, ";")?;

        // Exit depth tracking
        self.security.exit_depth();
        self.depth = self.depth.saturating_sub(1);

        Ok(Stmt::new(
            StmtKind::If {
                condition,
                then_body,
                elsif_branches,
                else_body,
            },
            start.merge(&end),
        ))
    }

    /// Parse CASE statement.
    fn parse_case(&mut self) -> ParseResult<Stmt> {
        let start = self.current.span;
        self.expect(&Token::Case, "CASE")?;

        // Track nesting depth
        self.depth += 1;
        if let Err(e) = self.security.enter_depth() {
            return Err(ParseError::new(
                ParseErrorKind::Security(e.to_string()),
                self.current.span,
            ));
        }

        let expr = self.parse_expression()?;
        self.expect(&Token::Of, "OF")?;

        let mut cases = Vec::new();
        while !self.check(&Token::Else) && !self.check(&Token::EndCase) && !self.at_end() {
            let case_start = self.current.span;

            // Parse case values
            let mut values = vec![self.parse_case_value()?];
            while self.eat(&Token::Comma) {
                values.push(self.parse_case_value()?);
            }
            self.expect(&Token::Colon, ":")?;

            // Parse case body - continue until we see a case label or end
            let body = self.parse_case_body()?;

            let case_end = self.previous.span;
            cases.push(CaseBranch {
                values,
                body,
                span: case_start.merge(&case_end),
            });
        }

        let else_body = if self.eat(&Token::Else) {
            Some(self.parse_statements_until(&[Token::EndCase])?)
        } else {
            None
        };

        let end = self.expect(&Token::EndCase, "END_CASE")?.span;
        self.expect(&Token::Semicolon, ";")?;

        // Exit depth tracking
        self.security.exit_depth();
        self.depth = self.depth.saturating_sub(1);

        Ok(Stmt::new(
            StmtKind::Case {
                expr,
                cases,
                else_body,
            },
            start.merge(&end),
        ))
    }

    /// Parse a single case value (may be range).
    fn parse_case_value(&mut self) -> ParseResult<CaseValue> {
        let from = self.parse_expression()?;
        if self.eat(&Token::DotDot) {
            let to = self.parse_expression()?;
            Ok(CaseValue::Range { from, to })
        } else {
            Ok(CaseValue::Single(from))
        }
    }

    /// Parse the body of a CASE branch.
    /// Continues until we see a new case label, ELSE, or END_CASE.
    fn parse_case_body(&mut self) -> ParseResult<Vec<Stmt>> {
        let mut stmts = Vec::new();
        while !self.at_end() {
            // Check for case termination
            if self.check(&Token::Else) || self.check(&Token::EndCase) {
                break;
            }
            // Check if this looks like a new case label (literal/ident followed by : or , or ..)
            if self.looks_like_case_label() {
                break;
            }
            stmts.push(self.parse_statement()?);
        }
        Ok(stmts)
    }

    /// Check if the current position looks like the start of a case label.
    fn looks_like_case_label(&self) -> bool {
        // A case label starts with an expression that's followed by ':', ',', or '..'
        // We look for patterns like:
        // - IntLiteral followed by : or , or ..
        // - Ident that is NOT followed by := (which would be assignment)
        match &self.current.token {
            Token::IntLiteral(_) | Token::RealLiteral(_) | Token::StringLiteral(_) => true,
            Token::Ident(_) => {
                // Need to peek at next token - if it's := it's assignment, else maybe case label
                // For now, simple heuristic: identifiers alone don't start case labels
                // This handles common case where assignments start case bodies
                false
            }
            _ => false,
        }
    }

    /// Parse FOR statement.
    fn parse_for(&mut self) -> ParseResult<Stmt> {
        let start = self.current.span;
        self.expect(&Token::For, "FOR")?;

        // Track nesting depth
        self.depth += 1;
        if let Err(e) = self.security.enter_depth() {
            return Err(ParseError::new(
                ParseErrorKind::Security(e.to_string()),
                self.current.span,
            ));
        }

        let var_tok = self.expect(&Token::Ident(String::new()), "identifier")?;
        let var = match var_tok.token {
            Token::Ident(name) => name,
            _ => unreachable!(),
        };

        self.expect(&Token::Assign, ":=")?;
        let from = self.parse_expression()?;
        self.expect(&Token::To, "TO")?;
        let to = self.parse_expression()?;

        let by = if self.eat(&Token::By) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.expect(&Token::Do, "DO")?;
        let body = self.parse_statements_until(&[Token::EndFor])?;
        let end = self.expect(&Token::EndFor, "END_FOR")?.span;
        self.expect(&Token::Semicolon, ";")?;

        // Exit depth tracking
        self.security.exit_depth();
        self.depth = self.depth.saturating_sub(1);

        Ok(Stmt::new(
            StmtKind::For {
                var,
                from,
                to,
                by,
                body,
            },
            start.merge(&end),
        ))
    }

    /// Parse WHILE statement.
    fn parse_while(&mut self) -> ParseResult<Stmt> {
        let start = self.current.span;
        self.expect(&Token::While, "WHILE")?;

        // Track nesting depth
        self.depth += 1;
        if let Err(e) = self.security.enter_depth() {
            return Err(ParseError::new(
                ParseErrorKind::Security(e.to_string()),
                self.current.span,
            ));
        }

        let condition = self.parse_expression()?;
        self.expect(&Token::Do, "DO")?;
        let body = self.parse_statements_until(&[Token::EndWhile])?;
        let end = self.expect(&Token::EndWhile, "END_WHILE")?.span;
        self.expect(&Token::Semicolon, ";")?;

        // Exit depth tracking
        self.security.exit_depth();
        self.depth = self.depth.saturating_sub(1);

        Ok(Stmt::new(
            StmtKind::While { condition, body },
            start.merge(&end),
        ))
    }

    /// Parse REPEAT statement.
    fn parse_repeat(&mut self) -> ParseResult<Stmt> {
        let start = self.current.span;
        self.expect(&Token::Repeat, "REPEAT")?;

        // Track nesting depth
        self.depth += 1;
        if let Err(e) = self.security.enter_depth() {
            return Err(ParseError::new(
                ParseErrorKind::Security(e.to_string()),
                self.current.span,
            ));
        }

        let body = self.parse_statements_until(&[Token::Until])?;
        self.expect(&Token::Until, "UNTIL")?;
        let until = self.parse_expression()?;
        let end = self.expect(&Token::Semicolon, ";")?.span;

        // Exit depth tracking
        self.security.exit_depth();
        self.depth = self.depth.saturating_sub(1);

        Ok(Stmt::new(
            StmtKind::Repeat { body, until },
            start.merge(&end),
        ))
    }

    // ========================================================================
    // POU Parsing
    // ========================================================================

    /// Parse a Program Organization Unit.
    pub fn parse_pou(&mut self) -> ParseResult<Pou> {
        let start = self.current.span;

        let (kind, end_token) = if self.eat(&Token::Program) {
            (PouKind::Program, Token::EndProgram)
        } else if self.eat(&Token::Function) {
            (PouKind::Function, Token::EndFunction)
        } else if self.eat(&Token::FunctionBlock) {
            (PouKind::FunctionBlock, Token::EndFunctionBlock)
        } else {
            return Err(ParseError::new(
                ParseErrorKind::UnexpectedToken {
                    expected: "PROGRAM, FUNCTION, or FUNCTION_BLOCK",
                },
                self.current.span,
            ));
        };

        // Parse name
        let name_tok = self.expect(&Token::Ident(String::new()), "identifier")?;
        let name = match name_tok.token {
            Token::Ident(n) => n,
            _ => unreachable!(),
        };

        // Parse return type for functions
        let return_type = if kind == PouKind::Function && self.eat(&Token::Colon) {
            Some(self.parse_type_spec()?)
        } else {
            None
        };

        // Parse variable blocks
        let mut var_blocks = Vec::new();
        while self.is_var_keyword() {
            var_blocks.push(self.parse_var_block()?);
        }

        // Parse body
        let body = self.parse_statements_until(std::slice::from_ref(&end_token))?;

        let end = self.expect(&end_token, "END_*")?.span;

        Ok(Pou {
            kind,
            name,
            return_type,
            var_blocks,
            body,
            span: start.merge(&end),
        })
    }

    /// Check if current token is a variable block keyword.
    fn is_var_keyword(&self) -> bool {
        matches!(
            self.current.token,
            Token::Var
                | Token::VarInput
                | Token::VarOutput
                | Token::VarInOut
                | Token::VarTemp
                | Token::VarGlobal
                | Token::VarExternal
        )
    }

    /// Parse a variable block.
    fn parse_var_block(&mut self) -> ParseResult<VarBlock> {
        let start = self.current.span;

        let kind = match &self.current.token {
            Token::Var => VarBlockKind::Var,
            Token::VarInput => VarBlockKind::VarInput,
            Token::VarOutput => VarBlockKind::VarOutput,
            Token::VarInOut => VarBlockKind::VarInOut,
            Token::VarTemp => VarBlockKind::VarTemp,
            Token::VarGlobal => VarBlockKind::VarGlobal,
            Token::VarExternal => VarBlockKind::VarExternal,
            _ => {
                return Err(ParseError::new(
                    ParseErrorKind::InvalidDeclaration,
                    self.current.span,
                ))
            }
        };
        self.advance();

        // Parse modifiers
        let constant = self.eat(&Token::Constant);
        let retain = if self.eat(&Token::Retain) {
            RetainKind::Retain
        } else if self.eat(&Token::NonRetain) {
            RetainKind::NonRetain
        } else {
            RetainKind::None
        };

        // Parse variable declarations
        let mut vars = Vec::new();
        while !self.check(&Token::EndVar) && !self.at_end() {
            vars.push(self.parse_var_decl()?);
        }

        let end = self.expect(&Token::EndVar, "END_VAR")?.span;

        Ok(VarBlock {
            kind,
            constant,
            retain,
            vars,
            span: start.merge(&end),
        })
    }

    /// Parse a single variable declaration.
    fn parse_var_decl(&mut self) -> ParseResult<VarDecl> {
        let start = self.current.span;

        let name_tok = self.expect(&Token::Ident(String::new()), "identifier")?;
        let name = match name_tok.token {
            Token::Ident(n) => n,
            _ => unreachable!(),
        };

        // Optional AT location
        let location = if self.eat(&Token::At) {
            // Parse location like %MW0.0
            let loc_tok = self.current.clone();
            self.advance();
            Some(format!("{:?}", loc_tok.token)) // Simplified for now
        } else {
            None
        };

        self.expect(&Token::Colon, ":")?;
        let var_type = self.parse_type_spec()?;

        // Optional initial value
        let initial = if self.eat(&Token::Assign) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        let end = self.expect(&Token::Semicolon, ";")?.span;

        Ok(VarDecl {
            name,
            var_type,
            initial,
            location,
            span: start.merge(&end),
        })
    }

    /// Parse a type specification.
    fn parse_type_spec(&mut self) -> ParseResult<TypeSpec> {
        let start = self.current.span;

        // ARRAY type
        if self.eat(&Token::Array) {
            self.expect(&Token::LBracket, "[")?;
            let mut ranges = vec![self.parse_array_range()?];
            while self.eat(&Token::Comma) {
                ranges.push(self.parse_array_range()?);
            }
            self.expect(&Token::RBracket, "]")?;
            self.expect(&Token::Of, "OF")?;
            let element = self.parse_type_spec()?;
            let span = start.merge(&element.span);
            return Ok(TypeSpec::new(
                TypeKind::Array {
                    ranges,
                    element: Box::new(element),
                },
                span,
            ));
        }

        // STRING type
        if self.eat(&Token::String_) {
            let length = if self.eat(&Token::LBracket) {
                let len_tok = self.expect(&Token::IntLiteral(0), "integer")?;
                let len = match len_tok.token {
                    Token::IntLiteral(v) => v as u32,
                    _ => unreachable!(),
                };
                self.expect(&Token::RBracket, "]")?;
                Some(len)
            } else {
                None
            };
            return Ok(TypeSpec::new(
                TypeKind::String { length },
                start.merge(&self.previous.span),
            ));
        }

        // WSTRING type
        if self.eat(&Token::WString) {
            let length = if self.eat(&Token::LBracket) {
                let len_tok = self.expect(&Token::IntLiteral(0), "integer")?;
                let len = match len_tok.token {
                    Token::IntLiteral(v) => v as u32,
                    _ => unreachable!(),
                };
                self.expect(&Token::RBracket, "]")?;
                Some(len)
            } else {
                None
            };
            return Ok(TypeSpec::new(
                TypeKind::WString { length },
                start.merge(&self.previous.span),
            ));
        }

        // Simple type name
        let name_tok = self.expect(&Token::Ident(String::new()), "type name")?;
        let name = match name_tok.token {
            Token::Ident(n) => n,
            _ => unreachable!(),
        };

        Ok(TypeSpec::new(TypeKind::Simple(name), name_tok.span))
    }

    /// Parse an array range.
    fn parse_array_range(&mut self) -> ParseResult<ArrayRange> {
        let start = self.current.span;
        let low = self.parse_expression()?;
        self.expect(&Token::DotDot, "..")?;
        let high = self.parse_expression()?;
        let span = start.merge(&high.span);
        Ok(ArrayRange { low, high, span })
    }

    // ========================================================================
    // TYPE Declarations
    // ========================================================================

    /// Parse a TYPE block containing type declarations.
    pub fn parse_type_block(&mut self) -> ParseResult<Vec<TypeDecl>> {
        self.expect(&Token::Type, "TYPE")?;

        let mut decls = Vec::new();
        while !self.check(&Token::EndType) && !self.at_end() {
            decls.push(self.parse_type_decl()?);
        }

        self.expect(&Token::EndType, "END_TYPE")?;
        Ok(decls)
    }

    /// Parse a single type declaration.
    fn parse_type_decl(&mut self) -> ParseResult<TypeDecl> {
        let start = self.current.span;

        let name_tok = self.expect(&Token::Ident(String::new()), "type name")?;
        let name = match name_tok.token {
            Token::Ident(n) => n,
            _ => unreachable!(),
        };

        self.expect(&Token::Colon, ":")?;

        let definition = self.parse_type_def()?;

        // Optional semicolon (some dialects require it, some don't)
        self.eat(&Token::Semicolon);

        let span = start.merge(&self.previous.span);
        Ok(TypeDecl { name, definition, span })
    }

    /// Parse a type definition (the right side of a type declaration).
    fn parse_type_def(&mut self) -> ParseResult<TypeDef> {
        // STRUCT
        if self.eat(&Token::Struct) {
            let mut fields = Vec::new();
            while !self.check(&Token::EndStruct) && !self.at_end() {
                fields.push(self.parse_var_decl()?);
            }
            self.expect(&Token::EndStruct, "END_STRUCT")?;
            return Ok(TypeDef::Struct { fields });
        }

        // ARRAY
        if self.eat(&Token::Array) {
            self.expect(&Token::LBracket, "[")?;
            let mut ranges = vec![self.parse_array_range()?];
            while self.eat(&Token::Comma) {
                ranges.push(self.parse_array_range()?);
            }
            self.expect(&Token::RBracket, "]")?;
            self.expect(&Token::Of, "OF")?;
            let element = self.parse_type_spec()?;
            return Ok(TypeDef::Array {
                ranges,
                element: Box::new(element),
            });
        }

        // Check for enumeration: (value1, value2, ...)
        if self.eat(&Token::LParen) {
            let mut values = vec![self.parse_enum_value()?];
            while self.eat(&Token::Comma) {
                values.push(self.parse_enum_value()?);
            }
            self.expect(&Token::RParen, ")")?;
            return Ok(TypeDef::Enum { values });
        }

        // Check for subrange: base_type (low..high)
        // or simple alias: base_type
        let base_tok = self.expect(&Token::Ident(String::new()), "type name")?;
        let base = match base_tok.token {
            Token::Ident(n) => n,
            _ => unreachable!(),
        };

        // Check for subrange
        if self.eat(&Token::LParen) {
            let low = self.parse_expression()?;
            self.expect(&Token::DotDot, "..")?;
            let high = self.parse_expression()?;
            self.expect(&Token::RParen, ")")?;
            return Ok(TypeDef::Subrange { base, low, high });
        }

        // Simple alias
        Ok(TypeDef::Alias(TypeSpec::new(
            TypeKind::Simple(base),
            base_tok.span,
        )))
    }

    /// Parse an enum value.
    fn parse_enum_value(&mut self) -> ParseResult<EnumValue> {
        let start = self.current.span;

        let name_tok = self.expect(&Token::Ident(String::new()), "enum value name")?;
        let name = match name_tok.token {
            Token::Ident(n) => n,
            _ => unreachable!(),
        };

        // Optional explicit value: := expr
        let value = if self.eat(&Token::Assign) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        let span = start.merge(&self.previous.span);
        Ok(EnumValue { name, value, span })
    }
}

// ============================================================================
// Public API Functions
// ============================================================================

/// Parse an expression from source.
pub fn parse_expression(source: &str) -> ParseResult<Expr> {
    Parser::new(source).parse_expression()
}

/// Parse a single statement from source.
pub fn parse_statement(source: &str) -> ParseResult<Stmt> {
    Parser::new(source).parse_statement()
}

/// Parse multiple statements from source.
pub fn parse_statements(source: &str) -> ParseResult<Vec<Stmt>> {
    Parser::new(source).parse_statements()
}

/// Parse a POU from source.
pub fn parse_pou(source: &str) -> ParseResult<Pou> {
    Parser::new(source).parse_pou()
}

/// Parse a TYPE block from source.
pub fn parse_type_block(source: &str) -> ParseResult<Vec<TypeDecl>> {
    Parser::new(source).parse_type_block()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_expression() {
        let expr = parse_expression("1 + 2").unwrap();
        assert!(matches!(expr.kind, ExprKind::BinaryOp { .. }));
    }

    #[test]
    fn test_parse_precedence() {
        let expr = parse_expression("1 + 2 * 3").unwrap();
        // Should be 1 + (2 * 3), not (1 + 2) * 3
        if let ExprKind::BinaryOp { op, right, .. } = &expr.kind {
            assert_eq!(*op, BinaryOp::Add);
            assert!(matches!(right.kind, ExprKind::BinaryOp { op: BinaryOp::Mul, .. }));
        } else {
            panic!("Expected BinaryOp");
        }
    }

    #[test]
    fn test_parse_function_call() {
        let expr = parse_expression("SIN(x)").unwrap();
        if let ExprKind::FunctionCall { name, args } = &expr.kind {
            assert_eq!(name, "SIN");
            assert_eq!(args.len(), 1);
        } else {
            panic!("Expected FunctionCall");
        }
    }

    #[test]
    fn test_parse_assignment() {
        let stmt = parse_statement("x := 1 + 2;").unwrap();
        assert!(matches!(stmt.kind, StmtKind::Assignment { .. }));
    }

    #[test]
    fn test_parse_if_statement() {
        let stmt = parse_statement("IF x > 0 THEN y := 1; END_IF;").unwrap();
        if let StmtKind::If { condition, then_body, .. } = &stmt.kind {
            assert!(matches!(condition.kind, ExprKind::BinaryOp { .. }));
            assert_eq!(then_body.len(), 1);
        } else {
            panic!("Expected If statement");
        }
    }

    #[test]
    fn test_parse_for_loop() {
        let stmt = parse_statement("FOR i := 1 TO 10 DO x := x + 1; END_FOR;").unwrap();
        if let StmtKind::For { var, body, .. } = &stmt.kind {
            assert_eq!(var, "i");
            assert_eq!(body.len(), 1);
        } else {
            panic!("Expected For statement");
        }
    }

    #[test]
    fn test_parse_while_loop() {
        let stmt = parse_statement("WHILE x < 10 DO x := x + 1; END_WHILE;").unwrap();
        assert!(matches!(stmt.kind, StmtKind::While { .. }));
    }

    #[test]
    fn test_parse_array_index() {
        let expr = parse_expression("arr[i, j]").unwrap();
        if let ExprKind::ArrayIndex { indices, .. } = &expr.kind {
            assert_eq!(indices.len(), 2);
        } else {
            panic!("Expected ArrayIndex");
        }
    }

    #[test]
    fn test_parse_member_access() {
        let expr = parse_expression("a.b.c").unwrap();
        assert!(matches!(expr.kind, ExprKind::MemberAccess { .. }));
    }

    #[test]
    fn test_parse_unary() {
        let expr = parse_expression("-x").unwrap();
        assert!(matches!(
            expr.kind,
            ExprKind::UnaryOp { op: UnaryOp::Neg, .. }
        ));

        let expr = parse_expression("NOT flag").unwrap();
        assert!(matches!(
            expr.kind,
            ExprKind::UnaryOp { op: UnaryOp::Not, .. }
        ));
    }

    #[test]
    fn test_parse_pou() {
        let source = r#"
            FUNCTION_BLOCK MyFB
                VAR_INPUT
                    input1 : INT;
                END_VAR
                VAR
                    local1 : REAL := 0.0;
                END_VAR
                output := input1 * 2;
            END_FUNCTION_BLOCK
        "#;
        let pou = parse_pou(source).unwrap();
        assert_eq!(pou.kind, PouKind::FunctionBlock);
        assert_eq!(pou.name, "MyFB");
        assert_eq!(pou.var_blocks.len(), 2);
        assert_eq!(pou.body.len(), 1);
    }
}
