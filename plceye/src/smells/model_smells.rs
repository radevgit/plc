//! Model-based smell detectors.
//!
//! These detectors work on plcmodel::Project for format-independent analysis.

use crate::analysis::ModelAnalysis;
use crate::config::{UnusedTagsConfig, UndefinedTagsConfig, EmptyRoutinesConfig};
use crate::report::{Report, Severity, Smell, SmellKind};

/// Detect unused tags using model analysis.
pub struct ModelUnusedTagsDetector<'a> {
    config: &'a UnusedTagsConfig,
}

impl<'a> ModelUnusedTagsDetector<'a> {
    pub fn new(config: &'a UnusedTagsConfig) -> Self {
        Self { config }
    }

    pub fn detect(&self, analysis: &ModelAnalysis, report: &mut Report) {
        if !self.config.enabled {
            return;
        }

        for var_def in analysis.unused_tags() {
            // Skip if matches ignore pattern
            if self.matches_ignore_pattern(&var_def.name) {
                continue;
            }

            // Skip if in ignored scope
            if self.matches_ignore_scope(&var_def.scope) {
                continue;
            }

            report.add(Smell::new(
                SmellKind::UnusedTag,
                Severity::Info,
                var_def.scope.clone(),
                var_def.name.clone(),
                format!("Tag '{}' is defined but never used", var_def.name),
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

    fn matches_ignore_scope(&self, scope: &str) -> bool {
        self.config.ignore_scopes.iter().any(|s| s == scope)
    }
}

/// Detect undefined tags using model analysis.
pub struct ModelUndefinedTagsDetector<'a> {
    config: &'a UndefinedTagsConfig,
}

impl<'a> ModelUndefinedTagsDetector<'a> {
    pub fn new(config: &'a UndefinedTagsConfig) -> Self {
        Self { config }
    }

    pub fn detect(&self, analysis: &ModelAnalysis, report: &mut Report) {
        if !self.config.enabled {
            return;
        }

        for tag_name in analysis.undefined_tags() {
            // Skip builtins
            if self.is_builtin(tag_name) {
                continue;
            }

            // Skip if matches ignore pattern
            if self.matches_ignore_pattern(tag_name) {
                continue;
            }

            report.add(Smell::new(
                SmellKind::UndefinedTag,
                Severity::Warning,
                "Project".to_string(),
                tag_name.clone(),
                format!("Tag '{}' is used but not defined", tag_name),
            ));
        }
    }

    fn is_builtin(&self, name: &str) -> bool {
        // Common builtin names
        matches!(name.to_uppercase().as_str(),
            "S:FS" | "S:Z" | "S:N" | "S:C" | "S:V" | "AFI" | "NOP" |
            "XIC" | "XIO" | "OTE" | "OTL" | "OTU" | "ONS" | "OSR" | "OSF" |
            "TON" | "TOF" | "RTO" | "CTU" | "CTD" | "RES" |
            "ADD" | "SUB" | "MUL" | "DIV" | "MOD" | "NEG" | "ABS" | "SQRT" |
            "MOV" | "COP" | "CPS" | "FLL" | "CLR" |
            "EQU" | "NEQ" | "LES" | "LEQ" | "GRT" | "GEQ" | "CMP" | "LIM" |
            "JSR" | "JMP" | "LBL" | "RET" | "SBR" | "TND" | "MCR" | "END"
        )
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

/// Detect empty POUs using model analysis.
pub struct ModelEmptyRoutinesDetector<'a> {
    config: &'a EmptyRoutinesConfig,
}

impl<'a> ModelEmptyRoutinesDetector<'a> {
    pub fn new(config: &'a EmptyRoutinesConfig) -> Self {
        Self { config }
    }

    pub fn detect(&self, analysis: &ModelAnalysis, report: &mut Report) {
        if !self.config.enabled {
            return;
        }

        for pou_name in &analysis.empty_pous {
            // Skip if matches ignore pattern
            if self.matches_ignore_pattern(pou_name) {
                continue;
            }

            report.add(Smell::new(
                SmellKind::EmptyBlock,
                Severity::Info,
                pou_name.clone(),
                pou_name.clone(),
                format!("POU '{}' has no code", pou_name),
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
                // * matches any sequence
                if pattern_chars.peek().is_none() {
                    return true; // trailing * matches everything
                }
                // Try matching rest of pattern at each position in remaining text
                let rest_pattern: String = pattern_chars.collect();
                let rest_text: String = text_chars.collect();
                // Try matching from each position
                for i in 0..=rest_text.len() {
                    if glob_match(&rest_pattern, &rest_text[i..]) {
                        return true;
                    }
                }
                return false;
            }
            '?' => {
                // ? matches any single character
                if text_chars.next().is_none() {
                    return false;
                }
            }
            c => {
                // Literal character must match
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
