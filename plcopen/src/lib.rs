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
//! - ST code extraction and parsing via `iecst`
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

/// Type alias for the root project element
pub type Project = Root_project_Inline;

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
        let path = std::path::Path::new(env!("HOME"))
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
        let base = std::path::Path::new(env!("HOME"))
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
        let path = std::path::Path::new(env!("HOME"))
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
        let base = std::path::Path::new(env!("HOME"))
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
        let base = std::path::Path::new(env!("HOME"))
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
            
            // Parse each ST block
            let result = crate::st::parse_st(code);
            assert!(result.is_ok(), "Failed to parse ST in {}: {:?}\nCode: {}", name, result.err(), code);
            parsed_count += 1;
        }
        
        assert!(parsed_count > 0, "Expected at least one parseable ST block");
    }
}
