//! RLL (Relay Ladder Logic) parsing utilities.
//!
//! Extract and parse ladder logic rungs from L5X structures.

use l5x::rll;
use l5x::{
    Routine, RoutineContent, Rung, RungContent,
    RungCollection, TextWide, TextWideContent,
};

use super::{RungLocation, LocatedRung};

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
