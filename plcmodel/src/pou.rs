//! Program Organization Unit (POU) definitions.

use crate::{Body, Variable};
use iectypes::PouType;

/// A Program Organization Unit.
///
/// POUs are the building blocks of IEC 61131-3 programs:
/// - **Program**: Stateful, scheduled by tasks
/// - **Function Block**: Stateful, instantiated and called
/// - **Function**: Stateless, returns a value
///
/// Maps to:
/// - L5X: `<Program>` for Programs, `<AddOnInstructionDefinition>` for FBs
/// - PLCopen: `<pou>` element with pouType attribute
#[derive(Debug, Clone)]
pub struct Pou {
    /// POU name (identifier)
    pub name: String,

    /// Type: Program, Function, or FunctionBlock
    pub pou_type: PouType,

    /// Optional description
    pub description: Option<String>,

    /// Interface (variable declarations)
    pub interface: PouInterface,

    /// Program body (the actual code)
    pub body: Option<Body>,
}

impl Pou {
    /// Create a new POU with the given name and type.
    pub fn new(name: impl Into<String>, pou_type: PouType) -> Self {
        Self {
            name: name.into(),
            pou_type,
            description: None,
            interface: PouInterface::default(),
            body: None,
        }
    }

    /// Check if this POU has any code.
    pub fn is_empty(&self) -> bool {
        self.body.as_ref().is_none_or(|b| b.is_empty())
    }

    /// Get all variables (input, output, local, etc.)
    pub fn all_variables(&self) -> impl Iterator<Item = &Variable> {
        self.interface.all_variables()
    }

    /// Find a variable by name.
    pub fn find_variable(&self, name: &str) -> Option<&Variable> {
        self.interface.find_variable(name)
    }
}

/// Interface section of a POU - variable declarations grouped by scope.
#[derive(Debug, Clone, Default)]
pub struct PouInterface {
    /// Input parameters (VAR_INPUT)
    pub inputs: Vec<Variable>,

    /// Output parameters (VAR_OUTPUT)
    pub outputs: Vec<Variable>,

    /// In/Out parameters (VAR_IN_OUT)
    pub in_outs: Vec<Variable>,

    /// Local variables (VAR)
    pub locals: Vec<Variable>,

    /// Temporary variables (VAR_TEMP)
    pub temps: Vec<Variable>,

    /// External references (VAR_EXTERNAL)
    pub externals: Vec<Variable>,

    /// Return type for Functions
    pub return_type: Option<String>,
}

impl PouInterface {
    /// Get all variables across all scopes.
    pub fn all_variables(&self) -> impl Iterator<Item = &Variable> {
        self.inputs
            .iter()
            .chain(self.outputs.iter())
            .chain(self.in_outs.iter())
            .chain(self.locals.iter())
            .chain(self.temps.iter())
            .chain(self.externals.iter())
    }

    /// Find a variable by name in any scope.
    pub fn find_variable(&self, name: &str) -> Option<&Variable> {
        self.all_variables().find(|v| v.name == name)
    }

    /// Total variable count.
    pub fn variable_count(&self) -> usize {
        self.inputs.len()
            + self.outputs.len()
            + self.in_outs.len()
            + self.locals.len()
            + self.temps.len()
            + self.externals.len()
    }
}
