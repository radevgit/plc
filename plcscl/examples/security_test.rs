use plcscl::{Parser, ParserLimits};

fn main() {
    println!("=== SCL Parser Security Tests ===\n");
    
    // Test 1: Normal input passes
    let normal = "FUNCTION_BLOCK Test VAR_INPUT x : INT; END_VAR BEGIN x := x; END_FUNCTION_BLOCK";
    match Parser::new(normal).parse_program() {
        Ok(_) => println!("✓ Normal input: PASS"),
        Err(e) => println!("✗ Normal input: FAIL - {}", e.message()),
    }
    
    // Test 2: Token limit - too many tokens (5K assignments = 20K tokens, exceeds strict limit of 10K iterations)
    let mut many_tokens = String::from("FUNCTION_BLOCK Test BEGIN ");
    for _ in 0..5_000 {
        many_tokens.push_str("x := x ; ");
    }
    many_tokens.push_str("END_FUNCTION_BLOCK");
    
    match Parser::with_limits(&many_tokens, ParserLimits::strict()).parse_program() {
        Ok(_) => println!("✗ Token limit: FAIL - should have been rejected"),
        Err(e) => {
            let msg = e.message();
            if msg.contains("Iteration") || msg.contains("token") || msg.contains("Too many") {
                println!("✓ Token limit: PASS - {}", msg);
            } else {
                println!("? Token limit: PARTIAL - {}", msg);
            }
        }
    }
    
    // Test 3: Deep nesting
    let mut deep_nesting = String::from("FUNCTION_BLOCK Test BEGIN ");
    for _ in 0..100 {
        deep_nesting.push_str("IF x THEN ");
    }
    deep_nesting.push_str("x := x;");
    for _ in 0..100 {
        deep_nesting.push_str(" END_IF;");
    }
    deep_nesting.push_str(" END_FUNCTION_BLOCK");
    match Parser::with_limits(&deep_nesting, ParserLimits::strict()).parse_program() {
        Ok(_) => println!("? Deep nesting: PASS (within limits)"),
        Err(e) => {
            let msg = e.message();
            if msg.contains("Recursion") || msg.contains("depth") {
                println!("✓ Deep nesting: PASS - {}", msg);
            } else {
                println!("? Deep nesting: ERROR - {}", msg);
            }
        }
    }
    
    // Test 4: Many variables (collection size limit - 2K is enough to test)
    let mut many_vars = String::from("FUNCTION_BLOCK Test VAR_INPUT ");
    for i in 0..2_000 {
        many_vars.push_str(&format!("x{} : INT; ", i));
    }
    many_vars.push_str("END_VAR BEGIN x0 := x0; END_FUNCTION_BLOCK");
    
    match Parser::with_limits(&many_vars, ParserLimits::strict()).parse_program() {
        Ok(_) => println!("✗ Collection limit: FAIL - should have been rejected"),
        Err(e) => {
            let msg = e.message();
            if msg.contains("Too many") || msg.contains("exceeds") {
                println!("✓ Collection limit: PASS - {}", msg);
            } else {
                println!("? Collection limit: PARTIAL - {}", msg);
            }
        }
    }
    
    // Test 5: Compare limits
    println!("\n=== Limit Comparisons ===");
    let limits_default = ParserLimits::default();
    let limits_strict = ParserLimits::strict();
    let limits_relaxed = ParserLimits::relaxed();
    
    println!("Default:  max_tokens={}, max_iterations={}, max_recursion={}",
        limits_default.max_tokens, limits_default.max_iterations, limits_default.max_recursion_depth);
    println!("Strict:   max_tokens={}, max_iterations={}, max_recursion={}",
        limits_strict.max_tokens, limits_strict.max_iterations, limits_strict.max_recursion_depth);
    println!("Relaxed:  max_tokens={}, max_iterations={}, max_recursion={}",
        limits_relaxed.max_tokens, limits_relaxed.max_iterations, limits_relaxed.max_recursion_depth);
    
    println!("\n=== Security Tests Complete ===");
}
