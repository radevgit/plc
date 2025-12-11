# Changelog

## 0.6.1 (2025-12-11)
- parser analysis added
- parser security added

## 0.6.0 (2025-12-11)
- The parser was rewritten with new architecture

## 0.5.0 (2025-12-09)
- Fixed security module

## 0.4.0 (2025-12-08)

### Added
- **Security module** - DoS attack protection for untrusted input
  - `ParserLimits` - configurable limits (strict/balanced/relaxed)
  - `ParserState` - runtime tracking of parser resource usage
  - `SecurityError` - comprehensive error types for limit violations
  - Input size, nesting depth, iteration, collection size limits
- Added `thiserror` dependency for error handling

## 0.3.0 (2025-12-06)

### Added
- **Nesting depth analysis** - new `analysis::nesting` module
  - `max_nesting_depth()` - calculate maximum nesting depth of control structures
  - Counts IF, CASE, FOR, WHILE, REPEAT as nesting levels
  - Useful for detecting overly complex code (M0003 deep nesting rule)

### Changed
- Re-export `max_nesting_depth` from `iecst`

## 0.2.0 (2025-12-06)

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

## 0.1.0 (2025-12-04)

- Initial release
- IEC 61131-3 Structured Text lexer
- ST parser with AST generation
- Symbol table and type checking
