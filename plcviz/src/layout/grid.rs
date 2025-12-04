//! Grid layout algorithm

use crate::graph::Graph;

/// Simple grid layout
pub struct GridLayout {
    pub columns: usize,
    pub cell_width: f64,
    pub cell_height: f64,
    pub padding: f64,
}

impl Default for GridLayout {
    fn default() -> Self {
        Self {
            columns: 4,
            cell_width: 150.0,
            cell_height: 80.0,
            padding: 20.0,
        }
    }
}

impl GridLayout {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn columns(mut self, columns: usize) -> Self {
        self.columns = columns;
        self
    }

    pub fn cell_size(mut self, width: f64, height: f64) -> Self {
        self.cell_width = width;
        self.cell_height = height;
        self
    }

    pub fn padding(mut self, padding: f64) -> Self {
        self.padding = padding;
        self
    }

    /// Apply layout to graph
    pub fn layout(&self, graph: &mut Graph) {
        for (i, node) in graph.nodes.iter_mut().enumerate() {
            let col = i % self.columns;
            let row = i / self.columns;
            
            node.x = self.padding + col as f64 * self.cell_width;
            node.y = self.padding + row as f64 * self.cell_height;
        }
    }

    /// Calculate required dimensions for the graph
    pub fn dimensions(&self, node_count: usize) -> (u32, u32) {
        let rows = (node_count + self.columns - 1) / self.columns;
        let width = self.padding * 2.0 + self.columns as f64 * self.cell_width;
        let height = self.padding * 2.0 + rows as f64 * self.cell_height;
        (width as u32, height as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Node;

    #[test]
    fn test_grid_layout() {
        let mut graph = Graph::new();
        graph.add_node(Node::routine("a", "A"));
        graph.add_node(Node::routine("b", "B"));
        graph.add_node(Node::routine("c", "C"));
        graph.add_node(Node::routine("d", "D"));
        graph.add_node(Node::routine("e", "E"));

        let layout = GridLayout::new().columns(3);
        layout.layout(&mut graph);

        // First row
        assert_eq!(graph.nodes[0].x, 20.0);
        assert_eq!(graph.nodes[1].x, 170.0);
        assert_eq!(graph.nodes[2].x, 320.0);
        
        // Second row
        assert_eq!(graph.nodes[3].y, 100.0);
    }
}
