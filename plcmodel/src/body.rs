//! Program body representations.

/// The body/implementation of a POU.
///
/// Different PLC programming languages have different representations,
/// but we normalize them for analysis purposes.
#[derive(Debug, Clone)]
pub enum Body {
    /// Structured Text
    St(String),

    /// Instruction List
    Il(String),

    /// Ladder Diagram (normalized to rungs)
    Ld(Vec<Rung>),

    /// Function Block Diagram (normalized to networks)
    Fbd(Vec<Network>),

    /// Sequential Function Chart
    Sfc(SfcBody),

    /// Raw/unparsed content (for unknown formats)
    Raw { language: String, content: String },
}

impl Body {
    /// Check if the body is empty.
    pub fn is_empty(&self) -> bool {
        match self {
            Body::St(s) | Body::Il(s) => s.trim().is_empty(),
            Body::Ld(rungs) => rungs.is_empty(),
            Body::Fbd(networks) => networks.is_empty(),
            Body::Sfc(sfc) => sfc.steps.is_empty(),
            Body::Raw { content, .. } => content.trim().is_empty(),
        }
    }

    /// Get the language name.
    pub fn language(&self) -> &str {
        match self {
            Body::St(_) => "ST",
            Body::Il(_) => "IL",
            Body::Ld(_) => "LD",
            Body::Fbd(_) => "FBD",
            Body::Sfc(_) => "SFC",
            Body::Raw { language, .. } => language,
        }
    }
}

/// A ladder diagram rung.
#[derive(Debug, Clone)]
pub struct Rung {
    /// Rung number (0-indexed)
    pub number: u32,

    /// Optional rung comment
    pub comment: Option<String>,

    /// Instructions in this rung
    pub instructions: Vec<Instruction>,

    /// Raw text representation (if available)
    pub raw_text: Option<String>,
}

impl Rung {
    /// Create an empty rung.
    pub fn new(number: u32) -> Self {
        Self {
            number,
            comment: None,
            instructions: Vec::new(),
            raw_text: None,
        }
    }
}

/// An instruction within a rung or network.
#[derive(Debug, Clone)]
pub struct Instruction {
    /// Instruction mnemonic (XIC, OTE, TON, etc.)
    pub mnemonic: String,

    /// Operands/parameters
    pub operands: Vec<Operand>,

    /// Source position (if available)
    pub position: Option<Position>,
}

impl Instruction {
    /// Create a simple instruction.
    pub fn new(mnemonic: impl Into<String>) -> Self {
        Self {
            mnemonic: mnemonic.into(),
            operands: Vec::new(),
            position: None,
        }
    }

    /// Add an operand.
    pub fn with_operand(mut self, operand: Operand) -> Self {
        self.operands.push(operand);
        self
    }
}

/// An operand to an instruction.
#[derive(Debug, Clone)]
pub enum Operand {
    /// Tag/variable reference
    Tag(String),

    /// Literal value
    Literal(String),

    /// Nested expression
    Expression(String),
}

/// Position in source.
#[derive(Debug, Clone, Copy)]
pub struct Position {
    /// Rung or network number
    pub rung: u32,
    /// Position within the rung
    pub column: u32,
}

/// A FBD network.
#[derive(Debug, Clone)]
pub struct Network {
    /// Network number
    pub number: u32,

    /// Optional network comment/label
    pub label: Option<String>,

    /// Instructions/blocks in this network
    pub instructions: Vec<Instruction>,
}

impl Network {
    /// Create an empty network.
    pub fn new(number: u32) -> Self {
        Self {
            number,
            label: None,
            instructions: Vec::new(),
        }
    }
}

/// Sequential Function Chart body.
#[derive(Debug, Clone, Default)]
pub struct SfcBody {
    /// Steps in the SFC
    pub steps: Vec<SfcStep>,

    /// Transitions between steps
    pub transitions: Vec<SfcTransition>,
}

/// An SFC step.
#[derive(Debug, Clone)]
pub struct SfcStep {
    /// Step name
    pub name: String,

    /// Is this the initial step?
    pub is_initial: bool,

    /// Actions associated with this step
    pub actions: Vec<SfcAction>,
}

/// An action within an SFC step.
#[derive(Debug, Clone)]
pub struct SfcAction {
    /// Action name
    pub name: String,

    /// Action qualifier (N, S, R, P, etc.)
    pub qualifier: String,

    /// Action body
    pub body: Option<Box<Body>>,
}

/// An SFC transition.
#[derive(Debug, Clone)]
pub struct SfcTransition {
    /// Transition name (optional)
    pub name: Option<String>,

    /// Source step(s)
    pub from_steps: Vec<String>,

    /// Target step(s)
    pub to_steps: Vec<String>,

    /// Transition condition
    pub condition: String,
}
