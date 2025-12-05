# plcopen

Parser for PLCopen TC6 XML files - the IEC 61131-3 standard exchange format for PLC programs.

## Features

- Type-safe parsing using quick-xml and serde
- Generated types from the official PLCopen TC6 XML schema (v2.01)
- Support for all IEC 61131-3 languages (ST, IL, LD, FBD, SFC)

## Usage

```rust
use plcopen::Project;

// Parse PLCopen XML file
let xml = std::fs::read_to_string("project.xml")?;
let project: Project = plcopen::from_str(&xml)?;
println!("Project: {:?}", project);
```

## PLCopen TC6 XML Format

PLCopen TC6 is an XML-based exchange format defined by the PLCopen organization 
for transferring PLC programs between different development environments. It supports:

- **Program Organization Units (POUs)**: Programs, Functions, Function Blocks
- **Data Types**: Elementary types (BOOL, INT, REAL, etc.) and user-defined types
- **Variables**: Input, Output, InOut, Local, Global, External
- **Languages**: ST (Structured Text), IL (Instruction List), LD (Ladder), FBD (Function Block Diagram), SFC (Sequential Function Chart)

## License

MIT
