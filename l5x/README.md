# l5x

A Rust library for parsing L5X files exported from Studio 5000 Logix Designer.

## Features

- **RLL (Relay Ladder Logic) parsing** - parse ladder logic instructions into AST
- **Tag reference extraction** - find all tag references in rungs
- **Security/Safety features** - protection against certan type of badly formed XML

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
l5x = "0.4"
```

## Usage

### Parse an L5X file

```rust
use l5x::Project;

let xml = std::fs::read_to_string("project.L5X")?;
let project: Project = l5x::from_str(&xml)?;

// Access controller information
println!("Controller: {}", project.controller.name);

// Iterate over programs
for program in &project.controller.programs.program {
    println!("Program: {}", program.name);
}
```

### Parse with security limits (recommended for untrusted files)

```rust
use l5x::{Project, from_str_secure, security::SecurityLimits};

// Use strict limits for untrusted input
let limits = SecurityLimits::strict();
let xml = std::fs::read_to_string("untrusted.L5X")?;

match from_str_secure::<Project>(&xml, &limits) {
    Ok(project) => println!("Parsed: {}", project.controller.name),
    Err(e) => eprintln!("Security validation failed: {}", e),
}

// Available profiles:
// - SecurityLimits::strict()    - For untrusted input (10 MB, 32 levels)
// - SecurityLimits::balanced()  - For typical files (100 MB, 100 levels) 
// - SecurityLimits::relaxed()   - For trusted files (500 MB, 256 levels)
```

### Parse ladder logic rungs

```rust
use l5x::rll::parse_rung;

let rung = parse_rung("XIC(Start)OTE(Motor);");

// Extract tag references
for tag in rung.tag_references() {
    println!("Tag: {} ({})", tag.name, if tag.is_write { "write" } else { "read" });
}
```

### Access tags and data types

```rust
use l5x::Project;

let project: Project = l5x::from_str(&xml)?;

// Controller-scoped tags
if let Some(tags) = &project.controller.tags {
    for tag in &tags.tag {
        println!("Tag: {} : {}", tag.name, tag.data_type);
    }
}

// User-defined types
if let Some(datatypes) = &project.controller.data_types {
    for dt in &datatypes.data_type {
        println!("DataType: {}", dt.name);
    }
}
```

## L5X File Format

L5X is an XML-based export format for Rockwell Automation PLCs. This crate supports:

- **Controllers**: ControlLogix, CompactLogix
- **Programs**: with routines in RLL, ST, FBD, SFC
- **Tags**: Controller and program-scoped
- **Data Types**: Built-in and user-defined (UDTs)
- **Add-On Instructions (AOIs)**
- **Modules**: I/O configuration
- **Tasks**: Continuous, periodic, event

## RLL Instruction Support

The RLL parser handles standard ladder logic syntax:

- **Instructions**: XIC, XIO, OTE, OTL, OTU, TON, TOF, CTU, CTD, MOV, ADD, etc.
- **Branches**: Parallel and series connections
- **Expressions**: Arithmetic and comparison in CMP, CPT, etc.
- **Arrays**: Multi-dimensional with literal or tag indices
- **Structured tags**: Member access (e.g., `Timer.DN`)

## Disclaimer

This is an independent open-source project and is not affiliated with, endorsed by, sponsored by, or associated with Rockwell Automation, Inc. or any of its subsidiaries or affiliates.

