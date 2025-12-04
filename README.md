# PLC - L5X Tools

Open source tools for parsing and analyzing L5X files exported from PLC programming software.

## Crates

- **l5x** - L5X file parser library
- **iecst** - IEC 61131-3 Structured Text parser
- **plceye** - PLC code smell detector CLI
- **plcviz** - PLC code visualization (SVG graph generator)

## Detected Issues

- **Unused Tags** - Tags defined but never referenced in code
- **Undefined Tags** - Tags referenced but not declared (may indicate typos or missing definitions)
- **Empty Routines** - Routines with no logic (RLL with no rungs, ST with no statements)

## Installation

```bash
cargo install --path plceye
cargo install --path plcviz
```

## Usage

### plceye

```bash
plceye --help

# Analyze an L5X file
plceye path/to/project.L5X

# Generate default config
plceye init

# Show only warnings and errors
plceye --severity warning path/to/project.L5X
```

### plcviz

```bash
plcviz --help

# Generate structure graph
plcviz path/to/project.L5X > graph.svg

# Generate call graph
plcviz -t call path/to/project.L5X > calls.svg

# Generate dataflow graph
plcviz -t dataflow path/to/project.L5X > dataflow.svg

# Generate combined graph
plcviz -t combined path/to/project.L5X > combined.svg
```
