//! Hierarchical layout algorithm
//!
//! Layers nodes based on L5X structure:
//! - Layer 0: Controller
//! - Layer 1: Tasks
//! - Layer 2: Programs, AOIs
//! - Layer 3: Routines
//! - Layer 4: Tags, UDTs

use std::collections::HashMap;
use crate::graph::{Graph, NodeId};

/// Hierarchical (layered) layout
/// 
/// Positions nodes in layers based on their type and parent relationships.
/// Similar to Sugiyama layout but using L5X structure hints.
pub struct HierarchicalLayout {
    pub layer_height: f64,
    pub node_spacing: f64,
    pub padding: f64,
    pub group_spacing: f64,
}

impl Default for HierarchicalLayout {
    fn default() -> Self {
        Self {
            layer_height: 100.0,
            node_spacing: 30.0,
            padding: 40.0,
            group_spacing: 50.0,
        }
    }
}

impl HierarchicalLayout {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn layer_height(mut self, height: f64) -> Self {
        self.layer_height = height;
        self
    }

    pub fn node_spacing(mut self, spacing: f64) -> Self {
        self.node_spacing = spacing;
        self
    }

    /// Apply hierarchical layout to graph
    pub fn layout(&self, graph: &mut Graph) {
        if graph.nodes.is_empty() {
            return;
        }

        // Group nodes by layer
        let mut layers: HashMap<u32, Vec<usize>> = HashMap::new();
        for (idx, node) in graph.nodes.iter().enumerate() {
            layers.entry(node.layer).or_default().push(idx);
        }

        // Sort layers by layer number
        let mut layer_keys: Vec<u32> = layers.keys().cloned().collect();
        layer_keys.sort();

        // Position each layer
        for (layer_idx, layer_num) in layer_keys.iter().enumerate() {
            let node_indices = &layers[layer_num];
            
            // Group nodes by parent within layer
            let groups = self.group_by_parent(graph, node_indices);
            
            // Calculate total width needed for this layer
            let _total_width = self.calculate_layer_width(graph, &groups);
            
            // Position nodes in this layer
            let mut x = self.padding;
            for group in groups {
                for &node_idx in &group {
                    let node = &mut graph.nodes[node_idx];
                    node.x = x;
                    node.y = self.padding + layer_idx as f64 * self.layer_height;
                    x += node.width + self.node_spacing;
                }
                x += self.group_spacing - self.node_spacing; // Extra space between groups
            }
        }

        // Center children under parents (optional refinement)
        self.center_children_under_parents(graph);
    }

    /// Group node indices by their parent
    fn group_by_parent(&self, graph: &Graph, node_indices: &[usize]) -> Vec<Vec<usize>> {
        let mut groups: HashMap<Option<&NodeId>, Vec<usize>> = HashMap::new();
        
        for &idx in node_indices {
            let parent = graph.nodes[idx].parent.as_ref();
            groups.entry(parent).or_default().push(idx);
        }

        // Sort groups by parent name for consistent ordering
        let mut group_keys: Vec<_> = groups.keys().cloned().collect();
        group_keys.sort_by(|a, b| {
            match (a, b) {
                (None, None) => std::cmp::Ordering::Equal,
                (None, Some(_)) => std::cmp::Ordering::Less,
                (Some(_), None) => std::cmp::Ordering::Greater,
                (Some(a), Some(b)) => a.cmp(b),
            }
        });

        group_keys.into_iter()
            .map(|k| groups.remove(&k).unwrap())
            .collect()
    }

    /// Calculate total width needed for a layer
    fn calculate_layer_width(&self, graph: &Graph, groups: &[Vec<usize>]) -> f64 {
        let mut width = self.padding * 2.0;
        
        for (i, group) in groups.iter().enumerate() {
            for &idx in group {
                width += graph.nodes[idx].width + self.node_spacing;
            }
            if i < groups.len() - 1 {
                width += self.group_spacing - self.node_spacing;
            }
        }
        
        width - self.node_spacing // Remove last spacing
    }

    /// Center children under their parent nodes
    fn center_children_under_parents(&self, graph: &mut Graph) {
        // Build parent -> children mapping
        let mut children_map: HashMap<NodeId, Vec<usize>> = HashMap::new();
        for (idx, node) in graph.nodes.iter().enumerate() {
            if let Some(ref parent_id) = node.parent {
                children_map.entry(parent_id.clone()).or_default().push(idx);
            }
        }

        // For each parent, center its children under it
        for node in graph.nodes.clone().iter() {
            if let Some(children_indices) = children_map.get(&node.id) {
                if children_indices.is_empty() {
                    continue;
                }

                // Calculate children center
                let children_left = children_indices.iter()
                    .map(|&i| graph.nodes[i].x)
                    .fold(f64::INFINITY, f64::min);
                let children_right = children_indices.iter()
                    .map(|&i| graph.nodes[i].x + graph.nodes[i].width)
                    .fold(f64::NEG_INFINITY, f64::max);
                let children_center = (children_left + children_right) / 2.0;

                // Calculate parent center
                let parent_center = node.x + node.width / 2.0;

                // Shift children to center under parent
                let shift = parent_center - children_center;
                if shift.abs() > 1.0 {
                    for &child_idx in children_indices {
                        graph.nodes[child_idx].x += shift;
                    }
                }
            }
        }
    }

    /// Calculate required dimensions for the graph
    pub fn dimensions(&self, graph: &Graph) -> (u32, u32) {
        if graph.nodes.is_empty() {
            return (400, 300);
        }

        let max_x = graph.nodes.iter()
            .map(|n| n.x + n.width)
            .fold(0.0f64, f64::max);
        let max_y = graph.nodes.iter()
            .map(|n| n.y + n.height)
            .fold(0.0f64, f64::max);

        ((max_x + self.padding) as u32, (max_y + self.padding) as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Node;

    #[test]
    fn test_hierarchical_layout() {
        let mut graph = Graph::new();
        
        // Create L5X-like structure
        graph.add_node(Node::program("prog1", "MainProgram"));
        graph.add_node(Node::routine("r1", "MainRoutine").with_parent("prog1"));
        graph.add_node(Node::routine("r2", "FaultRoutine").with_parent("prog1"));
        graph.add_node(Node::program("prog2", "CommProgram"));
        graph.add_node(Node::routine("r3", "EthRoutine").with_parent("prog2"));

        let layout = HierarchicalLayout::new();
        layout.layout(&mut graph);

        // Programs should be at layer 2 (y = padding + 0 * layer_height if no tasks)
        // Actually they'll be at layer 0 since we only have programs and routines
        let prog1 = graph.get_node("prog1").unwrap();
        let prog2 = graph.get_node("prog2").unwrap();
        assert_eq!(prog1.layer, 2);
        assert_eq!(prog2.layer, 2);

        // Routines should be below programs
        let r1 = graph.get_node("r1").unwrap();
        assert!(r1.y > prog1.y);
    }
}
