//! Visualization configuration

use std::str::FromStr;

/// Type of graph to generate
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GraphType {
    /// Structure/containment hierarchy (Controller → Programs → Routines)
    #[default]
    Structure,
    /// Call graph (Routine → Routine via JSR, AOI calls)
    CallGraph,
    /// Data flow graph (Tag read/write relationships)
    DataFlow,
    /// Combined structure + calls
    Combined,
}

impl FromStr for GraphType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "structure" | "struct" | "s" => Ok(GraphType::Structure),
            "call" | "callgraph" | "calls" | "c" => Ok(GraphType::CallGraph),
            "dataflow" | "data" | "flow" | "d" => Ok(GraphType::DataFlow),
            "combined" | "all" | "a" => Ok(GraphType::Combined),
            _ => Err(format!("Unknown graph type: '{}'. Valid: structure, call, dataflow, combined", s)),
        }
    }
}

impl GraphType {
    pub fn description(&self) -> &'static str {
        match self {
            GraphType::Structure => "Containment hierarchy (Programs → Routines)",
            GraphType::CallGraph => "Call graph (JSR, AOI calls)",
            GraphType::DataFlow => "Data flow (Tag read/write)",
            GraphType::Combined => "Combined structure + calls",
        }
    }
}

/// What elements to include in the graph
#[derive(Debug, Clone)]
pub struct ElementFilter {
    pub programs: bool,
    pub routines: bool,
    pub aois: bool,
    pub udts: bool,
    pub tags: bool,
}

impl Default for ElementFilter {
    fn default() -> Self {
        Self {
            programs: true,
            routines: true,
            aois: false,
            udts: false,
            tags: false,
        }
    }
}

/// Visual style for different node types
#[derive(Debug, Clone)]
pub struct NodeStyle {
    pub fill_color: u32,
    pub stroke_color: u32,
    pub rounded: usize,
}

impl NodeStyle {
    pub const fn new(fill: u32, stroke: u32, rounded: usize) -> Self {
        Self {
            fill_color: fill,
            stroke_color: stroke,
            rounded,
        }
    }
}

/// Default styles for different node types
pub struct NodeStyles {
    pub program: NodeStyle,
    pub routine: NodeStyle,
    pub aoi: NodeStyle,
    pub udt: NodeStyle,
    pub tag: NodeStyle,
}

impl Default for NodeStyles {
    fn default() -> Self {
        Self {
            // Light blue for programs
            program: NodeStyle::new(0xE3F2FD, 0x1976D2, 4),
            // White for routines
            routine: NodeStyle::new(0xFFFFFF, 0x424242, 4),
            // Light green for AOIs
            aoi: NodeStyle::new(0xE8F5E9, 0x388E3C, 8),
            // Light yellow for UDTs
            udt: NodeStyle::new(0xFFFDE7, 0xF9A825, 4),
            // Light gray for tags
            tag: NodeStyle::new(0xF5F5F5, 0x757575, 2),
        }
    }
}

/// Complete visualization configuration
#[derive(Debug, Clone)]
pub struct VizConfig {
    pub graph_type: GraphType,
    pub filter: ElementFilter,
    pub show_labels: bool,
    pub compact: bool,
}

impl Default for VizConfig {
    fn default() -> Self {
        Self {
            graph_type: GraphType::Structure,
            filter: ElementFilter::default(),
            show_labels: true,
            compact: false,
        }
    }
}

impl VizConfig {
    pub fn structure() -> Self {
        Self {
            graph_type: GraphType::Structure,
            ..Default::default()
        }
    }

    pub fn call_graph() -> Self {
        Self {
            graph_type: GraphType::CallGraph,
            filter: ElementFilter {
                programs: false,
                routines: true,
                aois: true,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn combined() -> Self {
        Self {
            graph_type: GraphType::Combined,
            ..Default::default()
        }
    }

    pub fn with_aois(mut self) -> Self {
        self.filter.aois = true;
        self
    }

    pub fn with_udts(mut self) -> Self {
        self.filter.udts = true;
        self
    }

    pub fn with_tags(mut self) -> Self {
        self.filter.tags = true;
        self
    }
}
