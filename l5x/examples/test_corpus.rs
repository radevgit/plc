use l5x::{from_str_secure, security::SecurityLimits};
use std::fs;
use std::env;
use walkdir::WalkDir;

fn main() {
    let corpus_dir = env::var("DATAPLC_DIR")
        .unwrap_or_else(|_| "../../../dataplc".to_string());
    
    println!("Testing l5x parser with real files...\n");
    
    let mut total = 0;
    let mut passed = 0;
    let mut failed = 0;
    let mut security_blocked = 0;
    
    // Test with balanced limits
    let limits = SecurityLimits::balanced();
    
    for entry in WalkDir::new(corpus_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension()
                .and_then(|s| s.to_str())
                .map(|s| s.eq_ignore_ascii_case("l5x"))
                .unwrap_or(false)
        })
        .take(20) // Limit to first 20 files
    {
        total += 1;
        let path = entry.path();
        
        print!("Testing {} ... ", path.file_name().unwrap().to_string_lossy());
        
        match fs::read_to_string(path) {
            Ok(content) => {
                // Test with security - parse as generic serde Value to avoid type issues
                match from_str_secure::<serde::de::IgnoredAny>(&content, &limits) {
                    Ok(_) => {
                        passed += 1;
                        println!("[PASS]");
                    }
                    Err(e) => {
                        if e.to_string().contains("Security") {
                            security_blocked += 1;
                            println!("[SEC] {}", e);
                        } else {
                            failed += 1;
                            // Show only first line of error
                            let err_msg = e.to_string();
                            let first_line = err_msg.lines().next().unwrap_or(&err_msg);
                            println!("[FAIL] {}", first_line);
                        }
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
