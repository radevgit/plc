//! L5X project analysis module.
//!
//! This module provides utilities to analyze L5X project structures,
//! parse RLL rungs and ST routines, and build cross-reference indices.
//!
//! This is the core analysis engine used by smell detectors.

mod l5x_analysis;
mod rll_parsing;
mod st_parsing;

pub use l5x_analysis::{
    ProjectAnalysis, ParseStats, RoutineSummary,
    RungLocation, LocatedRung, LocatedTagReference,
    STLocation, ParsedSTRoutine,
    AoiReference, AoiCallSource,
    analyze_controller,
};

pub use rll_parsing::{
    parse_routine, parse_rung_collection, extract_rung_text, extract_text_content,
};

pub use st_parsing::{
    parse_st_routine, parse_st_routines_from_program, extract_st_source,
    extract_st_call_names,
};
