# iecst

> **⚠️ DEPRECATED**: This crate is deprecated in favor of [`iec61131`](https://crates.io/crates/iec61131), which provides complete IEC 61131-3 support for all 5 programming languages (ST, IL, LD, FBD, SFC) based on the official specification. New projects should use `iec61131` instead.

IEC 61131-3 Structured Text parser for Rust.



[![Crates.io](https://img.shields.io/crates/v/iecst.svg)](https://crates.io/crates/iecst)
[![Documentation](https://docs.rs/iecst/badge.svg)](https://docs.rs/iecst)
[![License](https://img.shields.io/crates/l/iecst.svg)](LICENSE)

## Overview

`iecst` is a parser for IEC 61131-3 Structured Text (ST), the high-level programming language used in PLC (Programmable Logic Controller) programming.

## Features

- **Full ST syntax support** - expressions, statements, POUs, declarations
- **Minimal dependencies** (`winnow` for parsing)
- **Detailed error reporting** with source locations
- **AST output** for further analysis or transformation
- **Control Flow Graph (CFG)** - build CFGs from ST code for complexity analysis
- **Cyclomatic complexity** - calculate code complexity metrics
- **Nesting depth analysis** - detect deeply nested control structures
- **Basic static analysis** - type checking, unused variables, diagnostics
- **Security features** - DoS protection with configurable resource limits

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
iecst = "0.5"
```

## Quick Start

```rust
use iecst::{parse_statement, parse_expression};

// Parse a statement
let stmt = parse_statement("x := 1 + 2;").unwrap();

// Parse an expression
let expr = parse_expression("a AND b OR c").unwrap();
```

## Security

For untrusted input, use security limits to prevent DoS attacks:

```rust
use iecst::{Parser, security::ParserLimits};

// Use strict limits for untrusted input
let limits = ParserLimits::strict();
let parser = Parser::new_with_limits(source, limits)?;

// Available profiles:
// - ParserLimits::strict()    - For untrusted input (10 MB, 64 levels)
// - ParserLimits::balanced()  - For typical files (100 MB, 256 levels)
// - ParserLimits::relaxed()   - For trusted files (500 MB, 512 levels)
```

## Supported Constructs

### Expressions
- Literals: integers, reals, strings, booleans, time literals
- Identifiers and qualified names
- Binary operators: `+`, `-`, `*`, `/`, `MOD`, `**`, `AND`, `OR`, `XOR`, comparisons
- Unary operators: `-`, `NOT`
- Function calls: `SIN(x)`, `MAX(a, b)`
- Array indexing: `arr[i]`, `arr[i, j]`
- Member access: `struct.field`

### Statements
- Assignment: `x := expr;`
- IF/THEN/ELSIF/ELSE/END_IF
- CASE/OF/ELSE/END_CASE
- FOR/TO/BY/DO/END_FOR
- WHILE/DO/END_WHILE
- REPEAT/UNTIL/END_REPEAT
- EXIT, RETURN, CONTINUE

### Program Organization Units (POUs)
- PROGRAM/END_PROGRAM
- FUNCTION/END_FUNCTION
- FUNCTION_BLOCK/END_FUNCTION_BLOCK

### Declarations
- VAR/VAR_INPUT/VAR_OUTPUT/VAR_IN_OUT/VAR_TEMP/VAR_GLOBAL
- TYPE/END_TYPE (structures, enums, arrays, subranges)

## Example: Parsing a Function Block

```rust
use iecst::parse_pou;

let code = r#"
FUNCTION_BLOCK Counter
    VAR_INPUT
        Reset : BOOL;
    END_VAR
    VAR_OUTPUT
        Count : INT;
    END_VAR
    VAR
        _count : INT := 0;
    END_VAR
    
    IF Reset THEN
        _count := 0;
    ELSE
        _count := _count + 1;
    END_IF;
    
    Count := _count;
END_FUNCTION_BLOCK
"#;

let pou = parse_pou(code).unwrap();
println!("Parsed: {:?}", pou.name);
```

## Static Analysis

```rust
use iecst::{parse_pou, analyze_pou};

let code = "FUNCTION Test : INT VAR x : INT; END_VAR x := 1; Test := x; END_FUNCTION";
let pou = parse_pou(code).unwrap();
let diagnostics = analyze_pou(&pou);

for diag in diagnostics {
    println!("[{}] {}", diag.severity, diag.message);
}
```

## Control Flow Graph

Build a CFG from ST statements to analyze control flow and calculate complexity:

```rust
use iecst::{parse_statements, CfgBuilder};

let code = r#"
    IF x > 0 THEN
        y := 1;
    ELSIF x < 0 THEN
        y := -1;
    ELSE
        y := 0;
    END_IF;
"#;

let stmts = parse_statements(code).unwrap();
let cfg = CfgBuilder::new().build(&stmts);

// Calculate cyclomatic complexity
let complexity = cfg.cyclomatic_complexity();
println!("Cyclomatic complexity: {}", complexity); // Output: 3

// Export to Graphviz DOT format for visualization
let dot = cfg.to_dot();
println!("{}", dot);
```

## Nesting Depth Analysis

Detect deeply nested control structures:

```rust
use iecst::{parse_statements, max_nesting_depth};

let code = r#"
    IF a THEN
        FOR i := 1 TO 10 DO
            WHILE b DO
                x := x + 1;
            END_WHILE;
        END_FOR;
    END_IF;
"#;

let stmts = parse_statements(code).unwrap();
let depth = max_nesting_depth(&stmts);
println!("Max nesting depth: {}", depth); // Output: 3
```

### CFG Features

- **Cyclomatic complexity** - `cfg.cyclomatic_complexity()` using edge formula (E - N + 2)
- **Decision-based complexity** - `cfg.cyclomatic_complexity_decisions()` counting branch points
- **Unreachable code detection** - `cfg.unreachable_nodes()` finds dead code
- **Path analysis** - `cfg.has_path(from, to)` checks reachability
- **DOT export** - `cfg.to_dot()` for Graphviz visualization

