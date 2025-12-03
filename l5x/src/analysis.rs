//! L5X + RLL + ST integration for project-wide analysis.
//!
//! This module provides utilities to walk an L5X project structure and
//! parse all RLL rungs and ST routines, extracting tag references for 
//! cross-reference analysis.
//!
//! # Architecture
//!
//! This module extracts a "project model" from L5X files. The model can be
//! consumed by analysis tools (smell detectors, cross-reference viewers, etc.)
//!
//! Future: This will feed into a common `plc-model` abstraction that can be
//! generated from multiple PLC file formats (L5X, TIA Portal, Codesys, etc.)

use crate::rll::{self, ErrorContext, ParseError, Rung as ParsedRung, TagReference};
use crate::{
    AProgram, Controller, Routine, RoutineContent, Rung, RungContent,
    RungCollection, TextWide, TextWideContent, STContent, STContentContent, STLine,
    UDIDefinition, UDIDefinitionContent,
};
use std::collections::HashMap;

// Re-export iecst types for convenience
pub use iecst::{
    Pou, Stmt, Expr, Diagnostic, DiagnosticKind, Severity,
    ParseError as STParseError, ParseErrorKind as STParseErrorKind,
    analyze_pou as analyze_st_pou,
};

/// Location of a rung within a project.
#[derive(Debug, Clone, PartialEq)]
pub struct RungLocation {
    /// Program name (or "MainProgram" for controller-level)
    pub program: String,
    /// Routine name
    pub routine: String,
    /// Rung number
    pub rung_number: u32,
}

impl RungLocation {
    /// Create a new rung location.
    pub fn new(program: impl Into<String>, routine: impl Into<String>, rung_number: u32) -> Self {
        Self {
            program: program.into(),
            routine: routine.into(),
            rung_number,
        }
    }

    /// Format as a path string: "Program/Routine/Rung#"
    pub fn path(&self) -> String {
        format!("{}/{}/Rung#{}", self.program, self.routine, self.rung_number)
    }
}

/// A parsed rung with its location in the project.
#[derive(Debug, Clone)]
pub struct LocatedRung {
    /// Where this rung is located
    pub location: RungLocation,
    /// The parsed rung content
    pub parsed: ParsedRung,
}

impl LocatedRung {
    /// Get tag references with location context.
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

    /// Get a formatted parse error with full context, if parsing failed.
    pub fn parse_error(&self) -> Option<ParseError> {
        self.parsed.error.as_ref().map(|err| {
            ParseError::new(err.clone(), &self.parsed.raw_text).with_context(ErrorContext::new(
                &self.location.program,
                &self.location.routine,
                self.location.rung_number,
            ))
        })
    }

    /// Check if this rung has a parse error.
    pub fn has_error(&self) -> bool {
        self.parsed.error.is_some()
    }
}

/// A tag reference with its location in the project.
#[derive(Debug, Clone)]
pub struct LocatedTagReference {
    /// Where this reference was found
    pub location: RungLocation,
    /// The tag reference details
    pub reference: TagReference,
}

impl LocatedTagReference {
    /// Get the base tag name.
    pub fn tag_name(&self) -> &str {
        &self.reference.name
    }

    /// Get the full operand text.
    pub fn full_operand(&self) -> &str {
        &self.reference.full_operand
    }

    /// Get the instruction that uses this tag.
    pub fn instruction(&self) -> &str {
        &self.reference.instruction
    }
}

// ============================================================================
// AOI Cross-Reference
// ============================================================================

/// Location where an AOI is called.
#[derive(Debug, Clone, PartialEq)]
pub struct AoiReference {
    /// The AOI name being called
    pub aoi_name: String,
    /// Program where the call is made (or "AOI:<name>" if called from another AOI)
    pub program: String,
    /// Routine where the call is made
    pub routine: String,
    /// Rung number (for RLL calls)
    pub rung_number: Option<u32>,
    /// Source type: RLL or ST
    pub source: AoiCallSource,
}

/// Source of an AOI call.
#[derive(Debug, Clone, PartialEq)]
pub enum AoiCallSource {
    /// Called from RLL (ladder logic)
    Rll,
    /// Called from ST (structured text)
    St,
}

impl AoiReference {
    /// Create a new AOI reference from RLL.
    pub fn from_rll(aoi_name: &str, location: &RungLocation) -> Self {
        Self {
            aoi_name: aoi_name.to_string(),
            program: location.program.clone(),
            routine: location.routine.clone(),
            rung_number: Some(location.rung_number),
            source: AoiCallSource::Rll,
        }
    }

    /// Create a new AOI reference from ST.
    pub fn from_st(aoi_name: &str, location: &STLocation) -> Self {
        Self {
            aoi_name: aoi_name.to_string(),
            program: location.program.clone(),
            routine: location.routine.clone(),
            rung_number: None,
            source: AoiCallSource::St,
        }
    }

    /// Format as a path string.
    pub fn path(&self) -> String {
        match self.rung_number {
            Some(n) => format!("{}/{}/Rung#{}", self.program, self.routine, n),
            None => format!("{}/{}", self.program, self.routine),
        }
    }
}

/// Extract the text content from a TextWide element.
pub fn extract_text_content(text_wide: &TextWide) -> Option<String> {
    for content in &text_wide.content {
        if let TextWideContent::TextContent(text) = content {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

/// Extract rung text from a Rung element.
pub fn extract_rung_text(rung: &Rung) -> Option<String> {
    for content in &rung.content {
        if let RungContent::Text(text_wide) = content {
            if let Some(text) = extract_text_content(text_wide) {
                return Some(text);
            }
        }
    }
    None
}

/// Parse all rungs from a RungCollection (RLLContent).
pub fn parse_rung_collection(
    rungs: &RungCollection,
    program: &str,
    routine: &str,
) -> Vec<LocatedRung> {
    let mut results = Vec::new();

    for rung in &rungs.rung {
        let rung_number: u32 = rung
            .number
            .as_ref()
            .and_then(|n| n.parse().ok())
            .unwrap_or(0);

        if let Some(text) = extract_rung_text(rung) {
            let parsed = rll::parse_rung(&text);
            results.push(LocatedRung {
                location: RungLocation::new(program, routine, rung_number),
                parsed,
            });
        }
    }

    results
}

/// Parse all rungs from a Routine.
pub fn parse_routine(routine: &Routine, program: &str) -> Vec<LocatedRung> {
    let mut results = Vec::new();

    // Only process RLL routines
    if routine.r#type != "RLL" {
        return results;
    }

    for content in &routine.content {
        if let RoutineContent::RLLContent(rung_collection) = content {
            results.extend(parse_rung_collection(rung_collection, program, &routine.name));
        }
    }

    results
}

/// Parse all rungs from a Program.
pub fn parse_program(program: &AProgram) -> Vec<LocatedRung> {
    let mut results = Vec::new();

    if let Some(routines) = &program.routines {
        for routine in &routines.routine {
            results.extend(parse_routine(routine, &program.name));
        }
    }

    results
}

// ============================================================================
// Structured Text (ST) Parsing
// ============================================================================

/// Extract all function/FB call names from an ST POU.
/// 
/// This walks the AST and collects all function call names, which can then
/// be matched against AOI definitions for cross-reference.
pub fn extract_st_call_names(pou: &Pou) -> Vec<String> {
    let mut calls = Vec::new();
    
    fn visit_expr(expr: &Expr, calls: &mut Vec<String>) {
        use iecst::ExprKind;
        match &expr.kind {
            ExprKind::FunctionCall { name, args } => {
                calls.push(name.clone());
                for arg in args {
                    if let Some(ref value) = arg.value {
                        visit_expr(value, calls);
                    }
                }
            }
            ExprKind::BinaryOp { left, right, .. } => {
                visit_expr(left, calls);
                visit_expr(right, calls);
            }
            ExprKind::UnaryOp { expr, .. } => {
                visit_expr(expr, calls);
            }
            ExprKind::ArrayIndex { array, indices } => {
                visit_expr(array, calls);
                for idx in indices {
                    visit_expr(idx, calls);
                }
            }
            ExprKind::MemberAccess { expr, .. } => {
                visit_expr(expr, calls);
            }
            ExprKind::Paren(inner) => {
                visit_expr(inner, calls);
            }
            _ => {}
        }
    }
    
    fn visit_stmt(stmt: &iecst::Stmt, calls: &mut Vec<String>) {
        use iecst::StmtKind;
        match &stmt.kind {
            StmtKind::Assignment { target, value } => {
                visit_expr(target, calls);
                visit_expr(value, calls);
            }
            StmtKind::If { condition, then_body, elsif_branches, else_body } => {
                visit_expr(condition, calls);
                for s in then_body {
                    visit_stmt(s, calls);
                }
                for (cond, body) in elsif_branches {
                    visit_expr(cond, calls);
                    for s in body {
                        visit_stmt(s, calls);
                    }
                }
                if let Some(else_stmts) = else_body {
                    for s in else_stmts {
                        visit_stmt(s, calls);
                    }
                }
            }
            StmtKind::Case { expr, cases, else_body } => {
                visit_expr(expr, calls);
                for case in cases {
                    for s in &case.body {
                        visit_stmt(s, calls);
                    }
                }
                if let Some(else_stmts) = else_body {
                    for s in else_stmts {
                        visit_stmt(s, calls);
                    }
                }
            }
            StmtKind::For { from, to, by, body, .. } => {
                visit_expr(from, calls);
                visit_expr(to, calls);
                if let Some(by_expr) = by {
                    visit_expr(by_expr, calls);
                }
                for s in body {
                    visit_stmt(s, calls);
                }
            }
            StmtKind::While { condition, body } => {
                visit_expr(condition, calls);
                for s in body {
                    visit_stmt(s, calls);
                }
            }
            StmtKind::Repeat { body, until } => {
                for s in body {
                    visit_stmt(s, calls);
                }
                visit_expr(until, calls);
            }
            StmtKind::Return { value } => {
                if let Some(v) = value {
                    visit_expr(v, calls);
                }
            }
            StmtKind::Call { name, args } => {
                calls.push(name.clone());
                for arg in args {
                    if let Some(ref value) = arg.value {
                        visit_expr(value, calls);
                    }
                }
            }
            _ => {}
        }
    }
    
    for stmt in &pou.body {
        visit_stmt(stmt, &mut calls);
    }
    
    // Dedupe calls (same function may be called multiple times)
    calls.sort();
    calls.dedup();
    calls
}

/// Location of an ST routine within a project.
#[derive(Debug, Clone, PartialEq)]
pub struct STLocation {
    /// Program name (or AOI name for Add-On Instructions)
    pub program: String,
    /// Routine name
    pub routine: String,
}

impl STLocation {
    /// Create a new ST location.
    pub fn new(program: impl Into<String>, routine: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            routine: routine.into(),
        }
    }

    /// Format as a path string: "Program/Routine"
    pub fn path(&self) -> String {
        format!("{}/{}", self.program, self.routine)
    }
}

/// A parsed ST routine with its location and diagnostics.
#[derive(Debug)]
pub struct ParsedSTRoutine {
    /// Where this routine is located
    pub location: STLocation,
    /// The raw ST source code
    pub source: String,
    /// The parsed POU (if successful)
    pub pou: Option<Pou>,
    /// Parse error (if failed)
    pub parse_error: Option<iecst::ParseError>,
    /// Semantic analysis diagnostics
    pub diagnostics: Vec<Diagnostic>,
}

impl ParsedSTRoutine {
    /// Check if parsing was successful.
    pub fn is_parsed(&self) -> bool {
        self.pou.is_some()
    }

    /// Check if there are any errors (parse or semantic).
    pub fn has_errors(&self) -> bool {
        self.parse_error.is_some() 
            || self.diagnostics.iter().any(|d| matches!(d.severity, Severity::Error))
    }

    /// Get all error-level diagnostics.
    pub fn errors(&self) -> Vec<&Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| matches!(d.severity, Severity::Error))
            .collect()
    }

    /// Get all warning-level diagnostics.
    pub fn warnings(&self) -> Vec<&Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| matches!(d.severity, Severity::Warning))
            .collect()
    }
}

/// Extract the ST source code from an STLine element.
fn extract_st_line_text(line: &STLine) -> String {
    // First try the text content (CDATA)
    if let Some(text) = &line.text {
        return text.clone();
    }
    // Fall back to Value element if present
    if let Some(value) = &line.value {
        return value.clone();
    }
    String::new()
}

/// Extract all ST source code from an STContent element.
pub fn extract_st_source(st_content: &STContent) -> String {
    let mut lines: Vec<(u32, String)> = Vec::new();

    for content in &st_content.content {
        match content {
            STContentContent::Line(line) => {
                let line_num: u32 = line.number
                    .as_ref()
                    .and_then(|n| n.parse().ok())
                    .unwrap_or(0);
                let text = extract_st_line_text(line);
                lines.push((line_num, text));
            }
            STContentContent::TextContent(text) => {
                // Handle raw text content if present
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    lines.push((0, trimmed.to_string()));
                }
            }
            _ => {}
        }
    }

    // Sort by line number and join
    lines.sort_by_key(|(n, _)| *n);
    lines.into_iter().map(|(_, text)| text).collect::<Vec<_>>().join("\n")
}

/// Parse an ST routine from a Routine element.
pub fn parse_st_routine(routine: &Routine, program: &str) -> Option<ParsedSTRoutine> {
    // Only process ST routines
    if routine.r#type != "ST" {
        return None;
    }

    // Find STContent
    let st_content = routine.content.iter().find_map(|c| {
        if let RoutineContent::STContent(st) = c {
            Some(st)
        } else {
            None
        }
    })?;

    let source = extract_st_source(st_content);
    let location = STLocation::new(program, &routine.name);

    // Try to parse as statements (ST routines are typically just statements, not full POUs)
    // We'll wrap it in a synthetic PROGRAM for analysis
    let wrapped_source = format!(
        "PROGRAM {}\nVAR\nEND_VAR\n{}\nEND_PROGRAM",
        routine.name, source
    );

    match iecst::parse_pou(&wrapped_source) {
        Ok(pou) => {
            // Run semantic analysis
            let diagnostics = analyze_st_pou(&pou);
            Some(ParsedSTRoutine {
                location,
                source,
                pou: Some(pou),
                parse_error: None,
                diagnostics,
            })
        }
        Err(e) => {
            Some(ParsedSTRoutine {
                location,
                source,
                pou: None,
                parse_error: Some(e),
                diagnostics: Vec::new(),
            })
        }
    }
}

/// Parse all ST routines from a Program.
pub fn parse_st_routines_from_program(program: &AProgram) -> Vec<ParsedSTRoutine> {
    let mut results = Vec::new();

    if let Some(routines) = &program.routines {
        for routine in &routines.routine {
            if let Some(parsed) = parse_st_routine(routine, &program.name) {
                results.push(parsed);
            }
        }
    }

    results
}

/// Parse all RLL rungs from an Add-On Instruction (AOI).
pub fn parse_rll_from_aoi(aoi: &UDIDefinition) -> Vec<LocatedRung> {
    let mut results = Vec::new();
    let aoi_name = format!("AOI:{}", aoi.name);

    // Find the Routines element in the AOI content
    for content in &aoi.content {
        if let UDIDefinitionContent::Routines(routine_collection) = content {
            for routine in &routine_collection.routine {
                results.extend(parse_routine(routine, &aoi_name));
            }
        }
    }

    results
}

/// Parse all ST routines from an Add-On Instruction (AOI).
pub fn parse_st_routines_from_aoi(aoi: &UDIDefinition) -> Vec<ParsedSTRoutine> {
    let mut results = Vec::new();

    // Find the Routines element in the AOI content
    for content in &aoi.content {
        if let UDIDefinitionContent::Routines(routine_collection) = content {
            for routine in &routine_collection.routine {
                // Use AOI name as the "program" name for location
                let aoi_name = format!("AOI:{}", aoi.name);
                if let Some(parsed) = parse_st_routine(routine, &aoi_name) {
                    results.push(parsed);
                }
            }
        }
    }

    results
}

/// Parse all ST routines from a Controller (Programs + AOIs).
pub fn parse_st_routines(controller: &Controller) -> Vec<ParsedSTRoutine> {
    let mut results = Vec::new();

    // Parse ST routines from Programs
    if let Some(programs) = &controller.programs {
        for program in &programs.program {
            results.extend(parse_st_routines_from_program(program));
        }
    }

    // Parse ST routines from Add-On Instructions (AOIs)
    if let Some(aois) = &controller.add_on_instruction_definitions {
        for aoi in &aois.add_on_instruction_definition {
            results.extend(parse_st_routines_from_aoi(aoi));
        }
    }

    results
}

/// Statistics from parsing a project.
#[derive(Debug, Clone, Default)]
pub struct ParseStats {
    /// Total programs scanned
    pub programs: usize,
    /// Total Add-On Instructions (AOIs) scanned
    pub aois: usize,
    /// Total routines scanned (across programs and AOIs)
    pub routines: usize,
    /// Total rungs found (RLL)
    pub rungs: usize,
    /// RLL rungs from programs
    pub rll_rungs_programs: usize,
    /// RLL rungs from AOIs
    pub rll_rungs_aois: usize,
    /// Successfully parsed rungs (RLL)
    pub parsed_ok: usize,
    /// Rungs with parse errors (RLL)
    pub parsed_err: usize,
    /// Total tag references found
    pub tag_references: usize,
    /// Unique base tag names found
    pub unique_tags: usize,
    /// Total ST routines found (programs + AOIs)
    pub st_routines: usize,
    /// ST routines from programs
    pub st_routines_programs: usize,
    /// ST routines from AOIs
    pub st_routines_aois: usize,
    /// Successfully parsed ST routines
    pub st_parsed_ok: usize,
    /// ST routines with parse errors
    pub st_parsed_err: usize,
    /// Total ST diagnostics (errors + warnings)
    pub st_diagnostics: usize,
}

/// Result of parsing all RLL and ST in a controller.
#[derive(Debug)]
pub struct ProjectAnalysis {
    /// All parsed rungs with locations (RLL)
    pub rungs: Vec<LocatedRung>,
    /// All parsed ST routines
    pub st_routines: Vec<ParsedSTRoutine>,
    /// All tag references with locations
    pub tag_references: Vec<LocatedTagReference>,
    /// Cross-reference index: tag name → list of references
    pub tag_xref: HashMap<String, Vec<usize>>,
    /// Routine summaries
    pub routines: Vec<RoutineSummary>,
    /// Instruction usage: mnemonic → count
    pub instruction_usage: HashMap<String, usize>,
    /// Defined AOI names in this controller
    pub aoi_definitions: Vec<String>,
    /// AOI usage cross-reference: AOI name → list of call locations
    pub aoi_usage: HashMap<String, Vec<AoiReference>>,
    /// Parsing statistics
    pub stats: ParseStats,
}

/// Summary of a single routine.
#[derive(Debug, Clone)]
pub struct RoutineSummary {
    /// Program name
    pub program: String,
    /// Routine name
    pub routine: String,
    /// Routine type (RLL, ST, FBD, SFC)
    pub routine_type: String,
    /// Number of rungs (for RLL routines)
    pub rung_count: usize,
    /// Indices into ProjectAnalysis.rungs for this routine's rungs
    pub rung_indices: Vec<usize>,
    /// Parse error count
    pub parse_errors: usize,
    /// Unique tags used in this routine
    pub tags_used: Vec<String>,
    /// Instructions used in this routine (mnemonic → count)
    pub instructions: HashMap<String, usize>,
}

impl ProjectAnalysis {
    /// Get all references to a specific tag (by base name).
    pub fn references_to(&self, tag_name: &str) -> Vec<&LocatedTagReference> {
        if let Some(indices) = self.tag_xref.get(tag_name) {
            indices
                .iter()
                .filter_map(|&i| self.tag_references.get(i))
                .collect()
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

    /// Get tags grouped by instruction.
    pub fn tags_by_instruction(&self) -> HashMap<String, Vec<String>> {
        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        for r in &self.tag_references {
            map.entry(r.reference.instruction.clone())
                .or_default()
                .push(r.reference.name.clone());
        }
        // Dedupe each list
        for tags in map.values_mut() {
            tags.sort();
            tags.dedup();
        }
        map
    }

    /// Get routine summary by name.
    pub fn get_routine(&self, program: &str, routine: &str) -> Option<&RoutineSummary> {
        self.routines
            .iter()
            .find(|r| r.program == program && r.routine == routine)
    }

    /// Get all routines in a program.
    pub fn routines_in_program(&self, program: &str) -> Vec<&RoutineSummary> {
        self.routines
            .iter()
            .filter(|r| r.program == program)
            .collect()
    }

    /// Get program names.
    pub fn program_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self
            .routines
            .iter()
            .map(|r| r.program.as_str())
            .collect();
        names.sort();
        names.dedup();
        names
    }

    /// Get top N most used instructions.
    pub fn top_instructions(&self, n: usize) -> Vec<(&str, usize)> {
        let mut sorted: Vec<_> = self
            .instruction_usage
            .iter()
            .map(|(k, v)| (k.as_str(), *v))
            .collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.truncate(n);
        sorted
    }

    /// Get all RLL parse errors with full context.
    pub fn parse_errors(&self) -> Vec<ParseError> {
        self.rungs
            .iter()
            .filter_map(|rung| rung.parse_error())
            .collect()
    }

    /// Get all parse errors formatted as strings.
    pub fn format_parse_errors(&self) -> Vec<String> {
        self.parse_errors().iter().map(|e| e.format()).collect()
    }

    /// Print all parse errors to stderr.
    pub fn print_parse_errors(&self) {
        for error in self.parse_errors() {
            eprintln!("{}\n", error.format());
        }
    }

    /// Get all ST routines with parse errors.
    pub fn st_parse_errors(&self) -> Vec<&ParsedSTRoutine> {
        self.st_routines
            .iter()
            .filter(|r| r.parse_error.is_some())
            .collect()
    }

    /// Get all ST diagnostics across all routines.
    pub fn st_all_diagnostics(&self) -> Vec<(&ParsedSTRoutine, &Diagnostic)> {
        self.st_routines
            .iter()
            .flat_map(|r| r.diagnostics.iter().map(move |d| (r, d)))
            .collect()
    }

    /// Get ST routines with errors (parse or semantic).
    pub fn st_routines_with_errors(&self) -> Vec<&ParsedSTRoutine> {
        self.st_routines
            .iter()
            .filter(|r| r.has_errors())
            .collect()
    }

    /// Get an ST routine by program and routine name.
    pub fn get_st_routine(&self, program: &str, routine: &str) -> Option<&ParsedSTRoutine> {
        self.st_routines
            .iter()
            .find(|r| r.location.program == program && r.location.routine == routine)
    }

    /// Print summary of ST analysis to stdout.
    pub fn print_st_summary(&self) {
        println!("=== ST Analysis Summary ===");
        println!("ST Routines: {} total, {} parsed OK, {} with errors",
            self.stats.st_routines, self.stats.st_parsed_ok, self.stats.st_parsed_err);
        println!("Diagnostics: {}", self.stats.st_diagnostics);
        
        for routine in &self.st_routines {
            let status = if routine.is_parsed() { "OK" } else { "ERROR" };
            let diag_count = routine.diagnostics.len();
            println!("  {} - {} (diagnostics: {})", routine.location.path(), status, diag_count);
            
            if let Some(ref err) = routine.parse_error {
                println!("    Parse error: {:?} at {:?}", err.kind, err.span);
            }
            
            for diag in &routine.diagnostics {
                println!("    {:?}: {}", diag.severity, diag.kind);
            }
        }
    }

    // ========================================================================
    // AOI Cross-Reference Methods
    // ========================================================================

    /// Get all references to a specific AOI.
    pub fn aoi_references(&self, aoi_name: &str) -> Vec<&AoiReference> {
        self.aoi_usage
            .get(aoi_name)
            .map(|refs| refs.iter().collect())
            .unwrap_or_default()
    }

    /// Get AOIs that are defined but never used.
    pub fn unused_aois(&self) -> Vec<&str> {
        self.aoi_definitions
            .iter()
            .filter(|name| {
                self.aoi_usage
                    .get(*name)
                    .map(|refs| refs.is_empty())
                    .unwrap_or(true)
            })
            .map(|s| s.as_str())
            .collect()
    }

    /// Get AOIs sorted by usage count (most used first).
    pub fn aois_by_usage(&self) -> Vec<(&str, usize)> {
        let mut usage: Vec<_> = self.aoi_definitions
            .iter()
            .map(|name| {
                let count = self.aoi_usage
                    .get(name)
                    .map(|refs| refs.len())
                    .unwrap_or(0);
                (name.as_str(), count)
            })
            .collect();
        usage.sort_by(|a, b| b.1.cmp(&a.1));
        usage
    }

    /// Get programs that use a specific AOI.
    pub fn programs_using_aoi(&self, aoi_name: &str) -> Vec<&str> {
        let mut programs: Vec<&str> = self.aoi_usage
            .get(aoi_name)
            .map(|refs| {
                refs.iter()
                    .filter(|r| !r.program.starts_with("AOI:"))
                    .map(|r| r.program.as_str())
                    .collect()
            })
            .unwrap_or_default();
        programs.sort();
        programs.dedup();
        programs
    }

    /// Get AOIs that call other AOIs (nested AOI usage).
    pub fn aoi_calls_aoi(&self) -> Vec<(&str, &str)> {
        let mut calls = Vec::new();
        for (aoi_name, refs) in &self.aoi_usage {
            for r in refs {
                if r.program.starts_with("AOI:") {
                    let caller_aoi = r.program.strip_prefix("AOI:").unwrap_or(&r.program);
                    calls.push((caller_aoi, aoi_name.as_str()));
                }
            }
        }
        calls.sort();
        calls.dedup();
        calls
    }

    /// Print AOI cross-reference summary.
    pub fn print_aoi_summary(&self) {
        println!("=== AOI Cross-Reference Summary ===");
        println!("Defined AOIs: {}", self.aoi_definitions.len());
        
        let unused = self.unused_aois();
        if !unused.is_empty() {
            println!("\nUnused AOIs ({}):", unused.len());
            for aoi in &unused {
                println!("  - {}", aoi);
            }
        }

        println!("\nAOI Usage:");
        for (aoi, count) in self.aois_by_usage() {
            if count > 0 {
                println!("  {} - {} calls", aoi, count);
                for r in self.aoi_references(aoi) {
                    println!("    - {}", r.path());
                }
            }
        }

        let nested = self.aoi_calls_aoi();
        if !nested.is_empty() {
            println!("\nNested AOI Calls:");
            for (caller, callee) in &nested {
                println!("  {} -> {}", caller, callee);
            }
        }
    }
}

/// Parse all RLL logic from a Controller.
pub fn analyze_controller(controller: &Controller) -> ProjectAnalysis {
    let mut rungs = Vec::new();
    let mut routine_summaries = Vec::new();
    let mut stats = ParseStats::default();

    // Parse controller-level programs
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
                    
                    // Track rung indices for this routine
                    let rung_indices: Vec<usize> = (start_idx..start_idx + rung_count).collect();
                    
                    // Count parse errors and collect instruction usage
                    let mut parse_errors = 0;
                    let mut routine_tags: Vec<String> = Vec::new();
                    let mut routine_instructions: HashMap<String, usize> = HashMap::new();
                    
                    for rung in &routine_rungs {
                        if rung.parsed.is_parsed() {
                            // Collect tags
                            for tag_ref in rung.tag_references() {
                                routine_tags.push(tag_ref.reference.name.clone());
                                *routine_instructions.entry(tag_ref.reference.instruction.clone()).or_insert(0) += 1;
                            }
                        } else {
                            parse_errors += 1;
                        }
                    }
                    
                    // Dedupe tags
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

    // Parse RLL and ST from AOIs
    // Also collect AOI definitions
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

    // Build a set of AOI names for fast lookup
    let aoi_name_set: std::collections::HashSet<&str> = 
        aoi_definitions.iter().map(|s| s.as_str()).collect();

    // Collect all tag references and track AOI usage from RLL
    stats.rungs = rungs.len();
    let mut all_refs = Vec::new();
    let mut tag_xref: HashMap<String, Vec<usize>> = HashMap::new();
    let mut instruction_usage: HashMap<String, usize> = HashMap::new();
    let mut aoi_usage: HashMap<String, Vec<AoiReference>> = HashMap::new();

    // Initialize aoi_usage with empty vectors for all defined AOIs
    for aoi_name in &aoi_definitions {
        aoi_usage.insert(aoi_name.clone(), Vec::new());
    }

    for rung in &rungs {
        if rung.parsed.is_parsed() {
            stats.parsed_ok += 1;
            
            // Track AOI calls for this rung (to avoid duplicates)
            let mut rung_aoi_calls: std::collections::HashSet<String> = std::collections::HashSet::new();
            
            for tag_ref in rung.tag_references() {
                let ref_idx = all_refs.len();
                let tag_name = tag_ref.reference.name.clone();
                let instruction = tag_ref.reference.instruction.clone();
                
                // Check if this instruction is an AOI call (only add once per rung)
                if aoi_name_set.contains(instruction.as_str()) && !rung_aoi_calls.contains(&instruction) {
                    rung_aoi_calls.insert(instruction.clone());
                    aoi_usage
                        .entry(instruction.clone())
                        .or_default()
                        .push(AoiReference::from_rll(&instruction, &rung.location));
                }
                
                // Build cross-reference index
                tag_xref.entry(tag_name).or_default().push(ref_idx);
                
                // Track instruction usage
                *instruction_usage.entry(instruction).or_insert(0) += 1;
                
                all_refs.push(tag_ref);
            }
        } else {
            stats.parsed_err += 1;
        }
    }

    // Parse ST routines from Programs
    let mut st_routines = Vec::new();
    if let Some(programs) = &controller.programs {
        for program in &programs.program {
            let program_st = parse_st_routines_from_program(program);
            stats.st_routines_programs += program_st.len();
            st_routines.extend(program_st);
        }
    }

    // Parse ST routines from AOIs
    if let Some(aois) = &controller.add_on_instruction_definitions {
        for aoi in &aois.add_on_instruction_definition {
            let aoi_st = parse_st_routines_from_aoi(aoi);
            stats.st_routines_aois += aoi_st.len();
            st_routines.extend(aoi_st);
        }
    }

    // Track AOI usage from ST routines
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
        stats.st_diagnostics += st_routine.diagnostics.len();
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

    #[test]
    fn test_extract_st_source_basic() {
        // Simulate STContent with lines
        let st_content = STContent {
            r#use: None,
            start: None,
            count: None,
            online_edit_type: None,
            uid: None,
            content: vec![
                STContentContent::Line(STLine {
                    number: Some("0".to_string()),
                    uid: None,
                    metadata_id: None,
                    r#use: None,
                    custom_properties: None,
                    value: None,
                    text: Some("x := 1;".to_string()),
                }),
                STContentContent::Line(STLine {
                    number: Some("1".to_string()),
                    uid: None,
                    metadata_id: None,
                    r#use: None,
                    custom_properties: None,
                    value: None,
                    text: Some("y := x + 2;".to_string()),
                }),
            ],
        };

        let source = extract_st_source(&st_content);
        assert!(source.contains("x := 1;"));
        assert!(source.contains("y := x + 2;"));
    }
}
