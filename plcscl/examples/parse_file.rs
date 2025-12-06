use plcscl::{Lexer, Parser};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <scl_file>", args[0]);
        eprintln!("\nExample:");
        eprintln!("  {} test.scl", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    let source = fs::read_to_string(filename)?;

    println!("Parsing {}...", filename);
    println!("Source size: {} bytes", source.len());
    println!();

    let lexer = Lexer::new(&source);
    let mut parser = Parser::new(lexer)?.with_error_recovery();
    
    match parser.parse() {
        Ok(ast) => {
            // Check if there were any errors during parsing
            if !parser.errors.is_empty() {
                println!("⚠ Parsed with {} error(s):", parser.errors.len());
                for (i, err) in parser.errors.iter().enumerate() {
                    println!("  {}. {}", i + 1, err);
                }
                println!();
            } else {
                println!("✓ Parse successful!");
                println!();
            }
            
            println!("Blocks found: {}", ast.blocks.len());
            
            for (i, block) in ast.blocks.iter().enumerate() {
                println!("\nBlock #{}: {:?} '{}'", i + 1, block.kind, block.name);
                println!("  Pragmas: {}", block.pragmas.len());
                println!("  Variable sections: {}", block.variables.len());
                
                let total_vars: usize = block.variables.iter()
                    .map(|s| s.variables.len())
                    .sum();
                println!("  Total variables: {}", total_vars);
                println!("  Statements: {}", block.body.len());
                
                if let Some(ret) = &block.return_type {
                    println!("  Return type: {:?}", ret);
                }
            }
            
            println!("\n✓ Successfully parsed {} blocks", ast.blocks.len());
            
            if !parser.errors.is_empty() {
                println!("⚠ {} error(s) encountered during parsing", parser.errors.len());
                std::process::exit(2); // Exit with different code to indicate partial success
            }
        }
        Err(e) => {
            eprintln!("✗ Parse error: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
