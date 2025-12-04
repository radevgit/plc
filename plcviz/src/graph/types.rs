//! Graph types

use std::collections::HashMap;

/// Node identifier
pub type NodeId = String;

/// A node in the graph
#[derive(Debug, Clone)]
pub struct Node {
    pub id: NodeId,
    pub label: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub node_type: NodeType,
    /// Parent node ID (for hierarchical layout)
    pub parent: Option<NodeId>,
    /// Layer/depth hint from L5X structure
    pub layer: u32,
}

/// Type of node
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeType {
    Controller,
    Task,
    Program,
    Routine,
    Aoi,
    Tag,
    Udt,
}

impl NodeType {
    /// Default layer for this node type (based on L5X hierarchy)
    pub fn default_layer(&self) -> u32 {
        match self {
            NodeType::Controller => 0,
            NodeType::Task => 1,
            NodeType::Program => 2,
            NodeType::Routine => 3,
            NodeType::Aoi => 2,  // Same level as programs
            NodeType::Tag => 4,
            NodeType::Udt => 4,
        }
    }
}

impl Node {
    pub fn new(id: &str, label: &str, node_type: NodeType) -> Self {
        let layer = node_type.default_layer();
        Self {
            id: id.to_string(),
            label: label.to_string(),
            x: 0.0,
            y: 0.0,
            width: 120.0,
            height: 40.0,
            node_type,
            parent: None,
            layer,
        }
    }

    /// Create node with parent reference
    pub fn with_parent(mut self, parent: &str) -> Self {
        self.parent = Some(parent.to_string());
        self
    }

    /// Set explicit layer
    pub fn with_layer(mut self, layer: u32) -> Self {
        self.layer = layer;
        self
    }

    pub fn routine(id: &str, label: &str) -> Self {
        Self::new(id, label, NodeType::Routine)
    }

    pub fn program(id: &str, label: &str) -> Self {
        Self::new(id, label, NodeType::Program)
    }

    pub fn aoi(id: &str, label: &str) -> Self {
        Self::new(id, label, NodeType::Aoi)
    }

    /// Center X position
    pub fn cx(&self) -> f64 {
        self.x + self.width / 2.0
    }

    /// Center Y position
    pub fn cy(&self) -> f64 {
        self.y + self.height / 2.0
    }

    /// Bottom center (for outgoing edges)
    pub fn bottom(&self) -> (f64, f64) {
        (self.x + self.width / 2.0, self.y + self.height)
    }

    /// Top center (for incoming edges)
    pub fn top(&self) -> (f64, f64) {
        (self.x + self.width / 2.0, self.y)
    }
}

/// An edge in the graph
#[derive(Debug, Clone)]
pub struct Edge {
    pub from: NodeId,
    pub to: NodeId,
    pub edge_type: EdgeType,
}

/// Type of edge
#[derive(Debug, Clone, PartialEq)]
pub enum EdgeType {
    Call,       // Routine calls routine
    DataFlow,   // Tag data flows
    Contains,   // Parent contains child
    Uses,       // AOI uses AOI
}

impl Edge {
    pub fn new(from: &str, to: &str, edge_type: EdgeType) -> Self {
        Self {
            from: from.to_string(),
            to: to.to_string(),
            edge_type,
        }
    }

    pub fn call(from: &str, to: &str) -> Self {
        Self::new(from, to, EdgeType::Call)
    }

    pub fn data_flow(from: &str, to: &str) -> Self {
        Self::new(from, to, EdgeType::DataFlow)
    }
}

/// A directed graph
#[derive(Debug, Clone, Default)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    node_index: HashMap<NodeId, usize>,
}

impl Graph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, node: Node) {
        let id = node.id.clone();
        let idx = self.nodes.len();
        self.nodes.push(node);
        self.node_index.insert(id, idx);
    }

    /// Add an edge to the graph
    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }

    /// Get node by ID
    pub fn get_node(&self, id: &str) -> Option<&Node> {
        self.node_index.get(id).map(|&idx| &self.nodes[idx])
    }

    /// Get mutable node by ID
    pub fn get_node_mut(&mut self, id: &str) -> Option<&mut Node> {
        self.node_index.get(id).map(|&idx| &mut self.nodes[idx])
    }

    /// Get all edges from a node
    pub fn edges_from(&self, id: &str) -> Vec<&Edge> {
        self.edges.iter().filter(|e| e.from == id).collect()
    }

    /// Get all edges to a node
    pub fn edges_to(&self, id: &str) -> Vec<&Edge> {
        self.edges.iter().filter(|e| e.to == id).collect()
    }

    /// Number of nodes
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Number of edges
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_basic() {
        let mut graph = Graph::new();
        graph.add_node(Node::routine("main", "MainRoutine"));
        graph.add_node(Node::routine("sub", "SubRoutine"));
        graph.add_edge(Edge::call("main", "sub"));

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
        assert!(graph.get_node("main").is_some());
    }
}
