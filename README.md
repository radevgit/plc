# PLC - L5X Tools

Open source tools for parsing and analyzing L5X files exported from PLC programming software.

## Crates

- **plceye** - PLC code smell detector CLI
- **plcviz** - PLC code visualization (SVG graph generator)
- **l5x** - L5X file parser library
- **iecst** - IEC 61131-3 Structured Text parser


## Detected Issues

- **Unused Tags** - Tags defined but never referenced in code
- **Undefined Tags** - Tags referenced but not declared (may indicate typos or missing definitions)
- **Empty Routines** - Routines with no logic (RLL with no rungs, ST with no statements)

## Installation

### Binary Releases (Recommended)

Download pre-built binaries from [GitHub Releases](https://github.com/radevgit/plc/releases):

**Linux:**
```bash
# plcviz
curl -L https://github.com/radevgit/plc/releases/download/plcviz-v0.3.2/plcviz-v0.3.2-x86_64-linux.tar.gz | tar xz
sudo mv plcviz /usr/local/bin/
```

**Windows:**
```powershell
# Download from GitHub Releases and extract
# Add to PATH or run from current directory
.\plcviz.exe --version
```

### From Source

Requires Rust 1.70+:

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
