//! L5X file parser and utilities for PLC programs.
//!
//! This crate provides functionality for parsing, manipulating, and generating
//! L5X files exported from PLC programming software.
//!
//! # Parsing Approaches
//!
//! ## quick-xml (100% compatibility)
//! Fast, type-safe parsing using serde and generated structs.
//!
//! ## RLL (Ladder Logic) Parsing
//! Parse rung text into structured AST for analysis.
//!
//! # Example
//!
//! ```ignore
//! use l5x::Project;
//!
//! // Parse with quick-xml (fast, typed)
//! let xml = std::fs::read_to_string("project.L5X")?;
//! let project: Project = l5x::from_str(&xml)?;
//! println!("Controller: {:?}", project.controller);
//!
//! // Parse ladder logic rungs
//! use l5x::rll::parse_rung;
//! let rung = parse_rung("XIC(Start)OTE(Motor);");
//! let tags = rung.tag_references();
//!
//! // Analyze entire project
//! use l5x::analysis::analyze_controller;
//! let analysis = analyze_controller(&project.controller);
//! println!("Found {} unique tags", analysis.stats.unique_tags);
//! ```

#![allow(dead_code)]
#![allow(non_camel_case_types)]

use quick_xml::de::from_str as xml_from_str;
use quick_xml::se::to_string as xml_to_string;
use serde::{Deserialize, Serialize};

// RLL (Relay Ladder Logic) parser
pub mod rll;

// Project analysis (L5X + RLL integration)
pub mod analysis;

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
