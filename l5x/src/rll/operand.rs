//! Operand value parsing for tag extraction.
//!
//! This module parses operand strings to extract tag references from:
//! - Simple tags: `Motor`, `Start`
//! - Structured tags: `Timer1.DN`, `Motor.Running`
//! - Array access: `Array[0]`, `Data[idx]`
//! - Module I/O tags: `Local:1:I.Data.0` (base is `Local`)
//! - Indirect addressing: `Tag.[OtherTag.Member]` (extracts both tags)
//! - Expressions: `((1.0 - x) * y) + z`
//! - Function calls in CMP: `ATN(Tag) > 1.0`

/// A parsed operand value.
#[derive(Debug, Clone, PartialEq)]
pub enum OperandValue {
    /// A tag reference (simple, structured, or array)
    Tag(TagPath),
    /// A numeric literal
    Literal(String),
    /// An expression containing multiple terms
    Expression(Expression),
}

/// A tag path representing a tag reference.
/// Examples: `Motor`, `Timer1.DN`, `Array[0]`, `Local:1:I.Data`
#[derive(Debug, Clone, PartialEq)]
pub struct TagPath {
    /// The base tag name (before any `.` or `[`)
    pub base: String,
    /// Full path as written (for display/round-trip)
    pub full_path: String,
    /// Array indices if present (may contain tag references)
    pub indices: Vec<OperandValue>,
}

impl TagPath {
    /// Create a simple tag path
    pub fn simple(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            base: name.clone(),
            full_path: name,
            indices: Vec::new(),
        }
    }

    /// Create a tag path with the given base and full path
    pub fn new(base: impl Into<String>, full_path: impl Into<String>) -> Self {
        Self {
            base: base.into(),
            full_path: full_path.into(),
            indices: Vec::new(),
        }
    }

    /// Create a tag path with array indices
    pub fn with_indices(base: impl Into<String>, full_path: impl Into<String>, indices: Vec<OperandValue>) -> Self {
        Self {
            base: base.into(),
            full_path: full_path.into(),
            indices,
        }
    }

    /// Extract all tag references from this path (base + any tags in indices)
    pub fn all_tags(&self) -> Vec<String> {
        let mut tags = vec![self.base.clone()];
        for idx in &self.indices {
            tags.extend(idx.all_tags());
        }
        tags
    }
}

/// An expression containing operators and operands.
#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    /// The original expression text
    pub text: String,
    /// All terms (tags and literals) found in the expression
    pub terms: Vec<OperandValue>,
}

impl Expression {
    /// Create a new expression
    pub fn new(text: impl Into<String>, terms: Vec<OperandValue>) -> Self {
        Self {
            text: text.into(),
            terms,
        }
    }
}

impl OperandValue {
    /// Extract all tag base names from this operand value
    pub fn all_tags(&self) -> Vec<String> {
        match self {
            OperandValue::Tag(path) => path.all_tags(),
            OperandValue::Literal(_) => Vec::new(),
            OperandValue::Expression(expr) => {
                let mut tags = Vec::new();
                for term in &expr.terms {
                    tags.extend(term.all_tags());
                }
                tags
            }
        }
    }

    /// Get the base tag name if this is a simple tag reference
    pub fn base_tag(&self) -> Option<&str> {
        match self {
            OperandValue::Tag(path) => Some(&path.base),
            _ => None,
        }
    }
}

/// Parse an operand string into a structured OperandValue.
pub fn parse_operand_value(input: &str) -> OperandValue {
    let trimmed = input.trim();
    
    if trimmed.is_empty() {
        return OperandValue::Literal(String::new());
    }

    // Check if it's a numeric literal
    if is_numeric_literal(trimmed) {
        return OperandValue::Literal(trimmed.to_string());
    }

    // Check if it looks like an expression (contains operators outside of brackets)
    if looks_like_expression(trimmed) {
        return parse_expression(trimmed);
    }

    // Otherwise, parse as a tag path
    parse_tag_path(trimmed)
}

/// Check if a string is a numeric literal.
fn is_numeric_literal(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    // Check for radix prefix: 16#, 8#, 2#
    if s.starts_with("16#") || s.starts_with("8#") || s.starts_with("2#") {
        return true;
    }

    let s = s.strip_prefix('-').unwrap_or(s);
    let s = s.strip_prefix('+').unwrap_or(s);

    if s.is_empty() {
        return false;
    }

    let first = s.chars().next().unwrap();
    if !first.is_ascii_digit() {
        return false;
    }

    // Allow digits, '.', 'e', 'E', '+', '-', '_' for numeric literals
    s.chars().all(|c| {
        c.is_ascii_digit() || c == '.' || c == 'e' || c == 'E' || c == '+' || c == '-' || c == '_'
    })
}

/// Check if a string looks like an expression (contains operators at top level).
fn looks_like_expression(s: &str) -> bool {
    let mut paren_depth = 0;
    let mut bracket_depth = 0;
    let chars: Vec<char> = s.chars().collect();
    
    // Track if we've seen a non-whitespace char to detect subtraction vs negative
    let mut seen_term = false;
    
    for (_i, &c) in chars.iter().enumerate() {
        match c {
            '(' => paren_depth += 1,
            ')' => paren_depth -= 1,
            '[' => bracket_depth += 1,
            ']' => bracket_depth -= 1,
            '+' | '*' | '/' | '>' | '<' | '=' => {
                // These are expression operators when at top level
                if paren_depth == 0 && bracket_depth == 0 {
                    return true;
                }
            }
            '-' => {
                // Minus is subtraction if we've already seen a term
                if paren_depth == 0 && bracket_depth == 0 && seen_term {
                    return true;
                }
            }
            ' ' | '\t' | '\n' => {
                // Whitespace doesn't change seen_term
            }
            _ => {
                if paren_depth == 0 && bracket_depth == 0 {
                    seen_term = true;
                }
            }
        }
    }
    
    false
}

/// Parse a tag path: `Tag`, `Tag.Member`, `Tag[0]`, `Tag[idx].Member`, `Local:1:I.Data`
/// 
/// For Module I/O tags like `FlexIO:3:I.Pt01.Data`, the base is `FlexIO` (before first `:`)
/// For indirect addressing like `Tag.[OtherTag]`, extracts both tags.
fn parse_tag_path(input: &str) -> OperandValue {
    let mut chars = input.chars().peekable();
    let mut base = String::new();
    let mut indices = Vec::new();
    
    // Parse base name - stop at first `.`, `[`, or `:`
    // For Module I/O like `FlexIO:3:I.Data`, base is just `FlexIO`
    while let Some(&c) = chars.peek() {
        if c == '.' || c == '[' || c == ':' {
            break;
        }
        base.push(c);
        chars.next();
    }
    
    // Skip past any module address portion (`:slot:type`)
    // e.g., `FlexIO:3:I.Data` - skip `:3:I` part
    while let Some(&c) = chars.peek() {
        if c == ':' {
            chars.next(); // consume ':'
            // Skip until next `:` or `.` or `[`
            while let Some(&nc) = chars.peek() {
                if nc == ':' || nc == '.' || nc == '[' {
                    break;
                }
                chars.next();
            }
        } else {
            break;
        }
    }
    
    // Now parse array indices and handle indirect addressing
    while let Some(&c) = chars.peek() {
        if c == '[' {
            chars.next(); // consume '['
            let idx_str = collect_until_balanced(&mut chars, '[', ']');
            
            // Check if this looks like a numeric index (could have commas for multi-dim)
            // If it starts with a letter or underscore, it's a tag reference
            let trimmed = idx_str.trim();
            if !trimmed.is_empty() {
                let first_char = trimmed.chars().next().unwrap();
                if first_char.is_ascii_alphabetic() || first_char == '_' {
                    // It's a tag reference in the index
                    let idx_value = parse_operand_value(trimmed);
                    indices.push(idx_value);
                }
                // For numeric indices like `0` or `1,3`, we don't add them as tags
            }
        } else if c == '.' {
            chars.next(); // consume '.'
            // Check for indirect addressing: `.[ ... ]`
            if let Some(&nc) = chars.peek() {
                if nc == '[' {
                    chars.next(); // consume '['
                    let idx_str = collect_until_balanced(&mut chars, '[', ']');
                    // This is an indirect reference - parse the tag inside
                    let idx_value = parse_operand_value(&idx_str);
                    indices.push(idx_value);
                }
            }
            // Otherwise just skip member access (we only care about base tag)
        } else {
            break;
        }
    }
    
    OperandValue::Tag(TagPath::with_indices(base, input.to_string(), indices))
}

/// Parse an expression, extracting all tag references.
fn parse_expression(input: &str) -> OperandValue {
    let mut terms = Vec::new();
    extract_terms_from_expression(input, &mut terms);
    OperandValue::Expression(Expression::new(input, terms))
}

/// Recursively extract terms from an expression.
fn extract_terms_from_expression(input: &str, terms: &mut Vec<OperandValue>) {
    let mut current = String::new();
    let mut paren_depth = 0;
    let mut bracket_depth = 0;
    
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        let c = chars[i];
        
        match c {
            '(' => {
                paren_depth += 1;
                current.push(c);
            }
            ')' => {
                paren_depth -= 1;
                current.push(c);
            }
            '[' => {
                bracket_depth += 1;
                current.push(c);
            }
            ']' => {
                bracket_depth -= 1;
                current.push(c);
            }
            '+' | '*' | '/' | '>' | '<' | '=' => {
                if paren_depth == 0 && bracket_depth == 0 {
                    // End of term at top level
                    process_term(&current, terms);
                    current.clear();
                } else {
                    current.push(c);
                }
            }
            '-' => {
                // Minus is tricky - could be subtraction or negative sign
                if paren_depth == 0 && bracket_depth == 0 && !current.trim().is_empty() {
                    // Subtraction at top level
                    process_term(&current, terms);
                    current.clear();
                } else {
                    current.push(c);
                }
            }
            ' ' => {
                // Space at top level can be a separator
                if paren_depth == 0 && bracket_depth == 0 {
                    // Don't split on space, just skip it
                } else {
                    current.push(c);
                }
            }
            _ => {
                current.push(c);
            }
        }
        i += 1;
    }
    
    // Don't forget the last term
    process_term(&current, terms);
}

/// Process a term, stripping parens and recursively extracting tags.
fn process_term(term: &str, terms: &mut Vec<OperandValue>) {
    let trimmed = term.trim();
    if trimmed.is_empty() {
        return;
    }
    
    // Check for function call pattern: `FUNC(args)`
    // Functions like ATN, SIN, COS, etc. contain tags in their arguments
    if let Some(paren_pos) = trimmed.find('(') {
        let func_name = &trimmed[..paren_pos];
        // Check if it's a known math function (all uppercase or common functions)
        if is_known_function(func_name) {
            // Extract arguments and parse them
            if trimmed.ends_with(')') {
                let args = &trimmed[paren_pos + 1..trimmed.len() - 1];
                // Parse each comma-separated argument
                for arg in split_args(args) {
                    let arg_trimmed = arg.trim();
                    if !arg_trimmed.is_empty() {
                        let value = parse_operand_value(arg_trimmed);
                        match &value {
                            OperandValue::Literal(_) => {}
                            OperandValue::Tag(_) => terms.push(value),
                            OperandValue::Expression(e) => terms.extend(e.terms.clone()),
                        }
                    }
                }
            }
            return;
        }
    }
    
    // Strip outer parentheses and recurse
    let stripped = strip_outer_parens(trimmed);
    
    // Check if the stripped content contains operators (needs further parsing)
    if looks_like_expression(stripped) {
        extract_terms_from_expression(stripped, terms);
    } else {
        // Parse as a single value (tag or literal)
        let value = parse_operand_value(stripped);
        match &value {
            OperandValue::Literal(_) => {} // Skip literals
            OperandValue::Tag(_) => terms.push(value),
            OperandValue::Expression(e) => terms.extend(e.terms.clone()),
        }
    }
}

/// Check if a name is a known math/expression function.
fn is_known_function(name: &str) -> bool {
    // Common Rockwell/IEC 61131-3 expression functions
    matches!(name.to_uppercase().as_str(),
        "ABS" | "SQRT" | "LN" | "LOG" | "EXP" |
        "SIN" | "COS" | "TAN" | "ASN" | "ACS" | "ATN" |
        "DEG" | "RAD" | "TRUNC" | "NOT" | "AND" | "OR" | "XOR" |
        "MOD" | "FRD" | "TOD"
    )
}

/// Split function arguments by comma, respecting nested parens/brackets.
fn split_args(args: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut start = 0;
    let mut paren_depth = 0;
    let mut bracket_depth = 0;
    
    for (i, c) in args.char_indices() {
        match c {
            '(' => paren_depth += 1,
            ')' => paren_depth -= 1,
            '[' => bracket_depth += 1,
            ']' => bracket_depth -= 1,
            ',' if paren_depth == 0 && bracket_depth == 0 => {
                result.push(&args[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    
    // Don't forget the last argument
    if start < args.len() {
        result.push(&args[start..]);
    }
    
    result
}

/// Strip balanced outer parentheses from a string.
fn strip_outer_parens(s: &str) -> &str {
    let trimmed = s.trim();
    if trimmed.starts_with('(') && trimmed.ends_with(')') {
        // Check if they're balanced
        let inner = &trimmed[1..trimmed.len()-1];
        let mut depth = 0;
        for c in inner.chars() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth < 0 {
                        return trimmed; // Not balanced, keep original
                    }
                }
                _ => {}
            }
        }
        if depth == 0 {
            return strip_outer_parens(inner); // Recursively strip
        }
    }
    trimmed
}

/// Collect characters until we hit a balanced closing bracket.
fn collect_until_balanced(chars: &mut std::iter::Peekable<std::str::Chars>, open: char, close: char) -> String {
    let mut result = String::new();
    let mut depth = 1;
    
    while let Some(&c) = chars.peek() {
        if c == open {
            depth += 1;
        } else if c == close {
            depth -= 1;
            if depth == 0 {
                chars.next(); // consume closing bracket
                break;
            }
        }
        result.push(c);
        chars.next();
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_tag() {
        let val = parse_operand_value("Motor");
        assert_eq!(val.all_tags(), vec!["Motor"]);
    }

    #[test]
    fn test_parse_structured_tag() {
        let val = parse_operand_value("Timer1.DN");
        assert_eq!(val.all_tags(), vec!["Timer1"]);
        
        if let OperandValue::Tag(path) = val {
            assert_eq!(path.base, "Timer1");
            assert_eq!(path.full_path, "Timer1.DN");
        } else {
            panic!("Expected Tag");
        }
    }

    #[test]
    fn test_parse_io_tag() {
        // Module I/O tag: base is just the module name "Local"
        let val = parse_operand_value("Local:1:I.Data.0");
        assert_eq!(val.all_tags(), vec!["Local"]);
        
        if let OperandValue::Tag(path) = val {
            assert_eq!(path.base, "Local");
            assert_eq!(path.full_path, "Local:1:I.Data.0");
        } else {
            panic!("Expected Tag");
        }
    }

    #[test]
    fn test_parse_flexio_tag() {
        // FlexIO module tag
        let val = parse_operand_value("FlexIO:3:I.Pt01.Data");
        assert_eq!(val.all_tags(), vec!["FlexIO"]);
    }

    #[test]
    fn test_parse_array_literal_index() {
        let val = parse_operand_value("Array[0]");
        assert_eq!(val.all_tags(), vec!["Array"]);
    }

    #[test]
    fn test_parse_array_tag_index() {
        let val = parse_operand_value("Array[idx]");
        let tags = val.all_tags();
        assert!(tags.contains(&"Array".to_string()));
        assert!(tags.contains(&"idx".to_string()));
    }

    #[test]
    fn test_parse_numeric_literal() {
        let val = parse_operand_value("123.456");
        assert!(matches!(val, OperandValue::Literal(_)));
        assert!(val.all_tags().is_empty());
    }

    #[test]
    fn test_parse_hex_literal() {
        let val = parse_operand_value("16#FF00");
        assert!(matches!(val, OperandValue::Literal(_)));
        assert!(val.all_tags().is_empty());
    }

    #[test]
    fn test_parse_negative_literal() {
        let val = parse_operand_value("-2147483648");
        assert!(matches!(val, OperandValue::Literal(_)));
        assert!(val.all_tags().is_empty());
    }

    #[test]
    fn test_parse_expression() {
        let val = parse_operand_value("((1.0 - x) * y) + z");
        let tags = val.all_tags();
        assert!(tags.contains(&"x".to_string()), "Expected 'x' in {:?}", tags);
        assert!(tags.contains(&"y".to_string()), "Expected 'y' in {:?}", tags);
        assert!(tags.contains(&"z".to_string()), "Expected 'z' in {:?}", tags);
        assert_eq!(tags.len(), 3);
    }

    #[test]
    fn test_parse_expression_with_structured_tags() {
        let val = parse_operand_value("Timer1.ACC / Timer1.PRE * 100");
        let tags = val.all_tags();
        // Both Timer1.ACC and Timer1.PRE have base "Timer1"
        assert!(tags.iter().all(|t| t == "Timer1"));
    }

    #[test]
    fn test_parse_complex_expression() {
        // From real L5X: ((SP_In[10]-SP_In[9])*SRun_Tmr[10].ACC/SRun_Tmr[10].PRE)+SP_In[9]
        let val = parse_operand_value("((SP_In[10]-SP_In[9])*SRun_Tmr[10].ACC/SRun_Tmr[10].PRE)+SP_In[9]");
        let tags = val.all_tags();
        assert!(tags.contains(&"SP_In".to_string()));
        assert!(tags.contains(&"SRun_Tmr".to_string()));
    }

    #[test]
    fn test_parse_aoi_first_operand() {
        // AOI instances as operands
        let val = parse_operand_value("A_URNG");
        assert_eq!(val.all_tags(), vec!["A_URNG"]);
    }

    #[test]
    fn test_negative_in_expression() {
        // -2.0 should be a literal, not subtraction
        let val = parse_operand_value("-2.0");
        assert!(matches!(val, OperandValue::Literal(_)));
    }

    #[test]
    fn test_expression_with_negative() {
        let val = parse_operand_value("x * -2.0");
        let tags = val.all_tags();
        assert_eq!(tags, vec!["x"]);
    }

    #[test]
    fn test_indirect_addressing() {
        // Indirect addressing: Tag.[OtherTag.Member]
        let val = parse_operand_value("SimpleDint.[TestTag.IntMember]");
        let tags = val.all_tags();
        assert!(tags.contains(&"SimpleDint".to_string()), "Expected SimpleDint in {:?}", tags);
        assert!(tags.contains(&"TestTag".to_string()), "Expected TestTag in {:?}", tags);
    }

    #[test]
    fn test_multi_dimensional_array() {
        // Multi-dimensional array - should only extract the tag name
        let val = parse_operand_value("MultiDimArray[1,3].Member");
        let tags = val.all_tags();
        assert_eq!(tags, vec!["MultiDimArray"]);
    }

    #[test]
    fn test_atn_function_in_expression() {
        // CMP expression with ATN function
        let val = parse_operand_value("ATN(_Test) > 1.0");
        let tags = val.all_tags();
        assert!(tags.contains(&"_Test".to_string()), "Expected _Test in {:?}", tags);
    }

    #[test]
    fn test_sin_function_in_expression() {
        let val = parse_operand_value("SIN(Angle) + COS(Angle)");
        let tags = val.all_tags();
        assert!(tags.contains(&"Angle".to_string()), "Expected Angle in {:?}", tags);
    }

    #[test]
    fn test_complex_cmp_expression() {
        // Real pattern from L5X
        let val = parse_operand_value("ATN(_Test) > 1.0");
        let tags = val.all_tags();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0], "_Test");
    }
}
