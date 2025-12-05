//! Project and configuration structures.

use crate::{DataTypeDef, Pou, Task};

/// A PLC project - top-level container.
///
/// Maps to:
/// - L5X: `<RSLogix5000Content>` with `<Controller>`
/// - PLCopen: `<project>` element
#[derive(Debug, Clone, Default)]
pub struct Project {
    /// Project name
    pub name: String,

    /// Optional description
    pub description: Option<String>,

    /// User-defined data types
    pub data_types: Vec<DataTypeDef>,

    /// Program Organization Units (Programs, Functions, FBs)
    pub pous: Vec<Pou>,

    /// Configuration (hardware, tasks, resources)
    pub configuration: Option<Configuration>,

    /// Source format (e.g., "L5X", "PLCopen")
    pub source_format: Option<String>,
}

impl Project {
    /// Create an empty project with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Find a POU by name.
    pub fn find_pou(&self, name: &str) -> Option<&Pou> {
        self.pous.iter().find(|p| p.name == name)
    }

    /// Find a data type definition by name.
    pub fn find_data_type(&self, name: &str) -> Option<&DataTypeDef> {
        self.data_types.iter().find(|dt| dt.name == name)
    }

    /// Get all programs (POUs with type Program).
    pub fn programs(&self) -> impl Iterator<Item = &Pou> {
        self.pous
            .iter()
            .filter(|p| matches!(p.pou_type, iectypes::PouType::Program))
    }

    /// Get all function blocks.
    pub fn function_blocks(&self) -> impl Iterator<Item = &Pou> {
        self.pous
            .iter()
            .filter(|p| matches!(p.pou_type, iectypes::PouType::FunctionBlock))
    }

    /// Get all functions.
    pub fn functions(&self) -> impl Iterator<Item = &Pou> {
        self.pous
            .iter()
            .filter(|p| matches!(p.pou_type, iectypes::PouType::Function))
    }
}

/// Hardware/resource configuration.
///
/// Maps to:
/// - L5X: Controller modules and tasks
/// - PLCopen: `<configurations>` element
#[derive(Debug, Clone, Default)]
pub struct Configuration {
    /// Configuration name
    pub name: String,

    /// Resources (CPUs/PLCs in the configuration)
    pub resources: Vec<Resource>,
}

/// A resource within a configuration (typically a PLC or CPU).
#[derive(Debug, Clone, Default)]
pub struct Resource {
    /// Resource name
    pub name: String,

    /// Tasks scheduled on this resource
    pub tasks: Vec<Task>,

    /// Global variables for this resource
    pub global_vars: Vec<crate::Variable>,
}
