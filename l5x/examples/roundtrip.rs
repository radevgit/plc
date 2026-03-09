//! Round-trip serialization test for L5X files.
//!
//! Parses an L5X file, serializes it back to XML, then re-parses
//! to verify correctness. Reports any differences or issues.
//!
//! Usage:
//!   cargo run --example roundtrip -- path/to/file.L5X
//!

use l5x::{from_str, to_string, Project};
use std::{env, fs, path::Path};
use walkdir::WalkDir;

const XML_HEADER: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#;

fn test_file(path: &Path) -> Result<RoundtripResult, String> {
    let original = fs::read_to_string(path)
        .map_err(|e| format!("read error: {e}"))?;

    // Step 1: parse
    let project: Project = from_str(&original)
        .map_err(|e| format!("parse error: {e}"))?;

    // Step 2: serialize
    let serialized = to_string(&project)
        .map_err(|e| format!("serialize error: {e}"))?;

    // Prepend XML declaration (quick-xml se doesn't add it automatically)
    let with_header = format!("{XML_HEADER}\n{serialized}");

    // Step 3: re-parse the serialized output
    let reparsed: Project = from_str(&with_header)
        .map_err(|e| format!("re-parse error: {e}\n--- serialized output (first 2000 chars) ---\n{}", &serialized[..serialized.len().min(2000)]))?;

    // Step 4: compare key fields
    let issues = compare_projects(&project, &reparsed);

    Ok(RoundtripResult {
        serialized_len: serialized.len(),
        original_len: original.len(),
        issues,
    })
}

struct RoundtripResult {
    original_len: usize,
    serialized_len: usize,
    issues: Vec<String>,
}

fn compare_projects(a: &Project, b: &Project) -> Vec<String> {
    let mut issues = Vec::new();

    if a.schema_revision != b.schema_revision {
        issues.push(format!(
            "schema_revision mismatch: {:?} vs {:?}",
            a.schema_revision, b.schema_revision
        ));
    }
    if a.software_revision != b.software_revision {
        issues.push(format!(
            "software_revision mismatch: {:?} vs {:?}",
            a.software_revision, b.software_revision
        ));
    }
    if a.target_name != b.target_name {
        issues.push(format!(
            "target_name mismatch: {:?} vs {:?}",
            a.target_name, b.target_name
        ));
    }

    // Check controller name if present
    if let (Some(ca), Some(cb)) = (&a.controller, &b.controller) {
        if ca.name != cb.name {
            issues.push(format!(
                "controller.name mismatch: {:?} vs {:?}",
                ca.name, cb.name
            ));
        }
    } else if a.controller.is_some() != b.controller.is_some() {
        issues.push("controller presence mismatch".to_string());
    }

    issues
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() >= 2 {
        // Single file mode
        let path = Path::new(&args[1]);
        print!("{} ... ", path.file_name().unwrap().to_string_lossy());
        match test_file(path) {
            Ok(r) => print_result(&r),
            Err(e) => println!("[FAIL] {e}"),
        }
        return;
    }

    // Corpus mode
    let corpus_dir = env::var("DATAPLC_DIR").unwrap_or_else(|_| {
        eprintln!("Set DATAPLC_DIR to a directory containing .L5X files.");
        std::process::exit(1);
    });

    println!("Round-trip serialization test on: {corpus_dir}\n");

    let mut total = 0;
    let mut passed = 0;
    let mut failed = 0;
    let mut warned = 0;
    let mut total_original_bytes: u64 = 0;
    let mut total_serialized_bytes: u64 = 0;

    for entry in WalkDir::new(&corpus_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s.eq_ignore_ascii_case("l5x"))
                .unwrap_or(false)
        })
    {
        total += 1;
        let path = entry.path();
        print!("{} ... ", path.file_name().unwrap().to_string_lossy());

        match test_file(path) {
            Ok(r) => {
                total_original_bytes += r.original_len as u64;
                total_serialized_bytes += r.serialized_len as u64;
                if r.issues.is_empty() {
                    passed += 1;
                } else {
                    warned += 1;
                }
                print_result(&r);
            }
            Err(e) => {
                failed += 1;
                println!("[FAIL] {e}");
            }
        }
    }

    println!();
    println!("Results: {total} files — {passed} passed, {warned} warned, {failed} failed");
    if total_original_bytes > 0 {
        let ratio = total_serialized_bytes as f64 / total_original_bytes as f64;
        println!(
            "Total size: {} KB -> {} KB ({:.1}%)",
            total_original_bytes / 1024,
            total_serialized_bytes / 1024,
            ratio * 100.0
        );
    }
}

fn print_result(r: &RoundtripResult) {
    let ratio = r.serialized_len as f64 / r.original_len as f64;
    if r.issues.is_empty() {
        println!("[OK] {:.1}% size ({} -> {} bytes)", ratio * 100.0, r.original_len, r.serialized_len);
    } else {
        println!("[WARN] {:.1}% size — {} field issue(s):", ratio * 100.0, r.issues.len());
        for issue in &r.issues {
            println!("       - {issue}");
        }
    }
}
