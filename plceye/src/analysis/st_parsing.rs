//! Structured Text (ST) parsing utilities.
//!
//! Extract and parse ST routines from L5X structures.

use l5x::{
    AProgram, Routine, RoutineContent,
    STContent, STContentContent, STLine,
    UDIDefinition, UDIDefinitionContent,
};
use iecst::{Pou, Expr, Stmt, ExprKind, StmtKind};

use super::{STLocation, ParsedSTRoutine};

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

    // Wrap in synthetic PROGRAM for parsing
    let wrapped_source = format!(
        "PROGRAM {}\nVAR\nEND_VAR\n{}\nEND_PROGRAM",
        routine.name, source
    );

    match iecst::parse_pou(&wrapped_source) {
        Ok(pou) => {
            Some(ParsedSTRoutine {
                location,
                source,
                pou: Some(pou),
                parse_error: None,
            })
        }
        Err(e) => {
            Some(ParsedSTRoutine {
                location,
                source,
                pou: None,
                parse_error: Some(e),
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

/// Parse all ST routines from an Add-On Instruction (AOI).
pub fn parse_st_routines_from_aoi(aoi: &UDIDefinition) -> Vec<ParsedSTRoutine> {
    let mut results = Vec::new();

    for content in &aoi.content {
        if let UDIDefinitionContent::Routines(routine_collection) = content {
            for routine in &routine_collection.routine {
                let aoi_name = format!("AOI:{}", aoi.name);
                if let Some(parsed) = parse_st_routine(routine, &aoi_name) {
                    results.push(parsed);
                }
            }
        }
    }

    results
}

/// Extract all function/FB call names from an ST POU.
pub fn extract_st_call_names(pou: &Pou) -> Vec<String> {
    let mut calls = Vec::new();
    
    fn visit_expr(expr: &Expr, calls: &mut Vec<String>) {
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
    
    fn visit_stmt(stmt: &Stmt, calls: &mut Vec<String>) {
        match &stmt.kind {
            StmtKind::Assignment { target, value } => {
                visit_expr(target, calls);
                visit_expr(value, calls);
            }
            StmtKind::If { condition, then_body, elsif_branches, else_body } => {
                visit_expr(condition, calls);
                for s in then_body { visit_stmt(s, calls); }
                for (cond, body) in elsif_branches {
                    visit_expr(cond, calls);
                    for s in body { visit_stmt(s, calls); }
                }
                if let Some(else_stmts) = else_body {
                    for s in else_stmts { visit_stmt(s, calls); }
                }
            }
            StmtKind::Case { expr, cases, else_body } => {
                visit_expr(expr, calls);
                for case in cases {
                    for s in &case.body { visit_stmt(s, calls); }
                }
                if let Some(else_stmts) = else_body {
                    for s in else_stmts { visit_stmt(s, calls); }
                }
            }
            StmtKind::For { from, to, by, body, .. } => {
                visit_expr(from, calls);
                visit_expr(to, calls);
                if let Some(by_expr) = by { visit_expr(by_expr, calls); }
                for s in body { visit_stmt(s, calls); }
            }
            StmtKind::While { condition, body } => {
                visit_expr(condition, calls);
                for s in body { visit_stmt(s, calls); }
            }
            StmtKind::Repeat { body, until } => {
                for s in body { visit_stmt(s, calls); }
                visit_expr(until, calls);
            }
            StmtKind::Return { value: Some(v) } => {
                visit_expr(v, calls);
            }
            StmtKind::Return { value: None } => {}
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
    
    calls.sort();
    calls.dedup();
    calls
}
