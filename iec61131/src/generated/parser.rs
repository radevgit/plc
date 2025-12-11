//! Generated parser for IEC 61131-3

use super::lexer::{Lexer, SpannedToken, Token, Span};
use super::ast::*;

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse error at {:?}: {}", self.span, self.message)
    }
}

impl std::error::Error for ParseError {}

pub type ParseResult<T> = Result<T, ParseError>;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current: SpannedToken,
    previous: SpannedToken,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Lexer::new(input);
        let first = lexer.next_token();
        Self {
            lexer,
            current: first.clone(),
            previous: first,
        }
    }
    
    /// Parse a complete compilation unit
    pub fn parse(&mut self) -> ParseResult<CompilationUnit> {
        let start = self.current.span.start;
        let mut declarations = Vec::new();
        
        while !self.is_at_end() {
            declarations.push(self.parse_pou_declaration()?);
        }
        
        let end = self.previous.span.end;
        
        Ok(CompilationUnit {
            declarations,
            span: Span::new(start, end),
        })
    }
    
    /// Parse a POU declaration
    fn parse_pou_declaration(&mut self) -> ParseResult<PouDeclaration> {
        match &self.current.token {
            Token::Function => Ok(PouDeclaration::Function(self.parse_function()?)),
            Token::FunctionBlock => Ok(PouDeclaration::FunctionBlock(self.parse_function_block()?)),
            Token::Program => Ok(PouDeclaration::Program(self.parse_program()?)),
            Token::Class => Ok(PouDeclaration::Class(self.parse_class()?)),
            Token::Interface => Ok(PouDeclaration::Interface(self.parse_interface()?)),
            Token::Type => Ok(PouDeclaration::DataType(self.parse_data_type_decl()?)),
            Token::VarGlobal => Ok(PouDeclaration::GlobalVar(self.parse_global_var_decl()?)),
            Token::Namespace => Ok(PouDeclaration::Namespace(self.parse_namespace()?)),
            _ => Err(self.error(format!("Expected POU declaration, found {:?}", self.current.token))),
        }
    }
    
    /// Parse FUNCTION declaration
    fn parse_function(&mut self) -> ParseResult<FunctionDecl> {
        let start = self.current.span.start;
        self.expect(Token::Function)?;
        
        let name = self.expect_identifier()?;
        
        // Optional return type
        let return_type = if self.check(&Token::Colon) {
            self.advance();
            Some(self.parse_type_spec()?)
        } else {
            None
        };
        
        // Variable declarations
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        let mut in_outs = Vec::new();
        let mut vars = Vec::new();
        
        while self.check(&Token::VarInput) || self.check(&Token::VarOutput) 
            || self.check(&Token::VarInOut) || self.check(&Token::Var) {
            match &self.current.token {
                Token::VarInput => inputs.extend(self.parse_var_block()?),
                Token::VarOutput => outputs.extend(self.parse_var_block()?),
                Token::VarInOut => in_outs.extend(self.parse_var_block()?),
                Token::Var => vars.extend(self.parse_var_block()?),
                _ => break,
            }
        }
        
        // Function body
        let body = self.parse_statement_list()?;
        
        self.expect(Token::EndFunction)?;
        
        let end = self.previous.span.end;
        
        Ok(FunctionDecl {
            name,
            return_type,
            inputs,
            outputs,
            in_outs,
            vars,
            body,
            span: Span::new(start, end),
        })
    }
    
    /// Parse FUNCTION_BLOCK declaration
    fn parse_function_block(&mut self) -> ParseResult<FunctionBlockDecl> {
        let start = self.current.span.start;
        self.expect(Token::FunctionBlock)?;
        
        // Optional FINAL or ABSTRACT
        let is_final = self.match_token(&Token::Final);
        let is_abstract = if !is_final {
            self.match_token(&Token::Abstract)
        } else {
            false
        };
        
        let name = self.expect_identifier()?;
        
        // Optional EXTENDS
        let extends = if self.match_token(&Token::Extends) {
            Some(self.expect_identifier()?)
        } else {
            None
        };
        
        // Optional IMPLEMENTS
        let mut implements = Vec::new();
        if self.match_token(&Token::Implements) {
            loop {
                implements.push(self.expect_identifier()?);
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }
        
        // Variable declarations
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        let mut in_outs = Vec::new();
        let mut vars = Vec::new();
        let mut methods = Vec::new();
        
        loop {
            match &self.current.token {
                Token::VarInput => inputs.extend(self.parse_var_block()?),
                Token::VarOutput => outputs.extend(self.parse_var_block()?),
                Token::VarInOut => in_outs.extend(self.parse_var_block()?),
                Token::Var => vars.extend(self.parse_var_block()?),
                Token::Method => methods.push(self.parse_method()?),
                Token::EndFunctionBlock => break,
                _ => {
                    // Try to parse body
                    break;
                }
            }
        }
        
        // Optional body
        let body = if !self.check(&Token::EndFunctionBlock) {
            Some(self.parse_statement_list()?)
        } else {
            None
        };
        
        self.expect(Token::EndFunctionBlock)?;
        
        let end = self.previous.span.end;
        
        Ok(FunctionBlockDecl {
            name,
            extends,
            implements,
            is_final,
            is_abstract,
            inputs,
            outputs,
            in_outs,
            vars,
            methods,
            body,
            span: Span::new(start, end),
        })
    }
    
    /// Parse PROGRAM declaration
    fn parse_program(&mut self) -> ParseResult<ProgramDecl> {
        let start = self.current.span.start;
        self.expect(Token::Program)?;
        
        let name = self.expect_identifier()?;
        
        // Variable declarations
        let mut vars = Vec::new();
        while self.check(&Token::Var) || self.check(&Token::VarInput) 
            || self.check(&Token::VarOutput) || self.check(&Token::VarInOut) {
            vars.extend(self.parse_var_block()?);
        }
        
        // Program body
        let body = self.parse_statement_list()?;
        
        self.expect(Token::EndProgram)?;
        
        let end = self.previous.span.end;
        
        Ok(ProgramDecl {
            name,
            vars,
            body,
            span: Span::new(start, end),
        })
    }
    
    /// Parse CLASS declaration  
    fn parse_class(&mut self) -> ParseResult<ClassDecl> {
        let start = self.current.span.start;
        self.expect(Token::Class)?;
        
        let is_final = self.match_token(&Token::Final);
        let is_abstract = if !is_final {
            self.match_token(&Token::Abstract)
        } else {
            false
        };
        
        let name = self.expect_identifier()?;
        
        let extends = if self.match_token(&Token::Extends) {
            Some(self.expect_identifier()?)
        } else {
            None
        };
        
        let mut implements = Vec::new();
        if self.match_token(&Token::Implements) {
            loop {
                implements.push(self.expect_identifier()?);
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }
        
        let mut vars = Vec::new();
        let mut methods = Vec::new();
        
        while !self.check(&Token::EndClass) {
            if self.check(&Token::Var) {
                vars.extend(self.parse_var_block()?);
            } else if self.check(&Token::Method) {
                methods.push(self.parse_method()?);
            } else {
                break;
            }
        }
        
        self.expect(Token::EndClass)?;
        
        let end = self.previous.span.end;
        
        Ok(ClassDecl {
            name,
            extends,
            implements,
            is_final,
            is_abstract,
            vars,
            methods,
            span: Span::new(start, end),
        })
    }
    
    /// Parse INTERFACE declaration
    fn parse_interface(&mut self) -> ParseResult<InterfaceDecl> {
        let start = self.current.span.start;
        self.expect(Token::Interface)?;
        
        let name = self.expect_identifier()?;
        
        let mut extends = Vec::new();
        if self.match_token(&Token::Extends) {
            loop {
                extends.push(self.expect_identifier()?);
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }
        
        let mut methods = Vec::new();
        while self.check(&Token::Method) {
            methods.push(self.parse_method_prototype()?);
        }
        
        self.expect(Token::EndInterface)?;
        
        let end = self.previous.span.end;
        
        Ok(InterfaceDecl {
            name,
            extends,
            methods,
            span: Span::new(start, end),
        })
    }
    
    /// Parse METHOD declaration
    fn parse_method(&mut self) -> ParseResult<MethodDecl> {
        let start = self.current.span.start;
        self.expect(Token::Method)?;
        
        let access = self.parse_access_modifier();
        let is_final = self.match_token(&Token::Final);
        let is_abstract = if !is_final {
            self.match_token(&Token::Abstract)
        } else {
            false
        };
        let is_override = self.match_token(&Token::Override);
        
        let name = self.expect_identifier()?;
        
        let return_type = if self.check(&Token::Colon) {
            self.advance();
            Some(self.parse_type_spec()?)
        } else {
            None
        };
        
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        let mut in_outs = Vec::new();
        let mut vars = Vec::new();
        
        while self.check(&Token::VarInput) || self.check(&Token::VarOutput) 
            || self.check(&Token::VarInOut) || self.check(&Token::Var) {
            match &self.current.token {
                Token::VarInput => inputs.extend(self.parse_var_block()?),
                Token::VarOutput => outputs.extend(self.parse_var_block()?),
                Token::VarInOut => in_outs.extend(self.parse_var_block()?),
                Token::Var => vars.extend(self.parse_var_block()?),
                _ => break,
            }
        }
        
        let body = self.parse_statement_list()?;
        
        self.expect(Token::EndMethod)?;
        
        let end = self.previous.span.end;
        
        Ok(MethodDecl {
            name,
            access,
            return_type,
            is_final,
            is_abstract,
            is_override,
            inputs,
            outputs,
            in_outs,
            vars,
            body,
            span: Span::new(start, end),
        })
    }
    
    /// Parse method prototype (for interfaces)
    fn parse_method_prototype(&mut self) -> ParseResult<MethodPrototype> {
        let start = self.current.span.start;
        self.expect(Token::Method)?;
        
        let name = self.expect_identifier()?;
        
        let return_type = if self.check(&Token::Colon) {
            self.advance();
            Some(self.parse_type_spec()?)
        } else {
            None
        };
        
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        let mut in_outs = Vec::new();
        
        while self.check(&Token::VarInput) || self.check(&Token::VarOutput) 
            || self.check(&Token::VarInOut) {
            match &self.current.token {
                Token::VarInput => inputs.extend(self.parse_var_block()?),
                Token::VarOutput => outputs.extend(self.parse_var_block()?),
                Token::VarInOut => in_outs.extend(self.parse_var_block()?),
                _ => break,
            }
        }
        
        self.expect(Token::EndMethod)?;
        
        let end = self.previous.span.end;
        
        Ok(MethodPrototype {
            name,
            return_type,
            inputs,
            outputs,
            in_outs,
            span: Span::new(start, end),
        })
    }
    
    /// Parse access modifier
    fn parse_access_modifier(&mut self) -> AccessModifier {
        match &self.current.token {
            Token::Public => { self.advance(); AccessModifier::Public }
            Token::Protected => { self.advance(); AccessModifier::Protected }
            Token::Private => { self.advance(); AccessModifier::Private }
            Token::Internal => { self.advance(); AccessModifier::Internal }
            _ => AccessModifier::Public, // Default
        }
    }
    
    /// Parse variable block (VAR ... END_VAR)
    fn parse_var_block(&mut self) -> ParseResult<Vec<VarDecl>> {
        self.advance(); // Consume VAR* token
        
        let is_constant = self.match_token(&Token::Constant);
        let is_retain = self.match_token(&Token::Retain);
        
        let mut vars = Vec::new();
        
        while !self.check(&Token::EndVar) {
            vars.push(self.parse_var_decl(is_constant, is_retain)?);
            self.expect(Token::Semicolon)?;
        }
        
        self.expect(Token::EndVar)?;
        
        Ok(vars)
    }
    
    /// Parse single variable declaration
    fn parse_var_decl(&mut self, is_constant: bool, is_retain: bool) -> ParseResult<VarDecl> {
        let start = self.current.span.start;
        let name = self.expect_identifier()?;
        
        // Optional AT location
        let location = if self.match_token(&Token::At) {
            Some(self.parse_direct_variable()?)
        } else {
            None
        };
        
        self.expect(Token::Colon)?;
        let var_type = self.parse_type_spec()?;
        
        // Optional initialization
        let init_value = if self.match_token(&Token::Assign) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        let end = self.previous.span.end;
        
        Ok(VarDecl {
            name,
            var_type,
            init_value,
            is_constant,
            is_retain,
            location,
            span: Span::new(start, end),
        })
    }
    
    /// Parse type specification
    fn parse_type_spec(&mut self) -> ParseResult<TypeSpec> {
        if self.check(&Token::Array) {
            self.parse_array_type()
        } else if self.check(&Token::Struct) {
            self.parse_struct_type()
        } else if self.check(&Token::RefTo) {
            self.advance();
            Ok(TypeSpec::Ref(Box::new(self.parse_type_spec()?)))
        } else {
            // Elementary or user-defined type
            let name = self.expect_type_name()?;
            Ok(TypeSpec::Elementary(name))
        }
    }
    
    /// Parse array type
    fn parse_array_type(&mut self) -> ParseResult<TypeSpec> {
        self.expect(Token::Array)?;
        self.expect(Token::LBracket)?;
        
        let mut dimensions = Vec::new();
        loop {
            let start = self.parse_expression()?;
            self.expect(Token::DotDot)?;
            let end = self.parse_expression()?;
            dimensions.push(ArrayDimension { start, end });
            
            if !self.match_token(&Token::Comma) {
                break;
            }
        }
        
        self.expect(Token::RBracket)?;
        self.expect(Token::Of)?;
        
        let element_type = Box::new(self.parse_type_spec()?);
        
        Ok(TypeSpec::Array {
            dimensions,
            element_type,
        })
    }
    
    /// Parse struct type
    fn parse_struct_type(&mut self) -> ParseResult<TypeSpec> {
        self.expect(Token::Struct)?;
        
        let mut fields = Vec::new();
        
        while !self.check(&Token::EndStruct) {
            let name = self.expect_identifier()?;
            self.expect(Token::Colon)?;
            let field_type = self.parse_type_spec()?;
            
            let init_value = if self.match_token(&Token::Assign) {
                Some(self.parse_expression()?)
            } else {
                None
            };
            
            self.expect(Token::Semicolon)?;
            
            fields.push(StructField {
                name,
                field_type,
                init_value,
            });
        }
        
        self.expect(Token::EndStruct)?;
        
        Ok(TypeSpec::Struct { fields })
    }
    
    /// Parse direct variable (%IX0.0, etc.)
    fn parse_direct_variable(&mut self) -> ParseResult<DirectVariable> {
        let start = self.current.span.start;
        
        if let Token::DirectVariable(location) = &self.current.token {
            let location = location.clone();
            let end = self.current.span.end;
            self.advance();
            Ok(DirectVariable {
                location,
                span: Span::new(start, end),
            })
        } else {
            Err(self.error("Expected direct variable".to_string()))
        }
    }
    
    /// Parse DATA_TYPE declaration
    fn parse_data_type_decl(&mut self) -> ParseResult<DataTypeDecl> {
        self.expect(Token::Type)?;
        
        let name = self.expect_identifier()?;
        self.expect(Token::Colon)?;
        
        let decl = if self.check(&Token::Struct) {
            let TypeSpec::Struct { fields } = self.parse_struct_type()? else {
                return Err(self.error("Expected struct".to_string()));
            };
            DataTypeDecl::Struct { name, fields }
        } else if self.check(&Token::Array) {
            let spec = self.parse_array_type()?;
            DataTypeDecl::Array { name, spec }
        } else {
            // Simple type or subrange or enum - simplified for now
            let base_type = self.parse_type_spec()?;
            let init_value = if self.match_token(&Token::Assign) {
                Some(self.parse_expression()?)
            } else {
                None
            };
            DataTypeDecl::Simple {
                name,
                base_type,
                init_value,
            }
        };
        
        self.expect(Token::Semicolon)?;
        self.expect(Token::EndType)?;
        
        Ok(decl)
    }
    
    /// Parse global variable declaration
    fn parse_global_var_decl(&mut self) -> ParseResult<GlobalVarDecl> {
        let start = self.current.span.start;
        self.expect(Token::VarGlobal)?;
        
        let is_constant = self.match_token(&Token::Constant);
        let is_retain = self.match_token(&Token::Retain);
        
        let mut vars = Vec::new();
        
        while !self.check(&Token::EndVar) {
            vars.push(self.parse_var_decl(is_constant, is_retain)?);
            self.expect(Token::Semicolon)?;
        }
        
        self.expect(Token::EndVar)?;
        
        let end = self.previous.span.end;
        
        Ok(GlobalVarDecl {
            vars,
            is_constant,
            is_retain,
            span: Span::new(start, end),
        })
    }
    
    /// Parse namespace declaration
    fn parse_namespace(&mut self) -> ParseResult<NamespaceDecl> {
        let start = self.current.span.start;
        self.expect(Token::Namespace)?;
        
        let is_internal = self.match_token(&Token::Internal);
        
        let mut name = vec![self.expect_identifier()?];
        while self.match_token(&Token::Dot) {
            name.push(self.expect_identifier()?);
        }
        
        let mut using_directives = Vec::new();
        while self.match_token(&Token::Using) {
            let mut path = vec![self.expect_identifier()?];
            while self.match_token(&Token::Dot) {
                path.push(self.expect_identifier()?);
            }
            using_directives.push(path);
            self.expect(Token::Semicolon)?;
        }
        
        let mut elements = Vec::new();
        while !self.check(&Token::EndNamespace) {
            elements.push(self.parse_pou_declaration()?);
        }
        
        self.expect(Token::EndNamespace)?;
        
        let end = self.previous.span.end;
        
        Ok(NamespaceDecl {
            name,
            is_internal,
            using_directives,
            elements,
            span: Span::new(start, end),
        })
    }
    
    /// Parse statement list
    fn parse_statement_list(&mut self) -> ParseResult<StatementList> {
        let mut stmts = Vec::new();
        
        while !self.is_statement_list_end() {
            if let Some(stmt) = self.parse_statement()? {
                stmts.push(stmt);
            }
            
            // Statements end with semicolon
            if !self.check(&Token::Semicolon) {
                break;
            }
            self.advance();
        }
        
        Ok(stmts)
    }
    
    fn is_statement_list_end(&self) -> bool {
        matches!(&self.current.token,
            Token::EndFunction | Token::EndFunctionBlock |
            Token::EndProgram | Token::EndMethod | Token::EndClass |
            Token::EndIf | Token::EndWhile | Token::EndFor |
            Token::EndRepeat | Token::EndCase | Token::Elsif |
            Token::Else | Token::Until | Token::Eof
        )
    }
    
    /// Parse a single statement
    fn parse_statement(&mut self) -> ParseResult<Option<Statement>> {
        let start = self.current.span.start;
        
        match &self.current.token {
            Token::If => Ok(Some(self.parse_if_statement()?)),
            Token::Case => Ok(Some(self.parse_case_statement()?)),
            Token::For => Ok(Some(self.parse_for_statement()?)),
            Token::While => Ok(Some(self.parse_while_statement()?)),
            Token::Repeat => Ok(Some(self.parse_repeat_statement()?)),
            Token::Exit => {
                self.advance();
                Ok(Some(Statement::Exit {
                    span: Span::new(start, self.previous.span.end),
                }))
            }
            Token::Continue => {
                self.advance();
                Ok(Some(Statement::Continue {
                    span: Span::new(start, self.previous.span.end),
                }))
            }
            Token::Return => {
                self.advance();
                let value = if !self.check(&Token::Semicolon) {
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                Ok(Some(Statement::Return {
                    value,
                    span: Span::new(start, self.previous.span.end),
                }))
            }
            Token::Identifier(_) => {
                // Could be assignment or function call
                let var = self.parse_variable()?;
                
                if self.check(&Token::Assign) {
                    self.advance();
                    let value = self.parse_expression()?;
                    let end = self.previous.span.end;
                    Ok(Some(Statement::Assignment {
                        target: var,
                        value,
                        span: Span::new(start, end),
                    }))
                } else if self.check(&Token::LParen) {
                    // Function call
                    self.advance();
                    let arguments = self.parse_arguments()?;
                    self.expect(Token::RParen)?;
                    
                    if let Variable::Simple(name) = var {
                        let end = self.previous.span.end;
                        Ok(Some(Statement::FunctionCall {
                            name,
                            arguments,
                            span: Span::new(start, end),
                        }))
                    } else {
                        Err(self.error("Complex variable cannot be called as function".to_string()))
                    }
                } else {
                    // Just a variable reference - not a valid statement
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }
    
    /// Parse IF statement
    fn parse_if_statement(&mut self) -> ParseResult<Statement> {
        let start = self.current.span.start;
        self.expect(Token::If)?;
        
        let condition = self.parse_expression()?;
        self.expect(Token::Then)?;
        
        let then_body = self.parse_statement_list()?;
        
        let mut elsif_parts = Vec::new();
        while self.match_token(&Token::Elsif) {
            let elsif_cond = self.parse_expression()?;
            self.expect(Token::Then)?;
            let elsif_body = self.parse_statement_list()?;
            elsif_parts.push((elsif_cond, elsif_body));
        }
        
        let else_body = if self.match_token(&Token::Else) {
            Some(self.parse_statement_list()?)
        } else {
            None
        };
        
        self.expect(Token::EndIf)?;
        
        let end = self.previous.span.end;
        
        Ok(Statement::If {
            condition,
            then_body,
            elsif_parts,
            else_body,
            span: Span::new(start, end),
        })
    }
    
    /// Parse CASE statement
    fn parse_case_statement(&mut self) -> ParseResult<Statement> {
        let start = self.current.span.start;
        self.expect(Token::Case)?;
        
        let selector = self.parse_expression()?;
        self.expect(Token::Of)?;
        
        let mut cases = Vec::new();
        
        while !self.check(&Token::EndCase) && !self.check(&Token::Else) {
            let mut selectors = Vec::new();
            
            loop {
                let expr = self.parse_expression()?;
                
                if self.match_token(&Token::DotDot) {
                    let end_expr = self.parse_expression()?;
                    selectors.push(CaseSelector::Range(expr, end_expr));
                } else {
                    selectors.push(CaseSelector::Value(expr));
                }
                
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
            
            self.expect(Token::Colon)?;
            let body = self.parse_statement_list()?;
            
            cases.push(CaseItem { selectors, body });
        }
        
        let else_body = if self.match_token(&Token::Else) {
            Some(self.parse_statement_list()?)
        } else {
            None
        };
        
        self.expect(Token::EndCase)?;
        
        let end = self.previous.span.end;
        
        Ok(Statement::Case {
            selector,
            cases,
            else_body,
            span: Span::new(start, end),
        })
    }
    
    /// Parse FOR statement
    fn parse_for_statement(&mut self) -> ParseResult<Statement> {
        let start = self.current.span.start;
        self.expect(Token::For)?;
        
        let control_var = self.expect_identifier()?;
        self.expect(Token::Assign)?;
        
        let start_expr = self.parse_expression()?;
        self.expect(Token::To)?;
        let end_expr = self.parse_expression()?;
        
        let step = if self.match_token(&Token::By) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        self.expect(Token::Do)?;
        let body = self.parse_statement_list()?;
        self.expect(Token::EndFor)?;
        
        let end = self.previous.span.end;
        
        Ok(Statement::For {
            control_var,
            start: start_expr,
            end: end_expr,
            step,
            body,
            span: Span::new(start, end),
        })
    }
    
    /// Parse WHILE statement
    fn parse_while_statement(&mut self) -> ParseResult<Statement> {
        let start = self.current.span.start;
        self.expect(Token::While)?;
        
        let condition = self.parse_expression()?;
        self.expect(Token::Do)?;
        let body = self.parse_statement_list()?;
        self.expect(Token::EndWhile)?;
        
        let end = self.previous.span.end;
        
        Ok(Statement::While {
            condition,
            body,
            span: Span::new(start, end),
        })
    }
    
    /// Parse REPEAT statement
    fn parse_repeat_statement(&mut self) -> ParseResult<Statement> {
        let start = self.current.span.start;
        self.expect(Token::Repeat)?;
        
        let body = self.parse_statement_list()?;
        self.expect(Token::Until)?;
        let condition = self.parse_expression()?;
        self.expect(Token::EndRepeat)?;
        
        let end = self.previous.span.end;
        
        Ok(Statement::Repeat {
            body,
            condition,
            span: Span::new(start, end),
        })
    }
    
    /// Parse expression with operator precedence
    fn parse_expression(&mut self) -> ParseResult<Expression> {
        self.parse_or_expression()
    }
    
    fn parse_or_expression(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_xor_expression()?;
        
        while self.match_token(&Token::Or) {
            let right = self.parse_xor_expression()?;
            left = Expression::Binary {
                op: BinaryOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_xor_expression(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_and_expression()?;
        
        while self.match_token(&Token::Xor) {
            let right = self.parse_and_expression()?;
            left = Expression::Binary {
                op: BinaryOp::Xor,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_and_expression(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_comparison_expression()?;
        
        while self.match_token(&Token::And) || self.match_token(&Token::Ampersand) {
            let right = self.parse_comparison_expression()?;
            left = Expression::Binary {
                op: BinaryOp::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_comparison_expression(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_additive_expression()?;
        
        while let Some(op) = self.match_comparison_op() {
            let right = self.parse_additive_expression()?;
            left = Expression::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn match_comparison_op(&mut self) -> Option<BinaryOp> {
        let op = match &self.current.token {
            Token::Equal => BinaryOp::Eq,
            Token::NotEqual => BinaryOp::Ne,
            Token::Less => BinaryOp::Lt,
            Token::LessEqual => BinaryOp::Le,
            Token::Greater => BinaryOp::Gt,
            Token::GreaterEqual => BinaryOp::Ge,
            _ => return None,
        };
        self.advance();
        Some(op)
    }
    
    fn parse_additive_expression(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_multiplicative_expression()?;
        
        while self.check(&Token::Plus) || self.check(&Token::Minus) {
            let op = if self.match_token(&Token::Plus) {
                BinaryOp::Add
            } else {
                self.advance(); // Minus
                BinaryOp::Sub
            };
            
            let right = self.parse_multiplicative_expression()?;
            left = Expression::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_multiplicative_expression(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_power_expression()?;
        
        while self.check(&Token::Star) || self.check(&Token::Slash) || self.check(&Token::Mod) {
            let op = if self.match_token(&Token::Star) {
                BinaryOp::Mul
            } else if self.match_token(&Token::Slash) {
                BinaryOp::Div
            } else {
                self.advance(); // MOD
                BinaryOp::Mod
            };
            
            let right = self.parse_power_expression()?;
            left = Expression::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_power_expression(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_unary_expression()?;
        
        while self.match_token(&Token::Power) {
            let right = self.parse_unary_expression()?;
            left = Expression::Binary {
                op: BinaryOp::Power,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_unary_expression(&mut self) -> ParseResult<Expression> {
        if self.match_token(&Token::Not) {
            let operand = Box::new(self.parse_unary_expression()?);
            Ok(Expression::Unary {
                op: UnaryOp::Not,
                operand,
            })
        } else if self.match_token(&Token::Minus) {
            let operand = Box::new(self.parse_unary_expression()?);
            Ok(Expression::Unary {
                op: UnaryOp::Neg,
                operand,
            })
        } else if self.match_token(&Token::Plus) {
            // Unary plus is just ignored
            self.parse_unary_expression()
        } else {
            self.parse_primary_expression()
        }
    }
    
    fn parse_primary_expression(&mut self) -> ParseResult<Expression> {
        // Literals
        if let Token::IntLiteral(s) = &self.current.token {
            let value = s.parse::<i64>().unwrap_or(0);
            self.advance();
            return Ok(Expression::Literal(Literal::Integer(value)));
        }
        
        if let Token::RealLiteral(s) = &self.current.token {
            let value = s.parse::<f64>().unwrap_or(0.0);
            self.advance();
            return Ok(Expression::Literal(Literal::Real(value)));
        }
        
        if let Token::StringLiteral(s) = &self.current.token {
            let value = s.clone();
            self.advance();
            return Ok(Expression::Literal(Literal::String(value)));
        }
        
        if self.match_token(&Token::True) {
            return Ok(Expression::Literal(Literal::Bool(true)));
        }
        
        if self.match_token(&Token::False) {
            return Ok(Expression::Literal(Literal::Bool(false)));
        }
        
        if self.match_token(&Token::Null) {
            return Ok(Expression::Literal(Literal::Null));
        }
        
        // Parenthesized expression
        if self.match_token(&Token::LParen) {
            let expr = self.parse_expression()?;
            self.expect(Token::RParen)?;
            return Ok(Expression::Parenthesized(Box::new(expr)));
        }
        
        // Variable or function call
        if matches!(&self.current.token, Token::Identifier(_)) {
            let var = self.parse_variable()?;
            
            // Check if it's a function call
            if self.check(&Token::LParen) {
                self.advance();
                let arguments = self.parse_arguments()?;
                self.expect(Token::RParen)?;
                
                if let Variable::Simple(name) = var {
                    return Ok(Expression::Call {
                        function: name,
                        arguments,
                    });
                }
            }
            
            return Ok(Expression::Variable(var));
        }
        
        Err(self.error(format!("Unexpected token in expression: {:?}", self.current.token)))
    }
    
    /// Parse variable (with member access, array indexing, etc.)
    fn parse_variable(&mut self) -> ParseResult<Variable> {
        let mut var = if let Token::DirectVariable(_) = &self.current.token {
            Variable::Direct(self.parse_direct_variable()?)
        } else if let Token::Identifier(name) = &self.current.token {
            let name = name.clone();
            self.advance();
            Variable::Simple(name)
        } else {
            return Err(self.error("Expected variable".to_string()));
        };
        
        // Handle member access and array indexing
        loop {
            if self.match_token(&Token::Dot) {
                let member = self.expect_identifier()?;
                var = Variable::MemberAccess {
                    base: Box::new(var),
                    member,
                };
            } else if self.match_token(&Token::LBracket) {
                let mut indices = vec![self.parse_expression()?];
                
                while self.match_token(&Token::Comma) {
                    indices.push(self.parse_expression()?);
                }
                
                self.expect(Token::RBracket)?;
                
                var = Variable::ArrayAccess {
                    base: Box::new(var),
                    indices,
                };
            } else if self.match_token(&Token::Caret) {
                var = Variable::Dereference {
                    base: Box::new(var),
                };
            } else {
                break;
            }
        }
        
        Ok(var)
    }
    
    /// Parse function arguments
    fn parse_arguments(&mut self) -> ParseResult<Vec<Argument>> {
        let mut arguments = Vec::new();
        
        if self.check(&Token::RParen) {
            return Ok(arguments);
        }
        
        loop {
            // Check for named argument
            if let Token::Identifier(name) = &self.current.token {
                let name_copy = name.clone();
                self.advance();
                
                if self.match_token(&Token::Assign) {
                    // Named input argument
                    let value = self.parse_expression()?;
                    arguments.push(Argument::Named {
                        name: name_copy,
                        value,
                    });
                } else if self.match_token(&Token::Arrow) {
                    // Output argument
                    let variable = self.parse_variable()?;
                    arguments.push(Argument::Output {
                        name: name_copy,
                        variable,
                    });
                } else {
                    // Was actually a positional argument starting with identifier
                    // Need to backtrack - for now just treat as error
                    return Err(self.error("Invalid argument syntax".to_string()));
                }
            } else {
                // Positional argument
                let value = self.parse_expression()?;
                arguments.push(Argument::Positional(value));
            }
            
            if !self.match_token(&Token::Comma) {
                break;
            }
        }
        
        Ok(arguments)
    }
    
    // Helper methods
    
    fn advance(&mut self) {
        self.previous = self.current.clone();
        self.current = self.lexer.next_token();
    }
    
    fn check(&self, token: &Token) -> bool {
        std::mem::discriminant(&self.current.token) == std::mem::discriminant(token)
    }
    
    fn match_token(&mut self, token: &Token) -> bool {
        if self.check(token) {
            self.advance();
            true
        } else {
            false
        }
    }
    
    fn expect(&mut self, token: Token) -> ParseResult<()> {
        if self.check(&token) {
            self.advance();
            Ok(())
        } else {
            Err(self.error(format!("Expected {:?}, found {:?}", token, self.current.token)))
        }
    }
    
    fn expect_identifier(&mut self) -> ParseResult<String> {
        if let Token::Identifier(name) = &self.current.token {
            let name = name.clone();
            self.advance();
            Ok(name)
        } else {
            Err(self.error(format!("Expected identifier, found {:?}", self.current.token)))
        }
    }
    
    /// Accept identifier or type keyword as a name
    fn expect_type_name(&mut self) -> ParseResult<String> {
        match &self.current.token {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            }
            // Elementary types
            Token::Bool => { self.advance(); Ok("BOOL".to_string()) }
            Token::Byte => { self.advance(); Ok("BYTE".to_string()) }
            Token::Word => { self.advance(); Ok("WORD".to_string()) }
            Token::Dword => { self.advance(); Ok("DWORD".to_string()) }
            Token::Lword => { self.advance(); Ok("LWORD".to_string()) }
            Token::Sint => { self.advance(); Ok("SINT".to_string()) }
            Token::Int => { self.advance(); Ok("INT".to_string()) }
            Token::Dint => { self.advance(); Ok("DINT".to_string()) }
            Token::Lint => { self.advance(); Ok("LINT".to_string()) }
            Token::Usint => { self.advance(); Ok("USINT".to_string()) }
            Token::Uint => { self.advance(); Ok("UINT".to_string()) }
            Token::Udint => { self.advance(); Ok("UDINT".to_string()) }
            Token::Ulint => { self.advance(); Ok("ULINT".to_string()) }
            Token::Real => { self.advance(); Ok("REAL".to_string()) }
            Token::Lreal => { self.advance(); Ok("LREAL".to_string()) }
            Token::String => { self.advance(); Ok("STRING".to_string()) }
            Token::Wstring => { self.advance(); Ok("WSTRING".to_string()) }
            Token::Char => { self.advance(); Ok("CHAR".to_string()) }
            Token::Wchar => { self.advance(); Ok("WCHAR".to_string()) }
            Token::Time => { self.advance(); Ok("TIME".to_string()) }
            Token::Ltime => { self.advance(); Ok("LTIME".to_string()) }
            Token::Date => { self.advance(); Ok("DATE".to_string()) }
            Token::Ldate => { self.advance(); Ok("LDATE".to_string()) }
            Token::TimeOfDay => { self.advance(); Ok("TIME_OF_DAY".to_string()) }
            Token::Tod => { self.advance(); Ok("TOD".to_string()) }
            Token::Ltod => { self.advance(); Ok("LTOD".to_string()) }
            Token::LtimeOfDay => { self.advance(); Ok("LTIME_OF_DAY".to_string()) }
            Token::DateAndTime => { self.advance(); Ok("DATE_AND_TIME".to_string()) }
            Token::Dt => { self.advance(); Ok("DT".to_string()) }
            Token::Ldt => { self.advance(); Ok("LDT".to_string()) }
            Token::LdateAndTime => { self.advance(); Ok("LDATE_AND_TIME".to_string()) }
            _ => Err(self.error(format!("Expected type name or identifier, found {:?}", self.current.token)))
        }
    }
    
    fn is_at_end(&self) -> bool {
        matches!(&self.current.token, Token::Eof)
    }
    
    fn error(&self, message: String) -> ParseError {
        ParseError {
            message,
            span: self.current.span,
        }
    }
}
