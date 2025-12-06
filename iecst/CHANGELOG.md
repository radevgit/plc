# Changelog

## 0.2.0 (2024-12-06)

### Added
- **Control Flow Graph (CFG)** - new `analysis::cfg` module
  - `CfgBuilder` - construct CFG from ST statements
  - `Cfg` - graph structure with nodes and edges
  - `cyclomatic_complexity()` - calculate complexity using edge formula (E - N + 2)
  - `cyclomatic_complexity_decisions()` - count decision points (branches, loops)
  - `unreachable_nodes()` - detect dead code after RETURN/EXIT
  - `has_path(from, to)` - check reachability between nodes
  - `to_dot()` - export to Graphviz DOT format for visualization
- `count_expression_decisions()` - count AND/OR operators in conditions
- New CFG node types: Entry, Exit, Basic, Branch, LoopHeader, LoopExit
- New CFG edge types: Sequential, TrueBranch, FalseBranch, LoopBack, LoopExit, Return

### Changed
- Re-export CFG types from `iecst`: `Cfg`, `CfgBuilder`, `CfgNode`, `NodeId`, `NodeKind`

## 0.1.0 (2024-12-04)

- Initial release
- IEC 61131-3 Structured Text lexer
- ST parser with AST generation
- Symbol table and type checking
