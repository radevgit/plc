//! PLCopen TC6 XML parser for PLC programs.
//!
//! This crate provides functionality for parsing PLCopen TC6 XML files,
//! the IEC 61131-3 standard exchange format for PLC programs.
//!
//! # Features
//!
//! - Fast, type-safe parsing using quick-xml and serde
//! - Generated types from the official PLCopen TC6 XML schema (v2.01)
//! - Support for all IEC 61131-3 languages (ST, IL, LD, FBD, SFC)
//!
//! # Example
//!
//! ```ignore
//! use plcopen::Project;
//!
//! // Parse PLCopen XML file
//! let xml = std::fs::read_to_string("project.xml")?;
//! let project: Project = plcopen::from_str(&xml)?;
//! println!("Project: {:?}", project);
//! ```

#![allow(dead_code)]
#![allow(non_camel_case_types)]

use quick_xml::de::from_str as xml_from_str;
use quick_xml::se::to_string as xml_to_string;
use serde::{Deserialize, Serialize};

// Include pre-generated types (no build.rs needed)
#[path = "../generated/generated.rs"]
mod generated;
pub use generated::*;

/// Parse PLCopen XML string into a typed structure.
///
/// Uses quick-xml with serde for fast, type-safe parsing.
pub fn from_str<'a, T: Deserialize<'a>>(xml: &'a str) -> Result<T, quick_xml::DeError> {
    xml_from_str(xml)
}

/// Serialize a structure to PLCopen XML string.
pub fn to_string<T: Serialize>(value: &T) -> Result<String, quick_xml::SeError> {
    xml_to_string(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generated_types_exist() {
        // Basic test to verify generated code compiles
        // Verify key types exist
        let _: Option<ElementaryTypes> = None;
        let _: Option<DerivedTypes> = None;
    }
}
