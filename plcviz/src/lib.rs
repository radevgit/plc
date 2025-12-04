//! plcviz - PLC code visualization library
//!
//! Generate SVG diagrams from L5X files:
//! - Call graphs (routine → routine)
//! - Tag dataflow
//! - AOI dependencies
//! - Program structure

pub mod output;
pub mod graph;
pub mod layout;
pub mod analysis;

// Re-exports for convenience
pub use output::{SvgBuilder, Style, Color};
pub use output::{node_box, arrow_edge, arrow_edge_curved};
pub use graph::{Graph, Node, Edge, NodeType};
pub use layout::{GridLayout, HierarchicalLayout};
