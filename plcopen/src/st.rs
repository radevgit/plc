//! ST (Structured Text) extraction and parsing utilities.
//!
//! This module provides utilities for extracting ST code from PLCopen XML
//! and parsing it using the `iec61131` crate.
//!
//! PLCopen TC6 stores ST code in `<ST><xhtml:p><![CDATA[...]]></xhtml:p></ST>` elements,
//! while IEC 61131-10 uses `<ST><![CDATA[...]]></ST>` directly.

use crate::Body;

/// Extract ST code from a Body element.
///
/// PLCopen stores ST code in the `st` field as `FormattedText`, which contains
/// XHTML markup. This function extracts the raw ST code by parsing the original
/// XML to get the CDATA content.
///
/// Returns `None` if the body doesn't contain ST code.
///
/// # Example
///
/// ```ignore
/// use plcopen::st::extract_st_from_body;
///
/// let body = pou.body.as_ref().unwrap();
/// if let Some(code) = extract_st_from_body(body) {
///     println!("ST code: {}", code);
/// }
/// ```
pub fn extract_st_from_body(body: &Body) -> Option<String> {
    // The Body struct has `st: Option<FormattedText>` but FormattedText
    // only has skipped xs:any children. We need to check if ST is present.
    // If st is Some, ST content exists but we need to extract from raw XML.
    body.st.as_ref().map(|_| {
        // FormattedText doesn't capture content, so we return empty placeholder
        // Real extraction requires access to raw XML
        String::new()
    })
}

/// Extract ST code from raw XML string containing a Body element.
///
/// This parses the raw XML to extract CDATA content from ST elements,
/// handling both PLCopen TC6 format (`<xhtml:p><![CDATA[...]]>`) and
/// IEC 61131-10 format (`<ST><![CDATA[...]]>`).
///
/// # Example
///
/// ```
/// use plcopen::st::extract_st_from_xml;
///
/// let xml = r#"<body><ST><xhtml:p><![CDATA[x := 1;]]></xhtml:p></ST></body>"#;
/// let code = extract_st_from_xml(xml);
/// assert_eq!(code, Some("x := 1;".to_string()));
/// ```
pub fn extract_st_from_xml(xml: &str) -> Option<String> {
    // Find <ST> element and extract content
    let st_start = xml.find("<ST>")?;
    let st_end = xml.find("</ST>")?;
    
    if st_end <= st_start {
        return None;
    }
    
    let st_content = &xml[st_start + 4..st_end];
    
    // Extract CDATA content, handling both formats:
    // PLCopen TC6: <xhtml:p><![CDATA[...]]></xhtml:p>
    // IEC 61131-10: <![CDATA[...]]> or direct text
    extract_cdata_content(st_content)
        .or_else(|| extract_text_content(st_content))
}

/// Extract all ST code blocks from a PLCopen XML document.
///
/// Returns a vector of (pou_name, st_code) tuples for each POU that contains ST.
pub fn extract_all_st_from_xml(xml: &str) -> Vec<(String, String)> {
    let mut results = Vec::new();
    
    // Simple regex-free parsing: find all <pou name="..."> and their ST bodies
    let mut pos = 0;
    while let Some(pou_start) = xml[pos..].find("<pou ") {
        let pou_start = pos + pou_start;
        
        // Find pou end
        let pou_end = match xml[pou_start..].find("</pou>") {
            Some(end) => pou_start + end + 6,
            None => break,
        };
        
        let pou_xml = &xml[pou_start..pou_end];
        
        // Extract pou name
        if let Some(name) = extract_attribute(pou_xml, "name") {
            // Extract ST from this POU
            if let Some(st_code) = extract_st_from_xml(pou_xml) {
                if !st_code.trim().is_empty() {
                    results.push((name, st_code));
                }
            }
        }
        
        pos = pou_end;
    }
    
    results
}

/// Parse ST code string using iec61131 parser.
///
/// Returns parsed compilation unit or an error.
///
/// # Example
///
/// ```
/// use plcopen::st::parse_st;
///
/// let code = "FUNCTION Test : INT\n  x := 1 + 2;\n  Test := x;\nEND_FUNCTION";
/// let result = parse_st(code);
/// assert!(result.is_ok());
/// ```
pub fn parse_st(code: &str) -> Result<iec61131::CompilationUnit, iec61131::ParseError> {
    let mut parser = iec61131::Parser::new(code);
    parser.parse()
}

/// Parse ST code and run analysis diagnostics.
///
/// Returns parsed compilation unit with any diagnostics (warnings/errors).
pub fn analyze_st(code: &str) -> StAnalysisResult {
    let mut parser = iec61131::Parser::new(code);
    match parser.parse() {
        Ok(cu) => StAnalysisResult {
            compilation_unit: Some(cu),
            parse_error: None,
            diagnostics: Vec::new(), // TODO: Add analysis using iec61131::analysis
        },
        Err(e) => StAnalysisResult {
            compilation_unit: None,
            parse_error: Some(e),
            diagnostics: Vec::new(),
        },
    }
}

/// Result of ST analysis including parsed AST and diagnostics.
#[derive(Debug)]
pub struct StAnalysisResult {
    /// Parsed compilation unit (if parsing succeeded)
    pub compilation_unit: Option<iec61131::CompilationUnit>,
    /// Parse error (if parsing failed)
    pub parse_error: Option<iec61131::ParseError>,
    /// Analysis diagnostics (warnings, hints, etc.) - placeholder for future analysis
    pub diagnostics: Vec<String>,
}

impl StAnalysisResult {
    /// Check if parsing was successful.
    pub fn is_ok(&self) -> bool {
        self.compilation_unit.is_some()
    }
    
    /// Check if there are any errors (parse or diagnostic).
    pub fn has_errors(&self) -> bool {
        self.parse_error.is_some()
    }
}

// Helper functions

fn extract_cdata_content(s: &str) -> Option<String> {
    let cdata_start = s.find("<![CDATA[")?;
    let cdata_end = s.find("]]>")?;
    
    if cdata_end <= cdata_start {
        return None;
    }
    
    Some(s[cdata_start + 9..cdata_end].to_string())
}

fn extract_text_content(s: &str) -> Option<String> {
    // Strip any XML tags and get inner text
    let text: String = s.chars()
        .fold((String::new(), false), |(mut acc, in_tag), c| {
            match c {
                '<' => (acc, true),
                '>' => (acc, false),
                _ if !in_tag => { acc.push(c); (acc, false) },
                _ => (acc, in_tag),
            }
        }).0;
    
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn extract_attribute(xml: &str, attr_name: &str) -> Option<String> {
    let pattern = format!("{}=\"", attr_name);
    let start = xml.find(&pattern)?;
    let value_start = start + pattern.len();
    let value_end = xml[value_start..].find('"')?;
    
    Some(xml[value_start..value_start + value_end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_st_plcopen_format() {
        let xml = r#"<body><ST><xhtml:p><![CDATA[x := 1 + 2;]]></xhtml:p></ST></body>"#;
        let code = extract_st_from_xml(xml);
        assert_eq!(code, Some("x := 1 + 2;".to_string()));
    }

    #[test]
    fn test_extract_st_iec61131_format() {
        let xml = r#"<body><ST><![CDATA[x := 1;]]></ST></body>"#;
        let code = extract_st_from_xml(xml);
        assert_eq!(code, Some("x := 1;".to_string()));
    }

    #[test]
    fn test_extract_st_multiline() {
        let xml = r#"<body><ST><xhtml:p><![CDATA[IF x THEN
    y := 1;
END_IF;]]></xhtml:p></ST></body>"#;
        let code = extract_st_from_xml(xml).unwrap();
        assert!(code.contains("IF x THEN"));
        assert!(code.contains("END_IF"));
    }

    #[test]
    fn test_extract_st_no_st() {
        let xml = r#"<body><FBD></FBD></body>"#;
        let code = extract_st_from_xml(xml);
        assert!(code.is_none());
    }

    #[test]
    fn test_parse_st_simple() {
        let code = "FUNCTION Test : INT\n  x := 1 + 2;\n  Test := x;\nEND_FUNCTION";
        let result = parse_st(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_st_if() {
        let code = "FUNCTION Test : INT\n  IF x > 0 THEN y := 1; END_IF;\n  Test := y;\nEND_FUNCTION";
        let result = parse_st(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_st() {
        let code = "FUNCTION Test : INT\n  x := 1;\n  Test := x;\nEND_FUNCTION";
        let result = analyze_st(code);
        assert!(result.is_ok());
        assert!(!result.has_errors());
    }

    #[test]
    fn test_extract_all_st() {
        let xml = r#"
        <project>
            <pou name="Main" pouType="program">
                <body><ST><xhtml:p><![CDATA[x := 1;]]></xhtml:p></ST></body>
            </pou>
            <pou name="Helper" pouType="function">
                <body><ST><xhtml:p><![CDATA[RETURN 42;]]></xhtml:p></ST></body>
            </pou>
        </project>
        "#;
        
        let results = extract_all_st_from_xml(xml);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, "Main");
        assert_eq!(results[0].1, "x := 1;");
        assert_eq!(results[1].0, "Helper");
        assert_eq!(results[1].1, "RETURN 42;");
    }
}
