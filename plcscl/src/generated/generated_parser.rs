// Generated SCL parser

use super::lexer::{Lexer, Token, TokenKind};
use super::ast::*;

/// Internal parser limits (runtime checks during parsing)
/// Note: For external API, use security::ParserLimits which provides conversion
#[derive(Debug, Clone)]
pub struct ParserLimits {
    pub max_tokens: usize,
    pub max_iterations: usize,
    pub max_recursion_depth: usize,
    pub max_collection_size: usize,
    pub max_nesting_depth: usize,
    pub max_nodes: usize,
}

impl Default for ParserLimits {
    fn default() -> Self {
        ParserLimits {
            max_tokens: 1_000_000,
            max_iterations: 100_000,
            max_recursion_depth: 256,
            max_collection_size: 100_000,
            max_nesting_depth: 100,
            max_nodes: 10_000_000,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParseErrorKind {
    UnexpectedToken { expected: String, found: String },
    UnexpectedEof,
    TooManyTokens,
    TooManyIterations,
    RecursionLimitExceeded,
    CollectionSizeExceeded { collection: String },
    TooManyStatements,
    TooManyNodes,
}

impl std::fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseErrorKind::UnexpectedToken { expected, found } => 
                write!(f, "Expected {}, found {}", expected, found),
            ParseErrorKind::UnexpectedEof => 
                write!(f, "Unexpected end of file"),
            ParseErrorKind::TooManyTokens => 
                write!(f, "Too many tokens (possible memory bomb)"),
            ParseErrorKind::TooManyIterations => 
                write!(f, "Too many iterations (possible infinite loop)"),
            ParseErrorKind::RecursionLimitExceeded => 
                write!(f, "Recursion depth limit exceeded (possible stack overflow)"),
            ParseErrorKind::CollectionSizeExceeded { collection } => 
                write!(f, "Too many items in {} (possible complexity attack)", collection),
            ParseErrorKind::TooManyStatements => 
                write!(f, "Too many statements in block"),
            ParseErrorKind::TooManyNodes => 
                write!(f, "Too many AST nodes (possible complexity attack)"),
        }
    }
}

impl std::error::Error for ParseErrorKind {}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub span: (usize, usize),
    pub source: Option<String>,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.kind)
    }
}

impl ParseError {
    pub fn new(kind: ParseErrorKind, span: (usize, usize)) -> Self {
        ParseError { kind, span, source: None }
    }
    
    pub fn with_source(mut self, source: &str) -> Self {
        self.source = Some(source.to_string());
        self
    }
    
    pub fn message(&self) -> String {
        let base_msg = match &self.kind {
            ParseErrorKind::UnexpectedToken { expected, found } => 
                format!("Expected {}, found {}", expected, found),
            ParseErrorKind::UnexpectedEof => 
                "Unexpected end of file".to_string(),
            ParseErrorKind::TooManyTokens => 
                "Too many tokens (possible memory bomb)".to_string(),
            ParseErrorKind::TooManyIterations => 
                "Too many iterations (possible infinite loop)".to_string(),
            ParseErrorKind::RecursionLimitExceeded => 
                "Recursion depth limit exceeded (possible stack overflow)".to_string(),
            ParseErrorKind::CollectionSizeExceeded { collection } => 
                format!("Too many items in {} (possible complexity attack)", collection),
            ParseErrorKind::TooManyStatements => 
                "Too many statements in block".to_string(),
            ParseErrorKind::TooManyNodes => 
                "Too many AST nodes (possible complexity attack)".to_string(),
        };
        
        if let Some(source) = &self.source {
            format!("{}\n{}", base_msg, self.format_source_context(source))
        } else {
            base_msg
        }
    }
    
    pub fn suggestion(&self) -> Option<String> {
        match &self.kind {
            ParseErrorKind::UnexpectedToken { expected, found } => {
                // Provide helpful suggestions for common mistakes
                if expected.contains(";") && !found.contains(";") {
                    Some("Did you forget a semicolon?".to_string())
                } else if expected.contains("END_") {
                    Some(format!("Missing closing keyword {}", expected))
                } else if expected.contains("identifier") && found.contains("Operator") {
                    Some("Expected a name here".to_string())
                } else if expected.contains("Begin") {
                    Some("Block body must start with BEGIN".to_string())
                } else {
                    None
                }
            }
            ParseErrorKind::UnexpectedEof => {
                Some("File ended unexpectedly. Did you forget an END_FUNCTION_BLOCK or similar closing keyword?".to_string())
            }
            _ => None,
        }
    }
    
    fn format_source_context(&self, source: &str) -> String {
        let lines: Vec<&str> = source.lines().collect();
        let (start, end) = self.span;
        
        // Find line and column
        let mut current_pos = 0;
        let mut line_num = 0;
        let mut col_num = 0;
        
        for (i, line) in lines.iter().enumerate() {
            let line_end = current_pos + line.len() + 1; // +1 for newline
            if start < line_end {
                line_num = i;
                col_num = start - current_pos;
                break;
            }
            current_pos = line_end;
        }
        
        let mut output = String::new();
        output.push_str(&format!("\n  --> line {}:{}\n", line_num + 1, col_num + 1));
        
        // Show context: previous line, error line, next line
        let start_line = if line_num > 0 { line_num - 1 } else { 0 };
        let end_line = (line_num + 2).min(lines.len());
        
        for i in start_line..end_line {
            let marker = if i == line_num { ">" } else { " " };
            output.push_str(&format!("   {} | {}\n", marker, lines[i]));
            
            if i == line_num {
                // Add error indicator
                let padding = " ".repeat(col_num + 6); // 6 = " > | ".len()
                let indicator_len = (end - start).max(1).min(lines[i].len() - col_num);
                let indicator = "^".repeat(indicator_len);
                output.push_str(&format!("{}{}\n", padding, indicator));
            }
        }
        
        if let Some(suggestion) = self.suggestion() {
            output.push_str(&format!("\n  help: {}\n", suggestion));
        }
        
        output
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    source: String,              // Source code for error context
    errors: Vec<ParseError>,     // Collected errors
    // Security limits
    max_tokens: usize,           // Limit total tokens to prevent memory bombs
    max_iterations: usize,       // Limit loop iterations
    recursion_depth: usize,      // Current recursion depth
    max_recursion_depth: usize,  // Maximum allowed recursion
    max_collection_size: usize,  // Maximum items in any Vec (statements, declarations, etc)
    max_nesting_depth: usize,    // Maximum nesting of blocks/expressions
    nodes_parsed: usize,         // Total nodes parsed (detect complexity attacks)
    max_nodes: usize,            // Maximum total nodes
}

impl Parser {
    pub fn new(input: &str) -> Self {
        Self::with_limits(input, ParserLimits::default())
    }
    
    pub fn with_limits(input: &str, limits: ParserLimits) -> Self {
        let mut lexer = Lexer::new(input);
        let mut tokens = Vec::new();
        
        // Limit token generation to prevent memory exhaustion
        let mut token_count = 0;
        loop {
            let token = lexer.next_token();
            let is_eof = matches!(token.kind, TokenKind::Eof);
            tokens.push(token);
            token_count += 1;
            
            if token_count > limits.max_tokens {
                // Return parser with error token
                tokens.clear();
                tokens.push(Token::new(TokenKind::Eof, 0, 0));
                break;
            }
            
            if is_eof {
                break;
            }
        }
        
        Parser { 
            tokens, 
            pos: 0,
            source: input.to_string(),
            errors: Vec::new(),
            max_tokens: limits.max_tokens,
            max_iterations: limits.max_iterations,
            recursion_depth: 0,
            max_recursion_depth: limits.max_recursion_depth,
            max_collection_size: limits.max_collection_size,
            max_nesting_depth: limits.max_nesting_depth,
            nodes_parsed: 0,
            max_nodes: limits.max_nodes,
        }
    }
    
    fn check_complexity(&mut self) -> Result<(), ParseError> {
        self.nodes_parsed += 1;
        if self.nodes_parsed > self.max_nodes {
            return Err(self.make_error(
                ParseErrorKind::TooManyNodes,
                self.current().span,
            ));
        }
        Ok(())
    }
    
    fn check_collection_size(&self, size: usize, collection_name: &str) -> Result<(), ParseError> {
        if size >= self.max_collection_size {
            return Err(self.make_error(
                ParseErrorKind::CollectionSizeExceeded { collection: collection_name.to_string() },
                self.current().span,
            ));
        }
        Ok(())
    }
    
    fn check_recursion(&mut self) -> Result<(), ParseError> {
        self.recursion_depth += 1;
        if self.recursion_depth > self.max_recursion_depth {
            return Err(self.make_error(
                ParseErrorKind::RecursionLimitExceeded,
                self.current().span,
            ));
        }
        Ok(())
    }
    
    fn uncheck_recursion(&mut self) {
        if self.recursion_depth > 0 {
            self.recursion_depth -= 1;
        }
    }
    
    fn make_error(&self, kind: ParseErrorKind, span: (usize, usize)) -> ParseError {
        ParseError::new(kind, span).with_source(&self.source)
    }
    
    fn record_error(&mut self, error: ParseError) {
        self.errors.push(error);
    }
    
    pub fn get_errors(&self) -> &[ParseError] {
        &self.errors
    }
    
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
    
    fn current(&self) -> &Token {
        &self.tokens[self.pos]
    }
    
    fn peek(&self) -> &TokenKind {
        &self.current().kind
    }
    
    fn advance(&mut self) {
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
    }
    
    fn expect(&mut self, expected: TokenKind) -> Result<Token, ParseError> {
        let token = self.current().clone();
        if std::mem::discriminant(&token.kind) != std::mem::discriminant(&expected) {
            return Err(self.make_error(
                ParseErrorKind::UnexpectedToken { 
                    expected: format!("{:?}", expected), 
                    found: format!("{:?}", token.kind) 
                },
                token.span,
            ));
        }
        self.advance();
        Ok(token)
    }
    
    fn expect_identifier(&mut self) -> Result<String, ParseError> {
        let token = self.current().clone();
        match &token.kind {
            TokenKind::Identifier(name) => {
                self.advance();
                Ok(name.clone())
            }
            _ => Err(self.make_error(
                ParseErrorKind::UnexpectedToken { 
                    expected: "identifier".to_string(), 
                    found: format!("{:?}", token.kind) 
                },
                token.span,
            )),
        }
    }
    
    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        self.check_recursion()?;
        let mut blocks = Vec::new();
        let mut iterations = 0;
        
        while !matches!(self.peek(), TokenKind::Eof) {
            self.check_complexity()?;
            self.check_collection_size(blocks.len(), "program blocks")?;
            
            iterations += 1;
            if iterations > self.max_iterations {
                return Err(self.make_error(
                    ParseErrorKind::TooManyIterations,
                    self.current().span,
                ));
            }
            blocks.push(self.parse_block()?);
        }
        
        self.uncheck_recursion();
        Ok(Program { blocks })
    }
    
    fn parse_block(&mut self) -> Result<Block, ParseError> {
        self.check_recursion()?;
        let result = match self.peek() {
            TokenKind::FunctionBlock => Ok(Block::FunctionBlock(self.parse_function_block()?)),
            TokenKind::Function => Ok(Block::Function(self.parse_function()?)),
            TokenKind::DataBlock => Ok(Block::DataBlock(self.parse_data_block()?)),
            TokenKind::Type => Ok(Block::TypeDecl(self.parse_type_decl()?)),
            TokenKind::OrganizationBlock => Ok(Block::OrganizationBlock(self.parse_organization_block()?)),
            TokenKind::Program => Ok(Block::ProgramBlock(self.parse_program_block()?)),
            TokenKind::Class => Ok(Block::Class(self.parse_class_decl()?)),
            TokenKind::Interface => Ok(Block::Interface(self.parse_interface_decl()?)),
            kind => Err(self.make_error(
                ParseErrorKind::UnexpectedToken { 
                    expected: "block declaration".to_string(), 
                    found: format!("{:?}", kind) 
                },
                self.current().span,
            )),
        };
        self.uncheck_recursion();
        result
    }
    
    fn parse_function_block(&mut self) -> Result<FunctionBlock, ParseError> {
        self.check_recursion()?;
        self.expect(TokenKind::FunctionBlock)?;
        let name = self.expect_identifier()?;
        
        // Skip TIA Portal metadata and attributes (TITLE, AUTHOR, { attrs }, etc.)
        self.skip_tia_metadata();
        
        // Parse optional extends clause
        let extends = if matches!(self.peek(), TokenKind::Extends) {
            self.advance();
            Some(self.expect_identifier()?)
        } else {
            None
        };
        
        // Parse optional implements clause
        let mut implements = Vec::new();
        if matches!(self.peek(), TokenKind::Implements) {
            self.advance();
            implements.push(self.expect_identifier()?);
            while matches!(self.peek(), TokenKind::Operator(op) if op == ",") {
                self.advance();
                implements.push(self.expect_identifier()?);
            }
        }
        
        let mut var_sections = Vec::new();
        let mut methods = Vec::new();
        
        // Parse var sections and methods
        loop {
            match self.peek() {
                TokenKind::VarInput | TokenKind::VarOutput | TokenKind::VarInOut | TokenKind::VarTemp | TokenKind::Var => {
                    var_sections.push(self.parse_var_section()?);
                }
                TokenKind::Method => {
                    methods.push(self.parse_method_decl()?);
                }
                _ => break,
            }
        }
        
        // BEGIN is optional for TIA Portal compatibility
        if matches!(self.peek(), TokenKind::Begin) {
            self.advance();
        }
        
        let mut statements = Vec::new();
        let mut iterations = 0;
        while !matches!(self.peek(), TokenKind::EndFunctionBlock) {
            iterations += 1;
            if iterations > self.max_iterations {
                return Err(self.make_error(ParseErrorKind::TooManyStatements, self.current().span));
            }
            statements.push(self.parse_statement()?);
        }
        
        self.expect(TokenKind::EndFunctionBlock)?;
        
        self.uncheck_recursion();
        Ok(FunctionBlock { name, extends, implements, var_sections, methods, statements })
    }
    
    fn parse_var_section(&mut self) -> Result<VarSection, ParseError> {
        self.check_recursion()?;
        let result = match self.peek() {
            TokenKind::VarInput => {
                self.advance();
                // Check for optional RETAIN / NON_RETAIN
                let _retain_type = if matches!(self.peek(), TokenKind::Retain) {
                    self.advance();
                    "RETAIN"
                } else if matches!(self.peek(), TokenKind::NonRetain) {
                    self.advance();
                    "NON_RETAIN"
                } else {
                    ""
                };
                let mut declarations = Vec::new();
                let mut iterations = 0;
                while !matches!(self.peek(), TokenKind::EndVar) {
                    iterations += 1;
                    if iterations > self.max_iterations {
                        return Err(self.make_error(ParseErrorKind::TooManyIterations, self.current().span));
                    }
                    declarations.push(self.parse_var_declaration()?);
                }
                self.expect(TokenKind::EndVar)?;
                Ok(VarSection::Input(VarInput { declarations }))
            }
            TokenKind::VarOutput => {
                self.advance();
                // Check for optional RETAIN / NON_RETAIN
                let _retain_type = if matches!(self.peek(), TokenKind::Retain) {
                    self.advance();
                    "RETAIN"
                } else if matches!(self.peek(), TokenKind::NonRetain) {
                    self.advance();
                    "NON_RETAIN"
                } else {
                    ""
                };
                let mut declarations = Vec::new();
                let mut iterations = 0;
                while !matches!(self.peek(), TokenKind::EndVar) {
                    iterations += 1;
                    if iterations > self.max_iterations {
                        return Err(self.make_error(ParseErrorKind::TooManyIterations, self.current().span));
                    }
                    declarations.push(self.parse_var_declaration()?);
                }
                self.expect(TokenKind::EndVar)?;
                Ok(VarSection::Output(VarOutput { declarations }))
            }
            TokenKind::VarInOut => {
                self.advance();
                // Check for optional RETAIN / NON_RETAIN
                let _retain_type = if matches!(self.peek(), TokenKind::Retain) {
                    self.advance();
                    "RETAIN"
                } else if matches!(self.peek(), TokenKind::NonRetain) {
                    self.advance();
                    "NON_RETAIN"
                } else {
                    ""
                };
                let mut declarations = Vec::new();
                let mut iterations = 0;
                while !matches!(self.peek(), TokenKind::EndVar) {
                    iterations += 1;
                    if iterations > self.max_iterations {
                        return Err(self.make_error(ParseErrorKind::TooManyIterations, self.current().span));
                    }
                    declarations.push(self.parse_var_declaration()?);
                }
                self.expect(TokenKind::EndVar)?;
                Ok(VarSection::InOut(VarInout { declarations }))
            }
            TokenKind::VarTemp => {
                self.advance();
                // Check for optional RETAIN / NON_RETAIN
                let _retain_type = if matches!(self.peek(), TokenKind::Retain) {
                    self.advance();
                    "RETAIN"
                } else if matches!(self.peek(), TokenKind::NonRetain) {
                    self.advance();
                    "NON_RETAIN"
                } else {
                    ""
                };
                let mut declarations = Vec::new();
                let mut iterations = 0;
                while !matches!(self.peek(), TokenKind::EndVar) {
                    iterations += 1;
                    if iterations > self.max_iterations {
                        return Err(self.make_error(ParseErrorKind::TooManyIterations, self.current().span));
                    }
                    declarations.push(self.parse_var_declaration()?);
                }
                self.expect(TokenKind::EndVar)?;
                Ok(VarSection::Temp(VarTemp { declarations }))
            }
            TokenKind::Var => {
                self.advance();
                
                // Check for VAR CONSTANT / VAR RETAIN / VAR NON_RETAIN
                let var_type = if matches!(self.peek(), TokenKind::Constant) {
                    self.advance();
                    "CONSTANT"
                } else if matches!(self.peek(), TokenKind::Retain) {
                    self.advance();
                    "RETAIN"
                } else if matches!(self.peek(), TokenKind::NonRetain) {
                    self.advance();
                    "NON_RETAIN"
                } else {
                    "VAR"
                };
                
                let mut declarations = Vec::new();
                let mut iterations = 0;
                while !matches!(self.peek(), TokenKind::EndVar) {
                    iterations += 1;
                    if iterations > self.max_iterations {
                        return Err(self.make_error(ParseErrorKind::TooManyIterations, self.current().span));
                    }
                    declarations.push(self.parse_var_declaration()?);
                }
                self.expect(TokenKind::EndVar)?;
                
                match var_type {
                    "CONSTANT" => Ok(VarSection::Constant(VarDecl { declarations })),
                    "RETAIN" | "NON_RETAIN" | "VAR" => Ok(VarSection::Var(VarDecl { declarations })),
                    _ => Ok(VarSection::Var(VarDecl { declarations })),
                }
            }
            TokenKind::VarAccess => {
                self.advance();
                let mut declarations = Vec::new();
                let mut iterations = 0;
                while !matches!(self.peek(), TokenKind::EndVar) {
                    iterations += 1;
                    if iterations > self.max_iterations {
                        return Err(self.make_error(ParseErrorKind::TooManyIterations, self.current().span));
                    }
                    // VAR_ACCESS uses access_declaration format: name : path : type ;
                    declarations.push(self.parse_var_declaration()?);
                }
                self.expect(TokenKind::EndVar)?;
                Ok(VarSection::Var(VarDecl { declarations }))  // Treat as regular VAR for now
            }
            TokenKind::VarExternal => {
                self.advance();
                // Check for optional RETAIN / NON_RETAIN
                let _retain_type = if matches!(self.peek(), TokenKind::Retain) {
                    self.advance();
                    "RETAIN"
                } else if matches!(self.peek(), TokenKind::NonRetain) {
                    self.advance();
                    "NON_RETAIN"
                } else {
                    ""
                };
                let mut declarations = Vec::new();
                let mut iterations = 0;
                while !matches!(self.peek(), TokenKind::EndVar) {
                    iterations += 1;
                    if iterations > self.max_iterations {
                        return Err(self.make_error(ParseErrorKind::TooManyIterations, self.current().span));
                    }
                    declarations.push(self.parse_var_declaration()?);
                }
                self.expect(TokenKind::EndVar)?;
                Ok(VarSection::Var(VarDecl { declarations }))  // Treat as regular VAR for now
            }
            kind => Err(self.make_error(
                ParseErrorKind::UnexpectedToken { 
                    expected: "VAR section (VAR_INPUT/VAR_OUTPUT/VAR_INOUT/VAR_TEMP/VAR/VAR_ACCESS/VAR_EXTERNAL)".to_string(), 
                    found: format!("{:?}", kind) 
                },
                self.current().span,
            )),
        };
        self.uncheck_recursion();
        result
    }
    
    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        match self.peek() {
            TokenKind::If => Ok(Statement::If(self.parse_if_stmt()?)),
            TokenKind::Case => Ok(Statement::Case(self.parse_case_stmt()?)),
            TokenKind::For => Ok(Statement::For(self.parse_for_stmt()?)),
            TokenKind::While => Ok(Statement::While(self.parse_while_stmt()?)),
            TokenKind::Repeat => Ok(Statement::Repeat(self.parse_repeat_stmt()?)),
            TokenKind::Return => Ok(Statement::Return(self.parse_return_stmt()?)),
            TokenKind::Exit => {
                self.advance();
                self.expect(TokenKind::Operator(";".to_string()))?;
                Ok(Statement::Exit)
            }
            TokenKind::Continue => {
                self.advance();
                self.expect(TokenKind::Operator(";".to_string()))?;
                Ok(Statement::Continue)
            }
            TokenKind::Region => Ok(Statement::Region(self.parse_region()?)),
            TokenKind::Operator(op) if op == "%" => {
                // Absolute address assignment like %MW504 := value;
                Ok(Statement::Assignment(self.parse_assignment()?))
            }
            TokenKind::Identifier(_) => {
                // Could be assignment or function call
                // Look ahead past member access (.field) and array indexing ([...])
                let checkpoint = self.pos;
                let _name = self.expect_identifier()?;
                
                // Skip member access and array indexing
                while matches!(self.peek(), TokenKind::Operator(op) if op == "." || op == "[") {
                    match self.peek() {
                        TokenKind::Operator(op) if op == "." => {
                            self.advance();
                            let _ = self.expect_identifier()?;
                        }
                        TokenKind::Operator(op) if op == "[" => {
                            self.advance();
                            let mut depth = 1;
                            while depth > 0 && !matches!(self.peek(), TokenKind::Eof) {
                                match self.peek() {
                                    TokenKind::Operator(op) if op == "[" => depth += 1,
                                    TokenKind::Operator(op) if op == "]" => depth -= 1,
                                    _ => {}
                                }
                                if depth > 0 {
                                    self.advance();
                                }
                            }
                            self.expect(TokenKind::Operator("]".to_string()))?;
                        }
                        _ => break,
                    }
                }
                
                if matches!(self.peek(), TokenKind::Operator(op) if op == ":=") {
                    self.pos = checkpoint;
                    Ok(Statement::Assignment(self.parse_assignment()?))
                } else if matches!(self.peek(), TokenKind::Operator(op) if op == "?=") {
                    self.pos = checkpoint;
                    Ok(Statement::NullableAssignment(self.parse_nullable_assignment()?))
                } else if matches!(self.peek(), TokenKind::Operator(op) if op == "(") {
                    self.pos = checkpoint;
                    Ok(Statement::FunctionCall(self.parse_function_call_stmt()?))
                } else {
                    Err(self.make_error(
                        ParseErrorKind::UnexpectedToken { 
                            expected: ":= or ( after identifier".to_string(), 
                            found: format!("{:?}", self.peek()) 
                        },
                        self.current().span,
                    ))
                }
            }
            kind => Err(self.make_error(
                ParseErrorKind::UnexpectedToken { 
                    expected: "statement".to_string(), 
                    found: format!("{:?}", kind) 
                },
                self.current().span,
            )),
        }
    }
    // Additional parser methods
    
    fn parse_function(&mut self) -> Result<Function, ParseError> { 
        self.expect(TokenKind::Function)?;
        let name = self.expect_identifier()?;
        self.expect(TokenKind::Operator(":".to_string()))?;
        let return_type = self.parse_type_ref()?;
        
        // Skip TIA Portal metadata and attributes
        self.skip_tia_metadata();
        
        let mut var_sections = Vec::new();
        while matches!(self.peek(), TokenKind::VarInput | TokenKind::VarOutput | TokenKind::VarInOut | TokenKind::VarTemp | TokenKind::Var | TokenKind::VarAccess | TokenKind::VarExternal) {
            var_sections.push(self.parse_var_section()?);
        }
        
        // BEGIN is optional for TIA Portal compatibility
        if matches!(self.peek(), TokenKind::Begin) {
            self.advance();
        }
        
        let mut statements = Vec::new();
        while !matches!(self.peek(), TokenKind::EndFunction) {
            statements.push(self.parse_statement()?);
        }
        
        self.expect(TokenKind::EndFunction)?;
        
        Ok(Function { name, return_type, var_sections, statements })
    }
    
    fn parse_data_block(&mut self) -> Result<DataBlock, ParseError> { 
        self.expect(TokenKind::DataBlock)?;
        let name = self.expect_identifier()?;
        
        // Skip TIA Portal metadata and attributes
        self.skip_tia_metadata();
        
        let mut var_sections = Vec::new();
        while matches!(self.peek(), TokenKind::VarInput | TokenKind::VarOutput | TokenKind::VarInOut | TokenKind::VarTemp | TokenKind::Var | TokenKind::VarAccess | TokenKind::VarExternal) {
            var_sections.push(self.parse_var_section()?);
        }
        
        // BEGIN is optional for TIA Portal compatibility
        if matches!(self.peek(), TokenKind::Begin) {
            self.advance();
        }
        
        let mut statements = Vec::new();
        while !matches!(self.peek(), TokenKind::EndDataBlock) {
            statements.push(self.parse_statement()?);
        }
        
        self.expect(TokenKind::EndDataBlock)?;
        
        Ok(DataBlock { name, var_sections, statements })
    }
    
    fn parse_type_decl(&mut self) -> Result<TypeDecl, ParseError> { 
        self.expect(TokenKind::Type)?;
        
        let mut declarations = Vec::new();
        
        // Parse multiple type declarations until END_TYPE
        while !matches!(self.peek(), TokenKind::EndType) {
            let name = self.expect_identifier()?;
            self.expect(TokenKind::Operator(":".to_string()))?;
            let type_ref = self.parse_type_ref()?;
            
            // Optional initial value
            let initial_value = if matches!(self.peek(), TokenKind::Operator(op) if op == ":=") {
                self.advance(); // consume :=
                Some(self.parse_expression()?)
            } else {
                None
            };
            
            // Optional semicolon (be flexible)
            if matches!(self.peek(), TokenKind::Operator(op) if op == ";") {
                self.advance();
            }
            
            declarations.push((name, type_ref, initial_value));
            
            // Skip comments and whitespace
            if matches!(self.peek(), TokenKind::Eof | TokenKind::EndType) {
                break;
            }
        }
        
        self.expect(TokenKind::EndType)?;
        
        // For now, return first declaration (maintain compatibility)
        // TODO: Update TypeDecl to support multiple declarations
        if let Some((name, type_ref, _initial_value)) = declarations.first() {
            Ok(TypeDecl { name: name.clone(), type_spec: TypeSpec { type_ref: type_ref.clone() } })
        } else {
            Err(ParseError::new(
                ParseErrorKind::UnexpectedToken { 
                    expected: "type declaration".to_string(), 
                    found: "END_TYPE".to_string() 
                },
                (self.pos, self.pos)
            ))
        }
    }
    
    fn parse_var_declaration(&mut self) -> Result<VarDeclaration, ParseError> { 
        let mut names = vec![self.expect_identifier()?];
        self.skip_tia_attributes();
        
        while matches!(self.peek(), TokenKind::Operator(op) if op == ",") {
            self.advance();
            names.push(self.expect_identifier()?);
            self.skip_tia_attributes();
        }
        
        // Check for AT clause before colon: name AT %addr : type
        let at_address_before = if matches!(self.peek(), TokenKind::At) {
            self.advance();
            Some(self.expect_absolute_address()?)
        } else {
            None
        };
        
        self.expect(TokenKind::Operator(":".to_string()))?;
        let type_ref = self.parse_type_ref()?;
        
        // Parse optional AT clause after type: name : type AT %addr
        let at_address_after = if at_address_before.is_none() && matches!(self.peek(), TokenKind::At) {
            self.advance();
            Some(self.expect_absolute_address()?)
        } else {
            None
        };
        
        let at_address = at_address_before.or(at_address_after);
        
        let initializer = if matches!(self.peek(), TokenKind::Operator(op) if op == ":=") {
            self.advance();
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        self.expect(TokenKind::Operator(";".to_string()))?;
        
        Ok(VarDeclaration { names, type_ref, at_address, initializer })
    }
    
    fn expect_absolute_address(&mut self) -> Result<String, ParseError> {
        // AT addresses are: % followed by location like IW215, QB7, M*, etc.
        // The lexer splits this into % (operator) and IW215 (identifier/literal)
        
        self.expect(TokenKind::Operator("%".to_string()))?;
        
        // Next token could be identifier (like IW215, M*) or int literal (like 1, 42)
        let addr_part = match self.peek() {
            TokenKind::Identifier(id) => {
                let result = id.clone();
                self.advance();
                result
            }
            TokenKind::IntLit(num) => {
                let result = num.clone();
                self.advance();
                result
            }
            _ => return Err(self.make_error(
                ParseErrorKind::UnexpectedToken {
                    expected: "address location (e.g., IW215, QB7, I1, M*)".to_string(),
                    found: format!("{:?}", self.peek()),
                },
                self.current().span
            ))
        };
        
        // Handle additional parts like .5 or .7.9 in %QX7.5 or %MW1.7.9
        let mut full_address = format!("%{}", addr_part);
        while matches!(self.peek(), TokenKind::Operator(op) if op == ".") {
            self.advance();
            full_address.push('.');
            match self.peek() {
                TokenKind::IntLit(num) => {
                    full_address.push_str(&num);
                    self.advance();
                }
                TokenKind::Identifier(id) => {
                    full_address.push_str(&id);
                    self.advance();
                }
                _ => break,
            }
        }
        
        Ok(full_address)
    }
    
    fn parse_type_ref(&mut self) -> Result<TypeRef, ParseError> {
        match self.peek() {
            TokenKind::Array => {
                self.advance();
                self.expect(TokenKind::Operator("[".to_string()))?;
                let start = self.parse_expression()?;
                self.expect(TokenKind::Operator("..".to_string()))?;
                let end = self.parse_expression()?;
                self.expect(TokenKind::Operator("]".to_string()))?;
                self.expect(TokenKind::Of)?;
                let element_type = self.parse_type_ref()?;
                Ok(TypeRef::Array(Box::new(ArrayType { start, end, element_type })))
            }
            TokenKind::Struct => {
                self.advance();
                let mut fields = Vec::new();
                while !matches!(self.peek(), TokenKind::EndStruct) {
                    let name = self.expect_identifier()?;
                    self.expect(TokenKind::Operator(":".to_string()))?;
                    let type_ref = self.parse_type_ref()?;
                    self.expect(TokenKind::Operator(";".to_string()))?;
                    fields.push(StructField { name, type_ref });
                }
                self.expect(TokenKind::EndStruct)?;
                Ok(TypeRef::Struct(StructType { fields }))
            }
            TokenKind::Pointer => {
                self.advance();
                self.expect(TokenKind::To)?;
                let target_type = self.parse_type_ref()?;
                Ok(TypeRef::Pointer(Box::new(PointerType { target_type })))
            }
            TokenKind::Variant => {
                self.advance();
                Ok(TypeRef::Variant)
            }
            TokenKind::Any => {
                self.advance();
                Ok(TypeRef::Any)
            }
            TokenKind::AnyNum => {
                self.advance();
                Ok(TypeRef::Named("ANY_NUM".to_string()))
            }
            TokenKind::AnyReal => {
                self.advance();
                Ok(TypeRef::Named("ANY_REAL".to_string()))
            }
            TokenKind::AnyInt => {
                self.advance();
                Ok(TypeRef::Named("ANY_INT".to_string()))
            }
            TokenKind::AnyBit => {
                self.advance();
                Ok(TypeRef::Named("ANY_BIT".to_string()))
            }
            TokenKind::AnyDate => {
                self.advance();
                Ok(TypeRef::Named("ANY_DATE".to_string()))
            }
            TokenKind::AnyString => {
                self.advance();
                Ok(TypeRef::Named("ANY_STRING".to_string()))
            }
            TokenKind::Identifier(name) => {
                let mut name = name.clone();
                self.advance();
                
                // Handle sized types like String[23] or WString[100]
                if matches!(self.peek(), TokenKind::Operator(op) if op == "[") {
                    self.advance();
                    // Parse the size (could be integer or expression)
                    let _size = self.parse_expression()?;
                    self.expect(TokenKind::Operator("]".to_string()))?;
                    name = format!("{}[...]", name); // Simplified representation
                }
                
                // Handle subrange types like INT (-4095 .. 4095)
                if matches!(self.peek(), TokenKind::Operator(op) if op == "(") {
                    self.advance(); // consume (
                    let _min = self.parse_expression()?;
                    self.expect(TokenKind::Operator("..".to_string()))?;
                    let _max = self.parse_expression()?;
                    self.expect(TokenKind::Operator(")".to_string()))?;
                    // Store as subrange type (simplified for now)
                    name = format!("{}(subrange)", name);
                }
                
                Ok(TypeRef::Named(name))
            }
            kind => Err(self.make_error(
                ParseErrorKind::UnexpectedToken { 
                    expected: "type".to_string(), 
                    found: format!("{:?}", kind) 
                },
                self.current().span,
            ))
        }
    }
    
    fn parse_if_stmt(&mut self) -> Result<IfStmt, ParseError> { 
        self.expect(TokenKind::If)?;
        let condition = self.parse_expression()?;
        self.expect(TokenKind::Then)?;
        
        let mut then_body = Vec::new();
        while !matches!(self.peek(), TokenKind::Elsif | TokenKind::Else | TokenKind::EndIf) {
            then_body.push(self.parse_statement()?);
        }
        
        let mut elsif_parts = Vec::new();
        while matches!(self.peek(), TokenKind::Elsif) {
            self.advance();
            let condition = self.parse_expression()?;
            self.expect(TokenKind::Then)?;
            let mut body = Vec::new();
            while !matches!(self.peek(), TokenKind::Elsif | TokenKind::Else | TokenKind::EndIf) {
                body.push(self.parse_statement()?);
            }
            elsif_parts.push(ElsifPart { condition, body });
        }
        
        let else_part = if matches!(self.peek(), TokenKind::Else) {
            self.advance();
            let mut body = Vec::new();
            while !matches!(self.peek(), TokenKind::EndIf) {
                body.push(self.parse_statement()?);
            }
            Some(ElsePart { body })
        } else {
            None
        };
        
        self.expect(TokenKind::EndIf)?;
        self.expect(TokenKind::Operator(";".to_string()))?;
        
        Ok(IfStmt { condition, then_body, elsif_parts, else_part })
    }
    
    fn parse_case_stmt(&mut self) -> Result<CaseStmt, ParseError> { 
        self.expect(TokenKind::Case)?;
        let expression = self.parse_expression()?;
        self.expect(TokenKind::Of)?;
        
        let mut elements = Vec::new();
        while !matches!(self.peek(), TokenKind::Else | TokenKind::EndCase) {
            let mut values = vec![self.parse_expression()?];
            while matches!(self.peek(), TokenKind::Operator(op) if op == ",") {
                self.advance();
                values.push(self.parse_expression()?);
            }
            self.expect(TokenKind::Operator(":".to_string()))?;
            
            // Parse statements until we hit the next case element or ELSE/END_CASE
            // A new case element starts with Integer/Identifier followed by : or ,
            let mut body = Vec::new();
            let mut iterations = 0;
            loop {
                iterations += 1;
                if iterations > self.max_iterations {
                    return Err(self.make_error(ParseErrorKind::TooManyIterations, self.current().span));
                }
                
                // Stop if we see ELSE or END_CASE
                if matches!(self.peek(), TokenKind::Else | TokenKind::EndCase) {
                    break;
                }
                
                // Stop if we see the start of next case element (number/id followed by : or ,)
                if matches!(self.peek(), TokenKind::IntLit(_) | TokenKind::Identifier(_)) {
                    // Look ahead to see if this is a case label (followed by : or ,)
                    let saved_pos = self.pos;
                    self.advance();
                    let is_case_label = matches!(self.peek(), TokenKind::Operator(op) if op == ":" || op == ",");
                    self.pos = saved_pos;
                    
                    if is_case_label {
                        break;
                    }
                }
                
                body.push(self.parse_statement()?);
            }
            elements.push(CaseElement { values, body });
        }
        
        let else_part = if matches!(self.peek(), TokenKind::Else) {
            self.advance();
            let mut body = Vec::new();
            while !matches!(self.peek(), TokenKind::EndCase) {
                body.push(self.parse_statement()?);
            }
            Some(ElsePart { body })
        } else {
            None
        };
        
        self.expect(TokenKind::EndCase)?;
        self.expect(TokenKind::Operator(";".to_string()))?;
        
        Ok(CaseStmt { expression, elements, else_part })
    }
    
    fn parse_for_stmt(&mut self) -> Result<ForStmt, ParseError> { 
        self.expect(TokenKind::For)?;
        let variable = self.expect_identifier()?;
        self.expect(TokenKind::Operator(":=".to_string()))?;
        let start = self.parse_expression()?;
        self.expect(TokenKind::To)?;
        let end = self.parse_expression()?;
        
        let by = if matches!(self.peek(), TokenKind::By) {
            self.advance();
            Some(ByPart { value: self.parse_expression()? })
        } else {
            None
        };
        
        self.expect(TokenKind::Do)?;
        
        let mut body = Vec::new();
        while !matches!(self.peek(), TokenKind::EndFor) {
            body.push(self.parse_statement()?);
        }
        
        self.expect(TokenKind::EndFor)?;
        self.expect(TokenKind::Operator(";".to_string()))?;
        
        Ok(ForStmt { variable, start, end, by, body })
    }
    
    fn parse_while_stmt(&mut self) -> Result<WhileStmt, ParseError> { 
        self.expect(TokenKind::While)?;
        let condition = self.parse_expression()?;
        self.expect(TokenKind::Do)?;
        
        let mut body = Vec::new();
        while !matches!(self.peek(), TokenKind::EndWhile) {
            body.push(self.parse_statement()?);
        }
        
        self.expect(TokenKind::EndWhile)?;
        self.expect(TokenKind::Operator(";".to_string()))?;
        
        Ok(WhileStmt { condition, body })
    }
    
    fn parse_repeat_stmt(&mut self) -> Result<RepeatStmt, ParseError> { 
        self.expect(TokenKind::Repeat)?;
        
        let mut body = Vec::new();
        while !matches!(self.peek(), TokenKind::Until) {
            body.push(self.parse_statement()?);
        }
        
        self.expect(TokenKind::Until)?;
        let condition = self.parse_expression()?;
        self.expect(TokenKind::EndRepeat)?;
        self.expect(TokenKind::Operator(";".to_string()))?;
        
        Ok(RepeatStmt { body, condition })
    }
    
    fn parse_return_stmt(&mut self) -> Result<ReturnStmt, ParseError> { 
        self.expect(TokenKind::Return)?;
        self.expect(TokenKind::Operator(";".to_string()))?;
        Ok(ReturnStmt {})
    }
    
    fn parse_assignment(&mut self) -> Result<Assignment, ParseError> { 
        // Parse left-hand side (can be identifier or absolute_address)
        let mut target = if matches!(self.peek(), TokenKind::Operator(op) if op == "%") {
            // Absolute address like %MW504
            self.expect_absolute_address()?
        } else {
            self.expect_identifier()?
        };
        
        // Handle member access and array indexing (only for identifiers)
        if !target.starts_with('%') {
            while matches!(self.peek(), TokenKind::Operator(op) if op == "." || op == "[") {
            match self.peek() {
                TokenKind::Operator(op) if op == "." => {
                    self.advance();
                    let field = self.expect_identifier()?;
                    target = format!("{}.{}", target, field);
                }
                TokenKind::Operator(op) if op == "[" => {
                    self.advance();
                    // Skip the array index expression (we'll parse it properly later)
                    let mut depth = 1;
                    while depth > 0 && !matches!(self.peek(), TokenKind::Eof) {
                        match self.peek() {
                            TokenKind::Operator(op) if op == "[" => depth += 1,
                            TokenKind::Operator(op) if op == "]" => depth -= 1,
                            _ => {}
                        }
                        if depth > 0 {
                            self.advance();
                        }
                    }
                    self.expect(TokenKind::Operator("]".to_string()))?;
                    target = format!("{}[...]", target); // Placeholder
                }
                _ => break,
            }
            }
        }
        
        self.expect(TokenKind::Operator(":=".to_string()))?;
        let value = self.parse_expression()?;
        self.expect(TokenKind::Operator(";".to_string()))?;
        Ok(Assignment { target, value })
    }
    
    fn parse_nullable_assignment(&mut self) -> Result<NullableAssignment, ParseError> { 
        let target = self.expect_identifier()?;
        self.expect(TokenKind::Operator("?=".to_string()))?;
        let value = self.parse_expression()?;
        self.expect(TokenKind::Operator(";".to_string()))?;
        Ok(NullableAssignment { target, value })
    }
    
    fn parse_function_call_stmt(&mut self) -> Result<FunctionCallStmt, ParseError> { 
        let call = self.parse_function_call()?;
        self.expect(TokenKind::Operator(";".to_string()))?;
        Ok(FunctionCallStmt { call })
    }
    
    fn parse_function_call(&mut self) -> Result<FunctionCall, ParseError> {
        let name = self.expect_identifier()?;
        self.expect(TokenKind::Operator("(".to_string()))?;
        
        let mut arguments = Vec::new();
        if !matches!(self.peek(), TokenKind::Operator(op) if op == ")") {
            loop {
                // Try named argument
                let checkpoint = self.pos;
                let maybe_name = if matches!(self.peek(), TokenKind::Identifier(_)) {
                    let name = self.expect_identifier()?;
                    if matches!(self.peek(), TokenKind::Operator(op) if op == ":=") {
                        self.advance();
                        Some(name)
                    } else {
                        self.pos = checkpoint;
                        None
                    }
                } else {
                    None
                };
                
                let value = self.parse_expression()?;
                arguments.push(Argument { name: maybe_name, value });
                
                if matches!(self.peek(), TokenKind::Operator(op) if op == ",") {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        
        self.expect(TokenKind::Operator(")".to_string()))?;
        Ok(FunctionCall { name, arguments })
    }
    
    fn parse_expression(&mut self) -> Result<Expression, ParseError> { 
        self.parse_or_expr()
    }
    
    fn parse_or_expr(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_xor_expr()?;
        
        while matches!(self.peek(), TokenKind::Or) {
            self.advance();
            let right = self.parse_xor_expr()?;
            left = Expression::Or(OrExpr {
                left: Box::new(left),
                right: Box::new(right),
            });
        }
        
        Ok(left)
    }
    
    fn parse_xor_expr(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_and_expr()?;
        
        while matches!(self.peek(), TokenKind::Xor) {
            self.advance();
            let right = self.parse_and_expr()?;
            left = Expression::Xor(XorExpr {
                left: Box::new(left),
                right: Box::new(right),
            });
        }
        
        Ok(left)
    }
    
    fn parse_and_expr(&mut self) -> Result<Expression, ParseError> {
        self.check_recursion()?;
        let mut left = self.parse_comparison()?;
        
        let mut iterations = 0;
        loop {
            iterations += 1;
            if iterations > self.max_iterations {
                return Err(self.make_error(ParseErrorKind::TooManyIterations, self.current().span));
            }
            let is_and = matches!(self.peek(), TokenKind::And);
            let is_ampersand = matches!(self.peek(), TokenKind::Operator(op) if op == "&");
            
            if !is_and && !is_ampersand {
                break;
            }
            
            self.advance();
            let right = self.parse_comparison()?;
            left = Expression::And(AndExpr {
                left: Box::new(left),
                right: Box::new(right),
            });
        }
        
        self.uncheck_recursion();
        Ok(left)
    }
    
    fn parse_comparison(&mut self) -> Result<Expression, ParseError> {
        self.check_recursion()?;
        let left = self.parse_add_expr()?;
        
        let op = match self.peek() {
            TokenKind::Operator(s) if s == "=" => {
                self.advance();
                ComparisonOp::Eq
            }
            TokenKind::Operator(s) if s == "<>" => {
                self.advance();
                ComparisonOp::Ne
            }
            TokenKind::Operator(s) if s == "<" => {
                self.advance();
                ComparisonOp::Lt
            }
            TokenKind::Operator(s) if s == ">" => {
                self.advance();
                ComparisonOp::Gt
            }
            TokenKind::Operator(s) if s == "<=" => {
                self.advance();
                ComparisonOp::Le
            }
            TokenKind::Operator(s) if s == ">=" => {
                self.advance();
                ComparisonOp::Ge
            }
            _ => return Ok(left),
        };
        
        let right = self.parse_add_expr()?;
        Ok(Expression::Comparison(Comparison {
            left: Box::new(left),
            op,
            right: Box::new(right),
        }))
    }
    
    fn parse_add_expr(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_mult_expr()?;
        
        while matches!(self.peek(), TokenKind::Operator(op) if op == "+" || op == "-") {
            let op = if matches!(self.peek(), TokenKind::Operator(op) if op == "+") {
                self.advance();
                AddOp::Add
            } else {
                self.advance();
                AddOp::Sub
            };
            
            let right = self.parse_mult_expr()?;
            left = Expression::Add(AddExpr {
                left: Box::new(left),
                op,
                right: Box::new(right),
            });
        }
        
        Ok(left)
    }
    
    fn parse_mult_expr(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_unary_expr()?;
        
        while matches!(self.peek(), TokenKind::Operator(op) if op == "*" || op == "/") || matches!(self.peek(), TokenKind::Mod) {
            let op = match self.peek() {
                TokenKind::Operator(s) if s == "*" => {
                    self.advance();
                    MultOp::Mult
                }
                TokenKind::Operator(s) if s == "/" => {
                    self.advance();
                    MultOp::Div
                }
                TokenKind::Mod => {
                    self.advance();
                    MultOp::Mod
                }
                _ => unreachable!(),
            };
            
            let right = self.parse_unary_expr()?;
            left = Expression::Mult(MultExpr {
                left: Box::new(left),
                op,
                right: Box::new(right),
            });
        }
        
        Ok(left)
    }
    
    fn parse_unary_expr(&mut self) -> Result<Expression, ParseError> {
        let op = match self.peek() {
            TokenKind::Operator(s) if s == "+" => {
                self.advance();
                Some(UnaryOp::Plus)
            }
            TokenKind::Operator(s) if s == "-" => {
                self.advance();
                Some(UnaryOp::Minus)
            }
            TokenKind::Not => {
                self.advance();
                Some(UnaryOp::Not)
            }
            _ => None,
        };
        
        let primary = self.parse_primary()?;
        Ok(Expression::Unary(UnaryExpr {
            op,
            operand: Box::new(primary),
        }))
    }
    
    fn parse_primary(&mut self) -> Result<Primary, ParseError> {
        match self.peek() {
            TokenKind::Identifier(name) => {
                let mut name = name.clone();
                self.advance();
                
                // Handle member access (.field) and array indexing ([index])
                while matches!(self.peek(), TokenKind::Operator(op) if op == "." || op == "[") {
                    match self.peek() {
                        TokenKind::Operator(op) if op == "." => {
                            self.advance();
                            let field = self.expect_identifier()?;
                            name = format!("{}.{}", name, field);
                        }
                        TokenKind::Operator(op) if op == "[" => {
                            self.advance();
                            let mut depth = 1;
                            while depth > 0 && !matches!(self.peek(), TokenKind::Eof) {
                                match self.peek() {
                                    TokenKind::Operator(op) if op == "[" => depth += 1,
                                    TokenKind::Operator(op) if op == "]" => depth -= 1,
                                    _ => {}
                                }
                                if depth > 0 {
                                    self.advance();
                                }
                            }
                            self.expect(TokenKind::Operator("]".to_string()))?;
                            name = format!("{}[...]", name);
                        }
                        _ => break,
                    }
                }
                
                // Check if it's a function call
                if matches!(self.peek(), TokenKind::Operator(op) if op == "(") {
                    // For function calls, we need the base name without member access
                    // This is a simplification - proper handling would parse the call with the full path
                    self.pos -= 1;  // Backtrack
                    Ok(Primary::FunctionCall(self.parse_function_call()?))
                } else {
                    Ok(Primary::Identifier(name))
                }
            }
            TokenKind::IntLit(val) => {
                let val = val.clone();
                self.advance();
                Ok(Primary::Literal(Literal::IntLit(val)))
            }
            TokenKind::FloatLit(val) => {
                let val = val.clone();
                self.advance();
                Ok(Primary::Literal(Literal::FloatLit(val)))
            }
            TokenKind::StringLit(val) => {
                let val = val.clone();
                self.advance();
                Ok(Primary::Literal(Literal::String(val)))
            }
            TokenKind::True => {
                self.advance();
                Ok(Primary::Literal(Literal::Boolean(true)))
            }
            TokenKind::False => {
                self.advance();
                Ok(Primary::Literal(Literal::Boolean(false)))
            }
            TokenKind::Operator(op) if op == "(" => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(TokenKind::Operator(")".to_string()))?;
                Ok(Primary::Parenthesized(Box::new(expr)))
            }
            kind => Err(self.make_error(
                ParseErrorKind::UnexpectedToken { 
                    expected: "primary expression".to_string(), 
                    found: format!("{:?}", kind) 
                },
                self.current().span,
            ))
        }
    }
    
    fn parse_organization_block(&mut self) -> Result<OrganizationBlock, ParseError> {
        self.check_recursion()?;
        self.expect(TokenKind::OrganizationBlock)?;
        let name = self.expect_identifier()?;
        
        let mut var_sections = Vec::new();
        while matches!(self.peek(), TokenKind::VarInput | TokenKind::VarOutput | TokenKind::VarInOut | TokenKind::VarTemp | TokenKind::Var | TokenKind::VarAccess | TokenKind::VarExternal) {
            var_sections.push(self.parse_var_section()?);
        }
        
        // BEGIN is optional for TIA Portal compatibility
        if matches!(self.peek(), TokenKind::Begin) {
            self.advance();
        }
        
        let mut statements = Vec::new();
        let mut iterations = 0;
        while !matches!(self.peek(), TokenKind::EndOrganizationBlock) {
            iterations += 1;
            if iterations > self.max_iterations {
                return Err(self.make_error(ParseErrorKind::TooManyStatements, self.current().span));
            }
            statements.push(self.parse_statement()?);
        }
        
        self.expect(TokenKind::EndOrganizationBlock)?;
        self.uncheck_recursion();
        Ok(OrganizationBlock { name, var_sections, statements })
    }
    
    fn parse_program_block(&mut self) -> Result<ProgramBlock, ParseError> {
        self.check_recursion()?;
        self.expect(TokenKind::Program)?;
        let name = self.expect_identifier()?;
        
        // Skip TIA Portal metadata and attributes
        self.skip_tia_metadata();
        
        let mut var_sections = Vec::new();
        while matches!(self.peek(), TokenKind::VarInput | TokenKind::VarOutput | TokenKind::VarInOut | TokenKind::VarTemp | TokenKind::Var | TokenKind::VarAccess | TokenKind::VarExternal) {
            var_sections.push(self.parse_var_section()?);
        }
        
        // BEGIN is optional for TIA Portal compatibility
        if matches!(self.peek(), TokenKind::Begin) {
            self.advance();
        }
        
        let mut statements = Vec::new();
        let mut iterations = 0;
        while !matches!(self.peek(), TokenKind::EndProgram) {
            iterations += 1;
            if iterations > self.max_iterations {
                return Err(self.make_error(ParseErrorKind::TooManyStatements, self.current().span));
            }
            statements.push(self.parse_statement()?);
        }
        
        self.expect(TokenKind::EndProgram)?;
        self.uncheck_recursion();
        Ok(ProgramBlock { name, var_sections, statements })
    }
    
    fn parse_class_decl(&mut self) -> Result<ClassDecl, ParseError> {
        self.check_recursion()?;
        self.expect(TokenKind::Class)?;
        let name = self.expect_identifier()?;
        
        let extends = if matches!(self.peek(), TokenKind::Extends) {
            self.advance();
            Some(self.expect_identifier()?)
        } else {
            None
        };
        
        let mut implements = Vec::new();
        if matches!(self.peek(), TokenKind::Implements) {
            self.advance();
            implements.push(self.expect_identifier()?);
            while matches!(self.peek(), TokenKind::Operator(op) if op == ",") {
                self.advance();
                implements.push(self.expect_identifier()?);
            }
        }
        
        let mut var_sections = Vec::new();
        let mut methods = Vec::new();
        
        loop {
            match self.peek() {
                TokenKind::VarInput | TokenKind::VarOutput | TokenKind::VarInOut | TokenKind::VarTemp | TokenKind::Var => {
                    var_sections.push(self.parse_var_section()?);
                }
                TokenKind::Method => {
                    methods.push(self.parse_method_decl()?);
                }
                _ => break,
            }
        }
        
        self.expect(TokenKind::EndClass)?;
        self.uncheck_recursion();
        Ok(ClassDecl { name, extends, implements, var_sections, methods })
    }
    
    fn parse_interface_decl(&mut self) -> Result<InterfaceDecl, ParseError> {
        self.check_recursion()?;
        self.expect(TokenKind::Interface)?;
        let name = self.expect_identifier()?;
        
        let extends = if matches!(self.peek(), TokenKind::Extends) {
            self.advance();
            Some(self.expect_identifier()?)
        } else {
            None
        };
        
        let mut methods = Vec::new();
        while matches!(self.peek(), TokenKind::Method) {
            methods.push(self.parse_method_signature()?);
        }
        
        self.expect(TokenKind::EndInterface)?;
        self.uncheck_recursion();
        Ok(InterfaceDecl { name, extends, methods })
    }
    
    fn parse_method_decl(&mut self) -> Result<MethodDecl, ParseError> {
        self.check_recursion()?;
        self.expect(TokenKind::Method)?;
        
        let access = self.parse_access_modifier();
        let name = self.expect_identifier()?;
        self.expect(TokenKind::Operator(":".to_string()))?;
        let return_type = self.parse_type_ref()?;
        
        let mut var_sections = Vec::new();
        while matches!(self.peek(), TokenKind::VarInput | TokenKind::VarOutput | TokenKind::VarInOut | TokenKind::VarTemp | TokenKind::Var | TokenKind::VarAccess | TokenKind::VarExternal) {
            var_sections.push(self.parse_var_section()?);
        }
        
        // BEGIN is optional for TIA Portal compatibility
        if matches!(self.peek(), TokenKind::Begin) {
            self.advance();
        }
        
        let mut statements = Vec::new();
        let mut iterations = 0;
        while !matches!(self.peek(), TokenKind::EndMethod) {
            iterations += 1;
            if iterations > self.max_iterations {
                return Err(self.make_error(ParseErrorKind::TooManyStatements, self.current().span));
            }
            statements.push(self.parse_statement()?);
        }
        
        self.expect(TokenKind::EndMethod)?;
        self.uncheck_recursion();
        Ok(MethodDecl { access, name, return_type, var_sections, statements })
    }
    
    fn parse_method_signature(&mut self) -> Result<MethodSignature, ParseError> {
        self.check_recursion()?;
        self.expect(TokenKind::Method)?;
        let name = self.expect_identifier()?;
        self.expect(TokenKind::Operator(":".to_string()))?;
        let return_type = self.parse_type_ref()?;
        
        let mut var_sections = Vec::new();
        while matches!(self.peek(), TokenKind::VarInput | TokenKind::VarOutput | TokenKind::VarInOut | TokenKind::VarTemp | TokenKind::Var | TokenKind::VarAccess | TokenKind::VarExternal) {
            var_sections.push(self.parse_var_section()?);
        }
        
        self.expect(TokenKind::Operator(";".to_string()))?;
        self.uncheck_recursion();
        Ok(MethodSignature { name, return_type, var_sections })
    }
    
    fn parse_access_modifier(&mut self) -> Option<AccessModifier> {
        match self.peek() {
            TokenKind::Public => { self.advance(); Some(AccessModifier::Public) }
            TokenKind::Private => { self.advance(); Some(AccessModifier::Private) }
            TokenKind::Protected => { self.advance(); Some(AccessModifier::Protected) }
            TokenKind::Internal => { self.advance(); Some(AccessModifier::Internal) }
            _ => None
        }
    }
    
    fn parse_region(&mut self) -> Result<Region, ParseError> {
        self.check_recursion()?;
        self.expect(TokenKind::Region)?;
        
        // REGION can have multiple identifiers as name (e.g., "REGION delta time")
        // Consume identifiers until we hit something that clearly isn't part of the region name:
        // - Identifiers starting with # or % (local temps, absolute addresses)
        // - Statement keywords (IF, FOR, WHILE, etc.)
        // - Operators that indicate a statement (like :=, (, etc.)
        let mut name_parts = vec![self.expect_identifier()?];
        
        while matches!(self.peek(), TokenKind::Identifier(id) if !id.starts_with('#') && !id.starts_with('%')) {
            // Lookahead: check if the next identifier starts a statement
            let checkpoint = self.pos;
            let id = self.expect_identifier()?;
            
            // If followed by :=, (, or ?=, this identifier starts a statement
            if matches!(self.peek(), TokenKind::Operator(op) if op == ":=" || op == "(" || op == "?=") {
                self.pos = checkpoint;
                break;
            }
            
            name_parts.push(id);
        }
        
        let name = name_parts.join(" ");
        
        let mut statements = Vec::new();
        let mut iterations = 0;
        while !matches!(self.peek(), TokenKind::EndRegion) {
            iterations += 1;
            if iterations > self.max_iterations {
                return Err(self.make_error(ParseErrorKind::TooManyStatements, self.current().span));
            }
            statements.push(self.parse_statement()?);
        }
        
        self.expect(TokenKind::EndRegion)?;
        self.uncheck_recursion();
        Ok(Region { name, statements })
    }
    

    // Skip TIA Portal metadata lines (generated from tia_extensions.ebnf)
    fn skip_tia_metadata(&mut self) {
        loop {
            match self.peek() {
                // Skip AUTHOR : value
                TokenKind::Identifier(name) if name == "AUTHOR" => {
                    self.advance();
                    if matches!(self.peek(), TokenKind::Operator(op) if op == ":") {
                        self.advance();
                        self.advance(); // Skip the value
                    }
                }
                // Skip FAMILY : value
                TokenKind::Identifier(name) if name == "FAMILY" => {
                    self.advance();
                    if matches!(self.peek(), TokenKind::Operator(op) if op == ":") {
                        self.advance();
                        self.advance(); // Skip the value
                    }
                }
                // Skip TITLE = value
                TokenKind::Identifier(name) if name == "TITLE" => {
                    self.advance();
                    if matches!(self.peek(), TokenKind::Operator(op) if op == "=") {
                        self.advance();
                        self.advance(); // Skip the value
                    }
                }
                // Skip VERSION : number
                TokenKind::Identifier(name) if name == "VERSION" => {
                    self.advance();
                    if matches!(self.peek(), TokenKind::Operator(op) if op == ":") {
                        self.advance();
                        self.advance(); // Skip the value
                    }
                }
                // Skip { attributes }
                TokenKind::Operator(op) if op == "{" => {
                    self.advance();
                    let mut depth = 1;
                    while depth > 0 && !matches!(self.peek(), TokenKind::Eof) {
                        match self.peek() {
                            TokenKind::Operator(op) if op == "{" => depth += 1,
                            TokenKind::Operator(op) if op == "}" => depth -= 1,
                            _ => {}
                        }
                        self.advance();
                    }
                }
                _ => break,
            }
        }
    }
    
    // Skip TIA Portal attributes in variable declarations (generated from tia_extensions.ebnf)
    fn skip_tia_attributes(&mut self) {
        if matches!(self.peek(), TokenKind::Operator(op) if op == "{") {
            self.advance();
            let mut depth = 1;
            while depth > 0 && !matches!(self.peek(), TokenKind::Eof) {
                match self.peek() {
                    TokenKind::Operator(op) if op == "{" => depth += 1,
                    TokenKind::Operator(op) if op == "}" => depth -= 1,
                    _ => {}
                }
                self.advance();
            }
        }
    }
}
