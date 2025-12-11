//! Generated AST for IEC 61131-3

use super::lexer::Span;

/// Root compilation unit
#[derive(Debug, Clone)]
pub struct CompilationUnit {
    pub declarations: Vec<PouDeclaration>,
    pub span: Span,
}

/// Program Organization Unit declarations
#[derive(Debug, Clone)]
pub enum PouDeclaration {
    Function(FunctionDecl),
    FunctionBlock(FunctionBlockDecl),
    Program(ProgramDecl),
    Class(ClassDecl),
    Interface(InterfaceDecl),
    DataType(DataTypeDecl),
    GlobalVar(GlobalVarDecl),
    Namespace(NamespaceDecl),
}

/// Function declaration
#[derive(Debug, Clone)]
pub struct FunctionDecl {
    pub name: String,
    pub return_type: Option<TypeSpec>,
    pub inputs: Vec<VarDecl>,
    pub outputs: Vec<VarDecl>,
    pub in_outs: Vec<VarDecl>,
    pub vars: Vec<VarDecl>,
    pub body: StatementList,
    pub span: Span,
}

/// Function block declaration
#[derive(Debug, Clone)]
pub struct FunctionBlockDecl {
    pub name: String,
    pub extends: Option<String>,
    pub implements: Vec<String>,
    pub is_final: bool,
    pub is_abstract: bool,
    pub inputs: Vec<VarDecl>,
    pub outputs: Vec<VarDecl>,
    pub in_outs: Vec<VarDecl>,
    pub vars: Vec<VarDecl>,
    pub methods: Vec<MethodDecl>,
    pub body: Option<StatementList>,
    pub span: Span,
}

/// Program declaration
#[derive(Debug, Clone)]
pub struct ProgramDecl {
    pub name: String,
    pub vars: Vec<VarDecl>,
    pub body: StatementList,
    pub span: Span,
}

/// Class declaration
#[derive(Debug, Clone)]
pub struct ClassDecl {
    pub name: String,
    pub extends: Option<String>,
    pub implements: Vec<String>,
    pub is_final: bool,
    pub is_abstract: bool,
    pub vars: Vec<VarDecl>,
    pub methods: Vec<MethodDecl>,
    pub span: Span,
}

/// Interface declaration
#[derive(Debug, Clone)]
pub struct InterfaceDecl {
    pub name: String,
    pub extends: Vec<String>,
    pub methods: Vec<MethodPrototype>,
    pub span: Span,
}

/// Method declaration
#[derive(Debug, Clone)]
pub struct MethodDecl {
    pub name: String,
    pub access: AccessModifier,
    pub return_type: Option<TypeSpec>,
    pub is_final: bool,
    pub is_abstract: bool,
    pub is_override: bool,
    pub inputs: Vec<VarDecl>,
    pub outputs: Vec<VarDecl>,
    pub in_outs: Vec<VarDecl>,
    pub vars: Vec<VarDecl>,
    pub body: StatementList,
    pub span: Span,
}

/// Method prototype (for interfaces)
#[derive(Debug, Clone)]
pub struct MethodPrototype {
    pub name: String,
    pub return_type: Option<TypeSpec>,
    pub inputs: Vec<VarDecl>,
    pub outputs: Vec<VarDecl>,
    pub in_outs: Vec<VarDecl>,
    pub span: Span,
}

/// Access modifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessModifier {
    Public,
    Protected,
    Private,
    Internal,
}

/// Variable declaration
#[derive(Debug, Clone)]
pub struct VarDecl {
    pub name: String,
    pub var_type: TypeSpec,
    pub init_value: Option<Expression>,
    pub is_constant: bool,
    pub is_retain: bool,
    pub location: Option<DirectVariable>,
    pub span: Span,
}

/// Type specification
#[derive(Debug, Clone)]
pub enum TypeSpec {
    /// Elementary type (BOOL, INT, REAL, etc.)
    Elementary(String),
    /// Array type
    Array {
        dimensions: Vec<ArrayDimension>,
        element_type: Box<TypeSpec>,
    },
    /// Structure type
    Struct {
        fields: Vec<StructField>,
    },
    /// Reference type
    Ref(Box<TypeSpec>),
    /// User-defined type
    UserDefined(String),
}

#[derive(Debug, Clone)]
pub struct ArrayDimension {
    pub start: Expression,
    pub end: Expression,
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub field_type: TypeSpec,
    pub init_value: Option<Expression>,
}

/// Data type declaration
#[derive(Debug, Clone)]
pub enum DataTypeDecl {
    Simple {
        name: String,
        base_type: TypeSpec,
        init_value: Option<Expression>,
    },
    Subrange {
        name: String,
        base_type: String,
        min: Expression,
        max: Expression,
    },
    Enum {
        name: String,
        values: Vec<EnumValue>,
    },
    Array {
        name: String,
        spec: TypeSpec,
    },
    Struct {
        name: String,
        fields: Vec<StructField>,
    },
}

#[derive(Debug, Clone)]
pub struct EnumValue {
    pub name: String,
    pub value: Option<i64>,
}

/// Global variable declaration
#[derive(Debug, Clone)]
pub struct GlobalVarDecl {
    pub vars: Vec<VarDecl>,
    pub is_constant: bool,
    pub is_retain: bool,
    pub span: Span,
}

/// Namespace declaration
#[derive(Debug, Clone)]
pub struct NamespaceDecl {
    pub name: Vec<String>,
    pub is_internal: bool,
    pub using_directives: Vec<Vec<String>>,
    pub elements: Vec<PouDeclaration>,
    pub span: Span,
}

/// Direct variable (%IX0.0, %QW10, etc.)
#[derive(Debug, Clone)]
pub struct DirectVariable {
    pub location: String,
    pub span: Span,
}

/// Statement list
pub type StatementList = Vec<Statement>;

/// Statement
#[derive(Debug, Clone)]
pub enum Statement {
    /// Assignment: variable := expression
    Assignment {
        target: Variable,
        value: Expression,
        span: Span,
    },
    /// IF statement
    If {
        condition: Expression,
        then_body: StatementList,
        elsif_parts: Vec<(Expression, StatementList)>,
        else_body: Option<StatementList>,
        span: Span,
    },
    /// CASE statement
    Case {
        selector: Expression,
        cases: Vec<CaseItem>,
        else_body: Option<StatementList>,
        span: Span,
    },
    /// FOR loop
    For {
        control_var: String,
        start: Expression,
        end: Expression,
        step: Option<Expression>,
        body: StatementList,
        span: Span,
    },
    /// WHILE loop
    While {
        condition: Expression,
        body: StatementList,
        span: Span,
    },
    /// REPEAT loop
    Repeat {
        body: StatementList,
        condition: Expression,
        span: Span,
    },
    /// EXIT
    Exit { span: Span },
    /// CONTINUE
    Continue { span: Span },
    /// RETURN
    Return { value: Option<Expression>, span: Span },
    /// Function call
    FunctionCall {
        name: String,
        arguments: Vec<Argument>,
        span: Span,
    },
    /// Function block invocation
    FbInvocation {
        instance: String,
        arguments: Vec<Argument>,
        span: Span,
    },
}

#[derive(Debug, Clone)]
pub struct CaseItem {
    pub selectors: Vec<CaseSelector>,
    pub body: StatementList,
}

#[derive(Debug, Clone)]
pub enum CaseSelector {
    Value(Expression),
    Range(Expression, Expression),
}

#[derive(Debug, Clone)]
pub enum Argument {
    Positional(Expression),
    Named { name: String, value: Expression },
    Output { name: String, variable: Variable },
}

/// Variable (can be simple or complex with member access, array indexing)
#[derive(Debug, Clone)]
pub enum Variable {
    Direct(DirectVariable),
    Simple(String),
    MemberAccess {
        base: Box<Variable>,
        member: String,
    },
    ArrayAccess {
        base: Box<Variable>,
        indices: Vec<Expression>,
    },
    Dereference {
        base: Box<Variable>,
    },
}

/// Expression
#[derive(Debug, Clone)]
pub enum Expression {
    /// Literal value
    Literal(Literal),
    /// Variable reference
    Variable(Variable),
    /// Unary operation
    Unary {
        op: UnaryOp,
        operand: Box<Expression>,
    },
    /// Binary operation
    Binary {
        op: BinaryOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    /// Function call
    Call {
        function: String,
        arguments: Vec<Argument>,
    },
    /// Parenthesized expression
    Parenthesized(Box<Expression>),
}

#[derive(Debug, Clone)]
pub enum Literal {
    Bool(bool),
    Integer(i64),
    Real(f64),
    String(String),
    Time(String),
    Null,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

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
