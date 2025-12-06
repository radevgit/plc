//! IEC 61131-3 Structured Text parser.
//!
//! This crate provides a parser for IEC 61131-3 Structured Text (ST) language,
//! used in PLC programming.
//!
//! # Example
//!
//! ```
//! use iecst::parse_statement;
//!
//! let code = "x := 1 + 2;";
//! let result = parse_statement(code);
//! assert!(result.is_ok());
//! ```
//!
//! # Supported Constructs
//!
//! ## Expressions
//! - Literals: integers, reals, strings, booleans, time literals
//! - Identifiers and qualified names
//! - Binary operators: +, -, *, /, MOD, **, AND, OR, XOR, NOT, comparisons
//! - Unary operators: -, NOT
//! - Function calls: `SIN(x)`, `MAX(a, b)`
//! - Array indexing: `arr[i]`, `arr[i, j]`
//! - Member access: `struct.field`
//!
//! ## Statements
//! - Assignment: `x := expr;`
//! - IF/THEN/ELSIF/ELSE/END_IF
//! - CASE/OF/ELSE/END_CASE
//! - FOR/TO/BY/DO/END_FOR
//! - WHILE/DO/END_WHILE
//! - REPEAT/UNTIL/END_REPEAT
//! - EXIT, RETURN, CONTINUE
//! - Function/FB calls
//!
//! ## Program Organization Units (POUs)
//! - PROGRAM/END_PROGRAM
//! - FUNCTION/END_FUNCTION
//! - FUNCTION_BLOCK/END_FUNCTION_BLOCK
//!
//! ## Declarations
//! - VAR/VAR_INPUT/VAR_OUTPUT/VAR_IN_OUT/VAR_TEMP/VAR_GLOBAL
//! - TYPE/END_TYPE (structures, enums, arrays, subranges)

pub mod analysis;
mod ast;
mod error;
mod lexer;
mod parser;
mod span;

pub use ast::*;
pub use error::{ParseError, ParseErrorKind, ParseResult};
pub use parser::{parse_expression, parse_statement, parse_statements, parse_pou, parse_type_block};
pub use span::Span;

// Re-export key analysis types
pub use analysis::{analyze_pou, Diagnostic, DiagnosticKind, Severity, Type};
pub use analysis::{Cfg, CfgBuilder, CfgNode, NodeId, NodeKind};
