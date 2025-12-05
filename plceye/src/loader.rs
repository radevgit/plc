//! Unified project loading from multiple formats.
//!
//! This module provides format detection and loading for L5X and PLCopen files.

use std::path::Path;

use crate::error::{Error, L5xParseErrorKind, Result};

/// Detected file format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFormat {
    /// Rockwell L5X format
    L5x,
    /// PLCopen XML format
    PlcOpen,
}

impl std::fmt::Display for FileFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileFormat::L5x => write!(f, "L5X"),
            FileFormat::PlcOpen => write!(f, "PLCopen"),
        }
    }
}

impl FileFormat {
    /// Detect format from file extension.
    pub fn from_extension(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?.to_lowercase();
        match ext.as_str() {
            "l5x" | "l5k" => Some(FileFormat::L5x),
            "xml" => None, // Need content inspection for .xml
            _ => None,
        }
    }

    /// Detect format from content.
    pub fn from_content(content: &str) -> Option<Self> {
        // Check for L5X signature
        if content.contains("RSLogix5000Content") || content.contains("<Controller") {
            return Some(FileFormat::L5x);
        }
        
        // Check for PLCopen signature
        if content.contains("plcopen.org") || content.contains("<project") {
            return Some(FileFormat::PlcOpen);
        }
        
        None
    }
    
    /// Detect format, preferring extension, falling back to content.
    pub fn detect(path: &Path, content: &str) -> Option<Self> {
        Self::from_extension(path).or_else(|| Self::from_content(content))
    }
}

/// A loaded project with its format-specific data.
pub struct LoadedProject {
    /// The L5X controller (for L5X files)
    pub l5x_controller: Option<l5x::Controller>,
    
    /// The PLCopen project (for PLCopen files)
    pub plcopen_project: Option<plcopen::Project>,
    
    /// Detected format
    pub format: FileFormat,
    
    /// Source file path (if loaded from file)
    pub source_path: Option<String>,
}

impl LoadedProject {
    /// Load a project from file.
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| Error::FileRead {
            path: path.display().to_string(),
            source: e,
        })?;
        
        let mut project = Self::from_str(&content, Some(path))?;
        project.source_path = Some(path.display().to_string());
        Ok(project)
    }
    
    /// Load a project from string content.
    pub fn from_str(content: &str, path: Option<&Path>) -> Result<Self> {
        let format = path
            .and_then(|p| FileFormat::detect(p, content))
            .or_else(|| FileFormat::from_content(content))
            .ok_or(Error::L5xParse {
                kind: L5xParseErrorKind::XmlDeserialize,
            })?;
        
        match format {
            FileFormat::L5x => Self::load_l5x(content),
            FileFormat::PlcOpen => Self::load_plcopen(content),
        }
    }
    
    fn load_l5x(content: &str) -> Result<Self> {
        let project: l5x::Project = quick_xml::de::from_str(content)
            .map_err(|_| Error::L5xParse {
                kind: L5xParseErrorKind::XmlDeserialize,
            })?;
        
        Ok(LoadedProject {
            l5x_controller: project.controller,
            plcopen_project: None,
            format: FileFormat::L5x,
            source_path: None,
        })
    }
    
    fn load_plcopen(content: &str) -> Result<Self> {
        let project: plcopen::Project = plcopen::from_str(content)
            .map_err(|_| Error::L5xParse {
                kind: L5xParseErrorKind::XmlDeserialize,
            })?;
        
        Ok(LoadedProject {
            l5x_controller: None,
            plcopen_project: Some(project),
            format: FileFormat::PlcOpen,
            source_path: None,
        })
    }
    
    /// Check if this is an L5X file.
    pub fn is_l5x(&self) -> bool {
        self.l5x_controller.is_some()
    }
    
    /// Check if this is a PLCopen file.
    pub fn is_plcopen(&self) -> bool {
        self.plcopen_project.is_some()
    }
    
    /// Get the project name.
    pub fn name(&self) -> String {
        if let Some(ref c) = self.l5x_controller {
            return c.name.clone();
        }
        if let Some(ref p) = self.plcopen_project {
            if let Some(ref header) = p.content_header {
                return header.name.clone();
            }
        }
        "Unknown".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_detection_extension() {
        assert_eq!(
            FileFormat::from_extension(Path::new("test.L5X")),
            Some(FileFormat::L5x)
        );
        assert_eq!(
            FileFormat::from_extension(Path::new("test.l5x")),
            Some(FileFormat::L5x)
        );
        assert_eq!(
            FileFormat::from_extension(Path::new("test.xml")),
            None // Need content inspection
        );
    }

    #[test]
    fn test_format_detection_content() {
        let l5x = r#"<RSLogix5000Content><Controller Name="Test"/></RSLogix5000Content>"#;
        assert_eq!(FileFormat::from_content(l5x), Some(FileFormat::L5x));
        
        let plcopen = r#"<project xmlns="http://www.plcopen.org/xml/tc6_0200">"#;
        assert_eq!(FileFormat::from_content(plcopen), Some(FileFormat::PlcOpen));
        
        let unknown = r#"<something>else</something>"#;
        assert_eq!(FileFormat::from_content(unknown), None);
    }

    #[test]
    fn test_load_l5x() {
        let xml = r#"<?xml version="1.0"?>
        <RSLogix5000Content SchemaRevision="1.0" SoftwareRevision="32.00">
            <Controller Name="TestController">
                <Programs>
                    <Program Name="MainProgram"/>
                </Programs>
            </Controller>
        </RSLogix5000Content>"#;
        
        let loaded = LoadedProject::from_str(xml, None).expect("Should parse");
        assert!(loaded.is_l5x());
        assert_eq!(loaded.format, FileFormat::L5x);
        assert_eq!(loaded.name(), "TestController");
    }

    #[test]
    fn test_load_plcopen() {
        let xml = r#"<?xml version="1.0"?>
        <project xmlns="http://www.plcopen.org/xml/tc6_0200">
            <fileHeader companyName="Test" productName="TestProject" productVersion="1.0" creationDateTime="2024-01-01T00:00:00"/>
            <contentHeader name="Test"/>
            <types>
                <pous>
                    <pou name="Main" pouType="program"/>
                </pous>
            </types>
        </project>"#;
        
        let loaded = LoadedProject::from_str(xml, None).expect("Should parse");
        assert!(!loaded.is_l5x());
        assert!(loaded.is_plcopen());
        assert_eq!(loaded.format, FileFormat::PlcOpen);
        assert_eq!(loaded.name(), "Test");
    }
}
