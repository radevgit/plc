//! plcviz CLI - Generate SVG diagrams from L5X files

use std::path::PathBuf;
use std::fs;

use plcviz::L5xGraph;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return;
    }

    match args[1].as_str() {
        "help" | "--help" | "-h" => print_usage(),
        "example" => run_example(),
        path => generate_from_l5x(path),
    }
}

fn print_usage() {
    eprintln!("plcviz - Generate SVG diagrams from L5X files");
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  plcviz <file.l5x>        Generate call graph SVG");
    eprintln!("  plcviz example           Generate example SVG");
    eprintln!("  plcviz --help            Show this help");
    eprintln!();
    eprintln!("Output:");
    eprintln!("  Writes SVG to stdout, redirect to file:");
    eprintln!("  plcviz project.l5x > graph.svg");
}

fn run_example() {
    let mut graph = L5xGraph::new();
    
    // Programs
    graph.add_program("MainProgram");
    graph.add_program("CommProgram");
    
    // Routines under MainProgram
    graph.add_routine("MainProgram", "MainRoutine");
    graph.add_routine("MainProgram", "InitSequence");
    graph.add_routine("MainProgram", "FaultHandler");
    graph.add_routine("MainProgram", "MotorControl");
    
    // Routines under CommProgram
    graph.add_routine("CommProgram", "EthernetComm");
    
    // Call edges (using full routine IDs)
    graph.add_call("MainProgram.MainRoutine", "MainProgram.InitSequence");
    graph.add_call("MainProgram.MainRoutine", "MainProgram.MotorControl");
    graph.add_call("MainProgram.MainRoutine", "MainProgram.FaultHandler");
    graph.add_call("MainProgram.MotorControl", "MainProgram.FaultHandler");
    
    let svg = graph.render_svg();
    println!("{}", svg);
}

fn generate_from_l5x(path: &str) {
    let path = PathBuf::from(path);
    
    if !path.exists() {
        eprintln!("Error: File not found: {}", path.display());
        std::process::exit(1);
    }
    
    // Read and parse L5X
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            std::process::exit(1);
        }
    };
    
    let project: l5x::Project = match l5x::from_str(&content) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error parsing L5X: {}", e);
            std::process::exit(1);
        }
    };
    
    // Build graph from L5X structure
    let mut graph = L5xGraph::new();
    
    // Add programs and routines from controller
    if let Some(ref controller) = project.controller {
        if let Some(ref programs) = controller.programs {
            for program in &programs.program {
                let prog_name = &program.name;
                graph.add_program(prog_name);
                
                if let Some(ref routines) = program.routines {
                    for routine in &routines.routine {
                        graph.add_routine(prog_name, &routine.name);
                        
                        // Add containment edge from program to routine
                        let routine_id = format!("{}.{}", prog_name, routine.name);
                        graph.add_edge(prog_name, &routine_id, None);
                    }
                }
            }
        }
    }
    
    // TODO: Extract JSR calls from routines to build call edges
    // This requires parsing the RLL/ST content
    
    let svg = graph.render_svg();
    println!("{}", svg);
}
