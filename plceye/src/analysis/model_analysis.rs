//! Model-based project analysis.
//!
//! This module provides format-independent analysis using plcmodel,
//! enabling smell detection for any supported PLC format.

use std::collections::{HashMap, HashSet};

use plcmodel::{Body, Project, Pou};

/// Cross-reference data built from a plcmodel Project.
#[derive(Debug, Default)]
pub struct ModelAnalysis {
    /// All tag/variable names that are used in code
    pub used_tags: HashSet<String>,
    
    /// All defined variable names (from interfaces)
    pub defined_tags: HashSet<String>,
    
    /// Variable definitions: name -> (scope, data_type)
    pub variable_defs: HashMap<String, VariableDef>,
    
    /// POUs that are called/instantiated
    pub used_pous: HashSet<String>,
    
    /// All defined POU names
    pub defined_pous: HashSet<String>,
    
    /// Empty POUs (no body or empty body)
    pub empty_pous: Vec<String>,
    
    /// Statistics
    pub stats: ModelStats,
}

/// Definition info for a variable.
#[derive(Debug, Clone)]
pub struct VariableDef {
    pub name: String,
    pub scope: String,
    pub data_type: String,
    pub var_class: iectypes::VarClass,
}

/// Statistics from model analysis.
#[derive(Debug, Clone, Default)]
pub struct ModelStats {
    pub programs: usize,
    pub function_blocks: usize,
    pub functions: usize,
    pub total_variables: usize,
    pub tag_references: usize,
}

impl ModelAnalysis {
    /// Analyze a plcmodel Project.
    pub fn analyze(project: &Project) -> Self {
        let mut analysis = Self::default();
        
        // Collect defined POUs
        for pou in &project.pous {
            analysis.defined_pous.insert(pou.name.clone());
            
            // Count by type
            match pou.pou_type {
                iectypes::PouType::Program => analysis.stats.programs += 1,
                iectypes::PouType::FunctionBlock => analysis.stats.function_blocks += 1,
                iectypes::PouType::Function => analysis.stats.functions += 1,
            }
            
            // Check if empty
            if pou.body.is_none() || pou.body.as_ref().is_some_and(|b| is_body_empty(b)) {
                analysis.empty_pous.push(pou.name.clone());
            }
            
            // Collect defined variables
            analysis.collect_variables(pou);
            
            // Extract tag references from body
            if let Some(ref body) = pou.body {
                analysis.extract_references(body, &pou.name);
            }
        }
        
        analysis
    }
    
    fn collect_variables(&mut self, pou: &Pou) {
        let scope = &pou.name;
        
        for var in pou.interface.all_variables() {
            self.defined_tags.insert(var.name.clone());
            self.stats.total_variables += 1;
            
            self.variable_defs.insert(var.name.clone(), VariableDef {
                name: var.name.clone(),
                scope: scope.clone(),
                data_type: var.data_type.clone(),
                var_class: var.var_class.clone(),
            });
        }
    }
    
    fn extract_references(&mut self, body: &Body, _pou_name: &str) {
        match body {
            Body::St(code) => self.extract_from_text(code),
            Body::Il(code) => self.extract_from_text(code),
            Body::Raw { content, .. } => self.extract_from_rll(content),
            Body::Ld(rungs) => {
                for rung in rungs {
                    for instr in &rung.instructions {
                        self.used_pous.insert(instr.mnemonic.clone());
                        for op in &instr.operands {
                            if let plcmodel::Operand::Tag(name) = op {
                                self.used_tags.insert(name.clone());
                                self.stats.tag_references += 1;
                            }
                        }
                    }
                }
            }
            Body::Fbd(networks) => {
                for network in networks {
                    for instr in &network.instructions {
                        self.used_pous.insert(instr.mnemonic.clone());
                        for op in &instr.operands {
                            if let plcmodel::Operand::Tag(name) = op {
                                self.used_tags.insert(name.clone());
                                self.stats.tag_references += 1;
                            }
                        }
                    }
                }
            }
            Body::Sfc(sfc) => {
                for step in &sfc.steps {
                    for action in &step.actions {
                        if let Some(ref b) = action.body {
                            self.extract_references(b, _pou_name);
                        }
                    }
                }
                for trans in &sfc.transitions {
                    self.extract_from_text(&trans.condition);
                }
            }
        }
    }
    
    /// Extract tag references from RLL text (instruction(operand) format).
    fn extract_from_rll(&mut self, content: &str) {
        // Pattern: INSTR(op1,op2)INSTR(op) or with ; delimiters
        // Use regex-like parsing by finding all instruction(operands) patterns
        let mut remaining = content;
        
        while !remaining.is_empty() {
            remaining = remaining.trim_start();
            
            // Skip ; and newlines
            if remaining.starts_with(';') || remaining.starts_with('\n') {
                remaining = &remaining[1..];
                continue;
            }
            
            // Skip [ ] blocks
            if remaining.starts_with('[') {
                if let Some(end) = remaining.find(']') {
                    remaining = &remaining[end + 1..];
                    continue;
                }
            }
            
            // Find instruction name (until '(' or end of string)
            if let Some(paren_start) = remaining.find('(') {
                let instruction = remaining[..paren_start].trim();
                if !instruction.is_empty() {
                    self.used_pous.insert(instruction.to_string());
                }
                
                // Find matching closing paren (handle nested parens)
                let after_paren = &remaining[paren_start + 1..];
                if let Some(close_pos) = find_matching_paren(after_paren) {
                    let operands = &after_paren[..close_pos];
                    for op in operands.split(',') {
                        if let Some(base) = extract_base_tag(op.trim()) {
                            self.used_tags.insert(base);
                            self.stats.tag_references += 1;
                        }
                    }
                    remaining = &after_paren[close_pos + 1..];
                } else {
                    break;
                }
            } else {
                // No more instructions
                break;
            }
        }
    }
    
    /// Extract tag references from ST/IL text.
    fn extract_from_text(&mut self, code: &str) {
        // Simple extraction: find identifiers that could be tags
        for word in code.split(|c: char| !c.is_alphanumeric() && c != '_' && c != '.') {
            let word = word.trim();
            if !word.is_empty() && word.chars().next().map(|c| c.is_alphabetic()).unwrap_or(false) {
                // Skip keywords
                if !is_st_keyword(word) {
                    if let Some(base) = extract_base_tag(word) {
                        self.used_tags.insert(base);
                        self.stats.tag_references += 1;
                    }
                }
            }
        }
    }
    
    /// Get unused tags (defined but not used).
    pub fn unused_tags(&self) -> Vec<&VariableDef> {
        self.variable_defs
            .values()
            .filter(|v| !self.used_tags.contains(&v.name))
            .collect()
    }
    
    /// Get undefined tags (used but not defined).
    pub fn undefined_tags(&self) -> Vec<&String> {
        self.used_tags
            .iter()
            .filter(|t| !self.defined_tags.contains(*t))
            .collect()
    }
    
    /// Get unused POUs (defined but not called).
    pub fn unused_pous(&self) -> Vec<&String> {
        self.defined_pous
            .iter()
            .filter(|p| !self.used_pous.contains(*p))
            .collect()
    }
}

/// Check if a body is effectively empty.
fn is_body_empty(body: &Body) -> bool {
    match body {
        Body::St(s) | Body::Il(s) => s.trim().is_empty(),
        Body::Raw { content, .. } => content.trim().is_empty(),
        Body::Ld(rungs) => rungs.is_empty(),
        Body::Fbd(networks) => networks.is_empty(),
        Body::Sfc(sfc) => sfc.steps.is_empty(),
    }
}

/// Extract base tag name from an operand (strip array indices, struct members).
fn extract_base_tag(operand: &str) -> Option<String> {
    let s = operand.trim();
    if s.is_empty() {
        return None;
    }
    
    // Skip if starts with digit (literal)
    if s.chars().next()?.is_ascii_digit() {
        return None;
    }
    
    // Skip if looks like a direct address
    if s.starts_with('%') {
        return None;
    }
    
    // Get base name (before . or [)
    let base = s.split('.').next()?;
    let base = base.split('[').next()?;
    
    if base.is_empty() || !base.chars().next()?.is_alphabetic() {
        return None;
    }
    
    Some(base.to_string())
}

/// Check if a word is an ST keyword.
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
        "BOOL" | "INT" | "DINT" | "REAL" | "STRING" | "TIME" | "DATE" | "TOD" | "DT"
    )
}

/// Find the position of matching closing paren (handles nesting).
fn find_matching_paren(s: &str) -> Option<usize> {
    let mut depth = 0;
    for (i, c) in s.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                if depth == 0 {
                    return Some(i);
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use iectypes::PouType;

    #[test]
    fn test_extract_base_tag() {
        assert_eq!(extract_base_tag("Motor"), Some("Motor".to_string()));
        assert_eq!(extract_base_tag("Motor.Running"), Some("Motor".to_string()));
        assert_eq!(extract_base_tag("Values[5]"), Some("Values".to_string()));
        assert_eq!(extract_base_tag("Data[i].Field"), Some("Data".to_string()));
        assert_eq!(extract_base_tag("123"), None);
        assert_eq!(extract_base_tag("%I0.0"), None);
        assert_eq!(extract_base_tag(""), None);
    }

    #[test]
    fn test_analyze_empty_pou() {
        let mut project = Project::new("Test");
        project.pous.push(Pou::new("EmptyProgram", PouType::Program));
        
        let analysis = ModelAnalysis::analyze(&project);
        
        assert!(analysis.empty_pous.contains(&"EmptyProgram".to_string()));
    }

    #[test]
    fn test_analyze_rll_references() {
        let mut pou = Pou::new("Main", PouType::Program);
        pou.body = Some(Body::Raw {
            language: "RLL".to_string(),
            content: "XIC(Start)OTE(Motor);MOV(Counter,Dest);".to_string(),
        });
        
        let mut project = Project::new("Test");
        project.pous.push(pou);
        
        let analysis = ModelAnalysis::analyze(&project);
        
        assert!(analysis.used_tags.contains("Start"));
        assert!(analysis.used_tags.contains("Motor"));
        assert!(analysis.used_tags.contains("Counter"));
        assert!(analysis.used_tags.contains("Dest"));
        assert!(analysis.used_pous.contains("XIC"));
        assert!(analysis.used_pous.contains("OTE"));
        assert!(analysis.used_pous.contains("MOV"));
    }

    #[test]
    fn test_unused_tags() {
        use plcmodel::Variable;
        
        let mut pou = Pou::new("Main", PouType::Program);
        pou.interface.locals.push(Variable::new("Used", "BOOL"));
        pou.interface.locals.push(Variable::new("Unused", "BOOL"));
        pou.body = Some(Body::Raw {
            language: "RLL".to_string(),
            content: "XIC(Used)OTE(Output);".to_string(),
        });
        
        let mut project = Project::new("Test");
        project.pous.push(pou);
        
        let analysis = ModelAnalysis::analyze(&project);
        
        let unused: Vec<_> = analysis.unused_tags().iter().map(|v| &v.name).collect();
        assert!(unused.contains(&&"Unused".to_string()));
        assert!(!unused.contains(&&"Used".to_string()));
    }
}
