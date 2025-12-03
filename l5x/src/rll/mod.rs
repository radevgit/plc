//! RLL (Relay Ladder Logic) text parser for Rockwell L5X files.
//!
//! Parses the text content of ladder logic rungs into a structured AST.
//!
//! # Example
//!
//! ```
//! use l5x::rll::{parse_rung, RungElement, Operand};
//!
//! let text = "XIC(Start)OTE(Motor);";
//! let result = parse_rung(text);
//! assert!(result.is_parsed());
//! ```
//!
//! # Error Reporting
//!
//! When parsing fails, detailed error context is available:
//!
//! ```
//! use l5x::rll::{parse_rung, ParseError, ErrorContext};
//!
//! let text = "XIC(Tag[incomplete;";
//! let result = parse_rung(text);
//!
//! if let Some(err) = result.error {
//!     // Format error with source context
//!     let formatted = err.format_with_context(text);
//!     // Shows: error: unclosed bracket '[' at position 7
//!     //        1 | XIC(Tag[incomplete;
//!     //                  ^ here
//! }
//! ```

mod ast;
mod error;
mod operand;
mod parser;

pub use ast::*;
pub use error::*;
pub use operand::*;
pub use parser::parse_rung;
