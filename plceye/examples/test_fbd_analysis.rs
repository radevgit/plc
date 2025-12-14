//! Test FBD reference extraction

use plceye::analysis::{analyze_plcopen_project, PlcopenAnalysis};
use std::fs;

fn main() {
    let file_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| {
            eprintln!("Usage: cargo run --example test_fbd_analysis <file.xml>");
            std::process::exit(1);
        });

    println!("Analyzing: {}", file_path);
    let xml_content = fs::read_to_string(&file_path)
        .expect("Failed to read file");

    let project: plcopen::Root_project_Inline = quick_xml::de::from_str(&xml_content)
        .expect("Failed to parse XML");

    let analysis = analyze_plcopen_project(&project);

    println!("\n=== Project Analysis ===");
    println!("POUs: {}", analysis.stats.pous);
    println!("  - Programs: {}", analysis.stats.programs);
    println!("  - Function Blocks: {}", analysis.stats.function_blocks);
    println!("  - Functions: {}", analysis.stats.functions);
    println!("\nBodies by language:");
    println!("  - ST: {}", analysis.stats.st_bodies);
    println!("  - FBD: {}", analysis.stats.fbd_bodies);
    println!("  - LD: {}", analysis.stats.ld_bodies);
    println!("  - SFC: {}", analysis.stats.sfc_bodies);
    println!("  - IL: {}", analysis.stats.il_bodies);
    println!("\nVariables:");
    println!("  - Defined: {}", analysis.defined_variables.len());
    println!("  - Used: {}", analysis.used_variables.len());
    println!("  - Unused: {}", analysis.unused_variables().len());
    println!("  - Undefined: {}", analysis.undefined_variables().len());
    println!("\nPOUs:");
    println!("  - Called: {}", analysis.used_pous.len());
    println!("  - Empty: {}", analysis.empty_pous.len());

    // Show used POUs (from FBD blocks)
    if !analysis.used_pous.is_empty() {
        println!("\n=== Called POUs (from FBD blocks) ===");
        let mut pous: Vec<_> = analysis.used_pous.iter().collect();
        pous.sort();
        for pou in pous {
            println!("  - {}", pou);
        }
    }

    // Show used variables
    if !analysis.used_variables.is_empty() {
        println!("\n=== Used Variables (sample, first 20) ===");
        let mut vars: Vec<_> = analysis.used_variables.iter().collect();
        vars.sort();
        for var in vars.iter().take(20) {
            println!("  - {}", var);
        }
        if vars.len() > 20 {
            println!("  ... and {} more", vars.len() - 20);
        }
    }

    // Show unused variables
    if !analysis.unused_variables().is_empty() {
        println!("\n=== Unused Variables ===");
        let mut unused = analysis.unused_variables();
        unused.sort_by_key(|v| &v.name);
        for var in unused {
            println!("  - {} (in {})", var.name, var.pou_name);
        }
    }

    // Show undefined variables
    if !analysis.undefined_variables().is_empty() {
        println!("\n=== Undefined Variables ===");
        let mut undefined = analysis.undefined_variables();
        undefined.sort();
        for var in undefined {
            println!("  - {}", var);
        }
    }
}
