//! Variable definitions.

use iectypes::VarClass;

/// A variable declaration.
///
/// Maps to:
/// - L5X: `<Tag>` elements
/// - PLCopen: `<variable>` within interface sections
#[derive(Debug, Clone)]
pub struct Variable {
    /// Variable name
    pub name: String,

    /// Data type (as string, may be user-defined)
    pub data_type: String,

    /// Variable class (Input, Output, Local, etc.)
    pub var_class: VarClass,

    /// Initial value expression (if any)
    pub initial_value: Option<String>,

    /// Optional description/comment
    pub description: Option<String>,

    /// Direct address binding (%I0.0, %QW10, etc.)
    pub address: Option<String>,

    /// Array dimensions [10], [3,4], etc. Empty for scalar.
    pub dimensions: Vec<u32>,

    /// Is this variable a constant?
    pub is_constant: bool,

    /// Is this variable retained across power cycles?
    pub is_retain: bool,
}

impl Variable {
    /// Create a new local variable.
    pub fn new(name: impl Into<String>, data_type: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            data_type: data_type.into(),
            var_class: VarClass::Local,
            initial_value: None,
            description: None,
            address: None,
            dimensions: Vec::new(),
            is_constant: false,
            is_retain: false,
        }
    }

    /// Create an input variable.
    pub fn input(name: impl Into<String>, data_type: impl Into<String>) -> Self {
        let mut var = Self::new(name, data_type);
        var.var_class = VarClass::Input;
        var
    }

    /// Create an output variable.
    pub fn output(name: impl Into<String>, data_type: impl Into<String>) -> Self {
        let mut var = Self::new(name, data_type);
        var.var_class = VarClass::Output;
        var
    }

    /// Create an in/out variable.
    pub fn in_out(name: impl Into<String>, data_type: impl Into<String>) -> Self {
        let mut var = Self::new(name, data_type);
        var.var_class = VarClass::InOut;
        var
    }

    /// Check if this is an array type.
    pub fn is_array(&self) -> bool {
        !self.dimensions.is_empty()
    }

    /// Get total array size (product of dimensions).
    pub fn array_size(&self) -> u32 {
        if self.dimensions.is_empty() {
            1
        } else {
            self.dimensions.iter().product()
        }
    }
}
