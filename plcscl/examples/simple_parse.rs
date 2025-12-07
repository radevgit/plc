use plcscl::parse_scl;

fn main() {
    let source = r#"
FUNCTION_BLOCK MyFB
VAR_INPUT
    setpoint : REAL;
END_VAR
VAR_OUTPUT
    output : REAL;
END_VAR
BEGIN
    output := setpoint;
END_FUNCTION_BLOCK
"#;

    match parse_scl(source) {
        Ok(program) => {
            println!("✓ Successfully parsed!");
            println!("  Blocks: {}", program.blocks.len());
            for block in &program.blocks {
                match block {
                    plcscl::Block::FunctionBlock(fb) => {
                        println!("  FUNCTION_BLOCK '{}'", fb.name);
                        println!("    VAR sections: {}", fb.var_sections.len());
                        println!("    Statements: {}", fb.statements.len());
                    }
                    _ => {}
                }
            }
        }
        Err(e) => {
            eprintln!("✗ Parse error: {}", e.message);
            eprintln!("  At position: {:?}", e.span);
        }
    }
}
