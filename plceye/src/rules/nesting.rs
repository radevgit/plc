//! Deep nesting detector.
//!
//! Detects ST routines with excessively deep control structure nesting (M0003).

use iecst::max_nesting_depth;

use crate::analysis::ProjectAnalysis;
use crate::config::NestingConfig;
use crate::report::{Report, Rule, RuleKind, Severity};

/// Detector for deep nesting in ST routines.
pub struct NestingDetector<'a> {
    config: &'a NestingConfig,
}

impl<'a> NestingDetector<'a> {
    /// Create a new nesting detector with the given configuration.
    pub fn new(config: &'a NestingConfig) -> Self {
        Self { config }
    }

    /// Run detection on analyzed ST routines and add findings to the report.
    pub fn detect(&self, analysis: &ProjectAnalysis, report: &mut Report) {
        if !self.config.enabled {
            return;
        }

        for st_routine in &analysis.st_routines {
            // Skip if matches ignore pattern
            if self.matches_ignore_pattern(&st_routine.location.routine) {
                continue;
            }

            // Need a parsed POU to analyze
            let Some(ref pou) = st_routine.pou else {
                continue;
            };

            // Calculate maximum nesting depth
            let depth = max_nesting_depth(&pou.body);

            if depth > self.config.max_depth {
                report.add(Rule::new(
                    RuleKind::DeepNesting,
                    self.severity_for_depth(depth),
                    format!("Program:{}", st_routine.location.program),
                    st_routine.location.routine.clone(),
                    format!(
                        "Routine '{}' has nesting depth of {} (max: {})",
                        st_routine.location.routine,
                        depth,
                        self.config.max_depth
                    ),
                ));
            }
        }
    }

    /// Determine severity based on how much depth exceeds threshold.
    fn severity_for_depth(&self, depth: usize) -> Severity {
        let threshold = self.config.max_depth;
        // Severe: more than 2x the threshold
        if depth > threshold * 2 {
            Severity::Error
        // Moderate: more than 50% over threshold
        } else if depth > threshold + threshold / 2 {
            Severity::Warning
        } else {
            Severity::Info
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
    use crate::analysis::{ParsedSTRoutine, STLocation, ParseStats, RoutineSummary};
    use std::collections::HashMap;

    fn create_test_analysis(st_source: &str) -> ProjectAnalysis {
        // Wrap source in a PROGRAM to parse as POU
        let wrapped = format!("PROGRAM Test\nVAR\nEND_VAR\n{}\nEND_PROGRAM", st_source);
        let pou = iecst::parse_pou(&wrapped).ok();
        let st_routine = ParsedSTRoutine {
            location: STLocation::new("MainProgram", "TestRoutine"),
            source: st_source.to_string(),
            pou,
            parse_error: None,
        };

        ProjectAnalysis {
            rungs: vec![],
            st_routines: vec![st_routine],
            tag_references: vec![],
            tag_xref: HashMap::new(),
            routines: vec![RoutineSummary {
                program: "MainProgram".to_string(),
                routine: "TestRoutine".to_string(),
                routine_type: "ST".to_string(),
                rung_count: 0,
                rung_indices: vec![],
                parse_errors: 0,
                tags_used: vec![],
                instructions: HashMap::new(),
            }],
            instruction_usage: HashMap::new(),
            aoi_definitions: vec![],
            aoi_usage: HashMap::new(),
            stats: ParseStats::default(),
        }
    }

    #[test]
    fn test_shallow_routine_no_violation() {
        let config = NestingConfig {
            enabled: true,
            max_depth: 4,
            ignore_patterns: vec![],
        };
        let detector = NestingDetector::new(&config);

        // Simple routine with depth 1
        let analysis = create_test_analysis("IF a THEN x := 1; END_IF;");
        let mut report = Report::new();

        detector.detect(&analysis, &mut report);

        assert!(report.rules.is_empty(), "Shallow routine should not trigger nesting rule");
    }

    #[test]
    fn test_deep_routine_violation() {
        let config = NestingConfig {
            enabled: true,
            max_depth: 2,  // Low threshold for testing
            ignore_patterns: vec![],
        };
        let detector = NestingDetector::new(&config);

        // Routine with depth 3
        let analysis = create_test_analysis(
            r#"
            IF a THEN
                IF b THEN
                    IF c THEN
                        x := 1;
                    END_IF;
                END_IF;
            END_IF;
            "#
        );
        let mut report = Report::new();

        detector.detect(&analysis, &mut report);

        assert_eq!(report.rules.len(), 1);
        assert_eq!(report.rules[0].kind, RuleKind::DeepNesting);
        assert!(report.rules[0].message.contains("nesting depth of 3"));
    }

    #[test]
    fn test_disabled_detector() {
        let config = NestingConfig {
            enabled: false,
            max_depth: 1,
            ignore_patterns: vec![],
        };
        let detector = NestingDetector::new(&config);

        let analysis = create_test_analysis("IF a THEN IF b THEN x := 1; END_IF; END_IF;");
        let mut report = Report::new();

        detector.detect(&analysis, &mut report);

        assert!(report.rules.is_empty(), "Disabled detector should not report");
    }

    #[test]
    fn test_ignore_pattern() {
        let config = NestingConfig {
            enabled: true,
            max_depth: 1,
            ignore_patterns: vec!["Test*".to_string()],
        };
        let detector = NestingDetector::new(&config);

        let analysis = create_test_analysis("IF a THEN IF b THEN x := 1; END_IF; END_IF;");
        let mut report = Report::new();

        detector.detect(&analysis, &mut report);

        assert!(report.rules.is_empty(), "Ignored routine should not be reported");
    }

    #[test]
    fn test_severity_escalation() {
        let config = NestingConfig {
            enabled: true,
            max_depth: 4,
            ignore_patterns: vec![],
        };
        let detector = NestingDetector::new(&config);

        // With threshold 4:
        // - Info: depth <= 6 (4 + 4/2)
        // - Warning: 6 < depth <= 8 (4 * 2)
        // - Error: depth > 8
        
        // Mild violation: Info
        assert_eq!(detector.severity_for_depth(5), Severity::Info);

        // Moderate violation: Warning
        assert_eq!(detector.severity_for_depth(7), Severity::Warning);

        // Severe violation (>2x threshold): Error
        assert_eq!(detector.severity_for_depth(10), Severity::Error);
    }

    #[test]
    fn test_for_loop_nesting() {
        let config = NestingConfig {
            enabled: true,
            max_depth: 1,
            ignore_patterns: vec![],
        };
        let detector = NestingDetector::new(&config);

        let analysis = create_test_analysis(
            r#"
            FOR i := 1 TO 10 DO
                FOR j := 1 TO 10 DO
                    x := i + j;
                END_FOR;
            END_FOR;
            "#
        );
        let mut report = Report::new();
        detector.detect(&analysis, &mut report);

        assert_eq!(report.rules.len(), 1);
        assert_eq!(report.rules[0].kind, RuleKind::DeepNesting);
        assert!(report.rules[0].message.contains("nesting depth of 2"));
    }

    #[test]
    fn test_mixed_control_structures() {
        let config = NestingConfig {
            enabled: true,
            max_depth: 2,
            ignore_patterns: vec![],
        };
        let detector = NestingDetector::new(&config);

        // Mix of IF, FOR, WHILE at depth 3
        let analysis = create_test_analysis(
            r#"
            IF a THEN
                FOR i := 1 TO 10 DO
                    WHILE b DO
                        x := 1;
                    END_WHILE;
                END_FOR;
            END_IF;
            "#
        );
        let mut report = Report::new();
        detector.detect(&analysis, &mut report);

        assert_eq!(report.rules.len(), 1);
        assert!(report.rules[0].message.contains("nesting depth of 3"));
    }

    #[test]
    fn test_case_nesting() {
        let config = NestingConfig {
            enabled: true,
            max_depth: 1,
            ignore_patterns: vec![],
        };
        let detector = NestingDetector::new(&config);

        let analysis = create_test_analysis(
            r#"
            CASE x OF
                1:
                    IF a THEN
                        y := 1;
                    END_IF;
            END_CASE;
            "#
        );
        let mut report = Report::new();
        detector.detect(&analysis, &mut report);

        assert_eq!(report.rules.len(), 1);
        assert!(report.rules[0].message.contains("nesting depth of 2"));
    }

    #[test]
    fn test_unparseable_routine_skipped() {
        let config = NestingConfig {
            enabled: true,
            max_depth: 1,
            ignore_patterns: vec![],
        };
        let detector = NestingDetector::new(&config);

        // Create analysis with unparseable routine (pou = None)
        let st_routine = ParsedSTRoutine {
            location: STLocation::new("MainProgram", "BadRoutine"),
            source: "not valid ST {{{{".to_string(),
            pou: None,
            parse_error: None,
        };

        let analysis = ProjectAnalysis {
            rungs: vec![],
            st_routines: vec![st_routine],
            tag_references: vec![],
            tag_xref: HashMap::new(),
            routines: vec![],
            instruction_usage: HashMap::new(),
            aoi_definitions: vec![],
            aoi_usage: HashMap::new(),
            stats: ParseStats::default(),
        };

        let mut report = Report::new();
        detector.detect(&analysis, &mut report);

        assert!(report.rules.is_empty(), "Unparseable routine should not trigger rule");
    }
}
