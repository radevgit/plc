//! L5X Graph builder

use std::collections::HashMap;

use layout::core::base::Orientation;
use layout::core::geometry::Point;
use layout::core::style::StyleAttr;
use layout::std_shapes::shapes::{Arrow, Element, ShapeKind};
use layout::topo::layout::VisualGraph;

use super::SvgRenderer;

/// Graph builder for L5X visualization
///
/// Builds a graph from L5X structure, then renders using layout-rs
/// with our custom SVG renderer.
#[derive(Debug)]
pub struct L5xGraph {
    nodes: Vec<L5xNode>,
    edges: Vec<L5xEdge>,
    node_index: HashMap<String, usize>,
}

/// A node in the L5X graph
#[derive(Debug, Clone)]
pub struct L5xNode {
    pub id: String,
    pub label: String,
    pub node_type: L5xNodeType,
    pub parent: Option<String>,
}

/// Type of L5X element
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum L5xNodeType {
    Controller,
    Task,
    Program,
    Routine,
    Aoi,
    Tag,
    Udt,
}

impl L5xNodeType {
    /// Get fill color for this node type (RGB hex)
    pub fn fill_color(&self) -> u32 {
        match self {
            L5xNodeType::Controller => 0xE1BEE7, // Light purple
            L5xNodeType::Task => 0xFFCDD2,       // Light red
            L5xNodeType::Program => 0xBBDEFB,    // Light blue
            L5xNodeType::Routine => 0xFFFFFF,    // White
            L5xNodeType::Aoi => 0xC8E6C9,        // Light green
            L5xNodeType::Tag => 0xF5F5F5,        // Light gray
            L5xNodeType::Udt => 0xFFF9C4,        // Light yellow
        }
    }

    /// Get stroke color for this node type (RGB hex)
    pub fn stroke_color(&self) -> u32 {
        match self {
            L5xNodeType::Controller => 0x7B1FA2, // Purple
            L5xNodeType::Task => 0xC62828,       // Red
            L5xNodeType::Program => 0x1565C0,    // Blue
            L5xNodeType::Routine => 0x424242,    // Dark gray
            L5xNodeType::Aoi => 0x2E7D32,        // Green
            L5xNodeType::Tag => 0x616161,        // Gray
            L5xNodeType::Udt => 0xF9A825,        // Amber
        }
    }

    /// Get corner radius for this node type
    pub fn corner_radius(&self) -> usize {
        match self {
            L5xNodeType::Aoi => 12,  // More rounded for AOIs
            L5xNodeType::Tag => 2,   // Slight rounding
            _ => 4,                  // Default
        }
    }
}

/// Type of edge
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EdgeType {
    /// Containment edge (Program → Routine)
    #[default]
    Structure,
    /// Call edge (JSR call between routines)
    Call,
    /// Data flow edge (tag read/write)
    DataFlow,
}

/// An edge in the L5X graph
#[derive(Debug, Clone)]
pub struct L5xEdge {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
    pub edge_type: EdgeType,
}

impl Default for L5xGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl L5xGraph {
    /// Create a new empty graph
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            node_index: HashMap::new(),
        }
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, id: &str, label: &str, node_type: L5xNodeType) {
        let idx = self.nodes.len();
        self.nodes.push(L5xNode {
            id: id.to_string(),
            label: label.to_string(),
            node_type,
            parent: None,
        });
        self.node_index.insert(id.to_string(), idx);
    }

    /// Add a node with a parent reference
    pub fn add_node_with_parent(
        &mut self,
        id: &str,
        label: &str,
        node_type: L5xNodeType,
        parent: &str,
    ) {
        let idx = self.nodes.len();
        self.nodes.push(L5xNode {
            id: id.to_string(),
            label: label.to_string(),
            node_type,
            parent: Some(parent.to_string()),
        });
        self.node_index.insert(id.to_string(), idx);
    }

    /// Add a program node
    pub fn add_program(&mut self, name: &str) {
        self.add_node(name, name, L5xNodeType::Program);
    }

    /// Add a routine node under a program
    pub fn add_routine(&mut self, program: &str, name: &str) {
        let id = format!("{}.{}", program, name);
        self.add_node_with_parent(&id, name, L5xNodeType::Routine, program);
    }

    /// Add a call edge between routines
    pub fn add_call(&mut self, from: &str, to: &str) {
        self.edges.push(L5xEdge {
            from: from.to_string(),
            to: to.to_string(),
            label: None,
            edge_type: EdgeType::Call,
        });
    }

    /// Add a labeled edge (structure/containment by default)
    pub fn add_edge(&mut self, from: &str, to: &str, label: Option<&str>) {
        self.edges.push(L5xEdge {
            from: from.to_string(),
            to: to.to_string(),
            label: label.map(|s| s.to_string()),
            edge_type: EdgeType::Structure,
        });
    }

    /// Render the graph to SVG using layout-rs for layout, custom SVG output
    pub fn render_svg(&self) -> String {
        let mut vg = VisualGraph::new(Orientation::TopToBottom);

        // Map our node IDs to layout-rs handles
        let mut handles: HashMap<String, layout::adt::dag::NodeHandle> = HashMap::new();

        // Add nodes to the visual graph
        for node in &self.nodes {
            let shape = ShapeKind::Box(node.label.clone());
            
            // Use node-type-specific colors
            let mut look = StyleAttr::simple();
            look.fill_color = Some(layout::core::color::Color::new(
                node.node_type.fill_color() << 8 | 0xFF
            ));
            look.line_color = layout::core::color::Color::new(
                node.node_type.stroke_color() << 8 | 0xFF
            );
            look.rounded = node.node_type.corner_radius();
            
            // Calculate width based on text length (approx 8px per char + padding)
            let text_width = (node.label.len() as f64) * 8.0 + 20.0;
            let width = text_width.max(80.0).min(300.0); // min 80, max 300
            let sz = Point::new(width, 40.);

            let element = Element::create(shape, look, Orientation::LeftToRight, sz);

            let handle = vg.add_node(element);
            handles.insert(node.id.clone(), handle);
        }

        // Add edges
        for edge in &self.edges {
            if let (Some(&from_h), Some(&to_h)) = (handles.get(&edge.from), handles.get(&edge.to)) {
                // Encode edge type in label prefix for renderer to parse
                let type_prefix = match edge.edge_type {
                    EdgeType::Structure => "__STRUCT__",
                    EdgeType::Call => "__CALL__",
                    EdgeType::DataFlow => "__DATA__",
                };
                let label = match &edge.label {
                    Some(l) => format!("{}{}", type_prefix, l),
                    None => type_prefix.to_string(),
                };
                let arrow = Arrow::simple(&label);
                vg.add_edge(arrow, from_h, to_h);
            }
        }

        // Render using our custom SVG renderer
        let mut renderer = SvgRenderer::new();
        vg.do_it(false, false, false, &mut renderer);
        renderer.finalize()
    }

    /// Get node count
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get edge count
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_graph() {
        let mut graph = L5xGraph::new();
        graph.add_program("MainProgram");
        graph.add_routine("MainProgram", "MainRoutine");
        graph.add_routine("MainProgram", "SubRoutine");
        graph.add_call("MainProgram.MainRoutine", "MainProgram.SubRoutine");

        assert_eq!(graph.node_count(), 3);
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_render_svg() {
        let mut graph = L5xGraph::new();
        graph.add_program("Prog1");
        graph.add_routine("Prog1", "Main");
        graph.add_call("Prog1", "Prog1.Main");

        let svg = graph.render_svg();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("xmlns"));
    }

    #[test]
    fn test_svg_valid_structure() {
        let mut graph = L5xGraph::new();
        graph.add_node("A", "NodeA", L5xNodeType::Routine);
        graph.add_node("B", "NodeB", L5xNodeType::Routine);
        graph.add_call("A", "B");

        let svg = graph.render_svg();

        // Check for proper SVG elements
        assert!(svg.contains("<rect"));
        assert!(svg.contains("<text"));
        assert!(svg.contains("<path"));
        assert!(svg.contains("arrowhead"));
    }
}
