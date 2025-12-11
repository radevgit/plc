//! Cyclomatic complexity detector.
//!
//! Detects ST routines with high cyclomatic complexity (M0001).

use iec61131::analysis::CfgBuilder;

use crate::analysis::ProjectAnalysis;
use crate::config::ComplexityConfig;
use crate::report::{Report, Rule, RuleKind, Severity};

/// Detector for cyclomatic complexity in ST routines.
pub struct ComplexityDetector<'a> {
    config: &'a ComplexityConfig,
}

impl<'a> ComplexityDetector<'a> {
    /// Create a new complexity detector with the given configuration.
    pub fn new(config: &'a ComplexityConfig) -> Self {
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

            // Build CFG from POU body and calculate complexity
            let cfg = CfgBuilder::new().build(&pou.body);
            let complexity = cfg.cyclomatic_complexity();

            if complexity > self.config.max_complexity {
                report.add(Rule::new(
                    RuleKind::CyclomaticComplexity,
                    self.severity_for_complexity(complexity),
                    format!("Program:{}", st_routine.location.program),
                    st_routine.location.routine.clone(),
                    format!(
                        "Routine '{}' has cyclomatic complexity of {} (max: {})",
                        st_routine.location.routine,
                        complexity,
                        self.config.max_complexity
                    ),
                ));
            }
        }
    }

    /// Determine severity based on how much complexity exceeds threshold.
    fn severity_for_complexity(&self, complexity: usize) -> Severity {
        let threshold = self.config.max_complexity;
        // Severe: more than 2x the threshold
        if complexity > threshold * 2 {
            Severity::Error
        // Moderate: more than 50% over threshold
        } else if complexity > threshold + threshold / 2 {
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
        let pou = crate::analysis::parse_pou(&wrapped).ok();
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
    fn test_simple_routine_no_violation() {
        let config = ComplexityConfig {
            enabled: true,
            max_complexity: 10,
            ignore_patterns: vec![],
        };
        let detector = ComplexityDetector::new(&config);

        // Simple routine with complexity 1
        let analysis = create_test_analysis("x := 1;");
        let mut report = Report::new();

        detector.detect(&analysis, &mut report);

        assert!(report.rules.is_empty(), "Simple routine should not trigger complexity rule");
    }

    #[test]
    fn test_complex_routine_violation() {
        let config = ComplexityConfig {
            enabled: true,
            max_complexity: 2,  // Very low threshold for testing
            ignore_patterns: vec![],
        };
        let detector = ComplexityDetector::new(&config);

        // Routine with multiple branches (complexity > 2)
        let analysis = create_test_analysis(
            r#"
            IF a THEN
                x := 1;
            ELSIF b THEN
                x := 2;
            ELSIF c THEN
                x := 3;
            END_IF;
            "#
        );
        let mut report = Report::new();

        detector.detect(&analysis, &mut report);

        assert_eq!(report.rules.len(), 1);
        assert_eq!(report.rules[0].kind, RuleKind::CyclomaticComplexity);
        assert!(report.rules[0].message.contains("cyclomatic complexity"));
    }

    #[test]
    fn test_disabled_detector() {
        let config = ComplexityConfig {
            enabled: false,
            max_complexity: 1,
            ignore_patterns: vec![],
        };
        let detector = ComplexityDetector::new(&config);

        let analysis = create_test_analysis("IF a THEN x := 1; END_IF;");
        let mut report = Report::new();

        detector.detect(&analysis, &mut report);

        assert!(report.rules.is_empty(), "Disabled detector should not report");
    }

    #[test]
    fn test_ignore_pattern() {
        let config = ComplexityConfig {
            enabled: true,
            max_complexity: 1,
            ignore_patterns: vec!["Test*".to_string()],
        };
        let detector = ComplexityDetector::new(&config);

        let analysis = create_test_analysis("IF a THEN x := 1; END_IF;");
        let mut report = Report::new();

        detector.detect(&analysis, &mut report);

        assert!(report.rules.is_empty(), "Ignored routine should not be reported");
    }

    #[test]
    fn test_severity_escalation() {
        let config = ComplexityConfig {
            enabled: true,
            max_complexity: 10,
            ignore_patterns: vec![],
        };
        let detector = ComplexityDetector::new(&config);

        // With threshold 10:
        // - Info: complexity <= 15 (10 + 10/2)
        // - Warning: 15 < complexity <= 20 (10 * 2)
        // - Error: complexity > 20
        
        // Mild violation: Info
        assert_eq!(detector.severity_for_complexity(12), Severity::Info);

        // Moderate violation: Warning
        assert_eq!(detector.severity_for_complexity(18), Severity::Warning);

        // Severe violation (>2x threshold): Error
        assert_eq!(detector.severity_for_complexity(25), Severity::Error);
    }

    #[test]
    fn test_loop_adds_complexity() {
        let config = ComplexityConfig {
            enabled: true,
            max_complexity: 1,
            ignore_patterns: vec![],
        };
        let detector = ComplexityDetector::new(&config);

        // FOR loop adds complexity
        let analysis = create_test_analysis(
            r#"
            FOR i := 1 TO 10 DO
                x := x + 1;
            END_FOR;
            "#
        );
        let mut report = Report::new();
        detector.detect(&analysis, &mut report);

        assert_eq!(report.rules.len(), 1);
        assert_eq!(report.rules[0].kind, RuleKind::CyclomaticComplexity);
    }

    #[test]
    fn test_while_adds_complexity() {
        let config = ComplexityConfig {
            enabled: true,
            max_complexity: 1,
            ignore_patterns: vec![],
        };
        let detector = ComplexityDetector::new(&config);

        let analysis = create_test_analysis("WHILE a DO x := x + 1; END_WHILE;");
        let mut report = Report::new();
        detector.detect(&analysis, &mut report);

        assert_eq!(report.rules.len(), 1);
    }

    #[test]
    fn test_case_adds_complexity() {
        let config = ComplexityConfig {
            enabled: true,
            max_complexity: 1,
            ignore_patterns: vec![],
        };
        let detector = ComplexityDetector::new(&config);

        let analysis = create_test_analysis(
            r#"
            CASE x OF
                1: y := 1;
                2: y := 2;
                3: y := 3;
            END_CASE;
            "#
        );
        let mut report = Report::new();
        detector.detect(&analysis, &mut report);

        assert_eq!(report.rules.len(), 1);
    }

    #[test]
    fn test_unparseable_routine_skipped() {
        let config = ComplexityConfig {
            enabled: true,
            max_complexity: 1,
            ignore_patterns: vec![],
        };
        let detector = ComplexityDetector::new(&config);

        // Create analysis with unparseable routine (pou = None)
        let st_routine = ParsedSTRoutine {
            location: STLocation::new("MainProgram", "BadRoutine"),
            source: "this is not valid ST {{{{".to_string(),
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
