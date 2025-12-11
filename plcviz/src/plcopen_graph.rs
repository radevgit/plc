//! PLCopen graph building for plcviz.
//!
//! This module builds various graph types from PLCopen TC6 XML files.

use std::collections::HashSet;
use plcopen::Project;

use crate::graph::{L5xGraph, L5xNodeType};

/// Graph type for PLCopen projects
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlcopenGraphType {
    /// Project organization structure (Project → Programs → POUs)
    Structure,
    /// Call graph (function/FB calls)
    CallGraph,
    /// DataType dependencies (struct nesting)
    DataTypeDeps,
}

/// Build graph from PLCopen project
pub struct PlcopenGraphBuilder {
    project: Project,
    graph_type: PlcopenGraphType,
    raw_xml: Option<String>,
}

impl PlcopenGraphBuilder {
    /// Create a new PLCopen graph builder
    pub fn new(project: Project, graph_type: PlcopenGraphType) -> Self {
        Self { 
            project, 
            graph_type,
            raw_xml: None,
        }
    }

    /// Create a builder with raw XML for ST extraction
    pub fn with_xml(project: Project, graph_type: PlcopenGraphType, raw_xml: String) -> Self {
        Self {
            project,
            graph_type,
            raw_xml: Some(raw_xml),
        }
    }

    /// Build the graph
    pub fn build(self) -> L5xGraph {
        match self.graph_type {
            PlcopenGraphType::Structure => self.build_structure_graph(),
            PlcopenGraphType::CallGraph => self.build_call_graph(),
            PlcopenGraphType::DataTypeDeps => self.build_datatype_graph(),
        }
    }

    /// Build structure graph (Project → Programs → Functions/FBs)
    fn build_structure_graph(self) -> L5xGraph {
        let mut graph = L5xGraph::new();

        // Get project name from content header
        let project_name = self.project.content_header
            .as_ref()
            .map(|h| h.name.as_str())
            .unwrap_or("Project");

        // Add root project node
        let project_id = format!("project_{}", project_name);
        graph.add_node(&project_id, project_name, L5xNodeType::Controller);

        // Add POUs
        if let Some(ref types) = self.project.types {
            if let Some(ref pous) = types.pous {
                for pou in &pous.pou {
                    let pou_id = format!("pou_{}", pou.name);
                    let pou_type = &pou.pou_type;
                    
                    let node_type = match pou_type.to_lowercase().as_str() {
                        "program" => L5xNodeType::Program,
                        "function" => L5xNodeType::Routine,
                        "functionblock" => L5xNodeType::Aoi,
                        _ => L5xNodeType::Routine,
                    };

                    let label = format!("{} ({})", pou.name, pou_type);
                    graph.add_node_with_parent(&pou_id, &label, node_type, &project_id);

                    // Connect to project
                    graph.add_edge(&project_id, &pou_id, None);
                }
            }
        }

        graph
    }

    /// Build call graph (POU → POU calls from ST code)
    fn build_call_graph(self) -> L5xGraph {
        let mut graph = L5xGraph::new();

        // First pass: collect all POUs and add as nodes
        let mut pou_names = HashSet::new();
        if let Some(ref types) = self.project.types {
            if let Some(ref pous) = types.pous {
                for pou in &pous.pou {
                    pou_names.insert(pou.name.clone());
                    
                    let pou_id = format!("pou_{}", pou.name);
                    graph.add_node(&pou_id, &pou.name, L5xNodeType::Routine);
                }
            }
        }

        // Second pass: find calls and add edges
        if let Some(ref types) = self.project.types {
            if let Some(ref pous) = types.pous {
                for pou in &pous.pou {
                    let pou_id = format!("pou_{}", pou.name);

                    // Extract calls from bodies
                    let called_pous = if let Some(ref raw_xml) = self.raw_xml {
                        extract_calls_from_pou_with_xml(pou, &pou_names, raw_xml)
                    } else {
                        vec![]
                    };
                    
                    // Add edges for calls
                    for called in called_pous {
                        let called_id = format!("pou_{}", called);
                        graph.add_call(&pou_id, &called_id);
                    }
                }
            }
        }

        graph
    }

    /// Build datatype dependency graph
    fn build_datatype_graph(self) -> L5xGraph {
        let mut graph = L5xGraph::new();

        // Get datatypes
        if let Some(ref types) = self.project.types {
            if let Some(ref datatypes) = types.data_types {
                for datatype in &datatypes.data_type {
                    let dt_id = format!("datatype_{}", datatype.name);
                    graph.add_node(&dt_id, &datatype.name, L5xNodeType::Udt);

                    // TODO: Extract nested types from struct definitions
                    // This requires parsing the baseType element
                }
            }
        }

        graph
    }
}

/// Extract called POUs from a POU's body (legacy - without XML)
#[allow(dead_code)]
fn extract_calls_from_pou(pou: &plcopen::Root_project_InlineType_types_InlineType_pous_InlineType_pou_Inline, _known_pous: &HashSet<String>) -> Vec<String> {
    let mut calls = Vec::new();

    // Check main body for ST code
    for body in &pou.body {
        // Extract ST code if present
        if body.st.is_some() {
            // For now, skip extraction - would need raw XML
            // In a real implementation, we'd use plcopen::st::extract_st_from_xml
            // TODO: Store raw XML or use a different approach
        }
    }

    // Check actions
    if let Some(ref actions) = pou.actions {
        for action in &actions.action {
            if let Some(ref body) = action.body {
                if body.st.is_some() {
                    // Same as above - skip for now
                }
            }
        }
    }

    calls.sort();
    calls.dedup();
    calls
}

/// Extract called POUs from a POU's body using raw XML
fn extract_calls_from_pou_with_xml(
    pou: &plcopen::Root_project_InlineType_types_InlineType_pous_InlineType_pou_Inline, 
    known_pous: &HashSet<String>,
    raw_xml: &str
) -> Vec<String> {
    let mut calls = Vec::new();

    // Find this POU's XML section
    let pou_name = &pou.name;
    if let Some(pou_start) = raw_xml.find(&format!(r#"<pou name="{}""#, pou_name)) {
        if let Some(pou_end) = raw_xml[pou_start..].find("</pou>") {
            let pou_xml = &raw_xml[pou_start..pou_start + pou_end + 6];
            
            // Extract all ST code from this POU
            let st_blocks = plcopen::st::extract_all_st_from_xml(pou_xml);
            
            for (_, st_code) in st_blocks {
                let pou_calls = extract_calls_from_st(&st_code, known_pous);
                calls.extend(pou_calls);
            }
        }
    }

    calls.sort();
    calls.dedup();
    calls
}

/// Extract function/FB calls from ST code using iec61131 parser
fn extract_calls_from_st(code: &str, known_pous: &HashSet<String>) -> Vec<String> {
    let mut calls = Vec::new();

    // Parse ST code using iec61131
    let mut parser = iec61131::Parser::new(code);
    match parser.parse() {
        Ok(cu) => {
            // Walk the compilation unit to find function/FB calls
            for decl in &cu.declarations {
                extract_calls_from_declaration(decl, known_pous, &mut calls);
            }
        }
        Err(_) => {
            // If parsing fails, fall back to regex
            let re = regex::Regex::new(r"\b([A-Za-z_][A-Za-z0-9_]*)\s*\(").unwrap();
            
            for cap in re.captures_iter(code) {
                if let Some(name) = cap.get(1) {
                    let name_str = name.as_str().to_string();
                    if known_pous.contains(&name_str) {
                        calls.push(name_str);
                    }
                }
            }
        }
    }

    calls
}

/// Extract calls from a declaration (function, program, etc.)
fn extract_calls_from_declaration(decl: &iec61131::PouDeclaration, known_pous: &HashSet<String>, calls: &mut Vec<String>) {
    use iec61131::PouDeclaration;
    
    match decl {
        PouDeclaration::Function(func) => {
            for stmt in &func.body {
                extract_calls_from_statement(stmt, known_pous, calls);
            }
        }
        PouDeclaration::FunctionBlock(fb) => {
            if let Some(ref body) = fb.body {
                for stmt in body {
                    extract_calls_from_statement(stmt, known_pous, calls);
                }
            }
        }
        PouDeclaration::Program(prog) => {
            for stmt in &prog.body {
                extract_calls_from_statement(stmt, known_pous, calls);
            }
        }
        _ => {}
    }
}

/// Recursively extract calls from a statement
fn extract_calls_from_statement(stmt: &iec61131::Statement, known_pous: &HashSet<String>, calls: &mut Vec<String>) {
    use iec61131::Statement;
    
    match stmt {
        Statement::Assignment { value, .. } => {
            extract_calls_from_expression(value, known_pous, calls);
        }
        Statement::If { condition, then_body, elsif_parts, else_body, .. } => {
            extract_calls_from_expression(condition, known_pous, calls);
            for s in then_body {
                extract_calls_from_statement(s, known_pous, calls);
            }
            for (elsif_cond, elsif_body) in elsif_parts {
                extract_calls_from_expression(elsif_cond, known_pous, calls);
                for s in elsif_body {
                    extract_calls_from_statement(s, known_pous, calls);
                }
            }
            if let Some(else_stmts) = else_body {
                for s in else_stmts {
                    extract_calls_from_statement(s, known_pous, calls);
                }
            }
        }
        Statement::While { condition, body, .. } => {
            extract_calls_from_expression(condition, known_pous, calls);
            for s in body {
                extract_calls_from_statement(s, known_pous, calls);
            }
        }
        Statement::Repeat { body, condition, .. } => {
            for s in body {
                extract_calls_from_statement(s, known_pous, calls);
            }
            extract_calls_from_expression(condition, known_pous, calls);
        }
        Statement::For { start, end, step, body, .. } => {
            extract_calls_from_expression(start, known_pous, calls);
            extract_calls_from_expression(end, known_pous, calls);
            if let Some(step_expr) = step {
                extract_calls_from_expression(step_expr, known_pous, calls);
            }
            for s in body {
                extract_calls_from_statement(s, known_pous, calls);
            }
        }
        Statement::Case { selector, cases, else_body, .. } => {
            extract_calls_from_expression(selector, known_pous, calls);
            for case in cases {
                for s in &case.body {
                    extract_calls_from_statement(s, known_pous, calls);
                }
            }
            if let Some(else_stmts) = else_body {
                for s in else_stmts {
                    extract_calls_from_statement(s, known_pous, calls);
                }
            }
        }
        Statement::FunctionCall { name, arguments, .. } => {
            // Check if this is a known POU call
            if known_pous.contains(name) {
                calls.push(name.clone());
            }
            // Also check arguments for nested calls
            for arg in arguments {
                extract_calls_from_argument(arg, known_pous, calls);
            }
        }
        Statement::FbInvocation { instance: _, arguments, .. } => {
            // FB invocations use instance names, not POU names directly
            // But check arguments for nested calls
            for arg in arguments {
                extract_calls_from_argument(arg, known_pous, calls);
            }
        }
        Statement::Return { value, .. } => {
            if let Some(expr) = value {
                extract_calls_from_expression(expr, known_pous, calls);
            }
        }
        _ => {}
    }
}

/// Extract calls from an argument
fn extract_calls_from_argument(arg: &iec61131::Argument, known_pous: &HashSet<String>, calls: &mut Vec<String>) {
    use iec61131::Argument;
    
    match arg {
        Argument::Positional(expr) | Argument::Named { value: expr, .. } => {
            extract_calls_from_expression(expr, known_pous, calls);
        }
        Argument::Output { .. } => {}
    }
}

/// Extract calls from an expression
fn extract_calls_from_expression(expr: &iec61131::Expression, known_pous: &HashSet<String>, calls: &mut Vec<String>) {
    use iec61131::Expression;
    
    match expr {
        Expression::Binary { left, right, .. } => {
            extract_calls_from_expression(left, known_pous, calls);
            extract_calls_from_expression(right, known_pous, calls);
        }
        Expression::Unary { operand, .. } => {
            extract_calls_from_expression(operand, known_pous, calls);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_calls_from_st() {
        let code = r#"
            result := MyFunction(x, y);
            IF condition THEN
                AnotherFunction(z);
                SIN(angle);  // standard function, should be ignored
            END_IF;
        "#;

        let mut known_pous = HashSet::new();
        known_pous.insert("MyFunction".to_string());
        known_pous.insert("AnotherFunction".to_string());

        let calls = extract_calls_from_st(code, &known_pous);
        
        assert_eq!(calls.len(), 2);
        assert!(calls.contains(&"MyFunction".to_string()));
        assert!(calls.contains(&"AnotherFunction".to_string()));
        assert!(!calls.contains(&"SIN".to_string()));
    }
}
