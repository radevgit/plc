# iec61131

IEC 61131-3 Structured Text (ST) parser for PLC programming.

[![Crates.io](https://img.shields.io/crates/v/iec61131.svg)](https://crates.io/crates/iec61131)
[![Documentation](https://docs.rs/iec61131/badge.svg)](https://docs.rs/iec61131)
[![License](https://img.shields.io/crates/l/iec61131.svg)](LICENSE)

## Overview

`iec61131` is a parser for IEC 61131-3 Structured Text (ST), the international standard for PLC (Programmable Logic Controller) programming. Currently supports:

- **ST** (Structured Text) - High-level programming language ✅

**Planned support** for additional IEC 61131-3 languages:
- **IL** (Instruction List) - Low-level assembly-like language
- **LD** (Ladder Diagram) - Graphical ladder logic representation
- **FBD** (Function Block Diagram) - Graphical function block representation
- **SFC** (Sequential Function Chart) - State machine representation

## Features

- ✅ **Structured Text (ST) support** - Based on IEC 61131-3:2013 specification
- ✅ **Full ST syntax support** - Functions, function blocks, programs, classes, interfaces
- ✅ **Modern PLC features** - OOP (classes, interfaces), namespaces, references
- ✅ **Detailed error reporting** - Source locations and helpful messages
- ✅ **Security limits** - DoS protection with configurable resource limits
- ✅ **Production ready** - Comprehensive testing and validation

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
iec61131 = "0.7"
```

## Quick Start

```rust
use iec61131::Parser;

// Parse a Structured Text function
let code = r#"
FUNCTION Add : INT
    VAR_INPUT
        a : INT;
        b : INT;
    END_VAR
    
    Add := a + b;
END_FUNCTION
"#;

let mut parser = Parser::new(code);
let ast = parser.parse().expect("Parse error");

println!("Parsed {} declarations", ast.declarations.len());
```

## Supported Constructs

### Program Organization Units (POUs)

- `FUNCTION` / `END_FUNCTION`
- `FUNCTION_BLOCK` / `END_FUNCTION_BLOCK`
- `PROGRAM` / `END_PROGRAM`
- `CLASS` / `END_CLASS` (OOP)
- `INTERFACE` / `END_INTERFACE` (OOP)
- `METHOD` / `END_METHOD` (OOP)

### Variable Declarations

- `VAR`, `VAR_INPUT`, `VAR_OUTPUT`, `VAR_IN_OUT`
- `VAR_TEMP`, `VAR_EXTERNAL`, `VAR_GLOBAL`
- `VAR_ACCESS`, `VAR_CONFIG`
- `CONSTANT`, `RETAIN`, `NON_RETAIN`
- `AT` locations (direct variables)

### Data Types

- **Elementary**: BOOL, BYTE, WORD, DWORD, LWORD, SINT, INT, DINT, LINT, USINT, UINT, UDINT, ULINT, REAL, LREAL
- **Strings**: STRING, WSTRING, CHAR, WCHAR
- **Time**: TIME, LTIME, DATE, LDATE, TIME_OF_DAY, DATE_AND_TIME
- **Structured**: ARRAY, STRUCT
- **User-defined**: TYPE declarations, enumerations, subranges
- **References**: REF_TO

### Statements

- **Assignments**: `variable := expression`
- **Conditionals**: `IF...THEN...ELSIF...ELSE...END_IF`
- **Selection**: `CASE...OF...END_CASE`
- **Loops**: `FOR...TO...BY...DO...END_FOR`, `WHILE...DO...END_WHILE`, `REPEAT...UNTIL...END_REPEAT`
- **Control**: `EXIT`, `CONTINUE`, `RETURN`
- **Invocations**: Function calls, FB invocations

### Expressions

- **Arithmetic**: `+`, `-`, `*`, `/`, `MOD`, `**` (power)
- **Comparison**: `=`, `<>`, `<`, `<=`, `>`, `>=`
- **Logical**: `AND`, `OR`, `XOR`, `NOT`, `&`
- **Literals**: Integers, reals, strings, booleans, time values
- **Variables**: Simple, member access, array indexing, dereferencing

### Object-Oriented Programming

- Classes with inheritance (`EXTENDS`)
- Interfaces with implementation (`IMPLEMENTS`)
- Access modifiers (`PUBLIC`, `PROTECTED`, `PRIVATE`, `INTERNAL`)
- Method modifiers (`FINAL`, `ABSTRACT`, `OVERRIDE`)
- `THIS` and `SUPER` keywords

### Advanced Features

- Namespaces (`NAMESPACE...END_NAMESPACE`)
- Using directives (`USING`)
- Configuration and resources
- Sequential Function Charts (SFC)
- Direct variables (`%IX0.0`, `%QW10`)

## Examples

### Parse a Function Block

```rust
use iec61131::Parser;

let code = r#"
FUNCTION_BLOCK Counter
    VAR_INPUT
        reset : BOOL;
    END_VAR
    VAR_OUTPUT
        count : INT;
    END_VAR
    
    IF reset THEN
        count := 0;
    ELSE
        count := count + 1;
    END_IF
END_FUNCTION_BLOCK
"#;

let mut parser = Parser::new(code);
let ast = parser.parse()?;
```

## Security

For untrusted input, use security limits to prevent denial-of-service attacks:

```rust
use iec61131::{Parser, ParserLimits};

let input = load_untrusted_file()?;

// Use strict limits for untrusted input
let limits = ParserLimits::strict();
if input.len() > limits.max_input_size {
    return Err("Input too large");
}

let mut parser = Parser::new(&input);
let ast = parser.parse()?;
```

Available security profiles:

- `ParserLimits::strict()` - For untrusted/external input (10MB max, 64 depth)
- `ParserLimits::balanced()` - For general use (100MB max, 256 depth) [default]
- `ParserLimits::relaxed()` - For trusted/internal files (500MB max, 512 depth)

The limits protect against:
- Stack overflow from deeply nested code
- Memory exhaustion from huge arrays/collections
- CPU exhaustion from pathological inputs

### Parse a Program with Multiple POUs

```rust
use iec61131::Parser;

let code = r#"
TYPE
    Color : (Red, Green, Blue);
END_TYPE

FUNCTION_BLOCK Motor
    VAR_INPUT
        enable : BOOL;
        speed : INT;
    END_VAR
    // ...
END_FUNCTION_BLOCK

PROGRAM Main
    VAR
        motor1 : Motor;
        currentColor : Color;
    END_VAR
    
    motor1(enable := TRUE, speed := 100);
    
    CASE currentColor OF
        Red: motor1.speed := 50;
        Green: motor1.speed := 100;
        Blue: motor1.speed := 150;
    END_CASE
END_PROGRAM
"#;

let mut parser = Parser::new(code);
let ast = parser.parse()?;
```

## Architecture

The parser uses a two-stage approach:

1. **Lexer** - Tokenizes input into keywords, identifiers, operators, literals
2. **Parser** - Recursive descent parser that builds an Abstract Syntax Tree (AST)

The AST fully represents the structure of IEC 61131-3 programs and can be used for:
- Static analysis and linting
- Code transformation and optimization
- Code generation for different targets
- Documentation generation
- IDE support (syntax highlighting, completion, etc.)

## Comparison with `iecst`

This crate (`iec61131`) supersedes the older `iecst` crate with several improvements:

| Feature | iecst | iec61131 |
|---------|-------|----------|
| Languages | ST only | ST only (currently) |
| IEC Version | Partial | IEC 61131-3:2013 ST |
| OOP Support | Limited | Full (classes, interfaces) |
| Namespaces | No | Yes |
| Specification | Manual implementation | Based on official specification |
| AST | Basic | Comprehensive |

For new projects, use `iec61131`. The `iecst` crate remains available for backward compatibility.

## Development

This crate is generated from the official IEC 61131-3:2013 EBNF specification using the `plcp/iec61131` parser generator.

```bash
# Run tests
cargo test

# Build documentation
cargo doc --open
```

## License

MIT