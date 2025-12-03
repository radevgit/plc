//! RLL text parser using winnow.
//!
//! Grammar (EBNF):
//! ```text
//! rung           = element* ";"
//! element        = instruction | parallel
//! parallel       = "[" branch ("," branch)* "]"
//! branch         = element+
//! instruction    = MNEMONIC "(" operand_list? ")"
//! operand_list   = operand ("," operand)*
//! operand        = "?" | OPERAND_STRING
//! MNEMONIC       = [A-Za-z_][A-Za-z0-9_]*
//! OPERAND_STRING = (* balanced parens, stops at , ) ] *)
//! ```

use winnow::prelude::*;
use winnow::combinator::{alt, delimited, repeat, separated, terminated};
use winnow::token::{take_while, one_of, any};

use crate::rll::ast::{Branch, Instruction, Operand, RungContent, RungElement};
use crate::rll::error::{RllError, RllResult};
use crate::rll::Rung;

/// Parse a rung text string into a structured Rung.
///
/// This is permissive: if parsing fails, returns a Rung with the error
/// stored and the original text preserved.
pub fn parse_rung(input: &str) -> Rung {
    let trimmed = input.trim();
    
    if trimmed.is_empty() {
        return Rung::ok(input.to_string(), RungContent::new(Vec::new()));
    }

    match parse_rung_strict(trimmed) {
        Ok(content) => Rung::ok(input.to_string(), content),
        Err(e) => Rung::err(input.to_string(), e),
    }
}

/// Parse a rung strictly, returning an error on failure.
pub fn parse_rung_strict(input: &str) -> RllResult<RungContent> {
    let mut input = input;
    
    match rung_parser.parse_next(&mut input) {
        Ok(content) => Ok(content),
        Err(_) => {
            // Try to give a more specific error
            if !input.contains(';') {
                Err(RllError::MissingTerminator)
            } else if input.matches('[').count() != input.matches(']').count() {
                Err(RllError::UnclosedBracket { position: input.find('[').unwrap_or(0) })
            } else if input.matches('(').count() != input.matches(')').count() {
                Err(RllError::UnclosedParen { position: input.find('(').unwrap_or(0) })
            } else {
                Err(RllError::UnexpectedEof)
            }
        }
    }
}

/// Main rung parser: element* ";"
fn rung_parser(input: &mut &str) -> ModalResult<RungContent> {
    let elements = terminated(
        repeat(0.., element_parser),
        (take_while(0.., |c: char| c.is_whitespace()), ';')
    ).parse_next(input)?;
    
    Ok(RungContent::new(elements))
}

/// Parse a single element (instruction or parallel)
fn element_parser(input: &mut &str) -> ModalResult<RungElement> {
    // Skip optional whitespace before element
    let _ = take_while(0.., |c: char| c.is_whitespace()).parse_next(input)?;
    
    alt((
        parallel_parser.map(RungElement::Parallel),
        instruction_parser.map(RungElement::Instruction),
    )).parse_next(input)
}

/// Parse a parallel structure: [ branch (, branch)* ]
fn parallel_parser(input: &mut &str) -> ModalResult<Vec<Branch>> {
    // Open bracket
    let _ = '['.parse_next(input)?;
    
    // Parse branches separated by comma, allowing whitespace around comma
    let branches: Vec<Branch> = separated(1.., branch_parser, ws_comma_ws).parse_next(input)?;
    
    // Skip trailing whitespace before close bracket
    let _ = take_while(0.., |c: char| c.is_whitespace()).parse_next(input)?;
    
    // Close bracket
    let _ = ']'.parse_next(input)?;
    
    Ok(branches)
}

/// Parse comma with optional surrounding whitespace
fn ws_comma_ws(input: &mut &str) -> ModalResult<()> {
    let _ = take_while(0.., |c: char| c.is_whitespace()).parse_next(input)?;
    let _ = ','.parse_next(input)?;
    let _ = take_while(0.., |c: char| c.is_whitespace()).parse_next(input)?;
    Ok(())
}

/// Parse a branch: element+
fn branch_parser(input: &mut &str) -> ModalResult<Branch> {
    // Skip leading whitespace
    let _ = take_while(0.., |c: char| c.is_whitespace()).parse_next(input)?;
    
    let elements: Vec<RungElement> = repeat(1.., branch_element_parser).parse_next(input)?;
    Ok(Branch::new(elements))
}

/// Parse element within a branch (stops before , or ])
fn branch_element_parser(input: &mut &str) -> ModalResult<RungElement> {
    // Skip optional whitespace
    let _ = take_while(0.., |c: char| c.is_whitespace()).parse_next(input)?;
    
    // Peek to check if we're at end of branch
    if input.is_empty() || input.starts_with(',') || input.starts_with(']') {
        return Err(winnow::error::ErrMode::Backtrack(winnow::error::ContextError::new()));
    }
    
    alt((
        parallel_parser.map(RungElement::Parallel),
        instruction_parser.map(RungElement::Instruction),
    )).parse_next(input)
}

/// Parse an instruction: MNEMONIC "(" operand_list? ")"
fn instruction_parser(input: &mut &str) -> ModalResult<Instruction> {
    let mnemonic = mnemonic_parser.parse_next(input)?;
    let operands = delimited(
        '(',
        operand_list_parser,
        ')'
    ).parse_next(input)?;
    
    Ok(Instruction::new(mnemonic, operands))
}

/// Parse instruction mnemonic: [A-Za-z_][A-Za-z0-9_]*
fn mnemonic_parser(input: &mut &str) -> ModalResult<String> {
    let first = one_of(|c: char| c.is_ascii_alphabetic() || c == '_').parse_next(input)?;
    let rest: &str = take_while(0.., |c: char| c.is_ascii_alphanumeric() || c == '_').parse_next(input)?;
    
    let mut mnemonic = String::with_capacity(1 + rest.len());
    mnemonic.push(first);
    mnemonic.push_str(rest);
    Ok(mnemonic)
}

/// Parse operand list: operand ("," operand)*
fn operand_list_parser(input: &mut &str) -> ModalResult<Vec<Operand>> {
    // Handle empty operand list
    let _ = take_while(0.., |c: char| c.is_whitespace()).parse_next(input)?;
    
    if input.starts_with(')') {
        return Ok(Vec::new());
    }
    
    separated(1.., operand_parser, ',').parse_next(input)
}

/// Parse a single operand: "?" | OPERAND_STRING
fn operand_parser(input: &mut &str) -> ModalResult<Operand> {
    // Skip leading whitespace
    let _ = take_while(0.., |c: char| c.is_whitespace()).parse_next(input)?;
    
    // Check for inferred operand
    if input.starts_with('?') {
        let _ = any.parse_next(input)?; // consume '?'
        return Ok(Operand::inferred());
    }
    
    // Parse operand value (handles nested parens for expressions)
    let value = operand_value_parser.parse_next(input)?;
    
    // Skip trailing whitespace
    let _ = take_while(0.., |c: char| c.is_whitespace()).parse_next(input)?;
    
    Ok(Operand::value(value))
}

/// Parse operand value string, handling nested parentheses and brackets
fn operand_value_parser(input: &mut &str) -> ModalResult<String> {
    let mut result = String::new();
    let mut paren_depth = 0;
    let mut bracket_depth = 0;
    
    while !input.is_empty() {
        let c = input.chars().next().unwrap();
        
        match c {
            '(' => {
                paren_depth += 1;
                result.push(c);
                *input = &input[1..];
            }
            ')' => {
                if paren_depth == 0 {
                    // End of operand (instruction closing paren)
                    break;
                }
                paren_depth -= 1;
                result.push(c);
                *input = &input[1..];
            }
            '[' => {
                // Array index access - track depth
                bracket_depth += 1;
                result.push(c);
                *input = &input[1..];
            }
            ']' => {
                if bracket_depth == 0 {
                    // This is a parallel branch end, not array close
                    break;
                }
                bracket_depth -= 1;
                result.push(c);
                *input = &input[1..];
            }
            ',' => {
                if paren_depth == 0 && bracket_depth == 0 {
                    // End of operand (operand separator)
                    break;
                }
                result.push(c);
                *input = &input[1..];
            }
            _ => {
                result.push(c);
                *input = &input[1..];
            }
        }
    }
    
    let trimmed = result.trim().to_string();
    if trimmed.is_empty() {
        return Err(winnow::error::ErrMode::Backtrack(winnow::error::ContextError::new()));
    }
    
    Ok(trimmed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_nop() {
        let rung = parse_rung("NOP();");
        assert!(rung.is_parsed());
        let content = rung.content.unwrap();
        assert_eq!(content.elements.len(), 1);
        
        if let RungElement::Instruction(instr) = &content.elements[0] {
            assert_eq!(instr.mnemonic, "NOP");
            assert!(instr.operands.is_empty());
        } else {
            panic!("Expected instruction");
        }
    }

    #[test]
    fn test_parse_simple_instruction() {
        let rung = parse_rung("XIC(Start);");
        assert!(rung.is_parsed());
        let content = rung.content.unwrap();
        assert_eq!(content.elements.len(), 1);
        
        if let RungElement::Instruction(instr) = &content.elements[0] {
            assert_eq!(instr.mnemonic, "XIC");
            assert_eq!(instr.operands.len(), 1);
            assert_eq!(instr.operands[0], Operand::value("Start"));
        } else {
            panic!("Expected instruction");
        }
    }

    #[test]
    fn test_parse_series() {
        let rung = parse_rung("XIC(Start)XIC(Ready)OTE(Motor);");
        assert!(rung.is_parsed());
        let content = rung.content.unwrap();
        assert_eq!(content.elements.len(), 3);
    }

    #[test]
    fn test_parse_multiple_operands() {
        let rung = parse_rung("MOV(Source,Dest);");
        assert!(rung.is_parsed());
        let content = rung.content.unwrap();
        
        if let RungElement::Instruction(instr) = &content.elements[0] {
            assert_eq!(instr.mnemonic, "MOV");
            assert_eq!(instr.operands.len(), 2);
            assert_eq!(instr.operands[0], Operand::value("Source"));
            assert_eq!(instr.operands[1], Operand::value("Dest"));
        } else {
            panic!("Expected instruction");
        }
    }

    #[test]
    fn test_parse_inferred_operands() {
        let rung = parse_rung("TON(Timer1,?,?);");
        assert!(rung.is_parsed());
        let content = rung.content.unwrap();
        
        if let RungElement::Instruction(instr) = &content.elements[0] {
            assert_eq!(instr.mnemonic, "TON");
            assert_eq!(instr.operands.len(), 3);
            assert_eq!(instr.operands[0], Operand::value("Timer1"));
            assert_eq!(instr.operands[1], Operand::inferred());
            assert_eq!(instr.operands[2], Operand::inferred());
        } else {
            panic!("Expected instruction");
        }
    }

    #[test]
    fn test_parse_parallel_branches() {
        let rung = parse_rung("XIC(Start)[OTE(Motor),OTE(Light)];");
        assert!(rung.is_parsed());
        let content = rung.content.unwrap();
        assert_eq!(content.elements.len(), 2);
        
        // First element: XIC instruction
        if let RungElement::Instruction(instr) = &content.elements[0] {
            assert_eq!(instr.mnemonic, "XIC");
        } else {
            panic!("Expected instruction");
        }
        
        // Second element: parallel branches
        if let RungElement::Parallel(branches) = &content.elements[1] {
            assert_eq!(branches.len(), 2);
            assert_eq!(branches[0].elements.len(), 1);
            assert_eq!(branches[1].elements.len(), 1);
        } else {
            panic!("Expected parallel");
        }
    }

    #[test]
    fn test_parse_nested_parallel() {
        let rung = parse_rung("XIC(A)[XIC(B)[OTE(C),OTE(D)],OTE(E)];");
        assert!(rung.is_parsed());
    }

    #[test]
    fn test_parse_structured_tag() {
        let rung = parse_rung("XIC(Timer1.DN)OTE(Motor.Run);");
        assert!(rung.is_parsed());
        
        let refs = rung.tag_references();
        assert_eq!(refs.len(), 2);
        // Now extracts base tag names
        assert_eq!(refs[0].name, "Timer1");
        assert_eq!(refs[0].full_operand, "Timer1.DN");
        assert_eq!(refs[1].name, "Motor");
        assert_eq!(refs[1].full_operand, "Motor.Run");
    }

    #[test]
    fn test_parse_array_access() {
        let rung = parse_rung("XIC(Data[0])MOV(Array[1],Array[2]);");
        assert!(rung.is_parsed());
        
        let refs = rung.tag_references();
        // Now extracts base tag names only (indices are literals)
        assert_eq!(refs.len(), 3);
        assert_eq!(refs[0].name, "Data");
        assert_eq!(refs[0].full_operand, "Data[0]");
        assert_eq!(refs[1].name, "Array");
        assert_eq!(refs[2].name, "Array");
    }

    #[test]
    fn test_parse_expression_operand() {
        // CPT instruction with expression
        let rung = parse_rung("CPT(Result,((1.0 - x) * y) + z);");
        assert!(rung.is_parsed());
        let content = rung.content.unwrap();
        
        if let RungElement::Instruction(instr) = &content.elements[0] {
            assert_eq!(instr.mnemonic, "CPT");
            assert_eq!(instr.operands.len(), 2);
            assert_eq!(instr.operands[1], Operand::value("((1.0 - x) * y) + z"));
        } else {
            panic!("Expected instruction");
        }
    }

    #[test]
    fn test_parse_complex_real_example() {
        // From actual L5X file
        let rung = parse_rung("XIC(First_Scan)[XIC(Run_Cmd) XIO(Run_Mode) OTL(Run_Mode) ,XIO(Run_Cmd) OTU(Run_Mode) ];");
        assert!(rung.is_parsed(), "Error: {:?}", rung.error);
        let content = rung.content.unwrap();
        
        // Should have XIC instruction followed by parallel branches
        assert_eq!(content.elements.len(), 2);
    }

    #[test]
    fn test_parse_empty_rung() {
        let rung = parse_rung("");
        assert!(rung.is_parsed());
        assert!(rung.content.unwrap().elements.is_empty());
    }

    #[test]
    fn test_parse_whitespace_only() {
        let rung = parse_rung("  \n  ");
        assert!(rung.is_parsed());
    }

    #[test]
    fn test_permissive_missing_terminator() {
        use crate::rll::error::RllError;
        
        let rung = parse_rung("XIC(Start)");
        // Permissive mode: returns error in rung.error but doesn't panic
        assert!(!rung.is_parsed());
        assert!(rung.error.is_some());
        assert_eq!(rung.error.unwrap(), RllError::MissingTerminator);
    }

    #[test]
    fn test_tag_extraction() {
        let rung = parse_rung("XIC(Start)XIC(Ready)[OTE(Motor),TON(Timer1,?,?)];");
        assert!(rung.is_parsed());
        
        let refs = rung.tag_references();
        assert_eq!(refs.len(), 4);
        
        let names: Vec<&str> = refs.iter().map(|r| r.name.as_str()).collect();
        assert!(names.contains(&"Start"));
        assert!(names.contains(&"Ready"));
        assert!(names.contains(&"Motor"));
        assert!(names.contains(&"Timer1"));
    }

    #[test]
    fn test_parse_real_l5x_rungs() {
        // These are actual RLL text strings from data/Random_AOI_Test_01042020.L5X
        let real_rungs = [
            "NOP();",
            "MUL(Seed,A,Se)ADD(Se,B,Se)OR(Se,-2147483648,Seed_Out);",
            "OTU(S:V);",
            "DIV(Seed_Out,-2147483648.0,float_internal)CPT(Output_scaled,((1.0 - float_internal) * Out_lo) + (float_internal * Out_hi));",
            "XIO(AOI_FS)MOV(0,B_URNG.Seed_Out);",
            "[XIO(Odd_Even) ,XIO(AOI_FS) ]A_Uniform_RNG(A_URNG,B_URNG.Seed_Out,A_URNG.A,A_URNG.B,A_URNG.Out_lo,A_URNG.Out_hi,A_URNG.Output_scaled,Seed_A)A_Uniform_RNG(B_URNG,A_URNG.Seed_Out,B_URNG.A,B_URNG.B,B_URNG.Out_lo,B_URNG.Out_hi,B_URNG.Output_scaled,Seed_B);",
            "[XIO(Odd_Even) ,XIO(AOI_FS) ]LN(A_URNG.Output_scaled,Z1)MUL(Z1,-2.0,Z1)SQR(Z1,Z1);",
        ];

        for (i, rung_text) in real_rungs.iter().enumerate() {
            let rung = parse_rung(rung_text);
            assert!(
                rung.is_parsed(),
                "Failed to parse real rung {}: {:?}\nInput: {}",
                i,
                rung.error,
                rung_text
            );
        }
    }
}
