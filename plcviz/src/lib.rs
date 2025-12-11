//! plcviz - PLC code visualization library
//!
//! Generate SVG diagrams from L5X (Rockwell) and PLCopen XML (IEC 61131-3) files
//! using layout-rs for professional graph layout and custom SVG generation for
//! clean, valid output.
//!
//! # Supported Formats
//!
//! - **L5X**: Rockwell Automation (ControlLogix, CompactLogix, Studio 5000)
//! - **PLCopen**: IEC 61131-3 TC6 XML (Siemens, CODESYS, Beckhoff, B&R, Beremiz)
//!
//! # Graph Types
//!
//! - **Structure**: Containment hierarchy (Controller/Project → Programs/POUs → Routines)
//! - **CallGraph**: Execution flow (Routine → Routine via JSR/function calls)
//! - **DataFlow**: Tag relationships (L5X) or DataType dependencies (PLCopen)
//! - **Combined**: Both structure and calls
//!
//! # CLI Usage
//!
//! ```text
//! plcviz [OPTIONS] <FILE>
//!
//! Arguments:
//!   <FILE>  L5X or PLCopen XML file to visualize
//!
//! Options:
//!   -t, --type <TYPE>  Graph type: structure, call, dataflow, combined [default: structure]
//!   -a, --aois         Include AOIs in the graph (L5X only)
//!   -h, --help         Print help
//! ```
//!
//! # Examples
//!
//! ```bash
//! # Generate structure graph from L5X
//! plcviz project.L5X > graph.svg
//!
//! # Generate call graph from PLCopen
//! plcviz -t call project.xml > calls.svg
//!
//! # Generate combined graph with AOIs
//! plcviz -t combined -a project.L5X > combined.svg
//! ```

pub mod config;
pub mod graph;
pub mod svg;
pub mod plcopen_graph;

// Re-export main types
pub use config::{GraphType, VizConfig, ElementFilter, NodeStyle, NodeStyles};
pub use graph::{L5xGraph, L5xNode, L5xNodeType, L5xEdge};
pub use plcopen_graph::{PlcopenGraphBuilder, PlcopenGraphType};
