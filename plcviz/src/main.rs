//! plcviz CLI - Generate SVG diagrams from L5X files

use std::path::PathBuf;
use std::fs;

use plcviz::{Graph, Node, Edge, SvgBuilder, HierarchicalLayout};
use plcviz::output::{node_box, arrow_edge_curved};

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
    // Create example graph (L5X-like structure)
    let mut graph = Graph::new();
    
    // Programs (layer 2)
    graph.add_node(Node::program("main_prog", "MainProgram"));
    graph.add_node(Node::program("comm_prog", "CommProgram"));
    
    // Routines (layer 3)
    graph.add_node(Node::routine("main", "MainRoutine").with_parent("main_prog"));
    graph.add_node(Node::routine("init", "InitSequence").with_parent("main_prog"));
    graph.add_node(Node::routine("fault", "FaultHandler").with_parent("main_prog"));
    graph.add_node(Node::routine("motor", "MotorControl").with_parent("main_prog"));
    graph.add_node(Node::routine("eth", "EthernetComm").with_parent("comm_prog"));
    
    // Call edges
    graph.add_edge(Edge::call("main", "init"));
    graph.add_edge(Edge::call("main", "motor"));
    graph.add_edge(Edge::call("main", "fault"));
    graph.add_edge(Edge::call("motor", "fault"));
    
    // Apply hierarchical layout
    let layout = HierarchicalLayout::new()
        .layer_height(120.0)
        .node_spacing(40.0);
    layout.layout(&mut graph);
    
    // Calculate dimensions
    let (width, height) = layout.dimensions(&graph);
    
    // Build SVG
    let mut svg = SvgBuilder::new(width.max(600), height.max(400))
        .with_default_arrows()
        .with_default_styles();
    
    // Add edges first (below nodes)
    for edge in &graph.edges {
        if let (Some(from), Some(to)) = (graph.get_node(&edge.from), graph.get_node(&edge.to)) {
            let (x1, y1) = from.bottom();
            let (x2, y2) = to.top();
            svg.add(arrow_edge_curved(x1, y1, x2, y2, "arrow"));
        }
    }
    
    // Add nodes
    for node in &graph.nodes {
        svg.add(node_box(node.x, node.y, node.width, node.height, &node.label));
    }
    
    println!("{}", svg.build());
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
    let mut graph = Graph::new();
    
    // Add programs and routines from controller
    if let Some(ref controller) = project.controller {
        if let Some(ref programs) = controller.programs {
            for program in &programs.program {
                let prog_id = &program.name;
                graph.add_node(Node::program(prog_id, prog_id));
                
                if let Some(ref routines) = program.routines {
                    for routine in &routines.routine {
                        let routine_id = format!("{}.{}", prog_id, routine.name);
                        graph.add_node(
                            Node::routine(&routine_id, &routine.name)
                                .with_parent(prog_id)
                        );
                    }
                }
            }
        }
    }
    
    // TODO: Extract JSR calls from routines to build edges
    // This requires parsing the RLL/ST content
    
    // Apply layout
    let layout = HierarchicalLayout::new();
    layout.layout(&mut graph);
    
    let (width, height) = layout.dimensions(&graph);
    
    // Build SVG
    let mut svg = SvgBuilder::new(width.max(600), height.max(400))
        .with_default_arrows()
        .with_default_styles();
    
    for edge in &graph.edges {
        if let (Some(from), Some(to)) = (graph.get_node(&edge.from), graph.get_node(&edge.to)) {
            let (x1, y1) = from.bottom();
            let (x2, y2) = to.top();
            svg.add(arrow_edge_curved(x1, y1, x2, y2, "arrow"));
        }
    }
    
    for node in &graph.nodes {
        svg.add(node_box(node.x, node.y, node.width, node.height, &node.label));
    }
    
    println!("{}", svg.build());
}
