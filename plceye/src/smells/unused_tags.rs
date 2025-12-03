//! Unused tags detector.
//!
//! Detects tags that are defined but never used in any routine.

use std::collections::HashSet;

use l5x::Controller;

use crate::analysis::ProjectAnalysis;

use crate::config::UnusedTagsConfig;
use crate::report::{Report, Severity, Smell, SmellKind};

/// Detector for unused tags.
pub struct UnusedTagsDetector<'a> {
    config: &'a UnusedTagsConfig,
}

impl<'a> UnusedTagsDetector<'a> {
    /// Create a new unused tags detector with the given configuration.
    pub fn new(config: &'a UnusedTagsConfig) -> Self {
        Self { config }
    }

    /// Run detection on a controller and add findings to the report.
    pub fn detect(&self, controller: &Controller, analysis: &ProjectAnalysis, report: &mut Report) {
        if !self.config.enabled {
            return;
        }

        // Collect all defined tags from controller and program scopes
        let defined_tags = self.collect_defined_tags(controller);
        
        // Collect all used tags from analysis
        let used_tags: HashSet<&str> = analysis
            .tag_xref
            .keys()
            .map(|s| s.as_str())
            .collect();

        // Find unused tags
        for (tag_name, scope) in &defined_tags {
            // Skip if tag is used
            if used_tags.contains(tag_name.as_str()) {
                continue;
            }

            // Skip if matches ignore pattern
            if self.matches_ignore_pattern(tag_name) {
                continue;
            }

            // Skip if in ignored scope
            if self.matches_ignore_scope(scope) {
                continue;
            }

            report.add(Smell::new(
                SmellKind::UnusedTag,
                Severity::Info,
                scope.clone(),
                tag_name.clone(),
                format!("Tag '{}' is defined but never used", tag_name),
            ));
        }
    }

    /// Collect all defined tags from controller and program scopes.
    fn collect_defined_tags(&self, controller: &Controller) -> Vec<(String, String)> {
        let mut tags = Vec::new();

        // Controller-scope tags
        if let Some(ref tag_collection) = controller.tags {
            for tag in &tag_collection.tag {
                tags.push((tag.name.clone(), "Controller".to_string()));
            }
        }

        // Program-scope tags
        if let Some(ref programs) = controller.programs {
            for program in &programs.program {
                if let Some(ref tag_collection) = program.tags {
                    for tag in &tag_collection.tag {
                        let scope = format!("Program:{}", program.name);
                        tags.push((tag.name.clone(), scope));
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

    /// Check if a scope matches any ignored scope.
    fn matches_ignore_scope(&self, scope: &str) -> bool {
        self.config.ignore_scopes.iter().any(|s| s == scope)
    }
}

/// Simple glob matching (supports * and ? wildcards).
fn glob_match(pattern: &str, text: &str) -> bool {
    let pattern = pattern.to_lowercase();
    let text = text.to_lowercase();
    
    let mut p_chars = pattern.chars().peekable();
    let mut t_chars = text.chars().peekable();

    while let Some(p) = p_chars.next() {
        match p {
            '*' => {
                // * matches zero or more characters
                if p_chars.peek().is_none() {
                    return true;
                }
                let remaining_pattern: String = p_chars.collect();
                let remaining_text: String = t_chars.collect();
                for i in 0..=remaining_text.len() {
                    if glob_match(&remaining_pattern, &remaining_text[i..]) {
                        return true;
                    }
                }
                return false;
            }
            '?' => {
                if t_chars.next().is_none() {
                    return false;
                }
            }
            c => {
                if t_chars.next() != Some(c) {
                    return false;
                }
            }
        }
    }

    t_chars.next().is_none()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_match_literal() {
        assert!(glob_match("hello", "hello"));
        assert!(glob_match("hello", "HELLO"));
        assert!(!glob_match("hello", "world"));
    }

    #[test]
    fn test_glob_match_star() {
        assert!(glob_match("_*", "_internal"));
        assert!(glob_match("_*", "_"));
        assert!(!glob_match("_*", "public"));
        assert!(glob_match("HMI_*", "HMI_Button1"));
    }

    #[test]
    fn test_glob_match_question() {
        assert!(glob_match("Tag?", "Tag1"));
        assert!(!glob_match("Tag?", "Tag"));
        assert!(!glob_match("Tag?", "Tag12"));
    }

    #[test]
    fn test_matches_ignore_pattern() {
        let config = UnusedTagsConfig {
            enabled: true,
            ignore_patterns: vec!["_*".to_string(), "HMI_*".to_string()],
            ignore_scopes: vec![],
        };
        let detector = UnusedTagsDetector::new(&config);
        
        assert!(detector.matches_ignore_pattern("_internal"));
        assert!(detector.matches_ignore_pattern("HMI_Button"));
        assert!(!detector.matches_ignore_pattern("MyTag"));
    }
}
