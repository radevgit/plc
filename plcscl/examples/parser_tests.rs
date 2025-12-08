use plcscl::parse_scl;

fn main() {
    let tests = vec![
        ("Simple assignment", r#"
FUNCTION_BLOCK Test
VAR_INPUT x : INT; END_VAR
BEGIN
    x := x;
END_FUNCTION_BLOCK
"#),
        ("FOR loop", r#"
FUNCTION_BLOCK Test
VAR_TEMP i : INT; END_VAR
BEGIN
    FOR i := 1 TO 10 DO
        i := i;
    END_FOR;
END_FUNCTION_BLOCK
"#),
        ("IF statement", r#"
FUNCTION_BLOCK Test
VAR_INPUT x : INT; END_VAR
BEGIN
    IF x > 0 THEN
        x := 1;
    END_IF;
END_FUNCTION_BLOCK
"#),
    ];

    println!("Running {} parser tests...\n", tests.len());
    
    let mut passed = 0;
    let mut failed = 0;
    
    for (name, source) in &tests {
        print!("Test '{}': ", name);
        match parse_scl(source) {
            Ok(_) => {
                println!("✓ PASS");
                passed += 1;
            }
            Err(e) => {
                println!("✗ FAIL");
                println!("  Error: {}", e.message());
                failed += 1;
            }
        }
    }
    
    println!("\n=== Results ===");
    println!("Passed: {}/{}", passed, tests.len());
    println!("Failed: {}/{}", failed, tests.len());
    
    if failed > 0 {
        std::process::exit(1);
    }
}
