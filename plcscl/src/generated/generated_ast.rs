// Generated SCL AST

#[derive(Debug, Clone)]
pub struct Program {
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone)]
pub enum Block {
    FunctionBlock(FunctionBlock),
    Function(Function),
    DataBlock(DataBlock),
    TypeDecl(TypeDecl),
    OrganizationBlock(OrganizationBlock),
    ProgramBlock(ProgramBlock),
    Class(ClassDecl),
    Interface(InterfaceDecl),
}

#[derive(Debug, Clone)]
pub struct FunctionBlock {
    pub name: String,
    pub extends: Option<String>,
    pub implements: Vec<String>,
    pub var_sections: Vec<VarSection>,
    pub methods: Vec<MethodDecl>,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct OrganizationBlock {
    pub name: String,
    pub var_sections: Vec<VarSection>,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct ProgramBlock {
    pub name: String,
    pub var_sections: Vec<VarSection>,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct ClassDecl {
    pub name: String,
    pub extends: Option<String>,
    pub implements: Vec<String>,
    pub var_sections: Vec<VarSection>,
    pub methods: Vec<MethodDecl>,
}

#[derive(Debug, Clone)]
pub struct InterfaceDecl {
    pub name: String,
    pub extends: Option<String>,
    pub methods: Vec<MethodSignature>,
}

#[derive(Debug, Clone)]
pub struct MethodDecl {
    pub access: Option<AccessModifier>,
    pub name: String,
    pub return_type: TypeRef,
    pub var_sections: Vec<VarSection>,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct MethodSignature {
    pub name: String,
    pub return_type: TypeRef,
    pub var_sections: Vec<VarSection>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AccessModifier {
    Public,
    Private,
    Protected,
    Internal,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub return_type: TypeRef,
    pub var_sections: Vec<VarSection>,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct DataBlock {
    pub name: String,
    pub var_sections: Vec<VarSection>,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct TypeDecl {
    pub name: String,
    pub type_spec: TypeSpec,
}

#[derive(Debug, Clone)]
pub struct TypeSpec {
    pub type_ref: TypeRef,
}

#[derive(Debug, Clone)]
pub enum VarSection {
    Input(VarInput),
    Output(VarOutput),
    InOut(VarInout),
    Temp(VarTemp),
    Var(VarDecl),
    Constant(VarDecl),
}

#[derive(Debug, Clone)]
pub struct VarInput {
    pub declarations: Vec<VarDeclaration>,
}

#[derive(Debug, Clone)]
pub struct VarOutput {
    pub declarations: Vec<VarDeclaration>,
}

#[derive(Debug, Clone)]
pub struct VarInout {
    pub declarations: Vec<VarDeclaration>,
}

#[derive(Debug, Clone)]
pub struct VarTemp {
    pub declarations: Vec<VarDeclaration>,
}

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub declarations: Vec<VarDeclaration>,
}

#[derive(Debug, Clone)]
pub struct VarDeclaration {
    pub names: Vec<String>,
    pub type_ref: TypeRef,
    pub at_address: Option<String>,
    pub initializer: Option<Expression>,
}

#[derive(Debug, Clone)]
pub enum TypeRef {
    Named(String),
    Array(Box<ArrayType>),
    Struct(StructType),
    Pointer(Box<PointerType>),
    Variant,
    Any,
}

#[derive(Debug, Clone)]
pub struct ArrayType {
    pub start: Expression,
    pub end: Expression,
    pub element_type: TypeRef,
}

#[derive(Debug, Clone)]
pub struct StructType {
    pub fields: Vec<StructField>,
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub type_ref: TypeRef,
}

#[derive(Debug, Clone)]
pub struct PointerType {
    pub target_type: TypeRef,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Assignment(Assignment),
    NullableAssignment(NullableAssignment),
    If(IfStmt),
    Case(CaseStmt),
    For(ForStmt),
    While(WhileStmt),
    Repeat(RepeatStmt),
    Return(ReturnStmt),
    Exit,
    Continue,
    FunctionCall(FunctionCallStmt),
    Region(Region),
}

#[derive(Debug, Clone)]
pub struct Region {
    pub name: String,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub target: String,
    pub value: Expression,
}

#[derive(Debug, Clone)]
pub struct NullableAssignment {
    pub target: String,
    pub value: Expression,
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub condition: Expression,
    pub then_body: Vec<Statement>,
    pub elsif_parts: Vec<ElsifPart>,
    pub else_part: Option<ElsePart>,
}

#[derive(Debug, Clone)]
pub struct ElsifPart {
    pub condition: Expression,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct ElsePart {
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct CaseStmt {
    pub expression: Expression,
    pub elements: Vec<CaseElement>,
    pub else_part: Option<ElsePart>,
}

#[derive(Debug, Clone)]
pub struct CaseElement {
    pub values: Vec<Expression>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct ForStmt {
    pub variable: String,
    pub start: Expression,
    pub end: Expression,
    pub by: Option<ByPart>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct ByPart {
    pub value: Expression,
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub condition: Expression,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct RepeatStmt {
    pub body: Vec<Statement>,
    pub condition: Expression,
}

#[derive(Debug, Clone)]
pub struct ReturnStmt {}

#[derive(Debug, Clone)]
pub struct FunctionCallStmt {
    pub call: FunctionCall,
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: Vec<Argument>,
}

#[derive(Debug, Clone)]
pub struct Argument {
    pub name: Option<String>,
    pub value: Expression,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Or(OrExpr),
    Xor(XorExpr),
    And(AndExpr),
    Comparison(Comparison),
    Add(AddExpr),
    Mult(MultExpr),
    Unary(UnaryExpr),
    Primary(Primary),
}

#[derive(Debug, Clone)]
pub struct OrExpr {
    pub left: Box<Expression>,
    pub right: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct XorExpr {
    pub left: Box<Expression>,
    pub right: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct AndExpr {
    pub left: Box<Expression>,
    pub right: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct Comparison {
    pub left: Box<Expression>,
    pub op: ComparisonOp,
    pub right: Box<Expression>,
}

#[derive(Debug, Clone)]
pub enum ComparisonOp {
    Eq, Ne, Lt, Gt, Le, Ge,
}

#[derive(Debug, Clone)]
pub struct AddExpr {
    pub left: Box<Expression>,
    pub op: AddOp,
    pub right: Box<Expression>,
}

#[derive(Debug, Clone)]
pub enum AddOp {
    Add, Sub,
}

#[derive(Debug, Clone)]
pub struct MultExpr {
    pub left: Box<Expression>,
    pub op: MultOp,
    pub right: Box<Expression>,
}

#[derive(Debug, Clone)]
pub enum MultOp {
    Mult, Div, Mod,
}

#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub op: Option<UnaryOp>,
    pub operand: Box<Primary>,
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Plus, Minus, Not,
}

#[derive(Debug, Clone)]
pub enum Primary {
    Literal(Literal),
    Identifier(String),
    FunctionCall(FunctionCall),
    Parenthesized(Box<Expression>),
}

#[derive(Debug, Clone)]
pub enum Literal {
    IntLit(String),
    FloatLit(String),
    String(String),
    Boolean(bool),
    Time(String),
}
