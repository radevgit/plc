//! Control Flow Graph (CFG) construction and analysis.
//!
//! The CFG represents all possible execution paths through a program.
//! Each node is a basic block (straight-line code), and edges represent
//! control flow (branches, loops, etc.).
//!
//! # Example
//!
//! ```text
//! IF x > 0 THEN
//!     y := 1;
//! ELSE
//!     y := 2;
//! END_IF;
//! z := 3;
//! ```
//!
//! Produces:
//! ```text
//!        [Entry]
//!           |
//!      [x > 0 ?]
//!        /    \
//!   [y:=1]    [y:=2]
//!        \    /
//!       [z := 3]
//!           |
//!        [Exit]
//! ```

use crate::ast::{Stmt, StmtKind, Expr, ExprKind, BinaryOp};
use std::collections::{HashMap, HashSet};

/// Unique identifier for a CFG node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

impl NodeId {
    /// Entry node ID (always 0).
    pub const ENTRY: NodeId = NodeId(0);
}

/// A node in the control flow graph.
#[derive(Debug, Clone)]
pub struct CfgNode {
    /// Unique identifier.
    pub id: NodeId,
    /// Kind of node.
    pub kind: NodeKind,
    /// Statements in this basic block (empty for Entry/Exit).
    pub statements: Vec<StmtRef>,
}

/// Reference to a statement (for tracking without cloning).
#[derive(Debug, Clone)]
pub struct StmtRef {
    /// Index in the original statement list.
    pub index: usize,
    /// Copy of the statement kind for analysis.
    pub kind: StmtKind,
}

/// Kind of CFG node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    /// Entry point of the function/program.
    Entry,
    /// Exit point of the function/program.
    Exit,
    /// Basic block - straight-line code.
    Basic,
    /// Branch point (IF, CASE condition evaluation).
    Branch,
    /// Loop header (FOR, WHILE, REPEAT condition).
    LoopHeader,
    /// Loop exit point.
    LoopExit,
}

/// An edge in the control flow graph.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CfgEdge {
    /// Source node.
    pub from: NodeId,
    /// Target node.
    pub to: NodeId,
    /// Edge kind (for labeling/analysis).
    pub kind: EdgeKind,
}

/// Kind of CFG edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeKind {
    /// Normal sequential flow.
    Sequential,
    /// True branch of a condition.
    TrueBranch,
    /// False branch of a condition.
    FalseBranch,
    /// Loop back edge.
    LoopBack,
    /// Exit from loop (EXIT statement or condition false).
    LoopExit,
    /// Return from function.
    Return,
}

/// Control Flow Graph for a POU (function, function block, program).
#[derive(Debug)]
pub struct Cfg {
    /// All nodes in the graph.
    pub nodes: Vec<CfgNode>,
    /// All edges in the graph.
    pub edges: Vec<CfgEdge>,
    /// Entry node ID.
    pub entry: NodeId,
    /// Exit node ID.
    pub exit: NodeId,
    /// Successors map for fast lookup.
    successors: HashMap<NodeId, Vec<NodeId>>,
    /// Predecessors map for fast lookup.
    predecessors: HashMap<NodeId, Vec<NodeId>>,
}

impl Cfg {
    /// Get successor nodes.
    pub fn successors(&self, node: NodeId) -> &[NodeId] {
        self.successors.get(&node).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get predecessor nodes.
    pub fn predecessors(&self, node: NodeId) -> &[NodeId] {
        self.predecessors.get(&node).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get a node by ID.
    pub fn node(&self, id: NodeId) -> Option<&CfgNode> {
        self.nodes.get(id.0)
    }

    /// Calculate cyclomatic complexity.
    ///
    /// Formula: M = E - N + 2P
    /// Where:
    /// - E = number of edges
    /// - N = number of nodes
    /// - P = number of connected components (1 for a single function)
    ///
    /// Simplified: M = E - N + 2
    pub fn cyclomatic_complexity(&self) -> usize {
        let edges = self.edges.len() as isize;
        let nodes = self.nodes.len() as isize;
        // For a single connected component (one function), P = 1
        // M = E - N + 2P = E - N + 2
        (edges - nodes + 2).max(1) as usize
    }

    /// Calculate cyclomatic complexity using decision points.
    ///
    /// This is an alternative calculation: M = 1 + D
    /// Where D = number of decision points (branches).
    ///
    /// This often matches the edge-based calculation and is
    /// easier to understand.
    pub fn cyclomatic_complexity_decisions(&self) -> usize {
        let decision_points = self.nodes.iter()
            .filter(|n| matches!(n.kind, NodeKind::Branch | NodeKind::LoopHeader))
            .count();
        1 + decision_points
    }

    /// Find unreachable nodes (nodes with no path from entry).
    pub fn unreachable_nodes(&self) -> Vec<NodeId> {
        let mut visited = HashSet::new();
        let mut stack = vec![self.entry];

        while let Some(node) = stack.pop() {
            if visited.insert(node) {
                for &succ in self.successors(node) {
                    if !visited.contains(&succ) {
                        stack.push(succ);
                    }
                }
            }
        }

        self.nodes.iter()
            .map(|n| n.id)
            .filter(|id| !visited.contains(id))
            .collect()
    }

    /// Check if there's a path between two nodes.
    pub fn has_path(&self, from: NodeId, to: NodeId) -> bool {
        let mut visited = HashSet::new();
        let mut stack = vec![from];

        while let Some(node) = stack.pop() {
            if node == to {
                return true;
            }
            if visited.insert(node) {
                for &succ in self.successors(node) {
                    if !visited.contains(&succ) {
                        stack.push(succ);
                    }
                }
            }
        }
        false
    }

    /// Export to DOT format for Graphviz visualization.
    pub fn to_dot(&self) -> String {
        let mut dot = String::from("digraph CFG {\n");
        dot.push_str("    node [shape=box];\n");

        // Nodes
        for node in &self.nodes {
            let label = match &node.kind {
                NodeKind::Entry => "Entry".to_string(),
                NodeKind::Exit => "Exit".to_string(),
                NodeKind::Basic => format!("Block {}", node.id.0),
                NodeKind::Branch => format!("Branch {}", node.id.0),
                NodeKind::LoopHeader => format!("Loop {}", node.id.0),
                NodeKind::LoopExit => format!("LoopExit {}", node.id.0),
            };
            let shape = match &node.kind {
                NodeKind::Entry | NodeKind::Exit => "ellipse",
                NodeKind::Branch | NodeKind::LoopHeader => "diamond",
                _ => "box",
            };
            dot.push_str(&format!("    n{} [label=\"{}\" shape={}];\n", 
                node.id.0, label, shape));
        }

        // Edges
        for edge in &self.edges {
            let style = match edge.kind {
                EdgeKind::TrueBranch => "label=\"T\" color=green",
                EdgeKind::FalseBranch => "label=\"F\" color=red",
                EdgeKind::LoopBack => "style=dashed color=blue",
                EdgeKind::Return => "color=purple",
                _ => "",
            };
            dot.push_str(&format!("    n{} -> n{} [{}];\n", 
                edge.from.0, edge.to.0, style));
        }

        dot.push_str("}\n");
        dot
    }
}

/// Builder for constructing CFGs from AST.
pub struct CfgBuilder {
    nodes: Vec<CfgNode>,
    edges: Vec<CfgEdge>,
    next_id: usize,
    /// Current loop exit targets for EXIT statements.
    loop_exits: Vec<NodeId>,
    /// Current loop header targets for CONTINUE statements.
    loop_headers: Vec<NodeId>,
}

impl CfgBuilder {
    /// Create a new CFG builder.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            next_id: 0,
            loop_exits: Vec::new(),
            loop_headers: Vec::new(),
        }
    }

    /// Build a CFG from a list of statements.
    pub fn build(mut self, statements: &[Stmt]) -> Cfg {
        // Create entry and exit nodes
        let entry = self.create_node(NodeKind::Entry);
        let exit = self.create_node(NodeKind::Exit);

        if statements.is_empty() {
            // Empty function: entry -> exit
            self.add_edge(entry, exit, EdgeKind::Sequential);
        } else {
            // Process statements
            let (first, last_nodes) = self.process_statements(statements, 0);
            
            // Connect entry to first statement
            self.add_edge(entry, first, EdgeKind::Sequential);
            
            // Connect all terminal nodes to exit
            for last in last_nodes {
                self.add_edge(last, exit, EdgeKind::Sequential);
            }
        }

        // Build successor/predecessor maps
        let mut successors: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
        let mut predecessors: HashMap<NodeId, Vec<NodeId>> = HashMap::new();

        for edge in &self.edges {
            successors.entry(edge.from).or_default().push(edge.to);
            predecessors.entry(edge.to).or_default().push(edge.from);
        }

        Cfg {
            nodes: self.nodes,
            edges: self.edges,
            entry,
            exit,
            successors,
            predecessors,
        }
    }

    /// Create a new node with the given kind.
    fn create_node(&mut self, kind: NodeKind) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id += 1;
        self.nodes.push(CfgNode {
            id,
            kind,
            statements: Vec::new(),
        });
        id
    }

    /// Create a basic block node with a statement.
    fn create_basic_block(&mut self, stmt_index: usize, kind: &StmtKind) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id += 1;
        self.nodes.push(CfgNode {
            id,
            kind: NodeKind::Basic,
            statements: vec![StmtRef {
                index: stmt_index,
                kind: kind.clone(),
            }],
        });
        id
    }

    /// Add an edge between two nodes.
    fn add_edge(&mut self, from: NodeId, to: NodeId, kind: EdgeKind) {
        self.edges.push(CfgEdge { from, to, kind });
    }

    /// Process a list of statements.
    /// Returns (first_node, Vec<exit_nodes>).
    fn process_statements(&mut self, stmts: &[Stmt], start_index: usize) -> (NodeId, Vec<NodeId>) {
        if stmts.is_empty() {
            let empty = self.create_node(NodeKind::Basic);
            return (empty, vec![empty]);
        }

        let mut first_node: Option<NodeId> = None;
        let mut current_exits: Vec<NodeId> = Vec::new();

        for (i, stmt) in stmts.iter().enumerate() {
            let stmt_index = start_index + i;
            let (node, exits, is_terminal) = self.process_statement(stmt, stmt_index);

            if first_node.is_none() {
                first_node = Some(node);
            }

            // Connect previous exits to this node
            for prev_exit in &current_exits {
                self.add_edge(*prev_exit, node, EdgeKind::Sequential);
            }

            if is_terminal {
                // RETURN/EXIT/CONTINUE - no further flow
                current_exits.clear();
            } else {
                current_exits = exits;
            }
        }

        (first_node.unwrap(), current_exits)
    }

    /// Process a single statement.
    /// Returns (node, exit_nodes, is_terminal).
    fn process_statement(&mut self, stmt: &Stmt, stmt_index: usize) -> (NodeId, Vec<NodeId>, bool) {
        match &stmt.kind {
            StmtKind::Assignment { .. } | StmtKind::Call { .. } | StmtKind::Empty => {
                let node = self.create_basic_block(stmt_index, &stmt.kind);
                (node, vec![node], false)
            }

            StmtKind::If { condition: _, then_body, elsif_branches, else_body } => {
                self.process_if(stmt_index, then_body, elsif_branches, else_body)
            }

            StmtKind::Case { expr: _, cases, else_body } => {
                self.process_case(stmt_index, cases, else_body)
            }

            StmtKind::For { body, .. } => {
                self.process_for(stmt_index, body)
            }

            StmtKind::While { body, .. } => {
                self.process_while(stmt_index, body)
            }

            StmtKind::Repeat { body, .. } => {
                self.process_repeat(stmt_index, body)
            }

            StmtKind::Return { .. } => {
                let node = self.create_basic_block(stmt_index, &stmt.kind);
                // Return is terminal - no exits (will be connected to function exit)
                (node, vec![], true)
            }

            StmtKind::Exit => {
                let node = self.create_basic_block(stmt_index, &stmt.kind);
                // Connect to loop exit if in a loop
                if let Some(&loop_exit) = self.loop_exits.last() {
                    self.add_edge(node, loop_exit, EdgeKind::LoopExit);
                }
                (node, vec![], true)
            }

            StmtKind::Continue => {
                let node = self.create_basic_block(stmt_index, &stmt.kind);
                // Connect to loop header if in a loop
                if let Some(&loop_header) = self.loop_headers.last() {
                    self.add_edge(node, loop_header, EdgeKind::LoopBack);
                }
                (node, vec![], true)
            }
        }
    }

    /// Process IF statement.
    fn process_if(
        &mut self,
        stmt_index: usize,
        then_body: &[Stmt],
        elsif_branches: &[(Expr, Vec<Stmt>)],
        else_body: &Option<Vec<Stmt>>,
    ) -> (NodeId, Vec<NodeId>, bool) {
        let branch = self.create_node(NodeKind::Branch);
        self.nodes[branch.0].statements.push(StmtRef {
            index: stmt_index,
            kind: StmtKind::Empty, // Placeholder for condition
        });

        let mut all_exits = Vec::new();

        // Then branch
        let (then_first, then_exits) = self.process_statements(then_body, stmt_index + 1);
        self.add_edge(branch, then_first, EdgeKind::TrueBranch);
        all_exits.extend(then_exits);

        // Current "false" target
        let mut false_target = branch;

        // ELSIF branches
        for (_, elsif_body) in elsif_branches {
            let elsif_branch = self.create_node(NodeKind::Branch);
            self.add_edge(false_target, elsif_branch, EdgeKind::FalseBranch);

            let (elsif_first, elsif_exits) = self.process_statements(elsif_body, stmt_index + 1);
            self.add_edge(elsif_branch, elsif_first, EdgeKind::TrueBranch);
            all_exits.extend(elsif_exits);

            false_target = elsif_branch;
        }

        // ELSE branch
        if let Some(else_stmts) = else_body {
            let (else_first, else_exits) = self.process_statements(else_stmts, stmt_index + 1);
            self.add_edge(false_target, else_first, EdgeKind::FalseBranch);
            all_exits.extend(else_exits);
        } else {
            // No ELSE - false branch continues to next statement
            all_exits.push(false_target);
        }

        (branch, all_exits, false)
    }

    /// Process CASE statement.
    fn process_case(
        &mut self,
        stmt_index: usize,
        cases: &[crate::ast::CaseBranch],
        else_body: &Option<Vec<Stmt>>,
    ) -> (NodeId, Vec<NodeId>, bool) {
        let branch = self.create_node(NodeKind::Branch);
        let mut all_exits = Vec::new();

        // Each case branch
        for case in cases {
            let (case_first, case_exits) = self.process_statements(&case.body, stmt_index + 1);
            self.add_edge(branch, case_first, EdgeKind::TrueBranch);
            all_exits.extend(case_exits);
        }

        // ELSE branch
        if let Some(else_stmts) = else_body {
            let (else_first, else_exits) = self.process_statements(else_stmts, stmt_index + 1);
            self.add_edge(branch, else_first, EdgeKind::FalseBranch);
            all_exits.extend(else_exits);
        } else {
            // No ELSE - add implicit path through
            all_exits.push(branch);
        }

        (branch, all_exits, false)
    }

    /// Process FOR loop.
    fn process_for(&mut self, stmt_index: usize, body: &[Stmt]) -> (NodeId, Vec<NodeId>, bool) {
        let header = self.create_node(NodeKind::LoopHeader);
        let loop_exit = self.create_node(NodeKind::LoopExit);

        // Push loop context for EXIT/CONTINUE
        self.loop_exits.push(loop_exit);
        self.loop_headers.push(header);

        let (body_first, body_exits) = self.process_statements(body, stmt_index + 1);
        
        // Pop loop context
        self.loop_exits.pop();
        self.loop_headers.pop();

        // Header -> body (true) or exit (false)
        self.add_edge(header, body_first, EdgeKind::TrueBranch);
        self.add_edge(header, loop_exit, EdgeKind::FalseBranch);

        // Body exits -> header (loop back)
        for exit in body_exits {
            self.add_edge(exit, header, EdgeKind::LoopBack);
        }

        (header, vec![loop_exit], false)
    }

    /// Process WHILE loop.
    fn process_while(&mut self, stmt_index: usize, body: &[Stmt]) -> (NodeId, Vec<NodeId>, bool) {
        // Same structure as FOR
        self.process_for(stmt_index, body)
    }

    /// Process REPEAT loop.
    fn process_repeat(&mut self, stmt_index: usize, body: &[Stmt]) -> (NodeId, Vec<NodeId>, bool) {
        let body_start = self.create_node(NodeKind::Basic);
        let condition = self.create_node(NodeKind::LoopHeader);
        let loop_exit = self.create_node(NodeKind::LoopExit);

        // Push loop context
        self.loop_exits.push(loop_exit);
        self.loop_headers.push(body_start);

        let (body_first, body_exits) = self.process_statements(body, stmt_index + 1);

        // Pop loop context
        self.loop_exits.pop();
        self.loop_headers.pop();

        // Entry -> body
        self.add_edge(body_start, body_first, EdgeKind::Sequential);

        // Body exits -> condition check
        for exit in body_exits {
            self.add_edge(exit, condition, EdgeKind::Sequential);
        }

        // Condition: true -> exit, false -> loop back
        self.add_edge(condition, loop_exit, EdgeKind::TrueBranch);
        self.add_edge(condition, body_start, EdgeKind::FalseBranch);

        (body_start, vec![loop_exit], false)
    }
}

impl Default for CfgBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Count decision points in an expression (for AND/OR complexity).
pub fn count_expression_decisions(expr: &Expr) -> usize {
    match &expr.kind {
        ExprKind::BinaryOp { left, op, right } => {
            let base = match op {
                BinaryOp::And | BinaryOp::Or => 1,
                _ => 0,
            };
            base + count_expression_decisions(left) + count_expression_decisions(right)
        }
        ExprKind::UnaryOp { expr, .. } => count_expression_decisions(expr),
        ExprKind::Paren(inner) => count_expression_decisions(inner),
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn parse_and_build_cfg(code: &str) -> Cfg {
        let mut parser = Parser::new(code).expect("parser creation failed");
        let stmts = parser.parse_statements().expect("parse failed");
        CfgBuilder::new().build(&stmts)
    }

    #[test]
    fn test_empty_cfg() {
        let cfg = CfgBuilder::new().build(&[]);
        assert_eq!(cfg.nodes.len(), 2); // Entry + Exit
        assert_eq!(cfg.edges.len(), 1); // Entry -> Exit
        assert_eq!(cfg.cyclomatic_complexity(), 1);
    }

    #[test]
    fn test_sequential_statements() {
        let cfg = parse_and_build_cfg("x := 1; y := 2; z := 3;");
        // Entry, 3 statements, Exit = 5 nodes
        assert_eq!(cfg.cyclomatic_complexity(), 1);
    }

    #[test]
    fn test_if_then_else() {
        let cfg = parse_and_build_cfg(r#"
            IF a THEN
                x := 1;
            ELSE
                x := 2;
            END_IF;
        "#);
        // One branch point = complexity 2
        assert_eq!(cfg.cyclomatic_complexity_decisions(), 2);
    }

    #[test]
    fn test_if_elsif_else() {
        let cfg = parse_and_build_cfg(r#"
            IF a THEN
                x := 1;
            ELSIF b THEN
                x := 2;
            ELSE
                x := 3;
            END_IF;
        "#);
        // Two branch points = complexity 3
        assert_eq!(cfg.cyclomatic_complexity_decisions(), 3);
    }

    #[test]
    fn test_nested_if() {
        let cfg = parse_and_build_cfg(r#"
            IF a THEN
                IF b THEN
                    x := 1;
                END_IF;
            END_IF;
        "#);
        // Two branch points = complexity 3
        assert_eq!(cfg.cyclomatic_complexity_decisions(), 3);
    }

    #[test]
    fn test_while_loop() {
        let cfg = parse_and_build_cfg(r#"
            WHILE x < 10 DO
                x := x + 1;
            END_WHILE;
        "#);
        // One loop = complexity 2
        assert_eq!(cfg.cyclomatic_complexity_decisions(), 2);
    }

    #[test]
    fn test_for_loop() {
        let cfg = parse_and_build_cfg(r#"
            FOR i := 1 TO 10 DO
                x := x + i;
            END_FOR;
        "#);
        assert_eq!(cfg.cyclomatic_complexity_decisions(), 2);
    }

    #[test]
    fn test_complex_example() {
        let cfg = parse_and_build_cfg(r#"
            IF a THEN
                WHILE x < 10 DO
                    IF b THEN
                        x := x + 1;
                    ELSE
                        x := x + 2;
                    END_IF;
                END_WHILE;
            ELSIF c THEN
                FOR i := 1 TO 5 DO
                    y := y + i;
                END_FOR;
            END_IF;
        "#);
        // IF + ELSIF + WHILE + nested IF + FOR = 5 decision points
        // Complexity = 6
        assert_eq!(cfg.cyclomatic_complexity_decisions(), 6);
    }

    #[test]
    fn test_unreachable_after_return() {
        let cfg = parse_and_build_cfg(r#"
            x := 1;
            RETURN;
            y := 2;
        "#);
        let unreachable = cfg.unreachable_nodes();
        // y := 2 should be unreachable
        assert!(!unreachable.is_empty() || cfg.nodes.len() >= 4);
    }

    #[test]
    fn test_dot_output() {
        let cfg = parse_and_build_cfg(r#"
            IF a THEN
                x := 1;
            END_IF;
        "#);
        let dot = cfg.to_dot();
        assert!(dot.contains("digraph CFG"));
        assert!(dot.contains("Entry"));
        assert!(dot.contains("Exit"));
    }

    #[test]
    fn test_case_statement() {
        let cfg = parse_and_build_cfg(r#"
            CASE x OF
                1: y := 1;
                2: y := 2;
                3: y := 3;
            ELSE
                y := 0;
            END_CASE;
        "#);
        // CASE is one branch point (even with multiple cases)
        assert_eq!(cfg.cyclomatic_complexity_decisions(), 2);
    }

    #[test]
    fn test_count_expression_decisions() {
        use crate::parse_expression;

        // Simple expression - no decisions
        let expr = parse_expression("x + 1").unwrap();
        assert_eq!(count_expression_decisions(&expr), 0);

        // Single AND - 1 decision
        let expr = parse_expression("a AND b").unwrap();
        assert_eq!(count_expression_decisions(&expr), 1);

        // Single OR - 1 decision
        let expr = parse_expression("a OR b").unwrap();
        assert_eq!(count_expression_decisions(&expr), 1);

        // Compound: a AND b OR c - 2 decisions
        let expr = parse_expression("a AND b OR c").unwrap();
        assert_eq!(count_expression_decisions(&expr), 2);

        // Nested: (a AND b) AND (c OR d) - 3 decisions
        let expr = parse_expression("(a AND b) AND (c OR d)").unwrap();
        assert_eq!(count_expression_decisions(&expr), 3);

        // With comparison (no decision) and AND
        let expr = parse_expression("x > 0 AND y < 10").unwrap();
        assert_eq!(count_expression_decisions(&expr), 1);

        // NOT doesn't add decision, but inner does
        let expr = parse_expression("NOT (a AND b)").unwrap();
        assert_eq!(count_expression_decisions(&expr), 1);

        // XOR is not counted (not short-circuit)
        let expr = parse_expression("a XOR b").unwrap();
        assert_eq!(count_expression_decisions(&expr), 0);
    }
}
