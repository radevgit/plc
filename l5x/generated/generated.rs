// Auto-generated L5X types from XSD schema
// DO NOT EDIT MANUALLY

#![allow(clippy::large_enum_variant)]

use serde::{Deserialize, Serialize};

#[path = "generated_programs.rs"]
mod programs;
#[path = "generated_core.rs"]
mod core;
#[path = "generated_devices.rs"]
mod devices;
#[path = "generated_network.rs"]
mod network;
#[path = "generated_alarms.rs"]
mod alarms;
#[path = "generated_instructions.rs"]
mod instructions;
#[path = "generated_children.rs"]
mod children;
#[path = "generated_trends.rs"]
mod trends;
#[path = "generated_tags.rs"]
mod tags;
#[path = "generated_security.rs"]
mod security;
#[path = "generated_motion.rs"]
mod motion;
#[path = "generated_elements.rs"]
mod elements;
#[path = "generated_datatypes.rs"]
mod datatypes;
#[path = "generated_misc.rs"]
mod misc;
#[path = "generated_tasks.rs"]
mod tasks;
#[path = "generated_data.rs"]
mod data;
pub use self::{
    programs::*,
    core::*,
    devices::*,
    network::*,
    alarms::*,
    instructions::*,
    children::*,
    trends::*,
    tags::*,
    security::*,
    motion::*,
    elements::*,
    datatypes::*,
    misc::*,
    tasks::*,
    data::*
};

/// Placeholder for xs:any wildcard elements
/// quick-xml cannot deserialize dynamic element names, so these are skipped
/// Use xmltree-based parsing if you need to preserve xs:any content
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AnyElement;

/// DecoratedDataElements choice group
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum DecoratedDataElements {
    DataValue(DataValue),
    Array(Box<DataArray>),
    Structure(Box<DataStructure>),
}

