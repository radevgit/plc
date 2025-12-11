// Build script to generate parser code from IEC 61131-3 EBNF specification
// This runs at compile time and generates lexer.rs, ast.rs, and parser.rs

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Get the path to the generator crate
    let generator_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("plcp")
        .join("iec61131");

    // Tell cargo to rerun if the generator code changes
    println!("cargo:rerun-if-changed={}", generator_path.join("src").display());
    println!(
        "cargo:rerun-if-changed={}",
        generator_path.join("schema/iec61131-3_2013_official.ebnf").display()
    );

    // Read the EBNF grammar
    let ebnf_path = generator_path.join("schema/iec61131-3_2013_official.ebnf");
    let ebnf_content = fs::read_to_string(&ebnf_path).expect("Failed to read EBNF file");

    // Generate the parser code using plcp/iec61131
    // We need to inline the generator code since we can't depend on it directly
    // For now, just copy the generated code manually
    println!("cargo:warning=Build script running - parser generation not yet implemented");
    println!(
        "cargo:warning=Generator path: {}",
        generator_path.display()
    );
    println!("cargo:warning=EBNF path: {}", ebnf_path.display());
}
