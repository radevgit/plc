//! Abstract Syntax Tree for SCL programs.

use crate::span::Span;

/// A complete SCL compilation unit (file).
#[derive(Debug, Clone, PartialEq)]
pub struct CompilationUnit {
    /// Blocks defined in this unit
    pub blocks: Vec<Block>,
    /// Span of the entire unit
    pub span: Span,
}

/// A block definition (FUNCTION, FUNCTION_BLOCK, DATA_BLOCK, etc.).
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    /// Block type
    pub kind: BlockKind,
    /// Block name
    pub name: String,
    /// Block number (optional, e.g., FB10)
    pub number: Option<i32>,
    /// Pragmas attached to block
    pub pragmas: Vec<Pragma>,
    /// Variable declarations
    pub variables: Vec<VarSection>,
    /// Block body (statements)
    pub body: Vec<Statement>,
    /// Return type (for FUNCTION)
    pub return_type: Option<DataType>,
    /// Span
    pub span: Span,
}

/// Block type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockKind {
    Function,
    FunctionBlock,
    DataBlock,
    OrganizationBlock,
    Type,
}

/// A pragma annotation.
#[derive(Debug, Clone, PartialEq)]
pub struct Pragma {
    /// Pragma content (key-value pairs)
    pub content: String,
    /// Span
    pub span: Span,
}

/// Variable section.
#[derive(Debug, Clone, PartialEq)]
pub struct VarSection {
    /// Section kind
    pub kind: VarSectionKind,
    /// Variables declared
    pub variables: Vec<VarDeclaration>,
    /// Span
    pub span: Span,
}

/// Variable section kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarSectionKind {
    Input,
    Output,
    InOut,
    Temp,
    Stat,
    Constant,
    Global,
    External,
}

/// Variable declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct VarDeclaration {
    /// Variable name
    pub name: String,
    /// Data type
    pub data_type: DataType,
    /// Initial value
    pub initial_value: Option<Expression>,
    /// Is retained (RETAIN)
    pub retain: bool,
    /// Absolute address (AT %MW10)
    pub address: Option<Address>,
    /// Pragmas
    pub pragmas: Vec<Pragma>,
    /// Span
    pub span: Span,
}

/// Data type.
#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    // Elementary types
    Bool,
    Byte,
    Word,
    Dword,
    Lword,
    Sint,
    Int,
    Dint,
    Lint,
    Usint,
    Uint,
    Udint,
    Ulint,
    Real,
    Lreal,
    Char,
    Wchar,
    
    // String types
    String(Option<u32>),  // STRING[n]
    WString(Option<u32>), // WSTRING[n]
    
    // Time types
    Time,
    LTime,
    Date,
    TimeOfDay,
    DateAndTime,
    
    // Array
    Array {
        lower: Box<Expression>,
        upper: Box<Expression>,
        element_type: Box<DataType>,
    },
    
    // Struct
    Struct {
        members: Vec<VarDeclaration>,
    },
    
    // User-defined type
    UserDefined(String),
    
    // Pointer/Reference
    Pointer(Box<DataType>),
    Ref(Box<DataType>),
    
    // Special
    Any,
    Void,
}

/// Absolute or symbolic address.
#[derive(Debug, Clone, PartialEq)]
pub enum Address {
    /// Absolute address (%I0.0, %Q0.1, %MW10)
    Absolute {
        area: MemoryArea,
        size: Option<AddressSize>,
        byte_offset: u32,
        bit_offset: Option<u8>,
    },
    /// Data block address (DB10.DBW0)
    DataBlock {
        db_name: String,
        offset: u32,
        size: AddressSize,
    },
}

/// Memory area for absolute addressing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryArea {
    Input,    // %I
    Output,   // %Q
    Memory,   // %M
    Peripheral, // %P (peripheral I/O)
}

/// Size specifier for addresses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressSize {
    Bit,      // X
    Byte,     // B
    Word,     // W
    Dword,    // D
    Lword,    // L (64-bit)
}

/// Statement.
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// Assignment: x := expr;
    Assignment {
        target: Expression,
        operator: AssignOp,
        value: Expression,
        span: Span,
    },
    
    /// IF statement
    If {
        condition: Expression,
        then_branch: Vec<Statement>,
        elsif_branches: Vec<(Expression, Vec<Statement>)>,
        else_branch: Option<Vec<Statement>>,
        span: Span,
    },
    
    /// CASE statement
    Case {
        selector: Expression,
        branches: Vec<CaseBranch>,
        else_branch: Option<Vec<Statement>>,
        span: Span,
    },
    
    /// FOR loop
    For {
        variable: String,
        start: Expression,
        end: Expression,
        step: Option<Expression>,
        body: Vec<Statement>,
        span: Span,
    },
    
    /// WHILE loop
    While {
        condition: Expression,
        body: Vec<Statement>,
        span: Span,
    },
    
    /// REPEAT..UNTIL loop
    Repeat {
        body: Vec<Statement>,
        condition: Expression,
        span: Span,
    },
    
    /// CONTINUE
    Continue { span: Span },
    
    /// EXIT
    Exit { span: Span },
    
    /// RETURN
    Return {
        value: Option<Expression>,
        span: Span,
    },
    
    /// GOTO
    Goto {
        label: String,
        span: Span,
    },
    
    /// Label
    Label {
        name: String,
        span: Span,
    },
    
    /// Function/FB call
    Call {
        target: Expression,
        arguments: Vec<Argument>,
        span: Span,
    },
    
    /// REGION block
    Region {
        name: String,
        body: Vec<Statement>,
        span: Span,
    },
    
    /// Empty statement (just semicolon)
    Empty { span: Span },
}

/// Assignment operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignOp {
    Assign,    // :=
    AddAssign, // +=
    SubAssign, // -=
    MulAssign, // *=
    DivAssign, // /=
}

/// CASE branch.
#[derive(Debug, Clone, PartialEq)]
pub struct CaseBranch {
    /// Case values (constants or ranges)
    pub values: Vec<CaseValue>,
    /// Branch body
    pub body: Vec<Statement>,
    /// Span
    pub span: Span,
}

/// Case value (constant or range).
#[derive(Debug, Clone, PartialEq)]
pub enum CaseValue {
    /// Single value
    Single(Expression),
    /// Range: 1..10
    Range(Expression, Expression),
}

/// Function/FB argument.
#[derive(Debug, Clone, PartialEq)]
pub struct Argument {
    /// Parameter name (for named arguments)
    pub name: Option<String>,
    /// Argument value
    pub value: Expression,
    /// Is output (ENO=>...)
    pub is_output: bool,
    /// Span
    pub span: Span,
}

/// Expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// Literal value
    Literal(Literal),
    
    /// Variable or identifier
    Identifier {
        name: String,
        span: Span,
    },
    
    /// Member access: obj.member
    Member {
        object: Box<Expression>,
        member: String,
        span: Span,
    },
    
    /// Array indexing: arr[i]
    Index {
        array: Box<Expression>,
        index: Box<Expression>,
        span: Span,
    },
    
    /// Binary operation
    Binary {
        left: Box<Expression>,
        op: BinaryOp,
        right: Box<Expression>,
        span: Span,
    },
    
    /// Unary operation
    Unary {
        op: UnaryOp,
        operand: Box<Expression>,
        span: Span,
    },
    
    /// Function call
    Call {
        function: Box<Expression>,
        arguments: Vec<Argument>,
        span: Span,
    },
    
    /// Parenthesized expression
    Paren {
        inner: Box<Expression>,
        span: Span,
    },
    
    /// Type cast/conversion
    TypeCast {
        target_type: DataType,
        value: Box<Expression>,
        span: Span,
    },
    
    /// Absolute address: %MW10
    Address(Address),
}

impl Expression {
    /// Get the span of the expression.
    pub fn span(&self) -> Span {
        match self {
            Expression::Literal(lit) => lit.span,
            Expression::Identifier { span, .. } => *span,
            Expression::Member { span, .. } => *span,
            Expression::Index { span, .. } => *span,
            Expression::Binary { span, .. } => *span,
            Expression::Unary { span, .. } => *span,
            Expression::Call { span, .. } => *span,
            Expression::Paren { span, .. } => *span,
            Expression::TypeCast { span, .. } => *span,
            Expression::Address(_) => Span::default(),
        }
    }
}

/// Literal value.
#[derive(Debug, Clone, PartialEq)]
pub struct Literal {
    pub kind: LiteralKind,
    pub span: Span,
}

/// Literal kinds.
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralKind {
    Bool(bool),
    Integer(i64),
    Real(f64),
    String(String),
    WString(String),
    Time(String),   // Stored as string, parsed later
    Date(String),   // Stored as string, parsed later
    Binary(String), // 2#...
    Octal(String),  // 8#...
    Hex(String),    // 16#...
}

/// Binary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Exp, // **
    
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
    
    // Bitwise
    BitAnd,
    BitOr,
    BitXor,
}

impl BinaryOp {
    /// Get operator precedence (higher = binds tighter).
    /// Based on SCL specification precedence levels.
    pub fn precedence(self) -> u8 {
        match self {
            // Level 11: highest
            BinaryOp::Exp => 11,
            
            // Level 10: *, /, MOD
            BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => 10,
            
            // Level 9: +, -
            BinaryOp::Add | BinaryOp::Sub => 9,
            
            // Level 8: comparisons
            BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => 8,
            
            // Level 7: =, <>
            BinaryOp::Eq | BinaryOp::Ne => 7,
            
            // Level 5: AND, &
            BinaryOp::And | BinaryOp::BitAnd => 5,
            
            // Level 4: XOR
            BinaryOp::Xor | BinaryOp::BitXor => 4,
            
            // Level 3: OR
            BinaryOp::Or | BinaryOp::BitOr => 3,
        }
    }
}

/// Unary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,    // -
    Pos,    // +
    Not,    // NOT
    BitNot, // Bitwise NOT
}
