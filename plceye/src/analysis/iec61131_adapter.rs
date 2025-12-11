//! Adapter for iec61131 parser to provide iecst-like interface.
//!
//! This module bridges between iec61131's compilation-unit based parsing
//! and the routine-level parsing expected by plceye.

use iec61131::{Parser, ParseError as Iec61131ParseError, PouDeclaration, StatementList};

/// Parse error for iec61131
pub type ParseError = Iec61131ParseError;

/// Parsed POU (program organization unit)
///
/// This wraps iec61131's AST to provide the same interface as iecst's Pou.
#[derive(Debug)]
pub struct Pou {
    pub name: String,
    pub body: StatementList,
}

/// Parse a POU from ST source code.
///
/// This wraps the source in a PROGRAM/END_PROGRAM wrapper for parsing,
/// similar to what iecst::parse_pou did.
pub fn parse_pou(source: &str) -> Result<Pou, ParseError> {
    let mut parser = Parser::new(source);
    let cu = parser.parse()?;
    
    // Extract the first POU
    if let Some(decl) = cu.declarations.first() {
        match decl {
            PouDeclaration::Function(func) => Ok(Pou {
                name: func.name.clone(),
                body: func.body.clone(),
            }),
            PouDeclaration::FunctionBlock(fb) => {
                if let Some(body) = &fb.body {
                    Ok(Pou {
                        name: fb.name.clone(),
                        body: body.clone(),
                    })
                } else {
                    Err(ParseError {
                        message: "Function block has no body (abstract/interface)".to_string(),
                        span: fb.span,
                    })
                }
            }
            PouDeclaration::Program(prog) => Ok(Pou {
                name: prog.name.clone(),
                body: prog.body.clone(),
            }),
            _ => Err(ParseError {
                message: "Expected FUNCTION, FUNCTION_BLOCK, or PROGRAM".to_string(),
                span: cu.span,
            }),
        }
    } else {
        Err(ParseError {
            message: "Empty file - no POU declaration found".to_string(),
            span: cu.span,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pou_function() {
        let source = r#"
FUNCTION Test : INT
    Test := 42;
END_FUNCTION
"#;
        let pou = parse_pou(source).unwrap();
        assert_eq!(pou.name, "Test");
        assert_eq!(pou.body.len(), 1);
    }

    #[test]
    fn test_parse_pou_program() {
        let source = r#"
PROGRAM Main
    VAR
        x : INT;
    END_VAR
    x := 1;
END_PROGRAM
"#;
        let pou = parse_pou(source).unwrap();
        assert_eq!(pou.name, "Main");
        assert_eq!(pou.body.len(), 1);
    }
}
