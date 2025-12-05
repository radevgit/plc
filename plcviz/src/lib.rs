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
//! # CLI Usage
//!
//! ```text
//! plcviz [OPTIONS] <FILE>
//!
//! Arguments:
//!   <FILE>  L5X file to visualize
//!
//! Options:
//!   -t, --type <TYPE>  Graph type: structure, call, dataflow, combined [default: structure]
//!   -o, --output <FILE>  Output file (default: stdout)
//!   -h, --help         Print help
//! ```
//!
//! # Examples
//!
//! ```bash
//! # Generate structure graph
//! plcviz project.L5X > graph.svg
//!
//! # Generate call graph
//! plcviz -t call project.L5X > calls.svg
//!
//! # Generate combined graph to file
//! plcviz -t combined -o combined.svg project.L5X
//! ```

pub mod config;
pub mod graph;
pub mod svg;

// Re-export main types
pub use config::{GraphType, VizConfig, ElementFilter, NodeStyle, NodeStyles};
pub use graph::{L5xGraph, L5xNode, L5xNodeType, L5xEdge};
