// Build script to generate parser code from IEC 61131-3 EBNF specification
// This runs at compile time and generates lexer.rs, ast.rs, and parser.rs

use std::env;
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
    
    // Note: Parser code is pre-generated.
}
