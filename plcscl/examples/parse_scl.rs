use plcscl::parse::{SclParser, Rule};
use pest::Parser;
use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <scl_file>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    let content = fs::read_to_string(filename)
        .unwrap_or_else(|e| {
            eprintln!("Failed to read {}: {}", filename, e);
            std::process::exit(1);
        });

    match SclParser::parse(Rule::program, &content) {
        Ok(_pairs) => {
            println!("✓ Successfully parsed {}", filename);
        }
        Err(e) => {
            eprintln!("✗ Parse error in {}:\n{}", filename, e);
            std::process::exit(1);
        }
    }
}
