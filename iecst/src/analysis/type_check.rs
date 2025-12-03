//! Type checking for Structured Text.

use crate::ast::*;
use crate::Span;
use crate::analysis::{Diagnostic, DiagnosticKind, SymbolTable};

/// Built-in and derived types for type checking.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    // Elementary types
    Bool,
    Byte,
    Word,
    DWord,
    LWord,
    SInt,
    Int,
    DInt,
    LInt,
    USInt,
    UInt,
    UDInt,
    ULInt,
    Real,
    LReal,
    Time,
    Date,
    TimeOfDay,
    DateTime,
    String { max_length: Option<u32> },
    WString { max_length: Option<u32> },
    
    // Derived types
    Array { element: Box<Type>, dimensions: usize },
    Struct { name: String },
    Enum { name: String },
    FunctionBlock { name: String },
    
    // Special
    Any,
    Void,
    Unknown,
}

impl Type {
    /// Create a type from a type name.
    pub fn from_name(name: &str) -> Self {
        match name.to_uppercase().as_str() {
            "BOOL" => Type::Bool,
            "BYTE" => Type::Byte,
            "WORD" => Type::Word,
            "DWORD" => Type::DWord,
            "LWORD" => Type::LWord,
            "SINT" => Type::SInt,
            "INT" => Type::Int,
            "DINT" => Type::DInt,
            "LINT" => Type::LInt,
            "USINT" => Type::USInt,
            "UINT" => Type::UInt,
            "UDINT" => Type::UDInt,
            "ULINT" => Type::ULInt,
            "REAL" => Type::Real,
            "LREAL" => Type::LReal,
            "TIME" => Type::Time,
            "DATE" => Type::Date,
            "TOD" | "TIME_OF_DAY" => Type::TimeOfDay,
            "DT" | "DATE_AND_TIME" => Type::DateTime,
            "STRING" => Type::String { max_length: None },
            "WSTRING" => Type::WString { max_length: None },
            _ => Type::Struct { name: name.to_string() },
        }
    }

    /// Check if this is a numeric type.
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            Type::SInt | Type::Int | Type::DInt | Type::LInt |
            Type::USInt | Type::UInt | Type::UDInt | Type::ULInt |
            Type::Real | Type::LReal |
            Type::Byte | Type::Word | Type::DWord | Type::LWord
        )
    }

    /// Check if this is an integer type.
    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            Type::SInt | Type::Int | Type::DInt | Type::LInt |
            Type::USInt | Type::UInt | Type::UDInt | Type::ULInt |
            Type::Byte | Type::Word | Type::DWord | Type::LWord
        )
    }

    /// Check if this is a real (floating-point) type.
    pub fn is_real(&self) -> bool {
        matches!(self, Type::Real | Type::LReal)
    }

    /// Check if this is a boolean type.
    pub fn is_bool(&self) -> bool {
        matches!(self, Type::Bool)
    }

    /// Check if this is a string type.
    pub fn is_string(&self) -> bool {
        matches!(self, Type::String { .. } | Type::WString { .. })
    }

    /// Check if this is a time-related type.
    pub fn is_time(&self) -> bool {
        matches!(self, Type::Time | Type::Date | Type::TimeOfDay | Type::DateTime)
    }

    /// Get a display name for the type.
    pub fn display_name(&self) -> String {
        match self {
            Type::Bool => "BOOL".to_string(),
            Type::Byte => "BYTE".to_string(),
            Type::Word => "WORD".to_string(),
            Type::DWord => "DWORD".to_string(),
            Type::LWord => "LWORD".to_string(),
            Type::SInt => "SINT".to_string(),
            Type::Int => "INT".to_string(),
            Type::DInt => "DINT".to_string(),
            Type::LInt => "LINT".to_string(),
            Type::USInt => "USINT".to_string(),
            Type::UInt => "UINT".to_string(),
            Type::UDInt => "UDINT".to_string(),
            Type::ULInt => "ULINT".to_string(),
            Type::Real => "REAL".to_string(),
            Type::LReal => "LREAL".to_string(),
            Type::Time => "TIME".to_string(),
            Type::Date => "DATE".to_string(),
            Type::TimeOfDay => "TOD".to_string(),
            Type::DateTime => "DT".to_string(),
            Type::String { max_length: Some(n) } => format!("STRING[{}]", n),
            Type::String { max_length: None } => "STRING".to_string(),
            Type::WString { max_length: Some(n) } => format!("WSTRING[{}]", n),
            Type::WString { max_length: None } => "WSTRING".to_string(),
            Type::Array { element, dimensions } => {
                format!("ARRAY[{}] OF {}", dimensions, element.display_name())
            }
            Type::Struct { name } => name.clone(),
            Type::Enum { name } => name.clone(),
            Type::FunctionBlock { name } => name.clone(),
            Type::Any => "ANY".to_string(),
            Type::Void => "VOID".to_string(),
            Type::Unknown => "?".to_string(),
        }
    }

    /// Check if two types are compatible for assignment.
    pub fn is_assignable_from(&self, other: &Type) -> bool {
        if self == other {
            return true;
        }
        
        // Any accepts anything
        if matches!(self, Type::Any) || matches!(other, Type::Any) {
            return true;
        }
        
        // Unknown is compatible with everything (for error recovery)
        if matches!(self, Type::Unknown) || matches!(other, Type::Unknown) {
            return true;
        }
        
        // Numeric promotions
        if self.is_numeric() && other.is_numeric() {
            // Allow integer to real
            if self.is_real() && other.is_integer() {
                return true;
            }
            // Allow smaller integers to larger (simplified)
            if self.is_integer() && other.is_integer() {
                return true; // Simplified - allow all integer conversions
            }
        }
        
        // String compatibility
        if self.is_string() && other.is_string() {
            return true;
        }
        
        false
    }
}

/// Extended type information.
#[derive(Debug, Clone)]
pub struct TypeInfo {
    /// The type
    pub ty: Type,
    /// Whether the value is a constant
    pub is_const: bool,
    /// Whether the value is an l-value (can be assigned to)
    pub is_lvalue: bool,
}

impl TypeInfo {
    pub fn new(ty: Type) -> Self {
        Self {
            ty,
            is_const: false,
            is_lvalue: false,
        }
    }

    pub fn lvalue(ty: Type) -> Self {
        Self {
            ty,
            is_const: false,
            is_lvalue: true,
        }
    }

    pub fn constant(ty: Type) -> Self {
        Self {
            ty,
            is_const: true,
            is_lvalue: false,
        }
    }
}

/// Type checker for expressions and statements.
pub struct TypeChecker<'a> {
    symbols: &'a SymbolTable,
}

impl<'a> TypeChecker<'a> {
    /// Create a new type checker.
    pub fn new(symbols: &'a SymbolTable) -> Self {
        Self { symbols }
    }

    /// Check a statement and return any diagnostics.
    pub fn check_statement(&mut self, stmt: &Stmt) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        
        match &stmt.kind {
            StmtKind::Assignment { target, value } => {
                let target_type = self.infer_expr_type(target, &mut diagnostics);
                let value_type = self.infer_expr_type(value, &mut diagnostics);
                
                if !target_type.ty.is_assignable_from(&value_type.ty) {
                    diagnostics.push(Diagnostic::error(
                        DiagnosticKind::TypeMismatch {
                            expected: target_type.ty.display_name(),
                            found: value_type.ty.display_name(),
                        },
                        value.span,
                    ));
                }
                
                if target_type.is_const {
                    if let ExprKind::Ident(name) = &target.kind {
                        diagnostics.push(Diagnostic::error(
                            DiagnosticKind::AssignmentToConstant { name: name.clone() },
                            target.span,
                        ));
                    }
                }
            }
            
            StmtKind::If { condition, then_body, elsif_branches, else_body } => {
                self.check_condition(condition, &mut diagnostics);
                
                for stmt in then_body {
                    diagnostics.extend(self.check_statement(stmt));
                }
                
                for (cond, body) in elsif_branches {
                    self.check_condition(cond, &mut diagnostics);
                    for stmt in body {
                        diagnostics.extend(self.check_statement(stmt));
                    }
                }
                
                if let Some(else_stmts) = else_body {
                    for stmt in else_stmts {
                        diagnostics.extend(self.check_statement(stmt));
                    }
                }
            }
            
            StmtKind::Case { expr, cases, else_body } => {
                let _expr_type = self.infer_expr_type(expr, &mut diagnostics);
                
                for branch in cases {
                    for stmt in &branch.body {
                        diagnostics.extend(self.check_statement(stmt));
                    }
                }
                
                if let Some(else_stmts) = else_body {
                    for stmt in else_stmts {
                        diagnostics.extend(self.check_statement(stmt));
                    }
                }
            }
            
            StmtKind::For { from, to, by, body, .. } => {
                let from_type = self.infer_expr_type(from, &mut diagnostics);
                let to_type = self.infer_expr_type(to, &mut diagnostics);
                
                if !from_type.ty.is_integer() {
                    diagnostics.push(Diagnostic::error(
                        DiagnosticKind::TypeMismatch {
                            expected: "integer".to_string(),
                            found: from_type.ty.display_name(),
                        },
                        from.span,
                    ));
                }
                
                if !to_type.ty.is_integer() {
                    diagnostics.push(Diagnostic::error(
                        DiagnosticKind::TypeMismatch {
                            expected: "integer".to_string(),
                            found: to_type.ty.display_name(),
                        },
                        to.span,
                    ));
                }
                
                if let Some(by_expr) = by {
                    let by_type = self.infer_expr_type(by_expr, &mut diagnostics);
                    if !by_type.ty.is_integer() {
                        diagnostics.push(Diagnostic::error(
                            DiagnosticKind::TypeMismatch {
                                expected: "integer".to_string(),
                                found: by_type.ty.display_name(),
                            },
                            by_expr.span,
                        ));
                    }
                }
                
                for stmt in body {
                    diagnostics.extend(self.check_statement(stmt));
                }
            }
            
            StmtKind::While { condition, body } => {
                self.check_condition(condition, &mut diagnostics);
                for stmt in body {
                    diagnostics.extend(self.check_statement(stmt));
                }
            }
            
            StmtKind::Repeat { body, until } => {
                for stmt in body {
                    diagnostics.extend(self.check_statement(stmt));
                }
                self.check_condition(until, &mut diagnostics);
            }
            
            StmtKind::Call { args, .. } => {
                for arg in args {
                    if let Some(value) = &arg.value {
                        self.infer_expr_type(value, &mut diagnostics);
                    }
                }
            }
            
            StmtKind::Return { value } => {
                if let Some(expr) = value {
                    self.infer_expr_type(expr, &mut diagnostics);
                }
            }
            
            StmtKind::Exit | StmtKind::Continue | StmtKind::Empty => {}
        }
        
        diagnostics
    }

    /// Check that a condition is boolean.
    fn check_condition(&self, condition: &Expr, diagnostics: &mut Vec<Diagnostic>) {
        let cond_type = self.infer_expr_type(condition, diagnostics);
        if !cond_type.ty.is_bool() && !matches!(cond_type.ty, Type::Unknown) {
            diagnostics.push(Diagnostic::warning(
                DiagnosticKind::TypeMismatch {
                    expected: "BOOL".to_string(),
                    found: cond_type.ty.display_name(),
                },
                condition.span,
            ));
        }
    }

    /// Infer the type of an expression.
    pub fn infer_expr_type(&self, expr: &Expr, diagnostics: &mut Vec<Diagnostic>) -> TypeInfo {
        match &expr.kind {
            ExprKind::IntLiteral(_) => TypeInfo::constant(Type::DInt),
            ExprKind::RealLiteral(_) => TypeInfo::constant(Type::LReal),
            ExprKind::StringLiteral(_) => TypeInfo::constant(Type::String { max_length: None }),
            ExprKind::WStringLiteral(_) => TypeInfo::constant(Type::WString { max_length: None }),
            ExprKind::BoolLiteral(_) => TypeInfo::constant(Type::Bool),
            ExprKind::TimeLiteral(_) => TypeInfo::constant(Type::Time),
            ExprKind::DateLiteral(_) => TypeInfo::constant(Type::Date),
            ExprKind::TodLiteral(_) => TypeInfo::constant(Type::TimeOfDay),
            ExprKind::DateTimeLiteral(_) => TypeInfo::constant(Type::DateTime),
            
            ExprKind::Ident(name) => {
                if let Some(symbol) = self.symbols.lookup(name) {
                    let ty = symbol.type_info.clone().unwrap_or(Type::Unknown);
                    if symbol.mutable {
                        TypeInfo::lvalue(ty)
                    } else {
                        TypeInfo::constant(ty)
                    }
                } else {
                    diagnostics.push(Diagnostic::error(
                        DiagnosticKind::UndefinedIdentifier { name: name.clone() },
                        expr.span,
                    ));
                    TypeInfo::new(Type::Unknown)
                }
            }
            
            ExprKind::DirectAddress(_) => {
                // Direct addresses could be any type, default to BOOL for bit addresses
                TypeInfo::lvalue(Type::Bool)
            }
            
            ExprKind::BinaryOp { left, op, right } => {
                let left_type = self.infer_expr_type(left, diagnostics);
                let right_type = self.infer_expr_type(right, diagnostics);
                
                let result_type = self.binary_op_result_type(*op, &left_type.ty, &right_type.ty, expr.span, diagnostics);
                TypeInfo::new(result_type)
            }
            
            ExprKind::UnaryOp { op, expr: inner } => {
                let inner_type = self.infer_expr_type(inner, diagnostics);
                
                let result_type = match op {
                    UnaryOp::Neg => {
                        if inner_type.ty.is_numeric() {
                            inner_type.ty.clone()
                        } else {
                            diagnostics.push(Diagnostic::error(
                                DiagnosticKind::InvalidOperator {
                                    op: "-".to_string(),
                                    operand_type: inner_type.ty.display_name(),
                                },
                                expr.span,
                            ));
                            Type::Unknown
                        }
                    }
                    UnaryOp::Not => {
                        if inner_type.ty.is_bool() {
                            Type::Bool
                        } else if inner_type.ty.is_integer() {
                            inner_type.ty.clone() // Bitwise NOT
                        } else {
                            diagnostics.push(Diagnostic::error(
                                DiagnosticKind::InvalidOperator {
                                    op: "NOT".to_string(),
                                    operand_type: inner_type.ty.display_name(),
                                },
                                expr.span,
                            ));
                            Type::Unknown
                        }
                    }
                };
                
                TypeInfo::new(result_type)
            }
            
            ExprKind::FunctionCall { name, args } => {
                // Check argument types
                for arg in args {
                    if let Some(value) = &arg.value {
                        self.infer_expr_type(value, diagnostics);
                    }
                }
                
                // Return type based on known functions
                let return_type = self.builtin_function_type(name);
                TypeInfo::new(return_type)
            }
            
            ExprKind::ArrayIndex { array, indices } => {
                let array_type = self.infer_expr_type(array, diagnostics);
                
                // Check indices are integers
                for index in indices {
                    let idx_type = self.infer_expr_type(index, diagnostics);
                    if !idx_type.ty.is_integer() && !matches!(idx_type.ty, Type::Unknown) {
                        diagnostics.push(Diagnostic::error(
                            DiagnosticKind::NonIntegerArrayIndex,
                            index.span,
                        ));
                    }
                }
                
                // Get element type
                if let Type::Array { element, dimensions } = &array_type.ty {
                    if indices.len() != *dimensions {
                        diagnostics.push(Diagnostic::error(
                            DiagnosticKind::ArrayDimensionMismatch {
                                expected: *dimensions,
                                found: indices.len(),
                            },
                            expr.span,
                        ));
                    }
                    TypeInfo::lvalue((**element).clone())
                } else {
                    TypeInfo::new(Type::Unknown)
                }
            }
            
            ExprKind::MemberAccess { expr: inner, .. } => {
                self.infer_expr_type(inner, diagnostics);
                // Would need struct type info to determine member type
                TypeInfo::lvalue(Type::Unknown)
            }
            
            ExprKind::Paren(inner) => {
                self.infer_expr_type(inner, diagnostics)
            }
        }
    }

    /// Determine the result type of a binary operation.
    fn binary_op_result_type(
        &self,
        op: BinaryOp,
        left: &Type,
        right: &Type,
        span: Span,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Type {
        match op {
            // Arithmetic operators
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod | BinaryOp::Power => {
                if left.is_numeric() && right.is_numeric() {
                    // Promote to larger type
                    if left.is_real() || right.is_real() {
                        Type::LReal
                    } else {
                        Type::DInt
                    }
                } else if left.is_string() && right.is_string() && op == BinaryOp::Add {
                    Type::String { max_length: None }
                } else if left.is_time() && right.is_time() && matches!(op, BinaryOp::Add | BinaryOp::Sub) {
                    Type::Time
                } else if !matches!(left, Type::Unknown) && !matches!(right, Type::Unknown) {
                    diagnostics.push(Diagnostic::error(
                        DiagnosticKind::IncompatibleTypes {
                            left: left.display_name(),
                            right: right.display_name(),
                            op: format!("{:?}", op),
                        },
                        span,
                    ));
                    Type::Unknown
                } else {
                    Type::Unknown
                }
            }
            
            // Comparison operators
            BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                Type::Bool
            }
            
            // Logical operators
            BinaryOp::And | BinaryOp::Or | BinaryOp::Xor => {
                if left.is_bool() && right.is_bool() {
                    Type::Bool
                } else if left.is_integer() && right.is_integer() {
                    // Bitwise operation
                    Type::DInt
                } else if !matches!(left, Type::Unknown) && !matches!(right, Type::Unknown) {
                    diagnostics.push(Diagnostic::error(
                        DiagnosticKind::IncompatibleTypes {
                            left: left.display_name(),
                            right: right.display_name(),
                            op: format!("{:?}", op),
                        },
                        span,
                    ));
                    Type::Unknown
                } else {
                    Type::Unknown
                }
            }
        }
    }

    /// Get the return type of a builtin function.
    fn builtin_function_type(&self, name: &str) -> Type {
        match name.to_uppercase().as_str() {
            // Math functions
            "ABS" | "SQRT" | "LN" | "LOG" | "EXP" | "SIN" | "COS" | "TAN" | 
            "ASIN" | "ACOS" | "ATAN" | "ATAN2" => Type::LReal,
            
            // Type conversion
            "BOOL_TO_INT" | "REAL_TO_INT" | "TRUNC" | "ROUND" => Type::DInt,
            "INT_TO_REAL" | "DINT_TO_REAL" => Type::LReal,
            "INT_TO_STRING" | "REAL_TO_STRING" => Type::String { max_length: None },
            "STRING_TO_INT" => Type::DInt,
            "STRING_TO_REAL" => Type::LReal,
            
            // Bit operations
            "SHL" | "SHR" | "ROL" | "ROR" => Type::DWord,
            
            // Selection
            "SEL" | "MAX" | "MIN" | "LIMIT" | "MUX" => Type::Any,
            
            // String
            "LEN" | "FIND" => Type::DInt,
            "LEFT" | "RIGHT" | "MID" | "CONCAT" | "INSERT" | "DELETE" | "REPLACE" => {
                Type::String { max_length: None }
            }
            
            _ => Type::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_from_name() {
        assert_eq!(Type::from_name("INT"), Type::Int);
        assert_eq!(Type::from_name("int"), Type::Int);
        assert_eq!(Type::from_name("BOOL"), Type::Bool);
        assert_eq!(Type::from_name("REAL"), Type::Real);
    }

    #[test]
    fn test_numeric_checks() {
        assert!(Type::Int.is_numeric());
        assert!(Type::Real.is_numeric());
        assert!(!Type::Bool.is_numeric());
        assert!(!Type::String { max_length: None }.is_numeric());
    }

    #[test]
    fn test_type_compatibility() {
        let int_type = Type::Int;
        let real_type = Type::Real;
        let bool_type = Type::Bool;
        
        // Real can accept integer
        assert!(real_type.is_assignable_from(&int_type));
        // Bool cannot accept integer
        assert!(!bool_type.is_assignable_from(&int_type));
        // Same types are compatible
        assert!(int_type.is_assignable_from(&int_type));
    }
}
