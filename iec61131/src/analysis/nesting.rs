//! Nesting depth analysis for IEC 61131-3 Structured Text.
//!
//! This module provides utilities to calculate the maximum nesting depth
//! of control structures in ST code.

use crate::generated::ast::Statement;

/// Calculate the maximum nesting depth of statements.
///
///Nesting depth is incremented by:
/// - IF/ELSIF/ELSE blocks
/// - CASE branches
/// - FOR loops
/// - WHILE loops
/// - REPEAT loops
///
/// # Example
///
/// ```
/// use iec61131::{Parser, analysis::max_nesting_depth};
///
/// let code = r#"
/// FUNCTION Test : INT
///     IF a THEN
///         IF b THEN
///             x := 1;
///         END_IF;
///     END_IF;
/// END_FUNCTION
/// "#;
/// let mut parser = Parser::new(code);
/// let cu = parser.parse().unwrap();
/// // Extract statements from function and analyze
/// ```
pub fn max_nesting_depth(statements: &[Statement]) -> usize {
    calculate_depth(statements, 0)
}

fn calculate_depth(statements: &[Statement], current_depth: usize) -> usize {
    let mut max_depth = current_depth;

    for stmt in statements {
        let depth = match stmt {
            Statement::If {
                then_body,
                elsif_parts,
                else_body,
                ..
            } => {
                let nested = current_depth + 1;
                let mut branch_max = calculate_depth(then_body, nested);

                for (_, body) in elsif_parts {
                    branch_max = branch_max.max(calculate_depth(body, nested));
                }

                if let Some(body) = else_body {
                    branch_max = branch_max.max(calculate_depth(body, nested));
                }

                branch_max
            }

            Statement::Case { cases, else_body, .. } => {
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

            Statement::For { body, .. } => {
                calculate_depth(body, current_depth + 1)
            }

            Statement::While { body, .. } => {
                calculate_depth(body, current_depth + 1)
            }

            Statement::Repeat { body, .. } => {
                calculate_depth(body, current_depth + 1)
            }

            // Non-nesting statements
            Statement::Assignment { .. }
            | Statement::FunctionCall { .. }
            | Statement::FbInvocation { .. }
            | Statement::Exit { .. }
            | Statement::Continue { .. }
            | Statement::Return { .. } => current_depth,
        };

        max_depth = max_depth.max(depth);
    }

    max_depth
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nesting_depth_flat() {
        let stmts = vec![]; // Empty for now - will need parser integration
        assert_eq!(max_nesting_depth(&stmts), 0);
    }
}
