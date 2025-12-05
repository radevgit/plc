//! # plcmodel
//!
//! Vendor-neutral PLC model for IEC 61131-3 programs.
//!
//! This crate provides abstract representations that different PLC formats
//! (L5X, PLCopen XML, etc.) can map to, enabling format-independent analysis.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────┐     ┌─────────────┐     ┌──────────┐
//! │   L5X   │────▶│             │────▶│  plceye  │
//! └─────────┘     │   PlcModel  │     │ analysis │
//! ┌─────────┐     │             │     └──────────┘
//! │ PLCopen │────▶│             │────▶│  plcviz  │
//! └─────────┘     └─────────────┘     └──────────┘
//! ```
//!
//! ## Key Types
//!
//! - [`Project`] - Top-level container for all PLC configuration
//! - [`Pou`] - Program Organization Unit (Program, Function, FunctionBlock)
//! - [`Variable`] - Variables with scope and data type
//! - [`DataTypeDef`] - User-defined types (struct, enum, array)
//! - [`Task`] - Task configuration for scheduling

mod body;
mod data_type;
mod pou;
mod project;
mod task;
mod variable;

pub use body::{Body, Instruction, Network, Operand, Position, Rung, SfcAction, SfcBody, SfcStep, SfcTransition};
pub use data_type::{ArrayDef, ArrayDimension, DataTypeDef, DataTypeKind, EnumDef, EnumMember, StructDef, StructMember};
pub use pou::{Pou, PouInterface};
pub use project::{Configuration, Project, Resource};
pub use task::{Task, TaskTrigger};
pub use variable::Variable;

/// Trait for types that can be converted to the common PLC model.
///
/// Implemented by format-specific parsers (L5X, PLCopen, etc.)
pub trait ToPlcModel {
    /// Convert this type to a vendor-neutral [`Project`].
    fn to_plc_model(&self) -> Project;
}
