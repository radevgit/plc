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
//!
//! ## Security
//!
//! For untrusted input, use the `parse_scl_secure` function with appropriate limits:
//!
//! ```ignore
//! use plcscl::{parse_scl_secure, security::ParserLimits};
//!
//! let source = get_untrusted_input();
//! match parse_scl_secure(source, ParserLimits::strict()) {
//!     Ok(program) => println!("Parsed successfully!"),
//!     Err(e) => eprintln!("Parse error: {}", e),
//! }
//! ```

pub mod generated;
pub mod security;

pub use generated::*;
pub use security::{ParserLimits, ParserState, SecurityError};

/// Parse SCL source code into an AST
pub fn parse_scl(input: &str) -> Result<Program, ParseError> {
    let mut parser = Parser::new(input);
    parser.parse_program()
}

/// Parse SCL source code with security limits to prevent DoS attacks
///
/// This function enforces security limits during parsing to prevent
/// denial-of-service attacks via malicious SCL code.
///
/// The generated parser includes built-in runtime checks for:
/// - Input size (checked before parsing)
/// - Recursion depth (checked during block parsing)
/// - Complexity/node count (checked during AST construction)
/// - Iteration count (checked during statement parsing)
/// - Collection sizes (checked for Vec allocations)
///
/// # Arguments
///
/// * `input` - SCL source code to parse
/// * `limits` - Security limits (only `max_input_size` enforced in v0.1.0)
///
/// # Returns
///
/// Returns the parsed AST or an error if parsing fails or limits are exceeded.
///
/// # Example
///
/// ```ignore
/// use plcscl::{parse_scl_secure, security::ParserLimits};
///
/// let source = r#"FUNCTION_BLOCK MyFB ... END_FUNCTION_BLOCK"#;
/// match parse_scl_secure(source, ParserLimits::strict()) {
///     Ok(program) => println!("Parsed successfully!"),
///     Err(e) => eprintln!("Parse error: {}", e),
/// }
/// ```
pub fn parse_scl_secure(input: &str, limits: ParserLimits) -> Result<Program, SecureParseError> {
    // Check input size first (quick check before tokenization)
    if input.len() > limits.max_input_size {
        return Err(SecureParseError::Security(SecurityError::InputTooLarge {
            size: input.len(),
            limit: limits.max_input_size,
        }));
    }

    // Convert to internal parser limits and parse with security checks
    let parser_limits = limits.to_parser_limits();
    let mut parser = Parser::with_limits(input, parser_limits);
    parser.parse_program()
        .map_err(|e| SecureParseError::Parse(e.message().to_string()))
}

/// Error type for secure parsing that can be either a parse error or security violation
#[derive(Debug, thiserror::Error)]
pub enum SecureParseError {
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Security limit exceeded: {0}")]
    Security(#[from] SecurityError),
}
