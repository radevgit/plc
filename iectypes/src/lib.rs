//! IEC 61131-3 Data Types and Literals
//!
//! This crate provides common IEC 61131-3 type definitions used across
//! PLC programming tools.
//!
//! # Features
//!
//! - **Data Types**: Elementary types (BOOL, INT, REAL, STRING, TIME, etc.)
//! - **Literals**: Parsing of IEC literal formats (T#1s, 16#FF, etc.)
//! - **Keywords**: IEC 61131-3 reserved words
//!
//! # Example
//!
//! ```
//! use iectypes::{DataType, TimeLiteral};
//!
//! let dt = DataType::Int;
//! assert_eq!(dt.size_bits(), 16);
//!
//! let time = TimeLiteral::parse("T#1s500ms").unwrap();
//! assert_eq!(time.to_milliseconds(), 1500);
//! ```

mod data_types;
mod keywords;
mod literals;

pub use data_types::*;
pub use keywords::*;
pub use literals::*;
