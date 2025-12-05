//! Cross-reference index for tag/variable usage.

use std::collections::{HashMap, HashSet};

use crate::{Body, Instruction, Operand, Pou, Project, Rung};

/// A reference to a tag/variable in code.
#[derive(Debug, Clone)]
pub struct TagReference {
    /// The tag name being referenced
    pub tag_name: String,
    /// Full operand text (e.g., "Motor.Running" or "Values[5]")
    pub full_operand: String,
    /// Instruction mnemonic (e.g., "XIC", "OTE", "MOV")
    pub instruction: String,
    /// Location where this reference occurs
    pub location: ReferenceLocation,
}

/// Location of a reference within the project.
#[derive(Debug, Clone)]
pub struct ReferenceLocation {
    /// POU name
    pub pou: String,
    /// Routine name (if applicable)
    pub routine: Option<String>,
    /// Rung/network number (if applicable)
    pub rung: Option<u32>,
}

impl ReferenceLocation {
    /// Create a location for a POU.
    pub fn in_pou(pou: &str) -> Self {
        Self {
            pou: pou.to_string(),
            routine: None,
            rung: None,
        }
    }

    /// Create a location in a routine.
    pub fn in_routine(pou: &str, routine: &str) -> Self {
        Self {
            pou: pou.to_string(),
            routine: Some(routine.to_string()),
            rung: None,
        }
    }

    /// Create a location at a specific rung.
    pub fn at_rung(pou: &str, routine: Option<&str>, rung: u32) -> Self {
        Self {
            pou: pou.to_string(),
            routine: routine.map(|s| s.to_string()),
            rung: Some(rung),
        }
    }

    /// Get a path string for this location.
    pub fn path(&self) -> String {
        match (&self.routine, self.rung) {
            (Some(r), Some(n)) => format!("{}/{}/Rung#{}", self.pou, r, n),
            (Some(r), None) => format!("{}/{}", self.pou, r),
            (None, Some(n)) => format!("{}/Rung#{}", self.pou, n),
            (None, None) => self.pou.clone(),
        }
    }
}

/// Cross-reference index for a project.
#[derive(Debug, Default)]
pub struct CrossReference {
    /// All tag references found in code
    pub references: Vec<TagReference>,
    
    /// Index: tag_name -> indices into references
    tag_index: HashMap<String, Vec<usize>>,
    
    /// Set of all unique tag names referenced
    pub used_tags: HashSet<String>,
    
    /// Set of all POUs that are called/instantiated
    pub used_pous: HashSet<String>,
    
    /// Set of all data types that are used
    pub used_types: HashSet<String>,
}

impl CrossReference {
    /// Build a cross-reference from a project.
    pub fn build(project: &Project) -> Self {
        let mut xref = CrossReference::default();
        
        for pou in &project.pous {
            xref.analyze_pou(pou);
        }
        
        // Also mark interface variable types as used
        for pou in &project.pous {
            for var in pou.interface.all_variables() {
                xref.used_types.insert(var.data_type.clone());
            }
        }
        
        // Mark data type member types as used
        for dt in &project.data_types {
            xref.used_types.insert(dt.name.clone());
        }
        
        xref
    }
    
    /// Get all references to a specific tag.
    pub fn references_to(&self, tag_name: &str) -> Vec<&TagReference> {
        self.tag_index
            .get(tag_name)
            .map(|indices| {
                indices.iter().filter_map(|&i| self.references.get(i)).collect()
            })
            .unwrap_or_default()
    }
    
    /// Check if a tag is used anywhere.
    pub fn is_tag_used(&self, tag_name: &str) -> bool {
        self.used_tags.contains(tag_name)
    }
    
    /// Check if a POU is called anywhere.
    pub fn is_pou_used(&self, pou_name: &str) -> bool {
        self.used_pous.contains(pou_name)
    }
    
    /// Check if a data type is used anywhere.
    pub fn is_type_used(&self, type_name: &str) -> bool {
        self.used_types.contains(type_name)
    }
    
    fn analyze_pou(&mut self, pou: &Pou) {
        if let Some(ref body) = pou.body {
            self.analyze_body(body, &pou.name);
        }
    }
    
    fn analyze_body(&mut self, body: &Body, pou_name: &str) {
        match body {
            Body::St(code) => self.analyze_st(code, pou_name),
            Body::Il(code) => self.analyze_il(code, pou_name),
            Body::Ld(rungs) => self.analyze_rungs(rungs, pou_name),
            Body::Fbd(networks) => self.analyze_networks(networks, pou_name),
            Body::Sfc(sfc) => self.analyze_sfc(sfc, pou_name),
            Body::Raw { content, .. } => self.analyze_raw(content, pou_name),
        }
    }
    
    fn analyze_rungs(&mut self, rungs: &[Rung], pou_name: &str) {
        for (i, rung) in rungs.iter().enumerate() {
            let rung_num = i as u32;
            for instr in &rung.instructions {
                self.analyze_instruction(instr, pou_name, Some(rung_num));
            }
        }
    }
    
    fn analyze_instruction(&mut self, instr: &Instruction, pou_name: &str, rung: Option<u32>) {
        // Check if instruction is a POU call
        self.used_pous.insert(instr.mnemonic.clone());
        
        for operand in &instr.operands {
            self.analyze_operand(operand, &instr.mnemonic, pou_name, rung);
        }
    }
    
    fn analyze_operand(&mut self, operand: &Operand, instruction: &str, pou_name: &str, rung: Option<u32>) {
        let (tag_name, full_text) = match operand {
            Operand::Tag { name, full_text } => (name.clone(), full_text.clone()),
            Operand::Expression(expr) => {
                // Extract tag names from expression (simple approach)
                self.extract_tags_from_expr(expr, instruction, pou_name, rung);
                return;
            }
            Operand::Literal(_) => return,
            Operand::Address(_) => return,
        };
        
        self.add_reference(tag_name, full_text, instruction.to_string(), pou_name, rung);
    }
    
    fn add_reference(&mut self, tag_name: String, full_operand: String, instruction: String, pou_name: &str, rung: Option<u32>) {
        let location = ReferenceLocation::at_rung(pou_name, None, rung.unwrap_or(0));
        
        let idx = self.references.len();
        self.references.push(TagReference {
            tag_name: tag_name.clone(),
            full_operand,
            instruction,
            location,
        });
        
        self.tag_index.entry(tag_name.clone()).or_default().push(idx);
        self.used_tags.insert(tag_name);
    }
    
    fn extract_tags_from_expr(&mut self, expr: &str, instruction: &str, pou_name: &str, rung: Option<u32>) {
        // Simple extraction: split on operators and find identifiers
        for part in expr.split(&['+', '-', '*', '/', '(', ')', ',', ' ', '=', '<', '>', '!', '&', '|'][..]) {
            let trimmed = part.trim();
            if !trimmed.is_empty() && !trimmed.chars().next().unwrap().is_ascii_digit() {
                // Looks like an identifier
                let base_tag = trimmed.split('.').next().unwrap_or(trimmed);
                let base_tag = base_tag.split('[').next().unwrap_or(base_tag);
                if !base_tag.is_empty() && base_tag.chars().next().unwrap().is_alphabetic() {
                    self.add_reference(
                        base_tag.to_string(),
                        trimmed.to_string(),
                        instruction.to_string(),
                        pou_name,
                        rung,
                    );
                }
            }
        }
    }
    
    fn analyze_networks(&mut self, networks: &[crate::Network], pou_name: &str) {
        for (i, network) in networks.iter().enumerate() {
            for instr in &network.elements {
                self.analyze_instruction(instr, pou_name, Some(i as u32));
            }
        }
    }
    
    fn analyze_st(&mut self, code: &str, pou_name: &str) {
        // Simple ST analysis - extract identifiers
        self.extract_tags_from_expr(code, "ST", pou_name, None);
    }
    
    fn analyze_il(&mut self, code: &str, pou_name: &str) {
        self.extract_tags_from_expr(code, "IL", pou_name, None);
    }
    
    fn analyze_sfc(&mut self, sfc: &crate::SfcBody, pou_name: &str) {
        for step in &sfc.steps {
            for action in &step.actions {
                if let Some(ref body) = action.body {
                    self.analyze_body(body, pou_name);
                }
            }
        }
        for trans in &sfc.transitions {
            self.extract_tags_from_expr(&trans.condition, "SFC", pou_name, None);
        }
    }
    
    fn analyze_raw(&mut self, content: &str, pou_name: &str) {
        // For raw RLL content, try to extract tag references
        // Pattern: instruction(operand) like XIC(Tag) or MOV(src,dst)
        for cap in content.split(|c: char| c == ';' || c == '\n') {
            let trimmed = cap.trim();
            if let Some(paren_start) = trimmed.find('(') {
                if let Some(paren_end) = trimmed.rfind(')') {
                    let instruction = &trimmed[..paren_start];
                    let operands = &trimmed[paren_start + 1..paren_end];
                    
                    for op in operands.split(',') {
                        let op = op.trim();
                        if !op.is_empty() {
                            let base = op.split('.').next().unwrap_or(op);
                            let base = base.split('[').next().unwrap_or(base);
                            if !base.is_empty() && base.chars().next().map(|c| c.is_alphabetic()).unwrap_or(false) {
                                self.add_reference(
                                    base.to_string(),
                                    op.to_string(),
                                    instruction.to_string(),
                                    pou_name,
                                    None,
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Pou, Project};
    use iectypes::PouType;

    #[test]
    fn test_xref_from_raw_rll() {
        let mut pou = Pou::new("Main", PouType::Program);
        pou.body = Some(Body::Raw {
            language: "RLL".to_string(),
            content: "XIC(Start)OTE(Motor);MOV(Counter,Dest);".to_string(),
        });
        
        let mut project = Project::new("Test");
        project.pous.push(pou);
        
        let xref = CrossReference::build(&project);
        
        assert!(xref.is_tag_used("Start"));
        assert!(xref.is_tag_used("Motor"));
        assert!(xref.is_tag_used("Counter"));
        assert!(xref.is_tag_used("Dest"));
        assert!(!xref.is_tag_used("NotUsed"));
    }

    #[test]
    fn test_reference_location_path() {
        let loc = ReferenceLocation::at_rung("MainProgram", Some("MainRoutine"), 5);
        assert_eq!(loc.path(), "MainProgram/MainRoutine/Rung#5");
        
        let loc2 = ReferenceLocation::in_pou("FB_Motor");
        assert_eq!(loc2.path(), "FB_Motor");
    }
}
