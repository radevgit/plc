//! Project analysis module.
//!
//! This module provides utilities to analyze PLC projects,
//! parse code, and build cross-reference indices.
//!
//! - `l5x_analysis` - L5X-specific detailed analysis
//! - `plcopen_analysis` - PLCopen-specific analysis

mod iec61131_adapter;
mod l5x_analysis;
mod plcopen_analysis;
mod rll_parsing;
mod st_parsing;

pub use iec61131_adapter::{Pou, ParseError, parse_pou};

pub use l5x_analysis::{
    ProjectAnalysis, ParseStats, RoutineSummary,
    RungLocation, LocatedRung, LocatedTagReference,
    STLocation, ParsedSTRoutine,
    AoiReference, AoiCallSource,
    analyze_controller,
};

pub use plcopen_analysis::{
    PlcopenAnalysis, PlcopenStats, VariableDef, VarClass,
    analyze_project as analyze_plcopen_project,
};

pub use rll_parsing::{
    parse_routine, parse_rung_collection, extract_rung_text, extract_text_content,
};

pub use st_parsing::{
    parse_st_routine, parse_st_routines_from_program, extract_st_source,
    extract_st_call_names,
};
