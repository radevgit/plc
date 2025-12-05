//! L5X file parser for PLC programs.
//!
//! This crate provides functionality for parsing L5X files exported from
//! Rockwell Automation Studio 5000 Logix Designer.
//!
//! # Parsing Features
//!
//! - Fast, type-safe parsing using quick-xml and serde
//! - Generated types from the official L5X XSD schema
//! - RLL (Relay Ladder Logic) instruction parsing
//!
//! # Example
//!
//! ```ignore
//! use l5x::Project;
//!
//! // Parse L5X file
//! let xml = std::fs::read_to_string("project.L5X")?;
//! let project: Project = l5x::from_str(&xml)?;
//! println!("Controller: {:?}", project.controller);
//!
//! // Parse ladder logic rungs
//! use l5x::rll::parse_rung;
//! let rung = parse_rung("XIC(Start)OTE(Motor);");
//! let tags = rung.tag_references();
//! ```

#![allow(dead_code)]
#![allow(non_camel_case_types)]

use quick_xml::de::from_str as xml_from_str;
use quick_xml::se::to_string as xml_to_string;
use serde::{Deserialize, Serialize};

// RLL (Relay Ladder Logic) parser
pub mod rll;

// Conversion to plcmodel (when feature enabled)
#[cfg(feature = "model")]
pub mod to_model;

// Include pre-generated types (no build.rs needed)
#[path = "../generated/generated.rs"]
mod generated;
pub use generated::*;

/// Parse L5X XML string into a typed structure.
///
/// Uses quick-xml with serde for fast, type-safe parsing.
pub fn from_str<'a, T: Deserialize<'a>>(xml: &'a str) -> Result<T, quick_xml::DeError> {
    xml_from_str(xml)
}

/// Serialize a structure to L5X XML string.
pub fn to_string<T: Serialize>(value: &T) -> Result<String, quick_xml::SeError> {
    xml_to_string(value)
}
