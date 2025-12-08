use plcscl::parse_scl;
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

    match parse_scl(&source) {
        Ok(program) => {
            println!("✓ Successfully parsed!");
            println!("  Blocks: {}", program.blocks.len());
            
            for block in &program.blocks {
                match block {
                    plcscl::Block::FunctionBlock(fb) => {
                        println!("  FUNCTION_BLOCK '{}'", fb.name);
                        println!("    VAR sections: {}", fb.var_sections.len());
                        println!("    Statements: {}", fb.statements.len());
                    }
                    plcscl::Block::Function(f) => {
                        println!("  FUNCTION '{}'", f.name);
                        println!("    Statements: {}", f.statements.len());
                    }
                    _ => {
                        println!("  Other block type");
                    }
                }
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("✗ Parse error:");
            eprintln!("  {}", e.message());
            eprintln!("  At position: {:?}", e.span);
            std::process::exit(1);
        }
    }
}
