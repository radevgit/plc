use plcscl::{parse_scl, parse_scl_secure, security::ParserLimits};
use std::fs;
use std::path::PathBuf;

fn main() {
    let corpus_dir = PathBuf::from("/home/ross/devpublic/dataplc/SCL");
    
    println!("Testing plcscl parser with real files...\n");
    
    let mut total = 0;
    let mut passed = 0;
    let mut failed = 0;
    let mut security_blocked = 0;
    
    // Test with balanced limits
    let limits = ParserLimits::balanced();
    
    for entry in walkdir::WalkDir::new(&corpus_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension()
                .and_then(|s| s.to_str())
                .map(|s| s == "scl" || s == "st")
                .unwrap_or(false)
        })
    {
        total += 1;
        let path = entry.path();
        let rel_path = path.strip_prefix(&corpus_dir).unwrap_or(path);
        
        match fs::read_to_string(path) {
            Ok(content) => {
                // First test with security
                match parse_scl_secure(&content, limits.clone()) {
                    Ok(_) => {
                        passed += 1;
                        println!("✓ {}", rel_path.display());
                    }
                    Err(e) => {
                        if e.to_string().contains("Security") {
                            security_blocked += 1;
                            println!("🛡️ {} - {}", rel_path.display(), e);
                        } else {
                            failed += 1;
                            println!("✗ {} - {}", rel_path.display(), e);
                        }
                    }
                }
            }
            Err(e) => {
                failed += 1;
                println!("✗ {} - Failed to read: {}", rel_path.display(), e);
            }
        }
    }
    
    println!("\n=== Results ===");
    println!("Total files: {}", total);
    println!("Passed: {} ({:.1}%)", passed, (passed as f64 / total as f64) * 100.0);
    println!("Failed: {} ({:.1}%)", failed, (failed as f64 / total as f64) * 100.0);
    println!("Security blocked: {} ({:.1}%)", security_blocked, (security_blocked as f64 / total as f64) * 100.0);
}
