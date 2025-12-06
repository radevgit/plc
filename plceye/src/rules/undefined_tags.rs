//! Undefined tags detector.
//!
//! Detects tags that are referenced in code but not declared.
//! Note: Some "undefined" tags may be valid (aliases, module I/O, etc.)

use std::collections::HashSet;

use l5x::{Controller, UDIDefinitionContent};

use crate::analysis::ProjectAnalysis;

use crate::config::UndefinedTagsConfig;
use crate::report::{Report, Severity, Rule, RuleKind};

/// Detector for undefined tags.
pub struct UndefinedTagsDetector<'a> {
    config: &'a UndefinedTagsConfig,
}

impl<'a> UndefinedTagsDetector<'a> {
    /// Create a new undefined tags detector with the given configuration.
    pub fn new(config: &'a UndefinedTagsConfig) -> Self {
        Self { config }
    }

    /// Run detection on a controller and add findings to the report.
    pub fn detect(&self, controller: &Controller, analysis: &ProjectAnalysis, report: &mut Report) {
        if !self.config.enabled {
            return;
        }

        // Collect all defined tags
        let defined_tags = self.collect_defined_tags(controller);
        
        // Collect all AOI names (they can appear as instruction-like references)
        let aoi_names: HashSet<String> = analysis.aoi_definitions
            .iter()
            .cloned()
            .collect();

        // Check each referenced tag
        for tag_ref in analysis.unique_tags() {
            // Extract base tag name (before any dots or brackets)
            let base_name = extract_base_name(tag_ref);
            
            // Skip if defined
            if defined_tags.contains(base_name) {
                continue;
            }
            
            // Skip if it's an AOI name
            if aoi_names.contains(base_name) {
                continue;
            }

            // Skip if matches ignore pattern
            if self.matches_ignore_pattern(base_name) {
                continue;
            }

            // Skip known built-in tags
            if is_builtin_tag(base_name) {
                continue;
            }

            report.add(Rule::new(
                RuleKind::UndefinedTag,
                Severity::Warning,
                "Controller".to_string(),
                base_name.to_string(),
                format!("Tag '{}' is referenced but not defined (may be alias or I/O)", base_name),
            ));
        }
    }

    /// Collect all defined tag names (base names only).
    fn collect_defined_tags(&self, controller: &Controller) -> HashSet<String> {
        let mut tags = HashSet::new();

        // Controller-scope tags
        if let Some(ref tag_collection) = controller.tags {
            for tag in &tag_collection.tag {
                tags.insert(tag.name.clone());
            }
        }

        // Program-scope tags
        if let Some(ref programs) = controller.programs {
            for program in &programs.program {
                if let Some(ref tag_collection) = program.tags {
                    for tag in &tag_collection.tag {
                        tags.insert(tag.name.clone());
                    }
                }
            }
        }

        // AOI parameters and local tags
        if let Some(ref aois) = controller.add_on_instruction_definitions {
            for aoi in &aois.add_on_instruction_definition {
                for content in &aoi.content {
                    match content {
                        UDIDefinitionContent::Parameters(params) => {
                            for param in &params.parameter {
                                tags.insert(param.name.clone());
                            }
                        }
                        UDIDefinitionContent::LocalTags(local_tags) => {
                            for local_tag in &local_tags.local_tag {
                                tags.insert(local_tag.name.clone());
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        tags
    }

    /// Check if a tag name matches any ignore pattern.
    fn matches_ignore_pattern(&self, tag_name: &str) -> bool {
        for pattern in &self.config.ignore_patterns {
            if glob_match(pattern, tag_name) {
                return true;
            }
        }
        false
    }
}

/// Extract base tag name from a potentially qualified reference.
/// "MyTag.Member[0]" -> "MyTag"
fn extract_base_name(tag_ref: &str) -> &str {
    tag_ref
        .split('.')
        .next()
        .unwrap_or(tag_ref)
        .split('[')
        .next()
        .unwrap_or(tag_ref)
}

/// Check if a tag is a known built-in.
fn is_builtin_tag(name: &str) -> bool {
    let upper = name.to_uppercase();
    matches!(
        upper.as_str(),
        "S:FS" | "S:N" | "S:Z" | "S:V" | "S:C" | "S:MINOR" | "S:MAJOR" |
        "AFI" | "NOP" | "GSV" | "SSV" | "MSG" | "JSR" | "RET" | "SBR" |
        "UID" | "UIE" | "FOR" | "BRK" | "RES" | "EOT" | "EVENT" | "TASK"
    )
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
    fn test_extract_base_name() {
        assert_eq!(extract_base_name("MyTag"), "MyTag");
        assert_eq!(extract_base_name("MyTag.Member"), "MyTag");
        assert_eq!(extract_base_name("MyTag[0]"), "MyTag");
        assert_eq!(extract_base_name("MyTag.Member[0].Sub"), "MyTag");
    }

    #[test]
    fn test_builtin_detection() {
        assert!(is_builtin_tag("S:FS"));
        assert!(is_builtin_tag("GSV"));
        assert!(!is_builtin_tag("MyTag"));
    }
}
