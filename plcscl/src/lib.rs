//! # plcscl
//!
//! Parser and AST for Siemens SCL (Structured Control Language).
//!
//! SCL is Siemens' implementation of IEC 61131-3 Structured Text (ST) with
//! proprietary extensions for S7-300/400/1200/1500 PLCs.
//!
//! ## Features
//!
//! - Lexer for SCL tokens
//! - Parser for SCL programs
//! - AST representation
//! - Support for Siemens-specific extensions:
//!   - Pragmas (`{S7_Optimized_Access := 'TRUE'}`)
//!   - Absolute addressing (`%I0.0`, `DB10.DBW0`)
//!   - Regions (`REGION..END_REGION`)
//!   - Data blocks (DB, FB, FC)
//!
//! ## Example
//!
//! ```ignore
//! use plcscl::{Lexer, Parser};
//!
//! let source = r#"
//! FUNCTION_BLOCK "MyFB"
//! { S7_Optimized_Access := 'TRUE' }
//! VAR_INPUT
//!     setpoint : REAL;
//! END_VAR
//! VAR_OUTPUT
//!     output : REAL;
//! END_VAR
//! BEGIN
//!     output := setpoint * 2.0;
//! END_FUNCTION_BLOCK
//! "#;
//!
//! let lexer = Lexer::new(source);
//! let parser = Parser::new(lexer);
//! let ast = parser.parse();
//! ```

pub mod ast;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod span;

pub use ast::*;
pub use error::{Error, Result};
pub use lexer::Lexer;
pub use parser::Parser;
pub use span::Span;
