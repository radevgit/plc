use iecst::{parse_statements, security::ParserLimits};
use std::fs;
use std::env;
use walkdir::WalkDir;

fn main() {
    let corpus_dir = env::var("DATAPLC_DIR")
        .map(|p| format!("{}/SCL", p))
        .unwrap_or_else(|_| "../../../dataplc/SCL".to_string());
    
    println!("Testing iecst parser with real ST files...\n");
    
    let mut total = 0;
    let mut passed = 0;
    let mut failed = 0;
    let mut security_blocked = 0;
    
    // Test with balanced limits
    let limits = ParserLimits::balanced();
    
    for entry in WalkDir::new(&corpus_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension()
                .and_then(|s| s.to_str())
                .map(|s| s == "st")
                .unwrap_or(false)
        })
        .take(30) // Limit to first 30 files
    {
        total += 1;
        let path = entry.path();
        let rel_path = path.strip_prefix(&corpus_dir).unwrap_or(path);
        
        print!("Testing {} ... ", rel_path.display());
        
        match fs::read_to_string(path) {
            Ok(content) => {
                // Check input size limit
                if content.len() > limits.max_input_size {
                    security_blocked += 1;
                    println!("[SEC] Input too large: {} bytes", content.len());
                    continue;
                }
                
                // Test parsing
                match parse_statements(&content) {
                    Ok(_) => {
                        passed += 1;
                        println!("[PASS]");
                    }
                    Err(e) => {
                        failed += 1;
                        // Show only first line of error
                        let err_msg = format!("{:?}", e);
                        let first_line = err_msg.lines().next().unwrap_or(&err_msg);
                        println!("[FAIL] {}", first_line);
                    }
                }
            }
            Err(e) => {
                failed += 1;
                println!("[FAIL] Failed to read: {}", e);
            }
        }
    }
    
    println!("\n=== Results ===");
    println!("Total files: {}", total);
    println!("Passed: {} ({:.1}%)", passed, (passed as f64 / total as f64) * 100.0);
    println!("Failed: {} ({:.1}%)", failed, (failed as f64 / total as f64) * 100.0);
    println!("Security blocked: {} ({:.1}%)", security_blocked, (security_blocked as f64 / total as f64) * 100.0);
}
