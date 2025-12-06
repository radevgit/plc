//! PLCopen-specific rule detectors.

use crate::analysis::PlcopenAnalysis;
use crate::config::{EmptyRoutinesConfig, UndefinedTagsConfig, UnusedTagsConfig};
use crate::report::{Report, Severity, Rule, RuleKind};

/// Detect unused variables in PLCopen projects.
pub struct PlcopenUnusedVarsDetector<'a> {
    config: &'a UnusedTagsConfig,
}

impl<'a> PlcopenUnusedVarsDetector<'a> {
    pub fn new(config: &'a UnusedTagsConfig) -> Self {
        Self { config }
    }

    pub fn detect(&self, analysis: &PlcopenAnalysis, report: &mut Report) {
        if !self.config.enabled {
            return;
        }

        for var in analysis.unused_variables() {
            // Skip if matches ignore pattern
            if self.matches_ignore_pattern(&var.name) {
                continue;
            }

            report.add(Rule::new(
                RuleKind::UnusedTag,
                Severity::Warning,
                var.pou_name.clone(),
                var.name.clone(),
                format!("Variable '{}' is defined but never used", var.name),
            ));
        }
    }

    fn matches_ignore_pattern(&self, name: &str) -> bool {
        for pattern in &self.config.ignore_patterns {
            if glob_match(pattern, name) {
                return true;
            }
        }
        false
    }
}

/// Detect undefined variables in PLCopen projects.
pub struct PlcopenUndefinedVarsDetector<'a> {
    config: &'a UndefinedTagsConfig,
}

impl<'a> PlcopenUndefinedVarsDetector<'a> {
    pub fn new(config: &'a UndefinedTagsConfig) -> Self {
        Self { config }
    }

    pub fn detect(&self, analysis: &PlcopenAnalysis, report: &mut Report) {
        if !self.config.enabled {
            return;
        }

        for var_name in analysis.undefined_variables() {
            // Skip if it's a POU name (function/FB call)
            if analysis.pou_names.contains(var_name) {
                continue;
            }

            report.add(Rule::new(
                RuleKind::UndefinedTag,
                Severity::Info,
                "project".to_string(),
                var_name.clone(),
                format!("Variable '{}' is used but not defined (may be external)", var_name),
            ));
        }
    }
}

/// Detect empty POUs in PLCopen projects.
pub struct PlcopenEmptyPousDetector<'a> {
    config: &'a EmptyRoutinesConfig,
}

impl<'a> PlcopenEmptyPousDetector<'a> {
    pub fn new(config: &'a EmptyRoutinesConfig) -> Self {
        Self { config }
    }

    pub fn detect(&self, analysis: &PlcopenAnalysis, report: &mut Report) {
        if !self.config.enabled {
            return;
        }

        for pou_name in &analysis.empty_pous {
            // Skip if matches ignore pattern
            if self.matches_ignore_pattern(pou_name) {
                continue;
            }

            report.add(Rule::new(
                RuleKind::EmptyBlock,
                Severity::Info,
                pou_name.clone(),
                pou_name.clone(),
                format!("POU '{}' has no implementation", pou_name),
            ));
        }
    }

    fn matches_ignore_pattern(&self, name: &str) -> bool {
        for pattern in &self.config.ignore_patterns {
            if glob_match(pattern, name) {
                return true;
            }
        }
        false
    }
}

/// Simple glob pattern matching (supports * and ?).
fn glob_match(pattern: &str, text: &str) -> bool {
    let mut pattern_chars = pattern.chars().peekable();
    let mut text_chars = text.chars().peekable();

    while let Some(p) = pattern_chars.next() {
        match p {
            '*' => {
                if pattern_chars.peek().is_none() {
                    return true;
                }
                let rest_pattern: String = pattern_chars.collect();
                let rest_text: String = text_chars.collect();
                for i in 0..=rest_text.len() {
                    if glob_match(&rest_pattern, &rest_text[i..]) {
                        return true;
                    }
                }
                return false;
            }
            '?' => {
                if text_chars.next().is_none() {
                    return false;
                }
            }
            c => {
                if text_chars.next() != Some(c) {
                    return false;
                }
            }
        }
    }

    text_chars.peek().is_none()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_match() {
        assert!(glob_match("*", "anything"));
        assert!(glob_match("Motor*", "Motor_1"));
        assert!(glob_match("*_temp", "var_temp"));
        assert!(glob_match("?otor", "Motor"));
        assert!(!glob_match("Motor", "Pump"));
    }
}
