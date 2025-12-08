//! Security limits and protections for L5X parsing
//!
//! Protects against:
//! - XML bombs (billion laughs attack)
//! - Entity expansion attacks
//! - Deep nesting attacks
//! - Memory exhaustion
//! - External entity injection

use quick_xml::Reader;
use quick_xml::events::Event;
use std::io::BufRead;

/// Security limits for parsing L5X files
#[derive(Debug, Clone)]
pub struct SecurityLimits {
    /// Maximum XML file size in bytes
    pub max_file_size: usize,
    
    /// Maximum depth of nested XML elements
    pub max_nesting_depth: usize,
    
    /// Maximum number of attributes per element
    pub max_attributes_per_element: usize,
    
    /// Maximum attribute name/value length
    pub max_attribute_length: usize,
    
    /// Maximum text content length
    pub max_text_length: usize,
    
    /// Maximum number of elements in document
    pub max_elements: usize,
    
    /// Maximum buffer size for quick-xml
    pub max_buffer_size: usize,
}

impl Default for SecurityLimits {
    fn default() -> Self {
        SecurityLimits {
            max_file_size: 100 * 1024 * 1024,      // 100 MB
            max_nesting_depth: 100,                // 100 levels
            max_attributes_per_element: 100,       // 100 attributes
            max_attribute_length: 10_000,          // 10 KB
            max_text_length: 10 * 1024 * 1024,     // 10 MB
            max_elements: 1_000_000,               // 1M elements
            max_buffer_size: 10 * 1024 * 1024,     // 10 MB buffer
        }
    }
}

impl SecurityLimits {
    /// Balanced limits for typical industrial projects
    pub fn balanced() -> Self {
        Self::default()
    }
    
    /// Conservative limits for untrusted input
    pub fn strict() -> Self {
        SecurityLimits {
            max_file_size: 10 * 1024 * 1024,       // 10 MB
            max_nesting_depth: 32,                 // 32 levels
            max_attributes_per_element: 50,        // 50 attributes
            max_attribute_length: 1_000,           // 1 KB
            max_text_length: 1 * 1024 * 1024,      // 1 MB
            max_elements: 100_000,                 // 100K elements
            max_buffer_size: 1 * 1024 * 1024,      // 1 MB buffer
        }
    }
    
    /// Relaxed limits for trusted input
    pub fn relaxed() -> Self {
        SecurityLimits {
            max_file_size: 500 * 1024 * 1024,      // 500 MB
            max_nesting_depth: 256,                // 256 levels
            max_attributes_per_element: 500,       // 500 attributes
            max_attribute_length: 100_000,         // 100 KB
            max_text_length: 100 * 1024 * 1024,    // 100 MB
            max_elements: 10_000_000,              // 10M elements
            max_buffer_size: 100 * 1024 * 1024,    // 100 MB buffer
        }
    }
}

/// Validate XML input against security limits before parsing
pub fn validate_xml<R: BufRead>(
    reader: R,
    limits: &SecurityLimits,
) -> Result<(), SecurityError> {
    let mut xml_reader = Reader::from_reader(reader);
    
    // Configure quick-xml security settings
    xml_reader
        .config_mut()
        .expand_empty_elements = false;  // Don't expand empty elements
    
    xml_reader
        .config_mut()
        .trim_text(true);  // Trim whitespace to save memory
    
    let mut buf = Vec::new();
    let mut depth = 0;
    let mut element_count = 0;
    
    loop {
        match xml_reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                depth += 1;
                element_count += 1;
                
                // Check nesting depth
                if depth > limits.max_nesting_depth {
                    return Err(SecurityError::NestingTooDeep {
                        depth,
                        limit: limits.max_nesting_depth,
                    });
                }
                
                // Check element count
                if element_count > limits.max_elements {
                    return Err(SecurityError::TooManyElements {
                        count: element_count,
                        limit: limits.max_elements,
                    });
                }
                
                // Check attributes
                let attrs: Vec<_> = e.attributes().collect();
                if attrs.len() > limits.max_attributes_per_element {
                    return Err(SecurityError::TooManyAttributes {
                        count: attrs.len(),
                        limit: limits.max_attributes_per_element,
                    });
                }
                
                for attr_result in attrs {
                    let attr = attr_result.map_err(|e| SecurityError::XmlError(e.to_string()))?;
                    
                    // Check attribute key length
                    if attr.key.as_ref().len() > limits.max_attribute_length {
                        return Err(SecurityError::AttributeTooLong {
                            length: attr.key.as_ref().len(),
                            limit: limits.max_attribute_length,
                        });
                    }
                    
                    // Check attribute value length
                    if attr.value.len() > limits.max_attribute_length {
                        return Err(SecurityError::AttributeTooLong {
                            length: attr.value.len(),
                            limit: limits.max_attribute_length,
                        });
                    }
                }
            }
            Ok(Event::End(_)) => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape().map_err(|e| SecurityError::XmlError(e.to_string()))?;
                if text.len() > limits.max_text_length {
                    return Err(SecurityError::TextTooLong {
                        length: text.len(),
                        limit: limits.max_text_length,
                    });
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(SecurityError::XmlError(e.to_string())),
            _ => {}
        }
        
        buf.clear();
    }
    
    Ok(())
}

/// Security validation errors
#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    #[error("XML nesting too deep: {depth} levels (limit: {limit})")]
    NestingTooDeep { depth: usize, limit: usize },
    
    #[error("Too many XML elements: {count} (limit: {limit})")]
    TooManyElements { count: usize, limit: usize },
    
    #[error("Too many attributes: {count} (limit: {limit})")]
    TooManyAttributes { count: usize, limit: usize },
    
    #[error("Attribute too long: {length} bytes (limit: {limit})")]
    AttributeTooLong { length: usize, limit: usize },
    
    #[error("Text content too long: {length} bytes (limit: {limit})")]
    TextTooLong { length: usize, limit: usize },
    
    #[error("File too large: {size} bytes (limit: {limit})")]
    FileTooLarge { size: usize, limit: usize },
    
    #[error("XML parsing error: {0}")]
    XmlError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_xml() {
        let xml = r#"<Project><Controller Name="Test"></Controller></Project>"#;
        let result = validate_xml(xml.as_bytes(), &SecurityLimits::default());
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_deep_nesting() {
        let mut xml = String::from("<a>");
        for _ in 0..200 {
            xml.push_str("<b>");
        }
        for _ in 0..200 {
            xml.push_str("</b>");
        }
        xml.push_str("</a>");
        
        let result = validate_xml(xml.as_bytes(), &SecurityLimits::strict());
        assert!(matches!(result, Err(SecurityError::NestingTooDeep { .. })));
    }
    
    #[test]
    fn test_too_many_attributes() {
        let mut xml = String::from("<Element ");
        for i in 0..200 {
            xml.push_str(&format!("attr{}='value' ", i));
        }
        xml.push_str("/>");
        
        let result = validate_xml(xml.as_bytes(), &SecurityLimits::strict());
        assert!(matches!(result, Err(SecurityError::TooManyAttributes { .. })));
    }
}
