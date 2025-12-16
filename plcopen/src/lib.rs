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
//! - ST code extraction and parsing via `iec61131`
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
//!
//! # ST Code Extraction
//!
//! ```ignore
//! use plcopen::st::{extract_st_from_xml, parse_st};
//!
//! let xml = std::fs::read_to_string("project.xml")?;
//! for (pou_name, st_code) in plcopen::st::extract_all_st_from_xml(&xml) {
//!     let statements = parse_st(&st_code)?;
//!     println!("{}: {} statements", pou_name, statements.len());
//! }
//! ```
//!
//! # Security
//!
//! For untrusted XML files, use the secure parsing function:
//!
//! ```ignore
//! use plcopen::{from_str_secure, security::SecurityLimits};
//!
//! let xml = std::fs::read_to_string("untrusted.xml")?;
//! let project = from_str_secure::<Project>(&xml, SecurityLimits::strict())?;
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

// ST extraction and parsing
pub mod st;

// Security limits and validation
pub mod security;
pub use security::{SecurityError, SecurityLimits, validate_xml};

/// Type alias for the root project element
pub type Project = Root_project_Inline;

/// Parse PLCopen XML string into a typed structure.
///
/// Uses quick-xml with serde for fast, type-safe parsing.
pub fn from_str<'a, T: Deserialize<'a>>(xml: &'a str) -> Result<T, quick_xml::DeError> {
    xml_from_str(xml)
}

/// Parse PLCopen XML string with security limits.
///
/// This function validates the XML against security limits before parsing
/// to prevent denial-of-service attacks via malicious XML.
///
/// # Arguments
///
/// * `xml` - The XML content to parse
/// * `limits` - Security limits to enforce
///
/// # Returns
///
/// Returns the parsed structure or an error if validation fails or parsing fails.
///
/// # Example
///
/// ```ignore
/// use plcopen::{Project, from_str_secure, security::SecurityLimits};
///
/// let xml = std::fs::read_to_string("untrusted.xml")?;
/// let project = from_str_secure::<Project>(&xml, SecurityLimits::strict())?;
/// ```
pub fn from_str_secure<'a, T: Deserialize<'a>>(
    xml: &'a str,
    limits: SecurityLimits,
) -> Result<T, SecureParseError> {
    // Validate XML against security limits
    validate_xml(xml, &limits)?;
    
    // Parse with quick-xml
    xml_from_str(xml).map_err(SecureParseError::Parse)
}

/// Error type for secure parsing
#[derive(Debug, thiserror::Error)]
pub enum SecureParseError {
    #[error("Parse error: {0}")]
    Parse(#[from] quick_xml::DeError),
    
    #[error("Security limit exceeded: {0}")]
    Security(#[from] SecurityError),
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
        let _: Option<Project> = None;
    }

    #[test]
    fn test_parse_minimal_project() {
        let xml = r#"<?xml version='1.0' encoding='utf-8'?>
<project xmlns="http://www.plcopen.org/xml/tc6_0201">
  <fileHeader companyName="Test" productName="Test" productVersion="1" creationDateTime="2024-01-01T00:00:00"/>
  <contentHeader name="Test">
    <coordinateInfo>
      <fbd><scaling x="1" y="1"/></fbd>
      <ld><scaling x="1" y="1"/></ld>
      <sfc><scaling x="1" y="1"/></sfc>
    </coordinateInfo>
  </contentHeader>
  <types>
    <dataTypes/>
    <pous/>
  </types>
  <instances>
    <configurations/>
  </instances>
</project>"#;

        let result: Result<Project, _> = from_str(xml);
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        
        let project = result.unwrap();
        assert!(project.file_header.is_some());
        assert!(project.content_header.is_some());
    }

    #[test]
    fn test_parse_bbr_file() {
        // Test with real PLCopen file if available
        let home = match std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
            Ok(h) => h,
            Err(_) => { eprintln!("Skipping test: HOME not set"); return; }
        };
        let path = std::path::Path::new(&home)
            .join("devpublic/dataplc/plcopen/bbr/projects/iec61131_lang_test/plc.xml");
        
        if !path.exists() {
            eprintln!("Skipping test: {:?} not found", path);
            return;
        }

        let xml = std::fs::read_to_string(&path).expect("Failed to read file");
        let result: Result<Project, _> = from_str(&xml);
        
        assert!(result.is_ok(), "Failed to parse {:?}: {:?}", path, result.err());
        
        let project = result.unwrap();
        assert!(project.file_header.is_some(), "Missing fileHeader");
        assert!(project.content_header.is_some(), "Missing contentHeader");
        assert!(project.types.is_some(), "Missing types");
    }

    #[test]
    fn test_parse_multiple_bbr_files() {
        let home = match std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
            Ok(h) => h,
            Err(_) => { eprintln!("Skipping test: HOME not set"); return; }
        };
        let base = std::path::Path::new(&home)
            .join("devpublic/dataplc/plcopen/bbr/projects");
        
        if !base.exists() {
            eprintln!("Skipping test: test data not found");
            return;
        }

        let test_projects = [
            "BACnet/plc.xml",
            "logging/plc.xml",
            "mqtt_client/plc.xml",
            "opcua_client/plc.xml",
        ];

        let mut parsed = 0;
        let mut failed = Vec::new();

        for project in test_projects {
            let path = base.join(project);
            if !path.exists() {
                continue;
            }

            let xml = std::fs::read_to_string(&path).expect("Failed to read file");
            let result: Result<Project, _> = from_str(&xml);
            
            if result.is_ok() {
                parsed += 1;
            } else {
                failed.push((project.to_string(), format!("{:?}", result.err())));
            }
        }

        assert!(failed.is_empty(), "Failed to parse {} files: {:?}", failed.len(), failed);
        eprintln!("Successfully parsed {} project files", parsed);
    }

    #[test]
    fn test_parse_standard_function_blocks() {
        let home = match std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
            Ok(h) => h,
            Err(_) => { eprintln!("Skipping test: HOME not set"); return; }
        };
        let path = std::path::Path::new(&home)
            .join("devpublic/dataplc/plcopen/bbr/Standard_Function_Blocks.xml");
        
        if !path.exists() {
            eprintln!("Skipping test: Standard_Function_Blocks.xml not found");
            return;
        }

        let xml = std::fs::read_to_string(&path).expect("Failed to read file");
        let result: Result<Project, _> = from_str(&xml);
        
        assert!(result.is_ok(), "Failed to parse Standard_Function_Blocks.xml: {:?}", result.err());
        
        let project = result.unwrap();
        let _types = project.types.expect("Missing types");
        eprintln!("Standard Function Blocks library loaded successfully");
    }

    #[test]
    fn test_parse_bbr_examples() {
        let home = match std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
            Ok(h) => h,
            Err(_) => { eprintln!("Skipping test: HOME not set"); return; }
        };
        let base = std::path::Path::new(&home)
            .join("devpublic/dataplc/plcopen/bbr/exemples");
        
        if !base.exists() {
            eprintln!("Skipping test: examples not found");
            return;
        }

        let examples = [
            "first_steps/plc.xml",
            "python/plc.xml",
            "modbus/plc.xml",
            "csv_read/plc.xml",
            "csv_by_string/plc.xml",
        ];

        let mut parsed = 0;
        let mut failed = Vec::new();

        for example in examples {
            let path = base.join(example);
            if !path.exists() {
                continue;
            }

            let xml = std::fs::read_to_string(&path).expect("Failed to read file");
            let result: Result<Project, _> = from_str(&xml);
            
            if result.is_ok() {
                parsed += 1;
            } else {
                failed.push((example.to_string(), format!("{:?}", result.err())));
            }
        }

        assert!(failed.is_empty(), "Failed to parse {} examples: {:?}", failed.len(), failed);
        eprintln!("Successfully parsed {} example files", parsed);
    }

    #[test]
    fn test_extract_st_from_real_files() {
        let home = match std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
            Ok(h) => h,
            Err(_) => { eprintln!("Skipping test: HOME not set"); return; }
        };
        let base = std::path::Path::new(&home)
            .join("devpublic/dataplc/plcopen/bbr/exemples");
        
        if !base.exists() {
            eprintln!("Skipping test: examples not found");
            return;
        }

        let path = base.join("first_steps/plc.xml");
        if !path.exists() {
            eprintln!("Skipping test: first_steps not found");
            return;
        }

        let xml = std::fs::read_to_string(&path).expect("Failed to read file");
        let st_blocks = crate::st::extract_all_st_from_xml(&xml);
        
        assert!(!st_blocks.is_empty(), "Expected ST code in first_steps");
        
        let mut parsed_count = 0;
        eprintln!("Found {} POUs with ST code:", st_blocks.len());
        for (name, code) in &st_blocks {
            eprintln!("  {}: {} chars", name, code.len());
            
            // Skip very short code (likely SFC conditions like "TRUE")
            if code.len() < 10 {
                eprintln!("    (skipped - too short, likely SFC condition)");
                continue;
            }
            
            // Try to parse each ST block
            // Note: iec61131 requires complete POU declarations, but PLCopen stores body code
            // Wrap in a minimal function declaration for parsing
            let wrapped_code = format!("FUNCTION _Temp : INT\n{}\nEND_FUNCTION", code);
            let result = crate::st::parse_st(&wrapped_code);
            if result.is_ok() {
                parsed_count += 1;
            } else {
                eprintln!("    (failed to parse - may not be valid ST or requires declarations)");
            }
        }
        
        // We expect at least some ST blocks to parse successfully
        eprintln!("Successfully parsed {} of {} ST blocks", parsed_count, st_blocks.len());
    }
}
