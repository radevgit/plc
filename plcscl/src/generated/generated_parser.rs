// Generated SCL parser

use super::lexer::{Lexer, Token, TokenKind};
use super::ast::*;

/// Parser security limits to prevent DoS attacks
#[derive(Debug, Clone)]
pub struct ParserLimits {
    pub max_tokens: usize,           // Maximum tokens in input
    pub max_iterations: usize,       // Maximum loop iterations
    pub max_recursion_depth: usize,  // Maximum call stack depth
    pub max_collection_size: usize,  // Maximum items in collections
    pub max_nesting_depth: usize,    // Maximum block/expression nesting
    pub max_nodes: usize,            // Maximum total AST nodes
}

impl Default for ParserLimits {
    fn default() -> Self {
        ParserLimits {
            max_tokens: 1_000_000,        // 1M tokens (~10-50MB source)
            max_iterations: 100_000,      // 100K iterations per loop
            max_recursion_depth: 256,     // 256 levels deep
            max_collection_size: 100_000, // 100K items per collection
            max_nesting_depth: 100,       // 100 levels of nesting
            max_nodes: 10_000_000,        // 10M AST nodes total
        }
    }
}

impl ParserLimits {
    /// Conservative limits for untrusted input
    pub fn strict() -> Self {
        ParserLimits {
            max_tokens: 100_000,
            max_iterations: 10_000,
            max_recursion_depth: 64,
            max_collection_size: 10_000,
            max_nesting_depth: 32,
            max_nodes: 1_000_000,
        }
    }
    
    /// Relaxed limits for trusted input
    pub fn relaxed() -> Self {
        ParserLimits {
            max_tokens: 10_000_000,
            max_iterations: 1_000_000,
            max_recursion_depth: 512,
            max_collection_size: 1_000_000,
            max_nesting_depth: 256,
            max_nodes: 100_000_000,
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

#[derive(Debug, Clone)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub span: (usize, usize),
}

impl ParseError {
    pub fn new(kind: ParseErrorKind, span: (usize, usize)) -> Self {
        ParseError { kind, span }
    }
    
    pub fn message(&self) -> String {
        match &self.kind {
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
        }
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
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
            return Err(ParseError::new(
                ParseErrorKind::TooManyNodes,
                self.current().span,
            ));
        }
        Ok(())
    }
    
    fn check_collection_size(&self, size: usize, collection_name: &str) -> Result<(), ParseError> {
        if size >= self.max_collection_size {
            return Err(ParseError::new(
                ParseErrorKind::CollectionSizeExceeded { collection: collection_name.to_string() },
                self.current().span,
            ));
        }
        Ok(())
    }
    
    fn check_recursion(&mut self) -> Result<(), ParseError> {
        self.recursion_depth += 1;
        if self.recursion_depth > self.max_recursion_depth {
            return Err(ParseError::new(
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
            return Err(ParseError::new(
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
            _ => Err(ParseError::new(
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
                return Err(ParseError::new(
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
            kind => Err(ParseError::new(
                ParseErrorKind::UnexpectedToken { 
                    expected: "block declaration (FUNCTION_BLOCK/FUNCTION/DATA_BLOCK/TYPE)".to_string(), 
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
        
        let mut var_sections = Vec::new();
        while matches!(self.peek(), TokenKind::VarInput | TokenKind::VarOutput | TokenKind::VarInOut | TokenKind::VarTemp | TokenKind::Var) {
            var_sections.push(self.parse_var_section()?);
        }
        
        self.expect(TokenKind::Begin)?;
        
        let mut statements = Vec::new();
        let mut iterations = 0;
        while !matches!(self.peek(), TokenKind::EndFunctionBlock) {
            iterations += 1;
            if iterations > self.max_iterations {
                return Err(ParseError::new(ParseErrorKind::TooManyStatements, self.current().span));
            }
            statements.push(self.parse_statement()?);
        }
        
        self.expect(TokenKind::EndFunctionBlock)?;
        
        self.uncheck_recursion();
        Ok(FunctionBlock { name, var_sections, statements })
    }
    
    fn parse_var_section(&mut self) -> Result<VarSection, ParseError> {
        self.check_recursion()?;
        let result = match self.peek() {
            TokenKind::VarInput => {
                self.advance();
                let mut declarations = Vec::new();
                let mut iterations = 0;
                while !matches!(self.peek(), TokenKind::EndVar) {
                    iterations += 1;
                    if iterations > self.max_iterations {
                        return Err(ParseError::new(ParseErrorKind::TooManyIterations, self.current().span));
                    }
                    declarations.push(self.parse_var_declaration()?);
                }
                self.expect(TokenKind::EndVar)?;
                Ok(VarSection::Input(VarInput { declarations }))
            }
            TokenKind::VarOutput => {
                self.advance();
                let mut declarations = Vec::new();
                let mut iterations = 0;
                while !matches!(self.peek(), TokenKind::EndVar) {
                    iterations += 1;
                    if iterations > self.max_iterations {
                        return Err(ParseError::new(ParseErrorKind::TooManyIterations, self.current().span));
                    }
                    declarations.push(self.parse_var_declaration()?);
                }
                self.expect(TokenKind::EndVar)?;
                Ok(VarSection::Output(VarOutput { declarations }))
            }
            TokenKind::VarInOut => {
                self.advance();
                let mut declarations = Vec::new();
                let mut iterations = 0;
                while !matches!(self.peek(), TokenKind::EndVar) {
                    iterations += 1;
                    if iterations > self.max_iterations {
                        return Err(ParseError::new(ParseErrorKind::TooManyIterations, self.current().span));
                    }
                    declarations.push(self.parse_var_declaration()?);
                }
                self.expect(TokenKind::EndVar)?;
                Ok(VarSection::InOut(VarInout { declarations }))
            }
            TokenKind::VarTemp => {
                self.advance();
                let mut declarations = Vec::new();
                let mut iterations = 0;
                while !matches!(self.peek(), TokenKind::EndVar) {
                    iterations += 1;
                    if iterations > self.max_iterations {
                        return Err(ParseError::new(ParseErrorKind::TooManyIterations, self.current().span));
                    }
                    declarations.push(self.parse_var_declaration()?);
                }
                self.expect(TokenKind::EndVar)?;
                Ok(VarSection::Temp(VarTemp { declarations }))
            }
            TokenKind::Var => {
                self.advance();
                let mut declarations = Vec::new();
                let mut iterations = 0;
                while !matches!(self.peek(), TokenKind::EndVar) {
                    iterations += 1;
                    if iterations > self.max_iterations {
                        return Err(ParseError::new(ParseErrorKind::TooManyIterations, self.current().span));
                    }
                    declarations.push(self.parse_var_declaration()?);
                }
                self.expect(TokenKind::EndVar)?;
                Ok(VarSection::Var(VarDecl { declarations }))
            }
            kind => Err(ParseError::new(
                ParseErrorKind::UnexpectedToken { 
                    expected: "VAR section (VAR_INPUT/VAR_OUTPUT/VAR_INOUT/VAR_TEMP/VAR)".to_string(), 
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
            TokenKind::Identifier(_) => {
                // Could be assignment or function call
                let checkpoint = self.pos;
                let name = self.expect_identifier()?;
                
                if matches!(self.peek(), TokenKind::Operator(op) if op == ":=") {
                    self.pos = checkpoint;
                    Ok(Statement::Assignment(self.parse_assignment()?))
                } else if matches!(self.peek(), TokenKind::Operator(op) if op == "(") {
                    self.pos = checkpoint;
                    Ok(Statement::FunctionCall(self.parse_function_call_stmt()?))
                } else {
                    Err(ParseError::new(
                        ParseErrorKind::UnexpectedToken { 
                            expected: ":= or ( after identifier".to_string(), 
                            found: format!("{:?}", self.peek()) 
                        },
                        self.current().span,
                    ))
                }
            }
            kind => Err(ParseError::new(
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
        
        let mut var_sections = Vec::new();
        while matches!(self.peek(), TokenKind::VarInput | TokenKind::VarOutput | TokenKind::VarInOut | TokenKind::VarTemp | TokenKind::Var) {
            var_sections.push(self.parse_var_section()?);
        }
        
        self.expect(TokenKind::Begin)?;
        
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
        
        let mut var_sections = Vec::new();
        while matches!(self.peek(), TokenKind::VarInput | TokenKind::VarOutput | TokenKind::VarInOut | TokenKind::VarTemp | TokenKind::Var) {
            var_sections.push(self.parse_var_section()?);
        }
        
        self.expect(TokenKind::Begin)?;
        
        let mut statements = Vec::new();
        while !matches!(self.peek(), TokenKind::EndDataBlock) {
            statements.push(self.parse_statement()?);
        }
        
        self.expect(TokenKind::EndDataBlock)?;
        
        Ok(DataBlock { name, var_sections, statements })
    }
    
    fn parse_type_decl(&mut self) -> Result<TypeDecl, ParseError> { 
        self.expect(TokenKind::Type)?;
        let name = self.expect_identifier()?;
        self.expect(TokenKind::Operator(":".to_string()))?;
        let type_ref = self.parse_type_ref()?;
        self.expect(TokenKind::EndType)?;
        Ok(TypeDecl { name, type_spec: TypeSpec { type_ref } })
    }
    
    fn parse_var_declaration(&mut self) -> Result<VarDeclaration, ParseError> { 
        let mut names = vec![self.expect_identifier()?];
        
        while matches!(self.peek(), TokenKind::Operator(op) if op == ",") {
            self.advance();
            names.push(self.expect_identifier()?);
        }
        
        self.expect(TokenKind::Operator(":".to_string()))?;
        let type_ref = self.parse_type_ref()?;
        
        let initializer = if matches!(self.peek(), TokenKind::Operator(op) if op == ":=") {
            self.advance();
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        self.expect(TokenKind::Operator(";".to_string()))?;
        
        Ok(VarDeclaration { names, type_ref, initializer })
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
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(TypeRef::Named(name))
            }
            kind => Err(ParseError::new(
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
            
            let mut body = Vec::new();
            while !matches!(self.peek(), TokenKind::Integer(_) | TokenKind::Identifier(_) | TokenKind::Else | TokenKind::EndCase) {
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
        let target = self.expect_identifier()?;
        self.expect(TokenKind::Operator(":=".to_string()))?;
        let value = self.parse_expression()?;
        self.expect(TokenKind::Operator(";".to_string()))?;
        Ok(Assignment { target, value })
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
                return Err(ParseError::new(ParseErrorKind::TooManyIterations, self.current().span));
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
                let name = name.clone();
                self.advance();
                
                // Check if it's a function call
                if matches!(self.peek(), TokenKind::Operator(op) if op == "(") {
                    self.pos -= 1;  // Backtrack
                    Ok(Primary::FunctionCall(self.parse_function_call()?))
                } else {
                    Ok(Primary::Identifier(name))
                }
            }
            TokenKind::Integer(val) => {
                let val = val.clone();
                self.advance();
                Ok(Primary::Literal(Literal::Integer(val)))
            }
            TokenKind::Real(val) => {
                let val = val.clone();
                self.advance();
                Ok(Primary::Literal(Literal::Real(val)))
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
            kind => Err(ParseError::new(
                ParseErrorKind::UnexpectedToken { 
                    expected: "primary expression".to_string(), 
                    found: format!("{:?}", kind) 
                },
                self.current().span,
            ))
        }
    }
}
