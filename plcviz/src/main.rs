//! plcviz CLI - Generate SVG diagrams from L5X and PLCopen files

use std::path::PathBuf;
use std::fs;

use clap::{Parser, Subcommand, ValueEnum};
use plcviz::{L5xGraph, L5xNodeType, GraphType, PlcopenGraphBuilder, PlcopenGraphType};

#[derive(Parser)]
#[command(name = "plcviz")]
#[command(version, about = "Generate SVG diagrams from L5X and PLCopen XML files", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// L5X or PLCopen XML file to visualize
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,

    /// Graph type to generate
    #[arg(short = 't', long = "type", value_name = "TYPE", default_value = "structure")]
    graph_type: GraphTypeArg,

    /// Include AOIs in the graph (L5X only)
    #[arg(short = 'a', long = "aois")]
    show_aois: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate an example graph (no L5X file needed)
    Example,
}

/// Graph type for CLI argument parsing
#[derive(Clone, Copy, Default, ValueEnum)]
enum GraphTypeArg {
    /// Containment hierarchy (Programs → Routines)
    #[default]
    Structure,
    /// Call graph (JSR calls between routines)
    Call,
    /// Data flow (Tag read/write relationships)
    Dataflow,
    /// Structure + call edges combined
    Combined,
}

impl From<GraphTypeArg> for GraphType {
    fn from(arg: GraphTypeArg) -> Self {
        match arg {
            GraphTypeArg::Structure => GraphType::Structure,
            GraphTypeArg::Call => GraphType::CallGraph,
            GraphTypeArg::Dataflow => GraphType::DataFlow,
            GraphTypeArg::Combined => GraphType::Combined,
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let graph_type: GraphType = cli.graph_type.into();

    // Handle subcommands
    if let Some(Commands::Example) = cli.command {
        run_example(graph_type);
        return;
    }

    // Need a file for normal operation
    if let Some(path) = cli.file {
        generate_from_l5x(path.to_str().unwrap(), graph_type, cli.show_aois);
    } else {
        eprintln!("Error: No input file specified");
        eprintln!("Usage: plcviz <FILE> or plcviz example");
        eprintln!("Try 'plcviz --help' for more information.");
        std::process::exit(1);
    }
}

fn run_example(graph_type: GraphType) {
    eprintln!("Generating example: {}", graph_type.description());
    
    let mut graph = L5xGraph::new();
    
    match graph_type {
        GraphType::Structure | GraphType::Combined => {
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
            
            // Structure edges
            graph.add_edge("MainProgram", "MainProgram.MainRoutine", None);
            graph.add_edge("MainProgram", "MainProgram.InitSequence", None);
            graph.add_edge("MainProgram", "MainProgram.FaultHandler", None);
            graph.add_edge("MainProgram", "MainProgram.MotorControl", None);
            graph.add_edge("CommProgram", "CommProgram.EthernetComm", None);
            
            if graph_type == GraphType::Combined {
                // Also add call edges
                graph.add_call("MainProgram.MainRoutine", "MainProgram.InitSequence");
                graph.add_call("MainProgram.MainRoutine", "MainProgram.MotorControl");
                graph.add_call("MainProgram.MotorControl", "MainProgram.FaultHandler");
            }
        }
        GraphType::CallGraph => {
            // Only routines and call edges
            graph.add_node("MainRoutine", "MainRoutine", L5xNodeType::Routine);
            graph.add_node("InitSequence", "InitSequence", L5xNodeType::Routine);
            graph.add_node("MotorControl", "MotorControl", L5xNodeType::Routine);
            graph.add_node("FaultHandler", "FaultHandler", L5xNodeType::Routine);
            graph.add_node("EthernetComm", "EthernetComm", L5xNodeType::Routine);
            graph.add_node("Motor_AOI", "Motor_AOI", L5xNodeType::Aoi);
            
            graph.add_call("MainRoutine", "InitSequence");
            graph.add_call("MainRoutine", "MotorControl");
            graph.add_call("MotorControl", "FaultHandler");
            graph.add_call("MotorControl", "Motor_AOI");
        }
        GraphType::DataFlow => {
            // Tags and their relationships
            graph.add_node("MotorCmd", "MotorCmd", L5xNodeType::Tag);
            graph.add_node("MotorFb", "MotorFb", L5xNodeType::Tag);
            graph.add_node("Speed", "Speed", L5xNodeType::Tag);
            graph.add_node("Fault", "Fault", L5xNodeType::Tag);
            graph.add_node("MotorControl", "MotorControl", L5xNodeType::Routine);
            graph.add_node("FaultHandler", "FaultHandler", L5xNodeType::Routine);
            
            // MotorControl reads Speed, writes MotorCmd, MotorFb
            graph.add_edge("Speed", "MotorControl", Some("read"));
            graph.add_edge("MotorControl", "MotorCmd", Some("write"));
            graph.add_edge("MotorControl", "MotorFb", Some("write"));
            graph.add_edge("MotorFb", "FaultHandler", Some("read"));
            graph.add_edge("FaultHandler", "Fault", Some("write"));
        }
    }
    
    let svg = graph.render_svg();
    println!("{}", svg);
}

fn generate_from_l5x(path: &str, graph_type: GraphType, show_aois: bool) {
    let path = PathBuf::from(path);
    
    if !path.exists() {
        eprintln!("Error: File not found: {}", path.display());
        std::process::exit(1);
    }
    
    eprintln!("Generating {} from: {}", graph_type.description(), path.display());
    
    // Read file
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            std::process::exit(1);
        }
    };
    
    // Detect format and dispatch
    if is_plcopen_format(&content) {
        generate_from_plcopen(&content, graph_type);
    } else {
        generate_from_l5x_content(&content, graph_type, show_aois);
    }
}

/// Detect if the XML is PLCopen format
fn is_plcopen_format(content: &str) -> bool {
    // PLCopen files have <project> root element with xmlns PLCopen namespace
    content.contains("<project") && content.contains("http://www.plcopen.org/xml/tc6")
}

/// Generate from L5X content
fn generate_from_l5x_content(content: &str, graph_type: GraphType, show_aois: bool) {
    let project: l5x::Project = match l5x::from_str(content) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error parsing L5X: {}", e);
            std::process::exit(1);
        }
    };
    
    // Dispatch based on export type
    let target_type = project.target_type.as_deref().unwrap_or("Controller");
    eprintln!("L5X export type: {}", target_type);
    
    let svg = match target_type {
        "Program" => generate_program_export(&project, graph_type, show_aois),
        "AddOnInstructionDefinition" => generate_aoi_export(&project, graph_type),
        _ => generate_controller_export(&project, graph_type, show_aois),
    };
    
    println!("{}", svg);
}

/// Generate from PLCopen content
fn generate_from_plcopen(content: &str, graph_type: GraphType) {
    let project: plcopen::Project = match plcopen::from_str(content) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error parsing PLCopen XML: {}", e);
            std::process::exit(1);
        }
    };
    
    let project_name = project.content_header
        .as_ref()
        .map(|h| h.name.as_str())
        .unwrap_or("unknown");
    eprintln!("PLCopen project: {}", project_name);
    
    let plcopen_graph_type = match graph_type {
        GraphType::Structure | GraphType::Combined => PlcopenGraphType::Structure,
        GraphType::CallGraph => PlcopenGraphType::CallGraph,
        GraphType::DataFlow => PlcopenGraphType::DataTypeDeps,
    };
    
    // Pass raw XML for ST code extraction
    let builder = PlcopenGraphBuilder::with_xml(project, plcopen_graph_type, content.to_string());
    let graph = builder.build();
    let svg = graph.render_svg();
    
    println!("{}", svg);
}

/// Generate graph from Controller export (full project)
fn generate_controller_export(project: &l5x::Project, graph_type: GraphType, show_aois: bool) -> String {
    let mut graph = L5xGraph::new();
    
    if let Some(ref controller) = project.controller {
        // Add AOIs if requested
        if show_aois {
            if let Some(ref aois) = controller.add_on_instruction_definitions {
                for aoi in &aois.add_on_instruction_definition {
                    graph.add_node(&aoi.name, &aoi.name, L5xNodeType::Aoi);
                }
            }
        }
        
        if let Some(ref programs) = controller.programs {
            for program in &programs.program {
                add_program_to_graph(program, &mut graph, graph_type);
            }
        }
    }
    
    graph.render_svg()
}

/// Generate graph from Program export (single program)
fn generate_program_export(project: &l5x::Project, graph_type: GraphType, show_aois: bool) -> String {
    let mut graph = L5xGraph::new();
    
    if let Some(ref controller) = project.controller {
        // Add AOIs from context if requested
        if show_aois {
            if let Some(ref aois) = controller.add_on_instruction_definitions {
                for aoi in &aois.add_on_instruction_definition {
                    // Only add target AOIs or all if none are marked as target
                    if aoi.r#use.as_deref() == Some("Target") || aoi.r#use.is_none() {
                        graph.add_node(&aoi.name, &aoi.name, L5xNodeType::Aoi);
                    }
                }
            }
        }
        
        // Find the target program (Use="Target")
        if let Some(ref programs) = controller.programs {
            for program in &programs.program {
                // In program export, the target program has Use="Target"
                if program.r#use.as_deref() == Some("Target") {
                    add_program_to_graph(program, &mut graph, graph_type);
                }
            }
        }
    }
    
    graph.render_svg()
}

/// Generate graph from AddOnInstructionDefinition export (single AOI)
fn generate_aoi_export(project: &l5x::Project, graph_type: GraphType) -> String {
    let mut graph = L5xGraph::new();
    
    if let Some(ref controller) = project.controller {
        if let Some(ref aois) = controller.add_on_instruction_definitions {
            for aoi in &aois.add_on_instruction_definition {
                // Find the target AOI (Use="Target")
                if aoi.r#use.as_deref() == Some("Target") {
                    let aoi_name = &aoi.name;
                    
                    // Add AOI as main node
                    graph.add_node(aoi_name, aoi_name, L5xNodeType::Aoi);
                    
                    // Find routines in the AOI content
                    for item in &aoi.content {
                        if let l5x::UDIDefinitionContent::Routines(ref routines) = item {
                            for routine in &routines.routine {
                                let routine_id = format!("{}.{}", aoi_name, routine.name);
                                
                                if graph_type == GraphType::Structure || graph_type == GraphType::Combined {
                                    graph.add_node(&routine_id, &routine.name, L5xNodeType::Routine);
                                    graph.add_edge(aoi_name, &routine_id, None);
                                } else {
                                    graph.add_node(&routine_id, &routine.name, L5xNodeType::Routine);
                                }
                                
                                // Extract JSR calls for call graph
                                if graph_type == GraphType::CallGraph || graph_type == GraphType::Combined {
                                    extract_jsr_calls(&routine_id, aoi_name, routine, &mut graph);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    graph.render_svg()
}

/// Add a program and its routines to the graph
fn add_program_to_graph(program: &l5x::AProgram, graph: &mut L5xGraph, graph_type: GraphType) {
    let prog_name = &program.name;
    
    // Add program node for structure/combined
    if graph_type == GraphType::Structure || graph_type == GraphType::Combined {
        graph.add_program(prog_name);
    }
    
    if let Some(ref routines) = program.routines {
        for routine in &routines.routine {
            let routine_id = format!("{}.{}", prog_name, routine.name);
            
            // Add routine node
            if graph_type == GraphType::Structure || graph_type == GraphType::Combined {
                graph.add_routine(prog_name, &routine.name);
                graph.add_edge(prog_name, &routine_id, None);
            } else {
                // For call/dataflow, just add routine nodes
                graph.add_node(&routine_id, &routine.name, L5xNodeType::Routine);
            }
            
            // Extract JSR calls for call graph
            if graph_type == GraphType::CallGraph || graph_type == GraphType::Combined {
                extract_jsr_calls(&routine_id, prog_name, routine, graph);
            }
        }
    }
}

/// Extract JSR (Jump to Subroutine) calls from a routine
fn extract_jsr_calls(
    routine_id: &str,
    program_name: &str,
    routine: &l5x::Routine,
    graph: &mut L5xGraph,
) {
    // Iterate through routine content to find RLL or ST content
    for item in &routine.content {
        match item {
            l5x::RoutineContent::RLLContent(rll_content) => {
                for rung in &rll_content.rung {
                    // Find Text elements in rung content
                    for rung_item in &rung.content {
                        if let l5x::RungContent::Text(text_wide) = rung_item {
                            // Extract text from TextWide content
                            let text = extract_text_from_textwide(text_wide);
                            extract_jsr_from_text(routine_id, program_name, &text, graph);
                        }
                    }
                }
            }
            l5x::RoutineContent::STContent(st_content) => {
                for item in &st_content.content {
                    if let l5x::STContentContent::Line(line) = item {
                        // STLine has text: Option<String>
                        if let Some(ref text) = line.text {
                            extract_jsr_from_text(routine_id, program_name, text, graph);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

/// Extract text content from TextWide
fn extract_text_from_textwide(text_wide: &l5x::TextWide) -> String {
    let mut result = String::new();
    for item in &text_wide.content {
        if let l5x::TextWideContent::TextContent(s) = item {
            result.push_str(s);
        }
    }
    result
}

/// Parse text content to find JSR calls
fn extract_jsr_from_text(
    from_routine: &str,
    program_name: &str,
    text: &str,
    graph: &mut L5xGraph,
) {
    // Match JSR(RoutineName) or JSR(RoutineName,param1,param2)
    // Also match AOI calls which look like: AOI_Name(params...)
    
    let jsr_pattern = regex::Regex::new(r"JSR\s*\(\s*([A-Za-z_][A-Za-z0-9_]*)\s*[,)]").unwrap();
    
    for cap in jsr_pattern.captures_iter(text) {
        if let Some(routine_name) = cap.get(1) {
            let target_name = routine_name.as_str();
            // Build full routine ID (same program)
            let target_id = format!("{}.{}", program_name, target_name);
            graph.add_call(from_routine, &target_id);
        }
    }
}
