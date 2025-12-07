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
        ("Expression precedence", r#"
FUNCTION_BLOCK Test
VAR_INPUT a : INT; b : INT; c : INT; END_VAR
VAR_OUTPUT result : INT; END_VAR
BEGIN
    result := a;
END_FUNCTION_BLOCK
"#),
        ("FOR loop", r#"
FUNCTION_BLOCK Test
VAR_TEMP i : INT; END_VAR
BEGIN
    FOR i := i TO i DO
        i := i;
    END_FOR;
END_FUNCTION_BLOCK
"#),
        ("WHILE loop", r#"
FUNCTION_BLOCK Test
VAR_INPUT x : INT; END_VAR
BEGIN
    WHILE x DO
        x := x;
    END_WHILE;
END_FUNCTION_BLOCK
"#),
        ("REPEAT loop", r#"
FUNCTION_BLOCK Test
VAR_INPUT x : INT; END_VAR
BEGIN
    REPEAT
        x := x;
    UNTIL x
    END_REPEAT;
END_FUNCTION_BLOCK
"#),
        ("CASE statement", r#"
FUNCTION_BLOCK Test
VAR_INPUT x : INT; END_VAR
BEGIN
    CASE x OF
        x : x := x;
    END_CASE;
END_FUNCTION_BLOCK
"#),
    ];
    
    let mut passed = 0;
    let mut failed = 0;
    
    for (name, source) in tests {
        print!("Testing {:<25} ... ", name);
        match parse_scl(source) {
            Ok(_) => {
                println!("✓ PASS");
                passed += 1;
            }
            Err(e) => {
                println!("✗ FAIL: {}", e.message);
                failed += 1;
            }
        }
    }
    
    println!("\n{} passed, {} failed", passed, failed);
    if failed > 0 {
        std::process::exit(1);
    }
}
