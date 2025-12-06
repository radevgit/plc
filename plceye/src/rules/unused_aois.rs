//! Unused AOIs detector.
//!
//! Detects Add-On Instructions that are defined but never called.

use std::collections::HashSet;

use crate::analysis::ProjectAnalysis;
use crate::config::UnusedAoisConfig;
use crate::report::{Report, Severity, Rule, RuleKind};

/// Detector for unused AOIs.
pub struct UnusedAoisDetector<'a> {
    config: &'a UnusedAoisConfig,
}

impl<'a> UnusedAoisDetector<'a> {
    /// Create a new unused AOIs detector with the given configuration.
    pub fn new(config: &'a UnusedAoisConfig) -> Self {
        Self { config }
    }

    /// Run detection using analysis results and add findings to the report.
    pub fn detect(&self, analysis: &ProjectAnalysis, report: &mut Report) {
        if !self.config.enabled {
            return;
        }

        // Get all defined AOI names
        let defined_aois: HashSet<&str> = analysis.aoi_definitions.iter().map(|s| s.as_str()).collect();

        // Get all called AOI names from analysis
        let called_aois: HashSet<&str> = analysis.aoi_usage.keys().map(|s| s.as_str()).collect();

        // Find unused AOIs
        for aoi_name in &defined_aois {
            if called_aois.contains(*aoi_name) {
                continue;
            }

            // Skip if matches ignore pattern
            if self.matches_ignore_pattern(aoi_name) {
                continue;
            }

            report.add(Rule::new(
                RuleKind::UnusedAoi,
                Severity::Info,
                format!("AOI:{}", aoi_name),
                aoi_name.to_string(),
                format!("AOI '{}' is defined but never called", aoi_name),
            ));
        }
    }

    /// Check if an AOI name matches any ignore pattern.
    fn matches_ignore_pattern(&self, aoi_name: &str) -> bool {
        for pattern in &self.config.ignore_patterns {
            if glob_match(pattern, aoi_name) {
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
    use crate::analysis::{ProjectAnalysis, AoiReference, RungLocation, AoiCallSource, ParseStats};

    #[test]
    fn test_detects_unused_aoi() {
        let config = UnusedAoisConfig {
            enabled: true,
            ignore_patterns: vec![],
        };
        
        let mut analysis = ProjectAnalysis {
            rungs: vec![],
            st_routines: vec![],
            tag_references: vec![],
            tag_xref: Default::default(),
            routines: vec![],
            instruction_usage: Default::default(),
            aoi_definitions: vec!["UsedAOI".to_string(), "UnusedAOI".to_string()],
            aoi_usage: Default::default(),
            stats: ParseStats::default(),
        };
        
        // Add a call to UsedAOI
        analysis.aoi_usage.insert(
            "UsedAOI".to_string(),
            vec![AoiReference {
                aoi_name: "UsedAOI".to_string(),
                program: "Main".to_string(),
                routine: "Logic".to_string(),
                rung_number: Some(0),
                source: AoiCallSource::Rll,
            }],
        );
        
        let detector = UnusedAoisDetector::new(&config);
        let mut report = Report::new();
        detector.detect(&analysis, &mut report);
        
        assert_eq!(report.rules.len(), 1);
        assert_eq!(report.rules[0].kind, RuleKind::UnusedAoi);
        assert!(report.rules[0].message.contains("UnusedAOI"));
    }

    #[test]
    fn test_ignores_pattern() {
        let config = UnusedAoisConfig {
            enabled: true,
            ignore_patterns: vec!["Test_*".to_string()],
        };
        
        let analysis = ProjectAnalysis {
            rungs: vec![],
            st_routines: vec![],
            tag_references: vec![],
            tag_xref: Default::default(),
            routines: vec![],
            instruction_usage: Default::default(),
            aoi_definitions: vec!["Test_AOI".to_string()],
            aoi_usage: Default::default(),
            stats: ParseStats::default(),
        };
        
        let detector = UnusedAoisDetector::new(&config);
        let mut report = Report::new();
        detector.detect(&analysis, &mut report);
        
        assert_eq!(report.rules.len(), 0);
    }

    #[test]
    fn test_glob_match() {
        assert!(glob_match("Test*", "TestAOI"));
        assert!(glob_match("*AOI", "MyTestAOI"));
        assert!(glob_match("*", "anything"));
        assert!(glob_match("Test?", "TestA"));
        assert!(!glob_match("Test?", "TestAB"));
    }
}
