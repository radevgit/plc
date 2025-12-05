# plcmodel

Vendor-neutral PLC model for IEC 61131-3 programs.

## Overview

This crate provides an abstract representation that different PLC formats (L5X, PLCopen XML, etc.) can map to, enabling format-independent analysis and tooling.

```text
┌─────────┐     ┌─────────────┐     ┌──────────┐
│   L5X   │────▶│             │────▶│ Analysis │
└─────────┘     │   PlcModel  │     └──────────┘
┌─────────┐     │             │     ┌──────────┐
│ PLCopen │────▶│             │────▶│  Viz     │
└─────────┘     └─────────────┘     └──────────┘
```

## Key Types

- **`Project`** - Top-level container for all PLC configuration
- **`Pou`** - Program Organization Unit (Program, Function, FunctionBlock)
- **`Variable`** - Variables with scope and data type
- **`DataTypeDef`** - User-defined types (struct, enum, array)
- **`Task`** - Task configuration for scheduling
- **`Body`** - Program body (instructions, networks)

## Usage

```rust
use plcmodel::{Project, Pou, Variable, ToPlcModel};
use iectypes::{PouType, VarClass};

// Create a project
let mut project = Project::new("MyProject");

// Add a POU
let mut main = Pou::new("Main", PouType::Program);
main.interface.inputs.push(Variable::input("Start", "BOOL"));
main.interface.locals.push(Variable::new("Counter", "INT"));

project.pous.push(main);
```

## Trait: ToPlcModel

Format-specific parsers implement this trait to convert to the common model:

```rust
use plcmodel::{Project, ToPlcModel};

// Implemented by l5x::Controller, plcopen::Project, etc.
pub trait ToPlcModel {
    fn to_plc_model(&self) -> Project;
}
```

## License

MIT
