//! Security limits for PLCopen XML parser to prevent DoS attacks

/// Security limits for XML parsing to prevent various attacks
#[derive(Debug, Clone)]
pub struct SecurityLimits {
    /// Maximum XML file size in bytes (default: 100 MB)
    pub max_file_size: usize,
    /// Maximum depth of nested XML elements (default: 256)
    pub max_element_depth: usize,
    /// Maximum number of attributes per element (default: 100)
    pub max_attributes_per_element: usize,
    /// Maximum length of any single attribute value (default: 1 MB)
    pub max_attribute_length: usize,
    /// Maximum length of text content (default: 10 MB)
    pub max_text_length: usize,
    /// Maximum total number of XML elements (default: 1 million)
    pub max_total_elements: usize,
    /// Maximum number of POUs in the project (default: 10,000)
    pub max_pous: usize,
}

impl Default for SecurityLimits {
    fn default() -> Self {
        Self::balanced()
    }
}

impl SecurityLimits {
    /// Balanced limits suitable for most industrial projects
    pub fn balanced() -> Self {
        SecurityLimits {
            max_file_size: 100 * 1024 * 1024,       // 100 MB
            max_element_depth: 256,
            max_attributes_per_element: 100,
            max_attribute_length: 1024 * 1024,      // 1 MB
            max_text_length: 10 * 1024 * 1024,      // 10 MB
            max_total_elements: 1_000_000,          // 1M elements
            max_pous: 10_000,
        }
    }

    /// Strict limits for untrusted/external XML files
    pub fn strict() -> Self {
        SecurityLimits {
            max_file_size: 10 * 1024 * 1024,        // 10 MB
            max_element_depth: 64,
            max_attributes_per_element: 50,
            max_attribute_length: 64 * 1024,        // 64 KB
            max_text_length: 1024 * 1024,           // 1 MB
            max_total_elements: 100_000,            // 100K elements
            max_pous: 1_000,
        }
    }

    /// Relaxed limits for trusted internal projects
    pub fn relaxed() -> Self {
        SecurityLimits {
            max_file_size: 1024 * 1024 * 1024,      // 1 GB
            max_element_depth: 1024,
            max_attributes_per_element: 500,
            max_attribute_length: 10 * 1024 * 1024, // 10 MB
            max_text_length: 100 * 1024 * 1024,     // 100 MB
            max_total_elements: 10_000_000,         // 10M elements
            max_pous: 100_000,
        }
    }
}

/// Security-related errors
#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    #[error("XML file too large: {size} bytes > {limit} bytes")]
    FileTooLarge { size: usize, limit: usize },
    
    #[error("XML element nesting too deep: {depth} > {limit}")]
    NestingTooDeep { depth: usize, limit: usize },
    
    #[error("Too many attributes: {count} > {limit}")]
    TooManyAttributes { count: usize, limit: usize },
    
    #[error("Attribute value too long: {length} bytes > {limit} bytes")]
    AttributeTooLong { length: usize, limit: usize },
    
    #[error("Text content too long: {length} bytes > {limit} bytes")]
    TextTooLong { length: usize, limit: usize },
    
    #[error("Too many XML elements: {count} > {limit}")]
    TooManyElements { count: usize, limit: usize },
    
    #[error("Too many POUs: {count} > {limit}")]
    TooManyPous { count: usize, limit: usize },
    
    #[error("Potential XML bomb detected")]
    XmlBomb,
    
    #[error("Potential billion laughs attack detected")]
    BillionLaughs,
}

/// Validates XML content against security limits before parsing
///
/// This function performs basic validation to detect common XML attacks:
/// - File size bombs
/// - Deep nesting attacks
/// - Entity expansion attacks
///
/// # Arguments
///
/// * `xml` - The XML content as a string
/// * `limits` - Security limits to enforce
///
/// # Returns
///
/// Returns `Ok(())` if validation passes, otherwise returns a `SecurityError`
pub fn validate_xml(xml: &str, limits: &SecurityLimits) -> Result<(), SecurityError> {
    // Check file size
    if xml.len() > limits.max_file_size {
        return Err(SecurityError::FileTooLarge {
            size: xml.len(),
            limit: limits.max_file_size,
        });
    }

    // Check for entity expansion attacks (billion laughs)
    let entity_count = xml.matches("<!ENTITY").count();
    if entity_count > 10 {
        return Err(SecurityError::BillionLaughs);
    }

    // Check maximum element depth by scanning for deeply nested tags
    let mut max_depth = 0;
    let mut current_depth = 0;
    for c in xml.chars() {
        match c {
            '<' => {
                current_depth += 1;
                if current_depth > max_depth {
                    max_depth = current_depth;
                }
                if max_depth > limits.max_element_depth {
                    return Err(SecurityError::NestingTooDeep {
                        depth: max_depth,
                        limit: limits.max_element_depth,
                    });
                }
            }
            '>' => {
                if current_depth > 0 {
                    current_depth -= 1;
                }
            }
            _ => {}
        }
    }

    // Check total element count (rough estimate)
    let element_count = xml.matches('<').count() / 2; // Opening tags only
    if element_count > limits.max_total_elements {
        return Err(SecurityError::TooManyElements {
            count: element_count,
            limit: limits.max_total_elements,
        });
    }

    Ok(())
}
