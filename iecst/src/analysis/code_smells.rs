//! Code smell detection for Structured Text.

use crate::ast::*;
use crate::Span;
use crate::analysis::{Diagnostic, DiagnosticKind, Severity};

/// Configuration for code smell detection.
#[derive(Debug, Clone)]
pub struct SmellConfig {
    /// Maximum recommended nesting depth
    pub max_nesting: usize,
    /// Maximum recommended function length (statements)
    pub max_function_length: usize,
    /// Maximum recommended condition complexity
    pub max_condition_complexity: usize,
    /// Whether to warn about magic numbers
    pub warn_magic_numbers: bool,
    /// Magic number exceptions (e.g., 0, 1, -1)
    pub magic_number_exceptions: Vec<i64>,
}

impl Default for SmellConfig {
    fn default() -> Self {
        Self {
            max_nesting: 4,
            max_function_length: 50,
            max_condition_complexity: 4,
            warn_magic_numbers: true,
            magic_number_exceptions: vec![-1, 0, 1, 2, 10, 100],
        }
    }
}

/// A detected code smell.
#[derive(Debug, Clone)]
pub struct CodeSmell {
    /// Kind of smell
    pub kind: SmellKind,
    /// Location
    pub span: Span,
    /// Severity
    pub severity: Severity,
}

/// Kinds of code smells.
#[derive(Debug, Clone, PartialEq)]
pub enum SmellKind {
    /// Empty statement block
    EmptyBlock { block_type: String },
    /// Deeply nested code
    DeepNesting { depth: usize },
    /// Long function
    LongFunction { statement_count: usize },
    /// Complex condition
    ComplexCondition { complexity: usize },
    /// Magic number
    MagicNumber { value: i64 },
    /// Empty CASE branch
    EmptyCaseBranch,
    /// Missing ELSE in CASE
    MissingCaseElse,
    /// Duplicate comparison
    DuplicateComparison,
    /// Constant condition
    ConstantCondition { value: bool },
    /// Dead code after return/exit
    DeadCode,
}

/// Detector for code smells.
pub struct CodeSmellDetector {
    config: SmellConfig,
    current_nesting: usize,
}

impl CodeSmellDetector {
    /// Create a new code smell detector with default config.
    pub fn new() -> Self {
        Self::with_config(SmellConfig::default())
    }

    /// Create a detector with custom configuration.
    pub fn with_config(config: SmellConfig) -> Self {
        Self {
            config,
            current_nesting: 0,
        }
    }

    /// Analyze a POU for code smells.
    pub fn analyze_pou(&mut self, pou: &Pou) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Check function length
        let stmt_count = self.count_statements(&pou.body);
        if stmt_count > self.config.max_function_length {
            diagnostics.push(Diagnostic::warning(
                DiagnosticKind::LongFunction {
                    lines: stmt_count,
                    max_recommended: self.config.max_function_length,
                },
                pou.span,
            ));
        }

        // Check body for smells
        for stmt in &pou.body {
            diagnostics.extend(self.check_statement(stmt));
        }

        // Check for dead code (statements after unconditional return)
        diagnostics.extend(self.check_dead_code(&pou.body));

        diagnostics
    }

    /// Check a statement for code smells.
    fn check_statement(&mut self, stmt: &Stmt) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        match &stmt.kind {
            StmtKind::If { condition, then_body, elsif_branches, else_body } => {
                // Check condition complexity
                diagnostics.extend(self.check_condition_complexity(condition));

                // Check for constant condition
                diagnostics.extend(self.check_constant_condition(condition));

                // Check for empty blocks
                if then_body.is_empty() {
                    diagnostics.push(Diagnostic::warning(
                        DiagnosticKind::EmptyBlock { block_type: "IF".to_string() },
                        stmt.span,
                    ));
                }

                // Check nesting
                self.current_nesting += 1;
                if self.current_nesting > self.config.max_nesting {
                    diagnostics.push(Diagnostic::warning(
                        DiagnosticKind::DeepNesting {
                            depth: self.current_nesting,
                            max_recommended: self.config.max_nesting,
                        },
                        stmt.span,
                    ));
                }

                // Recurse into bodies
                for s in then_body {
                    diagnostics.extend(self.check_statement(s));
                }
                for (cond, body) in elsif_branches {
                    diagnostics.extend(self.check_condition_complexity(cond));
                    for s in body {
                        diagnostics.extend(self.check_statement(s));
                    }
                }
                if let Some(else_stmts) = else_body {
                    if else_stmts.is_empty() {
                        diagnostics.push(Diagnostic::hint(
                            DiagnosticKind::EmptyBlock { block_type: "ELSE".to_string() },
                            stmt.span,
                        ));
                    }
                    for s in else_stmts {
                        diagnostics.extend(self.check_statement(s));
                    }
                }

                self.current_nesting -= 1;
            }

            StmtKind::Case { cases, else_body, .. } => {
                // Check for empty branches
                for branch in cases {
                    if branch.body.is_empty() {
                        diagnostics.push(Diagnostic::warning(
                            DiagnosticKind::EmptyCaseBranch,
                            branch.span,
                        ));
                    }
                    for s in &branch.body {
                        diagnostics.extend(self.check_statement(s));
                    }
                }

                // Check for missing ELSE
                if else_body.is_none() {
                    diagnostics.push(Diagnostic::hint(
                        DiagnosticKind::MissingCaseElse,
                        stmt.span,
                    ));
                } else if let Some(else_stmts) = else_body {
                    for s in else_stmts {
                        diagnostics.extend(self.check_statement(s));
                    }
                }
            }

            StmtKind::For { body, .. } => {
                self.current_nesting += 1;
                if self.current_nesting > self.config.max_nesting {
                    diagnostics.push(Diagnostic::warning(
                        DiagnosticKind::DeepNesting {
                            depth: self.current_nesting,
                            max_recommended: self.config.max_nesting,
                        },
                        stmt.span,
                    ));
                }

                if body.is_empty() {
                    diagnostics.push(Diagnostic::warning(
                        DiagnosticKind::EmptyBlock { block_type: "FOR".to_string() },
                        stmt.span,
                    ));
                }

                for s in body {
                    diagnostics.extend(self.check_statement(s));
                }
                self.current_nesting -= 1;
            }

            StmtKind::While { condition, body } => {
                diagnostics.extend(self.check_condition_complexity(condition));
                diagnostics.extend(self.check_constant_condition(condition));

                self.current_nesting += 1;
                if self.current_nesting > self.config.max_nesting {
                    diagnostics.push(Diagnostic::warning(
                        DiagnosticKind::DeepNesting {
                            depth: self.current_nesting,
                            max_recommended: self.config.max_nesting,
                        },
                        stmt.span,
                    ));
                }

                if body.is_empty() {
                    diagnostics.push(Diagnostic::warning(
                        DiagnosticKind::EmptyBlock { block_type: "WHILE".to_string() },
                        stmt.span,
                    ));
                }

                for s in body {
                    diagnostics.extend(self.check_statement(s));
                }
                self.current_nesting -= 1;
            }

            StmtKind::Repeat { body, until } => {
                diagnostics.extend(self.check_condition_complexity(until));

                self.current_nesting += 1;
                if self.current_nesting > self.config.max_nesting {
                    diagnostics.push(Diagnostic::warning(
                        DiagnosticKind::DeepNesting {
                            depth: self.current_nesting,
                            max_recommended: self.config.max_nesting,
                        },
                        stmt.span,
                    ));
                }

                if body.is_empty() {
                    diagnostics.push(Diagnostic::warning(
                        DiagnosticKind::EmptyBlock { block_type: "REPEAT".to_string() },
                        stmt.span,
                    ));
                }

                for s in body {
                    diagnostics.extend(self.check_statement(s));
                }
                self.current_nesting -= 1;
            }

            StmtKind::Assignment { value, .. } => {
                // Check for magic numbers
                if self.config.warn_magic_numbers {
                    diagnostics.extend(self.check_magic_numbers(value));
                }
            }

            _ => {}
        }

        diagnostics
    }

    /// Check condition complexity (count of AND/OR/XOR operations).
    fn check_condition_complexity(&self, condition: &Expr) -> Vec<Diagnostic> {
        let complexity = self.count_logical_ops(condition);
        if complexity > self.config.max_condition_complexity {
            vec![Diagnostic::warning(
                DiagnosticKind::ComplexCondition {
                    complexity,
                    max_recommended: self.config.max_condition_complexity,
                },
                condition.span,
            )]
        } else {
            vec![]
        }
    }

    /// Count logical operators in an expression.
    fn count_logical_ops(&self, expr: &Expr) -> usize {
        match &expr.kind {
            ExprKind::BinaryOp { left, op, right } => {
                let count = if matches!(op, BinaryOp::And | BinaryOp::Or | BinaryOp::Xor) {
                    1
                } else {
                    0
                };
                count + self.count_logical_ops(left) + self.count_logical_ops(right)
            }
            ExprKind::UnaryOp { expr, .. } => self.count_logical_ops(expr),
            ExprKind::Paren(inner) => self.count_logical_ops(inner),
            _ => 0,
        }
    }

    /// Check for constant conditions (always true/false).
    fn check_constant_condition(&self, condition: &Expr) -> Vec<Diagnostic> {
        match &condition.kind {
            ExprKind::BoolLiteral(value) => {
                vec![Diagnostic::warning(
                    DiagnosticKind::RedundantCondition { always: *value },
                    condition.span,
                )]
            }
            _ => vec![],
        }
    }

    /// Check for magic numbers in an expression.
    fn check_magic_numbers(&self, expr: &Expr) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        match &expr.kind {
            ExprKind::IntLiteral(value) => {
                if !self.config.magic_number_exceptions.contains(value) {
                    diagnostics.push(Diagnostic::hint(
                        DiagnosticKind::MagicNumber { value: value.to_string() },
                        expr.span,
                    ));
                }
            }
            ExprKind::BinaryOp { left, right, .. } => {
                diagnostics.extend(self.check_magic_numbers(left));
                diagnostics.extend(self.check_magic_numbers(right));
            }
            ExprKind::UnaryOp { expr: inner, .. } => {
                diagnostics.extend(self.check_magic_numbers(inner));
            }
            ExprKind::FunctionCall { args, .. } => {
                for arg in args {
                    if let Some(value) = &arg.value {
                        diagnostics.extend(self.check_magic_numbers(value));
                    }
                }
            }
            ExprKind::ArrayIndex { array, indices } => {
                diagnostics.extend(self.check_magic_numbers(array));
                for idx in indices {
                    diagnostics.extend(self.check_magic_numbers(idx));
                }
            }
            ExprKind::Paren(inner) => {
                diagnostics.extend(self.check_magic_numbers(inner));
            }
            _ => {}
        }

        diagnostics
    }

    /// Check for dead code after return/exit.
    fn check_dead_code(&self, statements: &[Stmt]) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut found_terminal = false;

        for stmt in statements {
            if found_terminal {
                diagnostics.push(Diagnostic::warning(
                    DiagnosticKind::DeadCode {
                        reason: "code after RETURN or EXIT".to_string(),
                    },
                    stmt.span,
                ));
            }

            match &stmt.kind {
                StmtKind::Return { .. } | StmtKind::Exit => {
                    found_terminal = true;
                }
                _ => {}
            }
        }

        diagnostics
    }

    /// Count total statements recursively.
    fn count_statements(&self, statements: &[Stmt]) -> usize {
        let mut count = 0;
        for stmt in statements {
            count += 1;
            count += match &stmt.kind {
                StmtKind::If { then_body, elsif_branches, else_body, .. } => {
                    let mut inner = self.count_statements(then_body);
                    for (_, body) in elsif_branches {
                        inner += self.count_statements(body);
                    }
                    if let Some(else_stmts) = else_body {
                        inner += self.count_statements(else_stmts);
                    }
                    inner
                }
                StmtKind::Case { cases, else_body, .. } => {
                    let mut inner = 0;
                    for branch in cases {
                        inner += self.count_statements(&branch.body);
                    }
                    if let Some(else_stmts) = else_body {
                        inner += self.count_statements(else_stmts);
                    }
                    inner
                }
                StmtKind::For { body, .. } |
                StmtKind::While { body, .. } => self.count_statements(body),
                StmtKind::Repeat { body, .. } => self.count_statements(body),
                _ => 0,
            };
        }
        count
    }
}

impl Default for CodeSmellDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_pou;

    #[test]
    fn test_empty_block_detection() {
        let code = r#"
            PROGRAM Test
            VAR
                x : INT;
            END_VAR
                IF x > 0 THEN
                END_IF;
            END_PROGRAM
        "#;
        let pou = parse_pou(code).unwrap();
        let mut detector = CodeSmellDetector::new();
        let diagnostics = detector.analyze_pou(&pou);
        
        assert!(diagnostics.iter().any(|d| matches!(
            d.kind,
            DiagnosticKind::EmptyBlock { ref block_type } if block_type == "IF"
        )));
    }

    #[test]
    fn test_deep_nesting_detection() {
        let code = r#"
            PROGRAM Test
            VAR
                x : INT;
            END_VAR
                IF x > 0 THEN
                    IF x > 1 THEN
                        IF x > 2 THEN
                            IF x > 3 THEN
                                IF x > 4 THEN
                                    x := 5;
                                END_IF;
                            END_IF;
                        END_IF;
                    END_IF;
                END_IF;
            END_PROGRAM
        "#;
        let pou = parse_pou(code).unwrap();
        let mut detector = CodeSmellDetector::new();
        let diagnostics = detector.analyze_pou(&pou);
        
        assert!(diagnostics.iter().any(|d| matches!(
            d.kind,
            DiagnosticKind::DeepNesting { .. }
        )));
    }

    #[test]
    fn test_constant_condition_detection() {
        let code = r#"
            PROGRAM Test
            VAR
                x : INT;
            END_VAR
                IF TRUE THEN
                    x := 1;
                END_IF;
            END_PROGRAM
        "#;
        let pou = parse_pou(code).unwrap();
        let mut detector = CodeSmellDetector::new();
        let diagnostics = detector.analyze_pou(&pou);
        
        assert!(diagnostics.iter().any(|d| matches!(
            d.kind,
            DiagnosticKind::RedundantCondition { always: true }
        )));
    }

    #[test]
    fn test_missing_case_else() {
        let code = r#"
            PROGRAM Test
            VAR
                state : INT;
                x : INT;
            END_VAR
                CASE state OF
                    0: x := 1;
                    1: x := 2;
                END_CASE;
            END_PROGRAM
        "#;
        let pou = parse_pou(code).unwrap();
        let mut detector = CodeSmellDetector::new();
        let diagnostics = detector.analyze_pou(&pou);
        
        assert!(diagnostics.iter().any(|d| matches!(
            d.kind,
            DiagnosticKind::MissingCaseElse
        )));
    }
}
