//! Nesting depth analysis for IEC 61131-3 Structured Text.
//!
//! This module provides utilities to calculate the maximum nesting depth
//! of control structures in ST code.

use crate::ast::{Stmt, StmtKind};

/// Calculate the maximum nesting depth of statements.
///
/// Nesting depth is incremented by:
/// - IF/ELSIF/ELSE blocks
/// - CASE branches
/// - FOR loops
/// - WHILE loops
/// - REPEAT loops
///
/// # Example
///
/// ```
/// use iecst::{parse_statements, max_nesting_depth};
///
/// let code = r#"
///     IF a THEN
///         IF b THEN
///             x := 1;
///         END_IF;
///     END_IF;
/// "#;
/// let stmts = parse_statements(code).unwrap();
/// assert_eq!(max_nesting_depth(&stmts), 2);
/// ```
pub fn max_nesting_depth(statements: &[Stmt]) -> usize {
    calculate_depth(statements, 0)
}

fn calculate_depth(statements: &[Stmt], current_depth: usize) -> usize {
    let mut max_depth = current_depth;

    for stmt in statements {
        let depth = match &stmt.kind {
            StmtKind::If {
                then_body,
                elsif_branches,
                else_body,
                ..
            } => {
                let nested = current_depth + 1;
                let mut branch_max = calculate_depth(then_body, nested);

                for (_, body) in elsif_branches {
                    branch_max = branch_max.max(calculate_depth(body, nested));
                }

                if let Some(body) = else_body {
                    branch_max = branch_max.max(calculate_depth(body, nested));
                }

                branch_max
            }

            StmtKind::Case { cases, else_body, .. } => {
                let nested = current_depth + 1;
                let mut branch_max = nested;

                for case in cases {
                    branch_max = branch_max.max(calculate_depth(&case.body, nested));
                }

                if let Some(body) = else_body {
                    branch_max = branch_max.max(calculate_depth(body, nested));
                }

                branch_max
            }

            StmtKind::For { body, .. } => {
                calculate_depth(body, current_depth + 1)
            }

            StmtKind::While { body, .. } => {
                calculate_depth(body, current_depth + 1)
            }

            StmtKind::Repeat { body, .. } => {
                calculate_depth(body, current_depth + 1)
            }

            // Non-nesting statements
            StmtKind::Assignment { .. }
            | StmtKind::Call { .. }
            | StmtKind::Exit
            | StmtKind::Continue
            | StmtKind::Return { .. }
            | StmtKind::Empty => current_depth,
        };

        max_depth = max_depth.max(depth);
    }

    max_depth
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_statements;

    #[test]
    fn test_no_nesting() {
        let stmts = parse_statements("x := 1; y := 2;").unwrap();
        assert_eq!(max_nesting_depth(&stmts), 0);
    }

    #[test]
    fn test_single_if() {
        let stmts = parse_statements("IF a THEN x := 1; END_IF;").unwrap();
        assert_eq!(max_nesting_depth(&stmts), 1);
    }

    #[test]
    fn test_nested_if() {
        let code = r#"
            IF a THEN
                IF b THEN
                    x := 1;
                END_IF;
            END_IF;
        "#;
        let stmts = parse_statements(code).unwrap();
        assert_eq!(max_nesting_depth(&stmts), 2);
    }

    #[test]
    fn test_deeply_nested() {
        let code = r#"
            IF a THEN
                IF b THEN
                    IF c THEN
                        IF d THEN
                            IF e THEN
                                x := 1;
                            END_IF;
                        END_IF;
                    END_IF;
                END_IF;
            END_IF;
        "#;
        let stmts = parse_statements(code).unwrap();
        assert_eq!(max_nesting_depth(&stmts), 5);
    }

    #[test]
    fn test_elsif_same_depth() {
        let code = r#"
            IF a THEN
                x := 1;
            ELSIF b THEN
                x := 2;
            ELSE
                x := 3;
            END_IF;
        "#;
        let stmts = parse_statements(code).unwrap();
        assert_eq!(max_nesting_depth(&stmts), 1);
    }

    #[test]
    fn test_for_loop() {
        let code = r#"
            FOR i := 1 TO 10 DO
                x := x + 1;
            END_FOR;
        "#;
        let stmts = parse_statements(code).unwrap();
        assert_eq!(max_nesting_depth(&stmts), 1);
    }

    #[test]
    fn test_while_loop() {
        let code = r#"
            WHILE a DO
                x := x + 1;
            END_WHILE;
        "#;
        let stmts = parse_statements(code).unwrap();
        assert_eq!(max_nesting_depth(&stmts), 1);
    }

    #[test]
    fn test_repeat_loop() {
        let code = r#"
            REPEAT
                x := x + 1;
            UNTIL a;
        "#;
        let stmts = parse_statements(code).unwrap();
        assert_eq!(max_nesting_depth(&stmts), 1);
    }

    #[test]
    fn test_mixed_nesting() {
        let code = r#"
            FOR i := 1 TO 10 DO
                IF a THEN
                    WHILE b DO
                        x := x + 1;
                    END_WHILE;
                END_IF;
            END_FOR;
        "#;
        let stmts = parse_statements(code).unwrap();
        assert_eq!(max_nesting_depth(&stmts), 3);
    }

    #[test]
    fn test_case_statement() {
        let code = r#"
            CASE x OF
                1: y := 1;
                2: y := 2;
            ELSE
                y := 0;
            END_CASE;
        "#;
        let stmts = parse_statements(code).unwrap();
        assert_eq!(max_nesting_depth(&stmts), 1);
    }

    #[test]
    fn test_case_with_nested_if() {
        let code = r#"
            CASE x OF
                1:
                    IF a THEN
                        y := 1;
                    END_IF;
            END_CASE;
        "#;
        let stmts = parse_statements(code).unwrap();
        assert_eq!(max_nesting_depth(&stmts), 2);
    }

    #[test]
    fn test_parallel_branches_max() {
        // Two parallel branches, one deeper than the other
        let code = r#"
            IF a THEN
                x := 1;
            END_IF;
            IF b THEN
                IF c THEN
                    IF d THEN
                        x := 2;
                    END_IF;
                END_IF;
            END_IF;
        "#;
        let stmts = parse_statements(code).unwrap();
        assert_eq!(max_nesting_depth(&stmts), 3);
    }
}
