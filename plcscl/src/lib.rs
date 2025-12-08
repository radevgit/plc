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
/// **v0.1.0 Security Note**: Currently enforces input size validation only.
/// Runtime depth/iteration tracking planned for v0.2.0.
///
/// This function currently checks input size before parsing to prevent
/// memory exhaustion. Additional runtime security limits (depth, iterations,
/// statement count) are defined but not yet integrated into the parser.
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
    // v0.1.0: Check input size only
    // TODO v0.2.0: Integrate ParserState for runtime depth/iteration tracking
    if input.len() > limits.max_input_size {
        return Err(SecureParseError::Security(SecurityError::InputTooLarge {
            size: input.len(),
            limit: limits.max_input_size,
        }));
    }

    let mut parser = Parser::new(input);
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
