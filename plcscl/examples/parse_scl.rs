use plcscl::parse_scl;
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

    match parse_scl(&content) {
        Ok(program) => {
            println!("✓ Successfully parsed {}", filename);
            println!("  Blocks: {}", program.blocks.len());
        }
        Err(e) => {
            eprintln!("✗ Parse error in {}:\n{}", filename, e.message());
            std::process::exit(1);
        }
    }
}
