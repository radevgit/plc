//! Test FBD parsing with real PLCopen files

use plcopen::Root_project_Inline;
use std::fs;

fn main() {
    let file_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| {
            eprintln!("Usage: cargo run --example test_fbd_parsing <file.xml>");
            std::process::exit(1);
        });

    println!("Parsing: {}", file_path);
    let xml_content = fs::read_to_string(&file_path)
        .expect("Failed to read file");

    let root: Root_project_Inline = quick_xml::de::from_str(&xml_content)
        .expect("Failed to parse XML");

    println!("✓ Successfully parsed PLCopen project");
    let project_name = root.content_header.as_ref()
        .map(|h| h.name.as_str())
        .unwrap_or("<unnamed>");
    println!("  Name: {}", project_name);

    // Check for FBD bodies
    let mut fbd_count = 0;
    let mut ld_count = 0;
    let mut sfc_count = 0;
    let mut st_count = 0;

    if let Some(ref types) = root.types {
        if let Some(ref pous) = types.pous {
            for pou in &pous.pou {
                println!("\nPOU: {}", pou.name);
                for body_elem in &pou.body {
                    if let Some(ref fbd) = body_elem.fbd {
                        fbd_count += 1;
                        println!("  ✓ FBD body found");
                        
                        // Count FBD elements
                        println!("    - blocks: {}", fbd.block.len());
                        println!("    - inVariable: {}", fbd.in_variable.len());
                        println!("    - outVariable: {}", fbd.out_variable.len());
                        println!("    - inOutVariable: {}", fbd.in_out_variable.len());
                        println!("    - connectors: {}", fbd.connector.len());
                        println!("    - continuations: {}", fbd.continuation.len());
                        
                        // Show first block if exists
                        if let Some(first_block) = fbd.block.first() {
                            println!("    First block:");
                            println!("      - localId: {}", first_block.local_id);
                            println!("      - typeName: {:?}", first_block.type_name);
                            if let Some(ref instance_name) = first_block.instance_name {
                                println!("      - instanceName: {}", instance_name);
                            }
                        }
                        
                        // Show first inVariable if exists
                        if let Some(first_var) = fbd.in_variable.first() {
                            println!("    First inVariable:");
                            println!("      - localId: {}", first_var.local_id);
                            if let Some(ref expression) = first_var.expression {
                                println!("      - expression: {}", expression);
                            }
                        }
                    }
                    
                    if body_elem.ld.is_some() {
                        ld_count += 1;
                        println!("  ✓ LD body found");
                    }
                    if body_elem.sfc.is_some() {
                        sfc_count += 1;
                        println!("  ✓ SFC body found");
                    }
                    if body_elem.st.is_some() {
                        st_count += 1;
                        println!("  ✓ ST body found");
                    }
                }
            }
        }
    }

    println!("\n=== Summary ===");
    println!("FBD bodies: {}", fbd_count);
    println!("LD bodies: {}", ld_count);
    println!("SFC bodies: {}", sfc_count);
    println!("ST bodies: {}", st_count);
}
