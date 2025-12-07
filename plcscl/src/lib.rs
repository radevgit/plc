//! # plcscl
//!
//! Parser and AST for Siemens SCL (Structured Control Language).
//!
//! SCL is Siemens' implementation of IEC 61131-3 Structured Text (ST) with
//! proprietary extensions for S7-300/400/1200/1500 PLCs.
//!
//! ## Example
//!
//! ```ignore
//! use plcscl::parse_scl;
//!
//! let source = r#"
//! FUNCTION_BLOCK MyFB
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
//! match parse_scl(source) {
//!     Ok(program) => println!("Parsed successfully!"),
//!     Err(e) => eprintln!("Parse error: {}", e.message),
//! }
//! ```

pub mod generated;

pub use generated::*;

/// Parse SCL source code into an AST
pub fn parse_scl(input: &str) -> Result<Program, ParseError> {
    let mut parser = Parser::new(input);
    parser.parse_program()
}
