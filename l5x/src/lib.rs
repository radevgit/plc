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
use serde::{de::DeserializeOwned, Deserialize, Serialize};

// Security validation and limits
pub mod security;
pub use security::{SecurityLimits, SecurityError, validate_xml};

// RLL (Relay Ladder Logic) parser
pub mod rll;

// Include pre-generated types (no build.rs needed)
#[path = "../generated/generated.rs"]
mod generated;
pub use generated::*;

/// Parse L5X XML string into a typed structure.
///
/// Uses quick-xml with serde for fast, type-safe parsing.
///
/// Encrypted AOI content in Rockwell-protected files (long base64 text nodes
/// between sibling elements) is automatically stripped before parsing, since
/// it is opaque and not usable without the encryption key.
pub fn from_str<T: DeserializeOwned>(xml: &str) -> Result<T, quick_xml::DeError> {
    match strip_encrypted_text_nodes(xml) {
        Some(processed) => xml_from_str(&processed),
        None => xml_from_str(xml),
    }
}

/// Parse L5X XML string with security validation.
///
/// Validates input against security limits before parsing to protect
/// against XML bombs, entity expansion, and other attacks.
///
/// # Example
/// ```ignore
/// use l5x::{Project, SecurityLimits, from_str_secure};
///
/// let xml = std::fs::read_to_string("project.L5X")?;
/// let project: Project = from_str_secure(&xml, &SecurityLimits::strict())?;
/// ```
pub fn from_str_secure<T: DeserializeOwned>(
    xml: &str,
    limits: &SecurityLimits,
) -> Result<T, SecureParseError> {
    // First validate against security limits
    validate_xml(xml.as_bytes(), limits)?;

    // Then parse, stripping any encrypted blobs
    match strip_encrypted_text_nodes(xml) {
        Some(processed) => xml_from_str(&processed).map_err(SecureParseError::ParseError),
        None => xml_from_str(xml).map_err(SecureParseError::ParseError),
    }
}

/// Combined error type for secure parsing
#[derive(Debug, thiserror::Error)]
pub enum SecureParseError {
    #[error("Security validation failed: {0}")]
    SecurityError(#[from] SecurityError),
    
    #[error("XML parsing failed: {0}")]
    ParseError(quick_xml::DeError),
}

/// Serialize a structure to an L5X XML string.
///
/// The output does not include an XML declaration header. If you need one,
/// prepend it manually:
///
/// ```ignore
/// use l5x::{from_str, to_string, Project};
///
/// let xml = std::fs::read_to_string("project.L5X")?;
/// let project: Project = from_str(&xml)?;
///
/// // Modify the project here...
///
/// let output = format!(
///     "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n{}",
///     to_string(&project)?
/// );
/// std::fs::write("modified.L5X", output)?;
/// ```
pub fn to_string<T: Serialize>(value: &T) -> Result<String, quick_xml::SeError> {
    xml_to_string(value)
}

/// Strip encrypted blob text nodes from L5X XML before parsing.
///
/// Some Rockwell L5X files contain encrypted AOI definitions where the protected
/// content appears as a long base64-like text node between sibling child elements.
/// quick-xml's overlapped-lists mode routes these text events to the adjacent Vec
/// field deserializer rather than the `$text` field, causing a parse error.
///
/// Since the blobs are encrypted with a Rockwell-proprietary key and are not
/// usable without it, stripping them is safe. Returns `Some(processed)` if any
/// blobs were found (to avoid unnecessary allocation for the common case).
fn strip_encrypted_text_nodes(xml: &str) -> Option<String> {
    let mut result = String::with_capacity(xml.len());
    let mut modified = false;
    let mut chunk_start = 0usize;
    let bytes = xml.as_bytes();
    let mut i = 0usize;

    while i < bytes.len() {
        if bytes[i] == b'>' {
            let text_start = i + 1;
            let mut j = text_start;
            while j < bytes.len() && bytes[j] != b'<' {
                j += 1;
            }
            if j > text_start {
                let text_node = &xml[text_start..j];
                let trimmed = text_node.trim();
                if trimmed.len() > 100 && trimmed.bytes().all(is_base64_char) {
                    result.push_str(&xml[chunk_start..text_start]);
                    chunk_start = j;
                    modified = true;
                }
            }
            i = j;
        } else {
            i += 1;
        }
    }

    if modified {
        result.push_str(&xml[chunk_start..]);
        Some(result)
    } else {
        None
    }
}

#[inline]
fn is_base64_char(b: u8) -> bool {
    matches!(b, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'+' | b'/' | b'=' | b'\n' | b'\r' | b' ' | b'\t')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_node_between_elements() {
        // Rockwell puts encrypted text between sibling elements in protected files.
        // The parser should not error on unexpected text content.
        let xml = r#"<RSLogix5000Content SchemaRevision="1.0" SoftwareRevision="20.04">
some encrypted blob here
</RSLogix5000Content>"#;
        from_str::<Project>(xml).unwrap();
    }
}
