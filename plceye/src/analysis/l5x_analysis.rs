//! L5X project-wide analysis.
//!
//! This module provides the core analysis engine that walks L5X project
//! structures and builds cross-reference indices for smell detection.

use std::collections::{HashMap, HashSet};

use l5x::rll::{Rung as ParsedRung, TagReference, ErrorContext, ParseError};
use l5x::{
    Controller,
    UDIDefinition, UDIDefinitionContent,
};
use iecst::Pou;

use super::rll_parsing::parse_routine;
use super::st_parsing::{
    parse_st_routines_from_program, parse_st_routines_from_aoi,
    extract_st_call_names,
};

/// Location of a rung within a project.
#[derive(Debug, Clone, PartialEq)]
pub struct RungLocation {
    /// Program name
    pub program: String,
    /// Routine name
    pub routine: String,
    /// Rung number
    pub rung_number: u32,
}

impl RungLocation {
    pub fn new(program: impl Into<String>, routine: impl Into<String>, rung_number: u32) -> Self {
        Self {
            program: program.into(),
            routine: routine.into(),
            rung_number,
        }
    }

    pub fn path(&self) -> String {
        format!("{}/{}/Rung#{}", self.program, self.routine, self.rung_number)
    }
}

/// A parsed rung with its location in the project.
#[derive(Debug, Clone)]
pub struct LocatedRung {
    pub location: RungLocation,
    pub parsed: ParsedRung,
}

impl LocatedRung {
    pub fn tag_references(&self) -> Vec<LocatedTagReference> {
        self.parsed
            .tag_references()
            .into_iter()
            .map(|r| LocatedTagReference {
                location: self.location.clone(),
                reference: r,
            })
            .collect()
    }

    pub fn parse_error(&self) -> Option<ParseError> {
        self.parsed.error.as_ref().map(|err| {
            ParseError::new(err.clone(), &self.parsed.raw_text).with_context(ErrorContext::new(
                &self.location.program,
                &self.location.routine,
                self.location.rung_number,
            ))
        })
    }

    pub fn has_error(&self) -> bool {
        self.parsed.error.is_some()
    }
}

/// A tag reference with its location in the project.
#[derive(Debug, Clone)]
pub struct LocatedTagReference {
    pub location: RungLocation,
    pub reference: TagReference,
}

impl LocatedTagReference {
    pub fn tag_name(&self) -> &str {
        &self.reference.name
    }

    pub fn full_operand(&self) -> &str {
        &self.reference.full_operand
    }

    pub fn instruction(&self) -> &str {
        &self.reference.instruction
    }
}

/// Location of an ST routine within a project.
#[derive(Debug, Clone, PartialEq)]
pub struct STLocation {
    pub program: String,
    pub routine: String,
}

impl STLocation {
    pub fn new(program: impl Into<String>, routine: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            routine: routine.into(),
        }
    }

    pub fn path(&self) -> String {
        format!("{}/{}", self.program, self.routine)
    }
}

/// A parsed ST routine with its location.
#[derive(Debug)]
pub struct ParsedSTRoutine {
    pub location: STLocation,
    pub source: String,
    pub pou: Option<Pou>,
    pub parse_error: Option<iecst::ParseError>,
}

impl ParsedSTRoutine {
    pub fn is_parsed(&self) -> bool {
        self.pou.is_some()
    }
}

/// Location where an AOI is called.
#[derive(Debug, Clone, PartialEq)]
pub struct AoiReference {
    pub aoi_name: String,
    pub program: String,
    pub routine: String,
    pub rung_number: Option<u32>,
    pub source: AoiCallSource,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AoiCallSource {
    Rll,
    St,
}

impl AoiReference {
    pub fn from_rll(aoi_name: &str, location: &RungLocation) -> Self {
        Self {
            aoi_name: aoi_name.to_string(),
            program: location.program.clone(),
            routine: location.routine.clone(),
            rung_number: Some(location.rung_number),
            source: AoiCallSource::Rll,
        }
    }

    pub fn from_st(aoi_name: &str, location: &STLocation) -> Self {
        Self {
            aoi_name: aoi_name.to_string(),
            program: location.program.clone(),
            routine: location.routine.clone(),
            rung_number: None,
            source: AoiCallSource::St,
        }
    }

    pub fn path(&self) -> String {
        match self.rung_number {
            Some(n) => format!("{}/{}/Rung#{}", self.program, self.routine, n),
            None => format!("{}/{}", self.program, self.routine),
        }
    }
}

/// Statistics from parsing a project.
#[derive(Debug, Clone, Default)]
pub struct ParseStats {
    pub programs: usize,
    pub aois: usize,
    pub routines: usize,
    pub rungs: usize,
    pub rll_rungs_programs: usize,
    pub rll_rungs_aois: usize,
    pub parsed_ok: usize,
    pub parsed_err: usize,
    pub tag_references: usize,
    pub unique_tags: usize,
    pub st_routines: usize,
    pub st_routines_programs: usize,
    pub st_routines_aois: usize,
    pub st_parsed_ok: usize,
    pub st_parsed_err: usize,
}

/// Summary of a single routine.
#[derive(Debug, Clone)]
pub struct RoutineSummary {
    pub program: String,
    pub routine: String,
    pub routine_type: String,
    pub rung_count: usize,
    pub rung_indices: Vec<usize>,
    pub parse_errors: usize,
    pub tags_used: Vec<String>,
    pub instructions: HashMap<String, usize>,
}

/// Result of analyzing a controller.
#[derive(Debug)]
pub struct ProjectAnalysis {
    pub rungs: Vec<LocatedRung>,
    pub st_routines: Vec<ParsedSTRoutine>,
    pub tag_references: Vec<LocatedTagReference>,
    pub tag_xref: HashMap<String, Vec<usize>>,
    pub routines: Vec<RoutineSummary>,
    pub instruction_usage: HashMap<String, usize>,
    pub aoi_definitions: Vec<String>,
    pub aoi_usage: HashMap<String, Vec<AoiReference>>,
    pub stats: ParseStats,
}

impl ProjectAnalysis {
    /// Get all references to a specific tag.
    pub fn references_to(&self, tag_name: &str) -> Vec<&LocatedTagReference> {
        if let Some(indices) = self.tag_xref.get(tag_name) {
            indices.iter().filter_map(|&i| self.tag_references.get(i)).collect()
        } else {
            Vec::new()
        }
    }

    /// Get unique tag names.
    pub fn unique_tags(&self) -> Vec<&str> {
        let mut tags: Vec<&str> = self.tag_xref.keys().map(|s| s.as_str()).collect();
        tags.sort();
        tags
    }

    /// Get routine summary by name.
    pub fn get_routine(&self, program: &str, routine: &str) -> Option<&RoutineSummary> {
        self.routines.iter().find(|r| r.program == program && r.routine == routine)
    }

    /// Get all RLL parse errors.
    pub fn parse_errors(&self) -> Vec<ParseError> {
        self.rungs.iter().filter_map(|rung| rung.parse_error()).collect()
    }

    /// Get unused AOIs.
    pub fn unused_aois(&self) -> Vec<&str> {
        self.aoi_definitions
            .iter()
            .filter(|name| {
                self.aoi_usage.get(*name).map(|refs| refs.is_empty()).unwrap_or(true)
            })
            .map(|s| s.as_str())
            .collect()
    }

    /// Get references to a specific AOI.
    pub fn aoi_references(&self, aoi_name: &str) -> Vec<&AoiReference> {
        self.aoi_usage
            .get(aoi_name)
            .map(|refs| refs.iter().collect())
            .unwrap_or_default()
    }
}

/// Parse all RLL logic from an AOI.
fn parse_rll_from_aoi(aoi: &UDIDefinition) -> Vec<LocatedRung> {
    let mut results = Vec::new();
    let aoi_name = format!("AOI:{}", aoi.name);

    for content in &aoi.content {
        if let UDIDefinitionContent::Routines(routine_collection) = content {
            for routine in &routine_collection.routine {
                results.extend(parse_routine(routine, &aoi_name));
            }
        }
    }

    results
}

/// Analyze a Controller and build cross-reference indices.
pub fn analyze_controller(controller: &Controller) -> ProjectAnalysis {
    let mut rungs = Vec::new();
    let mut routine_summaries = Vec::new();
    let mut stats = ParseStats::default();

    // Parse programs
    if let Some(programs) = &controller.programs {
        for program in &programs.program {
            stats.programs += 1;

            if let Some(routines) = &program.routines {
                for routine in &routines.routine {
                    stats.routines += 1;

                    let start_idx = rungs.len();
                    let routine_rungs = parse_routine(routine, &program.name);
                    let rung_count = routine_rungs.len();
                    stats.rll_rungs_programs += rung_count;

                    let rung_indices: Vec<usize> = (start_idx..start_idx + rung_count).collect();

                    let mut parse_errors = 0;
                    let mut routine_tags: Vec<String> = Vec::new();
                    let mut routine_instructions: HashMap<String, usize> = HashMap::new();

                    for rung in &routine_rungs {
                        if rung.parsed.is_parsed() {
                            for tag_ref in rung.tag_references() {
                                routine_tags.push(tag_ref.reference.name.clone());
                                *routine_instructions.entry(tag_ref.reference.instruction.clone()).or_insert(0) += 1;
                            }
                        } else {
                            parse_errors += 1;
                        }
                    }

                    routine_tags.sort();
                    routine_tags.dedup();

                    routine_summaries.push(RoutineSummary {
                        program: program.name.clone(),
                        routine: routine.name.clone(),
                        routine_type: routine.r#type.clone(),
                        rung_count,
                        rung_indices,
                        parse_errors,
                        tags_used: routine_tags,
                        instructions: routine_instructions,
                    });

                    rungs.extend(routine_rungs);
                }
            }
        }
    }

    // Parse AOIs and collect definitions
    let mut aoi_definitions: Vec<String> = Vec::new();
    if let Some(aois) = &controller.add_on_instruction_definitions {
        stats.aois = aois.add_on_instruction_definition.len();
        for aoi in &aois.add_on_instruction_definition {
            aoi_definitions.push(aoi.name.clone());
            let aoi_rungs = parse_rll_from_aoi(aoi);
            stats.rll_rungs_aois += aoi_rungs.len();
            rungs.extend(aoi_rungs);
        }
    }

    let aoi_name_set: HashSet<&str> = aoi_definitions.iter().map(|s| s.as_str()).collect();

    // Build tag cross-reference and track AOI usage
    stats.rungs = rungs.len();
    let mut all_refs = Vec::new();
    let mut tag_xref: HashMap<String, Vec<usize>> = HashMap::new();
    let mut instruction_usage: HashMap<String, usize> = HashMap::new();
    let mut aoi_usage: HashMap<String, Vec<AoiReference>> = HashMap::new();

    for aoi_name in &aoi_definitions {
        aoi_usage.insert(aoi_name.clone(), Vec::new());
    }

    for rung in &rungs {
        if rung.parsed.is_parsed() {
            stats.parsed_ok += 1;

            let mut rung_aoi_calls: HashSet<String> = HashSet::new();

            for tag_ref in rung.tag_references() {
                let ref_idx = all_refs.len();
                let tag_name = tag_ref.reference.name.clone();
                let instruction = tag_ref.reference.instruction.clone();

                if aoi_name_set.contains(instruction.as_str()) && !rung_aoi_calls.contains(&instruction) {
                    rung_aoi_calls.insert(instruction.clone());
                    aoi_usage
                        .entry(instruction.clone())
                        .or_default()
                        .push(AoiReference::from_rll(&instruction, &rung.location));
                }

                tag_xref.entry(tag_name).or_default().push(ref_idx);
                *instruction_usage.entry(instruction).or_insert(0) += 1;

                all_refs.push(tag_ref);
            }
        } else {
            stats.parsed_err += 1;
        }
    }

    // Parse ST routines
    let mut st_routines = Vec::new();
    if let Some(programs) = &controller.programs {
        for program in &programs.program {
            let program_st = parse_st_routines_from_program(program);
            stats.st_routines_programs += program_st.len();
            st_routines.extend(program_st);
        }
    }

    if let Some(aois) = &controller.add_on_instruction_definitions {
        for aoi in &aois.add_on_instruction_definition {
            let aoi_st = parse_st_routines_from_aoi(aoi);
            stats.st_routines_aois += aoi_st.len();
            st_routines.extend(aoi_st);
        }
    }

    // Track AOI usage from ST
    for st_routine in &st_routines {
        if let Some(ref pou) = st_routine.pou {
            let call_names = extract_st_call_names(pou);
            for call_name in call_names {
                if aoi_name_set.contains(call_name.as_str()) {
                    aoi_usage
                        .entry(call_name.clone())
                        .or_default()
                        .push(AoiReference::from_st(&call_name, &st_routine.location));
                }
            }
        }
    }

    stats.st_routines = st_routines.len();
    for st_routine in &st_routines {
        if st_routine.is_parsed() {
            stats.st_parsed_ok += 1;
        } else {
            stats.st_parsed_err += 1;
        }
    }

    stats.tag_references = all_refs.len();
    stats.unique_tags = tag_xref.len();

    ProjectAnalysis {
        rungs,
        st_routines,
        tag_references: all_refs,
        tag_xref,
        routines: routine_summaries,
        instruction_usage,
        aoi_definitions,
        aoi_usage,
        stats,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rung_location_path() {
        let loc = RungLocation::new("MainProgram", "MainRoutine", 5);
        assert_eq!(loc.path(), "MainProgram/MainRoutine/Rung#5");
    }

    #[test]
    fn test_st_location_path() {
        let loc = STLocation::new("MainProgram", "Logic");
        assert_eq!(loc.path(), "MainProgram/Logic");
    }
}
