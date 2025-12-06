//! Unused DataTypes detector.
//!
//! Detects user-defined data types that are never used by any tag or AOI.

use std::collections::HashSet;

use l5x::{Controller, UDIDefinitionContent};

use crate::config::UnusedDataTypesConfig;
use crate::report::{Report, Severity, Rule, RuleKind};

/// Detector for unused DataTypes.
pub struct UnusedDataTypesDetector<'a> {
    config: &'a UnusedDataTypesConfig,
}

impl<'a> UnusedDataTypesDetector<'a> {
    /// Create a new unused DataTypes detector with the given configuration.
    pub fn new(config: &'a UnusedDataTypesConfig) -> Self {
        Self { config }
    }

    /// Run detection on a controller and add findings to the report.
    pub fn detect(&self, controller: &Controller, report: &mut Report) {
        if !self.config.enabled {
            return;
        }

        // Collect defined user data types
        let defined_types = self.collect_defined_datatypes(controller);
        
        // Collect all used data types (from tags, AOI parameters, etc.)
        let used_types = self.collect_used_datatypes(controller);

        // Find unused types
        for type_name in &defined_types {
            if used_types.contains(type_name.as_str()) {
                continue;
            }

            // Skip if matches ignore pattern
            if self.matches_ignore_pattern(type_name) {
                continue;
            }

            report.add(Rule::new(
                RuleKind::UnusedDataType,
                Severity::Info,
                "DataTypes".to_string(),
                type_name.clone(),
                format!("DataType '{}' is defined but never used", type_name),
            ));
        }
    }

    /// Collect all defined user data types.
    fn collect_defined_datatypes(&self, controller: &Controller) -> Vec<String> {
        let mut types = Vec::new();

        if let Some(ref datatypes) = controller.data_types {
            for dt in &datatypes.data_type {
                types.push(dt.name.clone());
            }
        }

        types
    }

    /// Collect all used data types from tags, AOI parameters, etc.
    fn collect_used_datatypes(&self, controller: &Controller) -> HashSet<String> {
        let mut used = HashSet::new();

        // Check controller-scope tags
        if let Some(ref tags) = controller.tags {
            for tag in &tags.tag {
                if let Some(ref dt) = tag.data_type {
                    used.insert(dt.clone());
                }
            }
        }

        // Check program-scope tags
        if let Some(ref programs) = controller.programs {
            for program in &programs.program {
                if let Some(ref tags) = program.tags {
                    for tag in &tags.tag {
                        if let Some(ref dt) = tag.data_type {
                            used.insert(dt.clone());
                        }
                    }
                }
            }
        }

        // Check AOI parameters and local tags
        if let Some(ref aois) = controller.add_on_instruction_definitions {
            for aoi in &aois.add_on_instruction_definition {
                for content in &aoi.content {
                    match content {
                        UDIDefinitionContent::Parameters(params) => {
                            for param in &params.parameter {
                                if let Some(ref dt) = param.data_type {
                                    used.insert(dt.clone());
                                }
                            }
                        }
                        UDIDefinitionContent::LocalTags(local_tags) => {
                            for tag in &local_tags.local_tag {
                                used.insert(tag.data_type.clone());
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // Check data type members (for nested types)
        if let Some(ref datatypes) = controller.data_types {
            for dt in &datatypes.data_type {
                if let Some(ref members) = dt.members {
                    for member in &members.member {
                        used.insert(member.data_type.clone());
                    }
                }
            }
        }

        used
    }

    /// Check if a type name matches any ignore pattern.
    fn matches_ignore_pattern(&self, type_name: &str) -> bool {
        for pattern in &self.config.ignore_patterns {
            if glob_match(pattern, type_name) {
                return true;
            }
        }
        false
    }
}

/// Simple glob pattern matching supporting * and ? wildcards.
fn glob_match(pattern: &str, text: &str) -> bool {
    let pattern_chars: Vec<char> = pattern.chars().collect();
    let text_chars: Vec<char> = text.chars().collect();
    glob_match_recursive(&pattern_chars, &text_chars, 0, 0)
}

fn glob_match_recursive(pattern: &[char], text: &[char], pi: usize, ti: usize) -> bool {
    if pi == pattern.len() {
        return ti == text.len();
    }

    match pattern[pi] {
        '*' => {
            for i in ti..=text.len() {
                if glob_match_recursive(pattern, text, pi + 1, i) {
                    return true;
                }
            }
            false
        }
        '?' => {
            if ti < text.len() {
                glob_match_recursive(pattern, text, pi + 1, ti + 1)
            } else {
                false
            }
        }
        c => {
            if ti < text.len() && c.eq_ignore_ascii_case(&text[ti]) {
                glob_match_recursive(pattern, text, pi + 1, ti + 1)
            } else {
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_match() {
        assert!(glob_match("UDT_*", "UDT_Motor"));
        assert!(glob_match("*_Type", "MyCustom_Type"));
        assert!(!glob_match("UDT_*", "Motor_UDT"));
    }
}
