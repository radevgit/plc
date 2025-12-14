//! PLCopen project analysis.
//!
//! This module provides analysis for PLCopen TC6 XML files,
//! extracting variables, code references, and building cross-reference indices.

use std::collections::{HashMap, HashSet};

use plcopen::{
    Project,
    Body,
    FormattedText,
    Root_project_InlineType_types_InlineType_pous_InlineType_pou_Inline as Pou,
    VarListPlain_variable_Inline as Variable,
};

/// Statistics from parsing a PLCopen project.
#[derive(Debug, Clone, Default)]
pub struct PlcopenStats {
    pub pous: usize,
    pub programs: usize,
    pub function_blocks: usize,
    pub functions: usize,
    pub variables: usize,
    pub st_bodies: usize,
    pub fbd_bodies: usize,
    pub ld_bodies: usize,
    pub sfc_bodies: usize,
    pub il_bodies: usize,
    pub empty_pous: usize,
}

/// A variable definition with its scope.
#[derive(Debug, Clone)]
pub struct VariableDef {
    pub name: String,
    pub pou_name: String,
    pub var_class: VarClass,
    pub data_type: Option<String>,
}

/// Variable class/scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarClass {
    Input,
    Output,
    InOut,
    Local,
    Temp,
    External,
    Global,
}

/// Analysis results for a PLCopen project.
#[derive(Debug)]
pub struct PlcopenAnalysis {
    /// All defined variables by name
    pub defined_variables: HashMap<String, VariableDef>,
    
    /// Set of all defined variable names (for quick lookup)
    pub defined_var_names: HashSet<String>,
    
    /// Variables referenced in code
    pub used_variables: HashSet<String>,
    
    /// POUs that are called/instantiated
    pub used_pous: HashSet<String>,
    
    /// POUs with empty bodies
    pub empty_pous: Vec<String>,
    
    /// All POU names
    pub pou_names: HashSet<String>,
    
    /// Statistics
    pub stats: PlcopenStats,
}

impl PlcopenAnalysis {
    /// Get unused variables (defined but not used).
    pub fn unused_variables(&self) -> Vec<&VariableDef> {
        self.defined_variables
            .values()
            .filter(|v| !self.used_variables.contains(&v.name))
            .collect()
    }
    
    /// Get undefined variables (used but not defined).
    pub fn undefined_variables(&self) -> Vec<&String> {
        self.used_variables
            .iter()
            .filter(|v| !self.defined_var_names.contains(*v) && !is_builtin(v))
            .collect()
    }
}

/// Analyze a PLCopen project.
pub fn analyze_project(project: &Project) -> PlcopenAnalysis {
    let mut analysis = PlcopenAnalysis {
        defined_variables: HashMap::new(),
        defined_var_names: HashSet::new(),
        used_variables: HashSet::new(),
        used_pous: HashSet::new(),
        empty_pous: Vec::new(),
        pou_names: HashSet::new(),
        stats: PlcopenStats::default(),
    };
    
    // Get POUs from types section
    if let Some(ref types) = project.types {
        if let Some(ref pous) = types.pous {
            for pou in &pous.pou {
                analyze_pou(pou, &mut analysis);
            }
        }
    }
    
    analysis
}

fn analyze_pou(pou: &Pou, analysis: &mut PlcopenAnalysis) {
    analysis.stats.pous += 1;
    analysis.pou_names.insert(pou.name.clone());
    
    // Count by type
    match pou.pou_type.to_lowercase().as_str() {
        "program" => analysis.stats.programs += 1,
        "functionblock" => analysis.stats.function_blocks += 1,
        "function" => analysis.stats.functions += 1,
        _ => {}
    }
    
    // Collect variables from interface
    if let Some(ref interface) = pou.interface {
        // Input variables
        for var_list in &interface.input_vars {
            for var in &var_list.variable {
                add_variable(var, &pou.name, VarClass::Input, analysis);
            }
        }
        
        // Output variables
        for var_list in &interface.output_vars {
            for var in &var_list.variable {
                add_variable(var, &pou.name, VarClass::Output, analysis);
            }
        }
        
        // InOut variables
        for var_list in &interface.in_out_vars {
            for var in &var_list.variable {
                add_variable(var, &pou.name, VarClass::InOut, analysis);
            }
        }
        
        // Local variables
        for var_list in &interface.local_vars {
            for var in &var_list.variable {
                add_variable(var, &pou.name, VarClass::Local, analysis);
            }
        }
        
        // Temp variables
        for var_list in &interface.temp_vars {
            for var in &var_list.variable {
                add_variable(var, &pou.name, VarClass::Temp, analysis);
            }
        }
        
        // External variables
        for var_list in &interface.external_vars {
            for var in &var_list.variable {
                add_variable(var, &pou.name, VarClass::External, analysis);
            }
        }
        
        // Global variables
        for var_list in &interface.global_vars {
            for var in &var_list.variable {
                add_variable(var, &pou.name, VarClass::Global, analysis);
            }
        }
    }
    
    // Analyze body
    let has_code = analyze_bodies(&pou.body, &pou.name, analysis);
    
    if !has_code {
        analysis.empty_pous.push(pou.name.clone());
        analysis.stats.empty_pous += 1;
    }
}

fn add_variable(var: &Variable, pou_name: &str, var_class: VarClass, analysis: &mut PlcopenAnalysis) {
    analysis.stats.variables += 1;
    
    let data_type = var.r#type.as_ref().and_then(|t| extract_type_name(t.as_ref()));
    
    let def = VariableDef {
        name: var.name.clone(),
        pou_name: pou_name.to_string(),
        var_class,
        data_type,
    };
    
    analysis.defined_var_names.insert(var.name.clone());
    analysis.defined_variables.insert(var.name.clone(), def);
}

fn analyze_bodies(bodies: &[Box<Body>], _pou_name: &str, analysis: &mut PlcopenAnalysis) -> bool {
    let mut has_code = false;
    
    for body in bodies {
        // ST body
        if let Some(ref st) = body.st {
            analysis.stats.st_bodies += 1;
            if let Some(text) = extract_formatted_text(st) {
                if !text.trim().is_empty() {
                    has_code = true;
                    extract_references_from_st(&text, analysis);
                }
            }
        }
        
        // IL body
        if let Some(ref il) = body.il {
            analysis.stats.il_bodies += 1;
            if let Some(text) = extract_formatted_text(il) {
                if !text.trim().is_empty() {
                    has_code = true;
                    extract_references_from_il(&text, analysis);
                }
            }
        }
        
        // FBD body
        if let Some(ref fbd) = body.fbd {
            analysis.stats.fbd_bodies += 1;
            has_code = true;
            extract_references_from_fbd(fbd, analysis);
        }
        
        // LD body
        if let Some(ref ld) = body.ld {
            analysis.stats.ld_bodies += 1;
            has_code = true;
            extract_references_from_ld(ld, analysis);
        }
        
        // SFC body
        if let Some(ref sfc) = body.sfc {
            analysis.stats.sfc_bodies += 1;
            has_code = true;
            extract_references_from_sfc(sfc, analysis);
        }
    }
    
    has_code
}

fn extract_formatted_text(ft: &FormattedText) -> Option<String> {
    // FormattedText now has a text field that captures the content
    ft.text.clone()
}

fn extract_references_from_st(code: &str, analysis: &mut PlcopenAnalysis) {
    // Simple extraction: find identifiers that could be variables
    for word in code.split(|c: char| !c.is_alphanumeric() && c != '_') {
        let word = word.trim();
        if !word.is_empty() 
            && word.chars().next().map(|c| c.is_alphabetic()).unwrap_or(false)
            && !is_st_keyword(word)
        {
            // Could be a variable or POU call
            analysis.used_variables.insert(word.to_string());
        }
    }
}

fn extract_references_from_il(code: &str, analysis: &mut PlcopenAnalysis) {
    // IL format: OPCODE OPERAND
    for line in code.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let operand = parts[1];
            if operand.chars().next().map(|c| c.is_alphabetic()).unwrap_or(false) {
                analysis.used_variables.insert(operand.to_string());
            }
        }
    }
}

fn extract_references_from_fbd(fbd: &plcopen::Body_FBD_Inline, analysis: &mut PlcopenAnalysis) {
    // Extract from blocks (function blocks, function calls)
    for block in &fbd.block {
        // typeName is the function block or function being called
        let type_name = block.type_name.trim();
        if !type_name.is_empty() && !is_builtin(type_name) {
            analysis.used_pous.insert(type_name.to_string());
        }
        
        // instanceName is the FB instance variable (optional for functions)
        if let Some(ref instance_name) = block.instance_name {
            let name = instance_name.trim();
            if !name.is_empty() {
                analysis.used_variables.insert(name.to_string());
            }
        }
    }
    
    // Extract from inVariable elements (input variables)
    for in_var in &fbd.in_variable {
        if let Some(ref expression) = in_var.expression {
            let expr = expression.trim();
            if !expr.is_empty() && is_identifier(expr) {
                analysis.used_variables.insert(expr.to_string());
            }
        }
    }
    
    // Extract from outVariable elements (output variables)
    for out_var in &fbd.out_variable {
        if let Some(ref expression) = out_var.expression {
            let expr = expression.trim();
            if !expr.is_empty() && is_identifier(expr) {
                analysis.used_variables.insert(expr.to_string());
            }
        }
    }
    
    // Extract from inOutVariable elements (in/out variables)
    for inout_var in &fbd.in_out_variable {
        if let Some(ref expression) = inout_var.expression {
            let expr = expression.trim();
            if !expr.is_empty() && is_identifier(expr) {
                analysis.used_variables.insert(expr.to_string());
            }
        }
    }
    
    // Extract from label/jump (for SFC-style control flow in FBD)
    for label in &fbd.label {
        let name = label.label.trim();
        if !name.is_empty() {
            analysis.used_variables.insert(name.to_string());
        }
    }
    
    for jump in &fbd.jump {
        let name = jump.label.trim();
        if !name.is_empty() {
            analysis.used_variables.insert(name.to_string());
        }
    }
}

fn extract_references_from_ld(ld: &plcopen::Body_LD_Inline, analysis: &mut PlcopenAnalysis) {
    // LD shares many elements with FBD, plus ladder-specific coils and contacts
    
    // Extract from blocks (same as FBD)
    for block in &ld.block {
        let type_name = block.type_name.trim();
        if !type_name.is_empty() && !is_builtin(type_name) {
            analysis.used_pous.insert(type_name.to_string());
        }
        if let Some(ref instance_name) = block.instance_name {
            let name = instance_name.trim();
            if !name.is_empty() {
                analysis.used_variables.insert(name.to_string());
            }
        }
    }
    
    // Extract from variables (same as FBD)
    for in_var in &ld.in_variable {
        if let Some(ref expression) = in_var.expression {
            let expr = expression.trim();
            if !expr.is_empty() && is_identifier(expr) {
                analysis.used_variables.insert(expr.to_string());
            }
        }
    }
    
    for out_var in &ld.out_variable {
        if let Some(ref expression) = out_var.expression {
            let expr = expression.trim();
            if !expr.is_empty() && is_identifier(expr) {
                analysis.used_variables.insert(expr.to_string());
            }
        }
    }
    
    for inout_var in &ld.in_out_variable {
        if let Some(ref expression) = inout_var.expression {
            let expr = expression.trim();
            if !expr.is_empty() && is_identifier(expr) {
                analysis.used_variables.insert(expr.to_string());
            }
        }
    }
    
    // Extract from coils (LD-specific: output variables)
    for coil in &ld.coil {
        if let Some(ref variable) = coil.variable {
            let var_name = variable.trim();
            if !var_name.is_empty() && is_identifier(var_name) {
                analysis.used_variables.insert(var_name.to_string());
            }
        }
    }
    
    // Extract from contacts (LD-specific: input variables/conditions)
    for contact in &ld.contact {
        if let Some(ref variable) = contact.variable {
            let var_name = variable.trim();
            if !var_name.is_empty() && is_identifier(var_name) {
                analysis.used_variables.insert(var_name.to_string());
            }
        }
    }
    
    // Extract from labels/jumps (same as FBD)
    for label in &ld.label {
        let name = label.label.trim();
        if !name.is_empty() {
            analysis.used_variables.insert(name.to_string());
        }
    }
    
    for jump in &ld.jump {
        let name = jump.label.trim();
        if !name.is_empty() {
            analysis.used_variables.insert(name.to_string());
        }
    }
}

fn extract_references_from_sfc(sfc: &plcopen::Body_SFC_Inline, analysis: &mut PlcopenAnalysis) {
    // SFC shares FBD/LD elements plus SFC-specific steps, transitions, actions
    
    // Extract from blocks (same as FBD/LD)
    for block in &sfc.block {
        let type_name = block.type_name.trim();
        if !type_name.is_empty() && !is_builtin(type_name) {
            analysis.used_pous.insert(type_name.to_string());
        }
        if let Some(ref instance_name) = block.instance_name {
            let name = instance_name.trim();
            if !name.is_empty() {
                analysis.used_variables.insert(name.to_string());
            }
        }
    }
    
    // Extract from variables (same as FBD/LD)
    for in_var in &sfc.in_variable {
        if let Some(ref expression) = in_var.expression {
            let expr = expression.trim();
            if !expr.is_empty() && is_identifier(expr) {
                analysis.used_variables.insert(expr.to_string());
            }
        }
    }
    
    for out_var in &sfc.out_variable {
        if let Some(ref expression) = out_var.expression {
            let expr = expression.trim();
            if !expr.is_empty() && is_identifier(expr) {
                analysis.used_variables.insert(expr.to_string());
            }
        }
    }
    
    for inout_var in &sfc.in_out_variable {
        if let Some(ref expression) = inout_var.expression {
            let expr = expression.trim();
            if !expr.is_empty() && is_identifier(expr) {
                analysis.used_variables.insert(expr.to_string());
            }
        }
    }
    
    // Extract from coils/contacts (if used in SFC)
    for coil in &sfc.coil {
        if let Some(ref variable) = coil.variable {
            let var_name = variable.trim();
            if !var_name.is_empty() && is_identifier(var_name) {
                analysis.used_variables.insert(var_name.to_string());
            }
        }
    }
    
    for contact in &sfc.contact {
        if let Some(ref variable) = contact.variable {
            let var_name = variable.trim();
            if !var_name.is_empty() && is_identifier(var_name) {
                analysis.used_variables.insert(var_name.to_string());
            }
        }
    }
    
    // Extract from SFC steps (step names are variables)
    for step in &sfc.step {
        let step_name = step.name.trim();
        if !step_name.is_empty() {
            analysis.used_variables.insert(step_name.to_string());
        }
    }
    
    // Extract from jump steps (references to step names)
    for jump_step in &sfc.jump_step {
        let target = jump_step.target_name.trim();
        if !target.is_empty() {
            analysis.used_variables.insert(target.to_string());
        }
    }
    
    // Extract from action blocks
    for action_block in &sfc.action_block {
        for action in &action_block.action {
            // Extract action name references
            if let Some(ref reference) = action.reference {
                let action_name = reference.name.trim();
                if !action_name.is_empty() {
                    // Action references are typically POU calls
                    analysis.used_pous.insert(action_name.to_string());
                }
            }
            // Inline actions would contain Body with ST/FBD/etc - already handled recursively
        }
    }
}

/// Check if a string is a valid identifier (not a literal or complex expression)
fn is_identifier(s: &str) -> bool {
    // Skip numeric literals
    if s.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        return false;
    }
    
    // Skip string literals
    if s.starts_with('\'') || s.starts_with('\"') {
        return false;
    }
    
    // Skip boolean literals
    if s == "TRUE" || s == "FALSE" {
        return false;
    }
    
    // Must start with letter or underscore and contain only alphanumeric/underscore
    let first_char = s.chars().next();
    if !matches!(first_char, Some(c) if c.is_alphabetic() || c == '_') {
        return false;
    }
    
    // Simple identifier check - no dots, brackets, or operators
    s.chars().all(|c| c.is_alphanumeric() || c == '_')
}

fn extract_type_name(_data: &plcopen::Data) -> Option<String> {
    // Data type extraction - the generated struct doesn't capture type details yet
    // TODO: Improve PLCopen codegen to capture type information
    None
}

fn is_st_keyword(word: &str) -> bool {
    matches!(word.to_uppercase().as_str(),
        "IF" | "THEN" | "ELSE" | "ELSIF" | "END_IF" |
        "FOR" | "TO" | "BY" | "DO" | "END_FOR" |
        "WHILE" | "END_WHILE" | "REPEAT" | "UNTIL" | "END_REPEAT" |
        "CASE" | "OF" | "END_CASE" |
        "VAR" | "VAR_INPUT" | "VAR_OUTPUT" | "VAR_IN_OUT" | "VAR_TEMP" | "VAR_GLOBAL" | "END_VAR" |
        "FUNCTION" | "FUNCTION_BLOCK" | "PROGRAM" | "END_FUNCTION" | "END_FUNCTION_BLOCK" | "END_PROGRAM" |
        "TRUE" | "FALSE" | "AND" | "OR" | "XOR" | "NOT" | "MOD" |
        "RETURN" | "EXIT" | "CONTINUE" |
        "BOOL" | "INT" | "DINT" | "SINT" | "LINT" | "UINT" | "UDINT" | "USINT" | "ULINT" |
        "REAL" | "LREAL" | "STRING" | "WSTRING" | "TIME" | "DATE" | "TOD" | "DT" | "BYTE" | "WORD" | "DWORD" | "LWORD"
    )
}

fn is_builtin(name: &str) -> bool {
    // Common IEC 61131-3 standard functions/function blocks
    matches!(name.to_uppercase().as_str(),
        // Timers
        "TON" | "TOF" | "TP" | "RTC" |
        // Counters
        "CTU" | "CTD" | "CTUD" |
        // Edge detection
        "R_TRIG" | "F_TRIG" |
        // Bistable
        "SR" | "RS" |
        // Type conversions
        "INT_TO_REAL" | "REAL_TO_INT" | "BOOL_TO_INT" | "INT_TO_BOOL" |
        // Math
        "ABS" | "SQRT" | "LN" | "LOG" | "EXP" | "SIN" | "COS" | "TAN" | "ASIN" | "ACOS" | "ATAN" |
        // String
        "LEN" | "LEFT" | "RIGHT" | "MID" | "CONCAT" | "INSERT" | "DELETE" | "REPLACE" | "FIND" |
        // Selection
        "SEL" | "MAX" | "MIN" | "LIMIT" | "MUX" |
        // Comparison
        "GT" | "GE" | "EQ" | "LE" | "LT" | "NE" |
        // Bit operations
        "SHL" | "SHR" | "ROL" | "ROR"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_is_st_keyword() {
        assert!(is_st_keyword("IF"));
        assert!(is_st_keyword("if"));
        assert!(is_st_keyword("THEN"));
        assert!(!is_st_keyword("MyVar"));
    }
    
    #[test]
    fn test_is_builtin() {
        assert!(is_builtin("TON"));
        assert!(is_builtin("CTU"));
        assert!(!is_builtin("MyFB"));
    }
}
