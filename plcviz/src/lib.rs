//! plcviz - PLC code visualization library
//!
//! Generate SVG diagrams from L5X files using layout-rs for
//! professional graph layout and our custom SVG generation for
//! clean, valid output.
//!
//! # Graph Types
//!
//! - **Structure**: Containment hierarchy (Controller → Programs → Routines)
//! - **CallGraph**: Execution flow (Routine → Routine via JSR/AOI calls)
//! - **Combined**: Both structure and calls
//!
//! # Example
//! ```ignore
//! use plcviz::{L5xGraph, VizConfig, GraphType};
//!
//! let mut graph = L5xGraph::new();
//! graph.add_program("MainProgram");
//! graph.add_routine("MainProgram", "MainRoutine");
//! graph.add_call("MainRoutine", "SubRoutine");
//!
//! let svg = graph.render_svg();
//! ```

pub mod config;
pub mod graph;
pub mod svg;

// Re-export main types
pub use config::{GraphType, VizConfig, ElementFilter, NodeStyle, NodeStyles};
pub use graph::{L5xGraph, L5xNode, L5xNodeType, L5xEdge};
