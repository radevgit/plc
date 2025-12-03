# iecst

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
- **Basic static analysis** - type checking, unused variables, code smells

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
iecst = "0.1"
```

## Quick Start

```rust
use iecst::{parse_statement, parse_expression};

// Parse a statement
let stmt = parse_statement("x := 1 + 2;").unwrap();

// Parse an expression
let expr = parse_expression("a AND b OR c").unwrap();
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

