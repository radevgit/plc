//! Semantic analysis for IEC 61131-3 Structured Text.
//!
//! This module provides:
//! - Symbol table management
//! - Type checking and inference
//! - Code smell detection
//! - Unused variable detection

mod symbol_table;
mod type_check;
mod diagnostics;
mod code_smells;

pub use symbol_table::{Symbol, SymbolKind, SymbolTable, Scope};
pub use type_check::{TypeChecker, Type, TypeInfo};
pub use diagnostics::{Diagnostic, DiagnosticKind, Severity};
pub use code_smells::{CodeSmellDetector, CodeSmell, SmellKind};

use crate::ast::*;

/// Analyze a POU and return all diagnostics.
pub fn analyze_pou(pou: &Pou) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    
    // Build symbol table
    let mut symbols = SymbolTable::new();
    symbols.enter_scope(&pou.name);
    
    // Register all variables
    for block in &pou.var_blocks {
        for var in &block.vars {
            if let Err(diag) = symbols.define(Symbol {
                name: var.name.clone(),
                kind: SymbolKind::Variable,
                type_info: Some(type_from_spec(&var.var_type)),
                span: var.span,
                mutable: !block.constant,
                used: false,
                assigned: var.initial.is_some(),
            }) {
                diagnostics.push(diag);
            }
        }
    }
    
    // Type check the body
    let mut type_checker = TypeChecker::new(&symbols);
    for stmt in &pou.body {
        diagnostics.extend(type_checker.check_statement(stmt));
    }
    
    // Detect code smells
    let mut smell_detector = CodeSmellDetector::new();
    diagnostics.extend(smell_detector.analyze_pou(pou));
    
    // Check for unused variables
    diagnostics.extend(symbols.check_unused());
    
    symbols.exit_scope();
    
    diagnostics
}

/// Convert a TypeSpec to a Type for analysis.
fn type_from_spec(spec: &TypeSpec) -> Type {
    match &spec.kind {
        TypeKind::Simple(name) => Type::from_name(name),
        TypeKind::Array { element, ranges } => Type::Array {
            element: Box::new(type_from_spec(element)),
            dimensions: ranges.len(),
        },
        TypeKind::String { length } => Type::String { max_length: *length },
        TypeKind::WString { length } => Type::WString { max_length: *length },
        TypeKind::Subrange { base, .. } => Type::from_name(base),
    }
}
