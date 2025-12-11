//! # iec61131
//!
//! Complete IEC 61131-3 parser for PLC programming languages.
//!
//! This crate provides a comprehensive parser for all 5 IEC 61131-3 programming languages:
//! - **ST** (Structured Text)
//! - **IL** (Instruction List)
//! - **LD** (Ladder Diagram)
//! - **FBD** (Function Block Diagram)
//! - **SFC** (Sequential Function Chart)
//!
//! ## Quick Start
//!
//! ```rust
//! use iec61131::Parser;
//!
//! let code = r#"
//! FUNCTION Add : INT
//!     VAR_INPUT
//!         a : INT;
//!         b : INT;
//!     END_VAR
//!     
//!     Add := a + b;
//! END_FUNCTION
//! "#;
//!
//! let mut parser = Parser::new(code);
//! match parser.parse() {
//!     Ok(ast) => println!("Parsed {} declarations", ast.declarations.len()),
//!     Err(e) => eprintln!("Parse error: {}", e),
//! }
//! ```
//!
//! ## Features
//!
//! - ✅ Complete IEC 61131-3:2013 support
//! - ✅ All 5 languages (ST, IL, LD, FBD, SFC)
//! - ✅ Functions, function blocks, programs
//! - ✅ Classes and interfaces (OOP)
//! - ✅ Namespaces and using directives
//! - ✅ Detailed error reporting with source locations
//! - ✅ Security limits to prevent DoS attacks
//! 
//! ## Security
//!
//! For untrusted input, use security limits to prevent denial-of-service attacks:
//!
//! ```rust
//! use iec61131::{Parser, security::ParserLimits};
//!
//! let input = "FUNCTION Test : INT\n  VAR x : INT; END_VAR\n  Test := x;\nEND_FUNCTION";
//! 
//! // Use strict limits for untrusted input
//! let limits = ParserLimits::strict();
//! // Check input size before parsing
//! if input.len() > limits.max_input_size {
//!     panic!("Input too large");
//! }
//! 
//! let mut parser = Parser::new(input);
//! let ast = parser.parse()?;
//! # Ok::<(), iec61131::ParseError>(())
//! ```
//! 
//! 
//! ## Analysis Features
//!
//! This crate now includes static analysis features previously available in iecst:
//!
//! ```rust
//! use iec61131::{Parser, analysis::{CfgBuilder, max_nesting_depth}};
//!
//! let code = r#"
//! FUNCTION Example : INT
//!     IF x > 0 THEN
//!         y := 1;
//!     ELSE
//!         y := 2;
//!     END_IF;
//!     Example := y;
//! END_FUNCTION
//! "#;
//! 
//! let mut parser = Parser::new(code);
//! let cu = parser.parse().unwrap();
//! 
//! // Extract function body and analyze
//! if let Some(iec61131::PouDeclaration::Function(func)) = cu.declarations.first() {
//!     let cfg = CfgBuilder::new().build(&func.body);
//!     println!("Cyclomatic complexity: {}", cfg.cyclomatic_complexity());
//!     println!("Nesting depth: {}", max_nesting_depth(&func.body));
//! }
//! ```

// Generated parser components
mod generated;

// Security features
pub mod security;

// Analysis features
pub mod analysis;

// Re-export the main types
pub use generated::ast::{
    CompilationUnit, PouDeclaration, FunctionDecl, FunctionBlockDecl, ProgramDecl, ClassDecl,
    InterfaceDecl, MethodDecl, Statement, Expression, VarDecl, TypeSpec,
    Variable, Argument, StatementList, BinaryOp, UnaryOp, Literal,
};

pub use generated::lexer::{Token, Lexer, Span};
pub use generated::parser::{Parser, ParseError};

// Re-export security types
pub use security::{ParserLimits, ParserState, SecurityError};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_function() {
        let code = r#"
FUNCTION Add : INT
    VAR_INPUT
        a : INT;
        b : INT;
    END_VAR
    
    Add := a + b;
END_FUNCTION
"#;

        let mut parser = Parser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let ast = result.unwrap();
        assert_eq!(ast.declarations.len(), 1);
    }

    #[test]
    fn test_parse_function_block() {
        let code = r#"
FUNCTION_BLOCK Counter
    VAR_INPUT
        reset : BOOL;
    END_VAR
    VAR_OUTPUT
        count : INT;
    END_VAR
    
    IF reset THEN
        count := 0;
    ELSE
        count := count + 1;
    END_IF
END_FUNCTION_BLOCK
"#;

        let mut parser = Parser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    }

    #[test]
    fn test_parse_program() {
        let code = r#"
PROGRAM Main
    VAR
        counter : INT := 0;
    END_VAR
    
    counter := counter + 1;
END_PROGRAM
"#;

        let mut parser = Parser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    }
}

