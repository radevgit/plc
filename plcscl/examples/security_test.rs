use plcscl::{parse_scl, parse_scl_secure, security::ParserLimits};

fn main() {
    println!("=== SCL Parser Security Tests ===\n");
    
    // Test 1: Normal input passes
    let normal = "FUNCTION_BLOCK Test VAR_INPUT x : INT; END_VAR BEGIN x := x; END_FUNCTION_BLOCK";
    match parse_scl(normal) {
        Ok(_) => println!("✓ Normal input: PASS"),
        Err(e) => println!("✗ Normal input: FAIL - {}", e.message()),
    }
    
    // Test 2: Token limit - too many tokens (5K assignments = 20K tokens, exceeds strict limit of 10K iterations)
    let mut many_tokens = String::from("FUNCTION_BLOCK Test BEGIN ");
    for _ in 0..5_000 {
        many_tokens.push_str("x := x ; ");
    }
    many_tokens.push_str("END_FUNCTION_BLOCK");
    
    match parse_scl_secure(&many_tokens, ParserLimits::strict()) {
        Ok(_) => println!("✗ Token limit: FAIL - should have been rejected"),
        Err(e) => {
            let msg = format!("{}", e);
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
    match parse_scl_secure(&deep_nesting, ParserLimits::strict()) {
        Ok(_) => println!("? Deep nesting: PASS (within limits)"),
        Err(e) => {
            let msg = format!("{}", e);
            if msg.contains("depth") || msg.contains("nesting") {
                println!("[PASS] Deep nesting: PASS - {}", msg);
            } else {
                println!("? Deep nesting: ERROR - {}", msg);
            }
        }
    }
    
    // Test 4: Input size limit
    let huge_input = "x".repeat(11 * 1024 * 1024); // 11 MB exceeds strict limit of 10 MB
    
    match parse_scl_secure(&huge_input, ParserLimits::strict()) {
        Ok(_) => println!("[FAIL] Input size limit: should have been rejected"),
        Err(e) => {
            let msg = format!("{}", e);
            if msg.contains("Input too large") || msg.contains("size") {
                println!("[PASS] Input size limit: {}", msg);
            } else {
                println!("[PARTIAL] Input size limit: {}", msg);
            }
        }
    }
    
    // Test 5: Compare limits
    println!("\\n=== Limit Comparisons ===");
    let limits_default = ParserLimits::default();
    let limits_strict = ParserLimits::strict();
    let limits_relaxed = ParserLimits::relaxed();
    
    println!("Default:  max_input_size={}MB, max_iterations={}, max_depth={}",
        limits_default.max_input_size / 1024 / 1024, limits_default.max_iterations, limits_default.max_depth);
    println!("Strict:   max_input_size={}MB, max_iterations={}, max_depth={}",
        limits_strict.max_input_size / 1024 / 1024, limits_strict.max_iterations, limits_strict.max_depth);
    println!("Relaxed:  max_input_size={}MB, max_iterations={}, max_depth={}",
        limits_relaxed.max_input_size / 1024 / 1024, limits_relaxed.max_iterations, limits_relaxed.max_depth);
    
    println!("\\n=== Security Tests Complete ===");
}
