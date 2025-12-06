//! Empty routines detector.
//!
//! Detects routines that have no logic (empty or only NOPs).

use l5x::Controller;

use crate::analysis::ProjectAnalysis;

use crate::config::EmptyRoutinesConfig;
use crate::report::{Report, Severity, Rule, RuleKind};

/// Detector for empty routines.
pub struct EmptyRoutinesDetector<'a> {
    config: &'a EmptyRoutinesConfig,
}

impl<'a> EmptyRoutinesDetector<'a> {
    /// Create a new empty routines detector with the given configuration.
    pub fn new(config: &'a EmptyRoutinesConfig) -> Self {
        Self { config }
    }

    /// Run detection on a controller and add findings to the report.
    pub fn detect(&self, _controller: &Controller, analysis: &ProjectAnalysis, report: &mut Report) {
        if !self.config.enabled {
            return;
        }

        for routine in &analysis.routines {
            // Skip if matches ignore pattern
            if self.matches_ignore_pattern(&routine.routine) {
                continue;
            }

            // Check if routine is empty based on type
            let is_empty = match routine.routine_type.as_str() {
                "RLL" => routine.rung_count == 0,
                "ST" => routine.tags_used.is_empty(),
                _ => false,
            };

            if is_empty {
                report.add(Rule::new(
                    RuleKind::EmptyBlock,
                    Severity::Info,
                    format!("Program:{}", routine.program),
                    routine.routine.clone(),
                    format!(
                        "Routine '{}' in program '{}' appears to be empty (type: {})",
                        routine.routine, routine.program, routine.routine_type
                    ),
                ));
            }
        }
    }

    /// Check if a routine name matches any ignore pattern.
    fn matches_ignore_pattern(&self, routine_name: &str) -> bool {
        for pattern in &self.config.ignore_patterns {
            if glob_match(pattern, routine_name) {
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
        assert!(glob_match("Main*", "MainRoutine"));
        assert!(glob_match("*_Test", "Unit_Test"));
        assert!(!glob_match("Main*", "OtherRoutine"));
    }
}
