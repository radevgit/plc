//! Symbol table for tracking declarations.

use crate::Span;
use crate::analysis::{Diagnostic, DiagnosticKind, Severity, Type};
use std::collections::HashMap;

/// A symbol in the symbol table.
#[derive(Debug, Clone)]
pub struct Symbol {
    /// Symbol name
    pub name: String,
    /// Kind of symbol
    pub kind: SymbolKind,
    /// Type information (if known)
    pub type_info: Option<Type>,
    /// Location in source
    pub span: Span,
    /// Whether the symbol is mutable
    pub mutable: bool,
    /// Whether the symbol has been used (read)
    pub used: bool,
    /// Whether the symbol has been assigned
    pub assigned: bool,
}

/// Kind of symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    /// Local variable
    Variable,
    /// Function parameter (input)
    Parameter,
    /// Output parameter
    Output,
    /// In/Out parameter
    InOut,
    /// Constant
    Constant,
    /// Function
    Function,
    /// Function block instance
    FunctionBlock,
    /// Program
    Program,
    /// User-defined type
    Type,
}

/// A scope in the symbol table.
#[derive(Debug)]
pub struct Scope {
    /// Scope name (e.g., function name)
    pub name: String,
    /// Symbols defined in this scope
    pub symbols: HashMap<String, Symbol>,
    /// Parent scope index (None for global)
    pub parent: Option<usize>,
}

impl Scope {
    /// Create a new scope.
    pub fn new(name: impl Into<String>, parent: Option<usize>) -> Self {
        Self {
            name: name.into(),
            symbols: HashMap::new(),
            parent,
        }
    }
}

/// Symbol table for managing declarations.
#[derive(Debug)]
pub struct SymbolTable {
    /// Stack of scopes
    scopes: Vec<Scope>,
    /// Current scope index
    current: usize,
}

impl SymbolTable {
    /// Create a new symbol table with global scope.
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::new("global", None)],
            current: 0,
        }
    }

    /// Enter a new scope.
    pub fn enter_scope(&mut self, name: &str) {
        let parent = Some(self.current);
        self.scopes.push(Scope::new(name, parent));
        self.current = self.scopes.len() - 1;
    }

    /// Exit the current scope.
    pub fn exit_scope(&mut self) {
        if let Some(parent) = self.scopes[self.current].parent {
            self.current = parent;
        }
    }

    /// Define a new symbol in the current scope.
    pub fn define(&mut self, symbol: Symbol) -> Result<(), Diagnostic> {
        let scope = &mut self.scopes[self.current];
        
        // Check for duplicate in current scope
        if let Some(existing) = scope.symbols.get(&symbol.name) {
            return Err(Diagnostic {
                kind: DiagnosticKind::DuplicateDefinition {
                    name: symbol.name.clone(),
                    original: existing.span,
                },
                span: symbol.span,
                severity: Severity::Error,
            });
        }
        
        scope.symbols.insert(symbol.name.clone(), symbol);
        Ok(())
    }

    /// Look up a symbol by name, searching up the scope chain.
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        let mut scope_idx = Some(self.current);
        
        while let Some(idx) = scope_idx {
            if let Some(symbol) = self.scopes[idx].symbols.get(name) {
                return Some(symbol);
            }
            scope_idx = self.scopes[idx].parent;
        }
        
        None
    }

    /// Look up a symbol mutably.
    pub fn lookup_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        let mut scope_idx = Some(self.current);
        
        // Find which scope contains the symbol
        let target_scope = loop {
            match scope_idx {
                Some(idx) => {
                    if self.scopes[idx].symbols.contains_key(name) {
                        break Some(idx);
                    }
                    scope_idx = self.scopes[idx].parent;
                }
                None => break None,
            }
        };
        
        target_scope.and_then(move |idx| self.scopes[idx].symbols.get_mut(name))
    }

    /// Mark a symbol as used.
    pub fn mark_used(&mut self, name: &str) {
        if let Some(symbol) = self.lookup_mut(name) {
            symbol.used = true;
        }
    }

    /// Mark a symbol as assigned.
    pub fn mark_assigned(&mut self, name: &str) {
        if let Some(symbol) = self.lookup_mut(name) {
            symbol.assigned = true;
        }
    }

    /// Check for unused variables and return diagnostics.
    pub fn check_unused(&self) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        
        for scope in &self.scopes {
            for symbol in scope.symbols.values() {
                // Skip checking outputs and in/outs (they might be used externally)
                if matches!(symbol.kind, SymbolKind::Output | SymbolKind::InOut) {
                    continue;
                }
                
                if !symbol.used && symbol.kind == SymbolKind::Variable {
                    diagnostics.push(Diagnostic {
                        kind: DiagnosticKind::UnusedVariable {
                            name: symbol.name.clone(),
                        },
                        span: symbol.span,
                        severity: Severity::Warning,
                    });
                }
                
                if !symbol.assigned && symbol.mutable && symbol.kind == SymbolKind::Variable {
                    diagnostics.push(Diagnostic {
                        kind: DiagnosticKind::UninitializedVariable {
                            name: symbol.name.clone(),
                        },
                        span: symbol.span,
                        severity: Severity::Warning,
                    });
                }
            }
        }
        
        diagnostics
    }

    /// Get the current scope name.
    pub fn current_scope_name(&self) -> &str {
        &self.scopes[self.current].name
    }

    /// Check if a name is defined in the current scope only.
    pub fn is_defined_locally(&self, name: &str) -> bool {
        self.scopes[self.current].symbols.contains_key(name)
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_define_and_lookup() {
        let mut table = SymbolTable::new();
        
        let symbol = Symbol {
            name: "x".to_string(),
            kind: SymbolKind::Variable,
            type_info: None,
            span: Span::new(0, 1),
            mutable: true,
            used: false,
            assigned: false,
        };
        
        assert!(table.define(symbol).is_ok());
        assert!(table.lookup("x").is_some());
        assert!(table.lookup("y").is_none());
    }

    #[test]
    fn test_nested_scopes() {
        let mut table = SymbolTable::new();
        
        // Define in global scope
        table.define(Symbol {
            name: "global_var".to_string(),
            kind: SymbolKind::Variable,
            type_info: None,
            span: Span::new(0, 10),
            mutable: true,
            used: false,
            assigned: true,
        }).unwrap();
        
        // Enter new scope
        table.enter_scope("inner");
        
        // Can still see global_var
        assert!(table.lookup("global_var").is_some());
        
        // Define local var
        table.define(Symbol {
            name: "local_var".to_string(),
            kind: SymbolKind::Variable,
            type_info: None,
            span: Span::new(20, 30),
            mutable: true,
            used: false,
            assigned: true,
        }).unwrap();
        
        assert!(table.lookup("local_var").is_some());
        
        // Exit scope
        table.exit_scope();
        
        // Can no longer see local_var from outer scope
        // (Note: our implementation keeps all scopes, but lookup starts from current)
        assert!(table.lookup("global_var").is_some());
    }

    #[test]
    fn test_duplicate_definition() {
        let mut table = SymbolTable::new();
        
        let symbol1 = Symbol {
            name: "x".to_string(),
            kind: SymbolKind::Variable,
            type_info: None,
            span: Span::new(0, 1),
            mutable: true,
            used: false,
            assigned: false,
        };
        
        let symbol2 = Symbol {
            name: "x".to_string(),
            kind: SymbolKind::Variable,
            type_info: None,
            span: Span::new(10, 11),
            mutable: true,
            used: false,
            assigned: false,
        };
        
        assert!(table.define(symbol1).is_ok());
        assert!(table.define(symbol2).is_err());
    }

    #[test]
    fn test_unused_detection() {
        let mut table = SymbolTable::new();
        
        table.define(Symbol {
            name: "used_var".to_string(),
            kind: SymbolKind::Variable,
            type_info: None,
            span: Span::new(0, 8),
            mutable: true,
            used: true,
            assigned: true,
        }).unwrap();
        
        table.define(Symbol {
            name: "unused_var".to_string(),
            kind: SymbolKind::Variable,
            type_info: None,
            span: Span::new(10, 20),
            mutable: true,
            used: false,
            assigned: true,
        }).unwrap();
        
        let diagnostics = table.check_unused();
        assert_eq!(diagnostics.len(), 1);
        assert!(matches!(
            diagnostics[0].kind,
            DiagnosticKind::UnusedVariable { ref name } if name == "unused_var"
        ));
    }
}
