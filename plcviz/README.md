# plcviz

PLC code visualization tool - generate SVG diagrams from L5X files.

## Features

- **Structure graphs**: Program → Routine containment hierarchy
- **Call graphs**: JSR (Jump to Subroutine) relationships between routines
- **Combined graphs**: Structure + call edges together
- **Multiple export types**: Controller, Program, and AOI exports

## Installation

```bash
cargo install --path .
```

## Usage

```bash
# Generate structure graph (default)
plcviz project.L5X > structure.svg

# Generate call graph
plcviz -t call project.L5X > calls.svg

# Generate combined graph (structure + calls)
plcviz -t combined project.L5X > combined.svg

# Include AOIs in the graph
plcviz -a project.L5X > with_aois.svg

# Generate example graph (no L5X file needed)
plcviz example > example.svg
```

## Graph Types

| Type | Description |
|------|-------------|
| `structure` | Containment hierarchy (Programs → Routines) |
| `call` | JSR calls between routines |
| `dataflow` | Tag read/write relationships (example only) |
| `combined` | Structure + call edges |

## Supported L5X Export Types

- **Controller**: Full project exports (multiple programs)
- **Program**: Single program exports  
- **AddOnInstructionDefinition**: AOI exports with internal routines

## License

MIT
