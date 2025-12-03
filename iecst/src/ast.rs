//! Abstract Syntax Tree for IEC 61131-3 Structured Text.

use crate::Span;

// ============================================================================
// Expressions
// ============================================================================

/// An expression node.
#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

impl Expr {
    pub fn new(kind: ExprKind, span: Span) -> Self {
        Self { kind, span }
    }
}

/// Expression kinds.
#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    /// Integer literal
    IntLiteral(i64),
    /// Real (floating-point) literal
    RealLiteral(f64),
    /// String literal
    StringLiteral(String),
    /// Wide string literal
    WStringLiteral(String),
    /// Boolean literal
    BoolLiteral(bool),
    /// Time duration literal
    TimeLiteral(String),
    /// Date literal
    DateLiteral(String),
    /// Time of day literal
    TodLiteral(String),
    /// Date and time literal
    DateTimeLiteral(String),

    /// Identifier reference
    Ident(String),

    /// Direct address (%I0.0, %MW100, %QX0.0)
    DirectAddress(DirectAddress),

    /// Binary operation: left op right
    BinaryOp {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },

    /// Unary operation: op expr
    UnaryOp { op: UnaryOp, expr: Box<Expr> },

    /// Function call: func(args) with optional named parameters
    FunctionCall { name: String, args: Vec<FunctionArg> },

    /// Array indexing: array[index]
    ArrayIndex { array: Box<Expr>, indices: Vec<Expr> },

    /// Member access: expr.member
    MemberAccess { expr: Box<Expr>, member: String },

    /// Parenthesized expression
    Paren(Box<Expr>),
}

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Power,

    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    // Logical
    And,
    Or,
    Xor,
}

impl BinaryOp {
    /// Get the precedence of this operator (higher = binds tighter).
    pub fn precedence(&self) -> u8 {
        match self {
            BinaryOp::Or => 1,
            BinaryOp::Xor => 2,
            BinaryOp::And => 3,
            BinaryOp::Eq | BinaryOp::Ne => 4,
            BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => 5,
            BinaryOp::Add | BinaryOp::Sub => 6,
            BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => 7,
            BinaryOp::Power => 8,
        }
    }

    /// Is this operator right-associative?
    pub fn is_right_assoc(&self) -> bool {
        matches!(self, BinaryOp::Power)
    }
}

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    /// Negation: -x
    Neg,
    /// Logical NOT: NOT x
    Not,
}

/// Direct address (I/O or memory location).
/// Examples: %IX0.0, %QW5, %MW100, %MD10
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectAddress {
    /// Location prefix: I (input), Q (output), M (memory)
    pub location: LocationPrefix,
    /// Size prefix: X (bit), B (byte), W (word), D (double word), L (long word)
    pub size: SizePrefix,
    /// The address (e.g., "0.0", "100", "5")
    pub address: String,
}

/// Location prefix for direct addresses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocationPrefix {
    /// Input: %I
    Input,
    /// Output: %Q
    Output,
    /// Memory: %M
    Memory,
}

/// Size prefix for direct addresses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizePrefix {
    /// Bit: X (default, often omitted)
    Bit,
    /// Byte: B
    Byte,
    /// Word (16-bit): W
    Word,
    /// Double word (32-bit): D
    DoubleWord,
    /// Long word (64-bit): L
    LongWord,
}

/// Function/FB call argument (may be named).
/// 
/// Supports Rockwell-specific empty arguments like `GSV(a, , b)` where
/// the second argument is intentionally omitted.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionArg {
    /// Optional parameter name (for named arguments: param := value)
    pub name: Option<String>,
    /// The argument value (None for empty arguments in Rockwell ST)
    pub value: Option<Expr>,
    pub span: Span,
}

// ============================================================================
// Statements
// ============================================================================

/// A statement node.
#[derive(Debug, Clone, PartialEq)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

impl Stmt {
    pub fn new(kind: StmtKind, span: Span) -> Self {
        Self { kind, span }
    }
}

/// Statement kinds.
#[derive(Debug, Clone, PartialEq)]
pub enum StmtKind {
    /// Assignment: target := value
    Assignment { target: Expr, value: Expr },

    /// IF statement
    If {
        condition: Expr,
        then_body: Vec<Stmt>,
        elsif_branches: Vec<(Expr, Vec<Stmt>)>,
        else_body: Option<Vec<Stmt>>,
    },

    /// CASE statement
    Case {
        expr: Expr,
        cases: Vec<CaseBranch>,
        else_body: Option<Vec<Stmt>>,
    },

    /// FOR loop
    For {
        var: String,
        from: Expr,
        to: Expr,
        by: Option<Expr>,
        body: Vec<Stmt>,
    },

    /// WHILE loop
    While { condition: Expr, body: Vec<Stmt> },

    /// REPEAT loop
    Repeat { body: Vec<Stmt>, until: Expr },

    /// EXIT statement (break from loop)
    Exit,

    /// CONTINUE statement (skip to next iteration)
    Continue,

    /// RETURN statement
    Return { value: Option<Expr> },

    /// Function/FB call as statement
    Call { name: String, args: Vec<CallArg> },

    /// Empty statement (just a semicolon)
    Empty,
}

/// A branch in a CASE statement.
#[derive(Debug, Clone, PartialEq)]
pub struct CaseBranch {
    /// The values to match (can be ranges)
    pub values: Vec<CaseValue>,
    /// Body to execute
    pub body: Vec<Stmt>,
    pub span: Span,
}

/// A value or range in a CASE branch.
#[derive(Debug, Clone, PartialEq)]
pub enum CaseValue {
    /// Single value
    Single(Expr),
    /// Range: from..to
    Range { from: Expr, to: Expr },
}

/// Argument in a function/FB call.
/// 
/// Supports Rockwell-specific empty arguments like `GSV(a, , b)` where
/// the second argument is intentionally omitted.
#[derive(Debug, Clone, PartialEq)]
pub struct CallArg {
    /// Optional parameter name (for named arguments)
    pub name: Option<String>,
    /// The value (None for empty arguments in Rockwell ST)
    pub value: Option<Expr>,
    pub span: Span,
}

// ============================================================================
// Declarations
// ============================================================================

/// Variable declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct VarDecl {
    /// Variable name
    pub name: String,
    /// Variable type
    pub var_type: TypeSpec,
    /// Initial value
    pub initial: Option<Expr>,
    /// Location (AT %MW0.0)
    pub location: Option<String>,
    pub span: Span,
}

/// Variable block (VAR, VAR_INPUT, etc.)
#[derive(Debug, Clone, PartialEq)]
pub struct VarBlock {
    pub kind: VarBlockKind,
    pub constant: bool,
    pub retain: RetainKind,
    pub vars: Vec<VarDecl>,
    pub span: Span,
}

/// Kind of variable block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarBlockKind {
    Var,
    VarInput,
    VarOutput,
    VarInOut,
    VarTemp,
    VarGlobal,
    VarExternal,
}

/// Retain attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RetainKind {
    #[default]
    None,
    Retain,
    NonRetain,
}

// ============================================================================
// Types
// ============================================================================

/// Type specification.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeSpec {
    pub kind: TypeKind,
    pub span: Span,
}

impl TypeSpec {
    pub fn new(kind: TypeKind, span: Span) -> Self {
        Self { kind, span }
    }
}

/// Type kinds.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
    /// Simple type name (BOOL, INT, REAL, UserType, etc.)
    Simple(String),

    /// Array type: ARRAY[lo..hi] OF element
    Array {
        ranges: Vec<ArrayRange>,
        element: Box<TypeSpec>,
    },

    /// String type with optional length: STRING[80]
    String { length: Option<u32> },

    /// Wide string type with optional length
    WString { length: Option<u32> },

    /// Subrange type: INT(0..100)
    Subrange {
        base: String,
        low: Box<Expr>,
        high: Box<Expr>,
    },
}

/// Array dimension range.
#[derive(Debug, Clone, PartialEq)]
pub struct ArrayRange {
    pub low: Expr,
    pub high: Expr,
    pub span: Span,
}

// ============================================================================
// Program Organization Units (POUs)
// ============================================================================

/// A Program Organization Unit.
#[derive(Debug, Clone, PartialEq)]
pub struct Pou {
    pub kind: PouKind,
    pub name: String,
    pub return_type: Option<TypeSpec>,
    pub var_blocks: Vec<VarBlock>,
    pub body: Vec<Stmt>,
    pub span: Span,
}

/// Kind of POU.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PouKind {
    Program,
    Function,
    FunctionBlock,
}

// ============================================================================
// Type Declarations
// ============================================================================

/// A type declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeDecl {
    pub name: String,
    pub definition: TypeDef,
    pub span: Span,
}

/// Type definition body.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeDef {
    /// Alias: MyInt : INT;
    Alias(TypeSpec),

    /// Structure
    Struct { fields: Vec<VarDecl> },

    /// Enumeration
    Enum { values: Vec<EnumValue> },

    /// Subrange
    Subrange {
        base: String,
        low: Expr,
        high: Expr,
    },

    /// Array type
    Array {
        ranges: Vec<ArrayRange>,
        element: Box<TypeSpec>,
    },
}

/// Enumeration value.
#[derive(Debug, Clone, PartialEq)]
pub struct EnumValue {
    pub name: String,
    pub value: Option<Expr>,
    pub span: Span,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_op_precedence() {
        assert!(BinaryOp::Mul.precedence() > BinaryOp::Add.precedence());
        assert!(BinaryOp::And.precedence() > BinaryOp::Or.precedence());
        assert!(BinaryOp::Power.precedence() > BinaryOp::Mul.precedence());
    }

    #[test]
    fn test_expr_construction() {
        let span = Span::new(0, 5);
        let expr = Expr::new(ExprKind::IntLiteral(42), span);
        assert_eq!(expr.span.start, 0);
        assert_eq!(expr.span.end, 5);
        if let ExprKind::IntLiteral(v) = expr.kind {
            assert_eq!(v, 42);
        } else {
            panic!("Expected IntLiteral");
        }
    }
}
