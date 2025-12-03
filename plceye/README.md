# plceye

A static analyzer and code smell detector for Rockwell Automation L5X files (Studio 5000 Logix Designer).

## Features

- **Unused Tags** - Detect tags that are defined but never referenced
- **Undefined Tags** - Find tags referenced in code but not declared
- **Empty Routines** - Identify routines with no logic
- **Configurable** - Customize detection via `plceye.toml`

## Installation

```bash
cargo install plceye
```

## Usage

```bash
# Analyze a single file
plceye project.L5X

# Analyze multiple files
plceye *.L5X

# Use custom configuration
plceye --config plceye.toml project.L5X

# Generate default configuration
plceye init
```

## Configuration

Create a `plceye.toml` file to customize detection:

```toml
[unused_tags]
enabled = true
severity = "info"
# Ignore tags matching these patterns
ignore = ["zz*", "Spare_*", "_*"]

[undefined_tags]
enabled = true
severity = "warning"
# Known valid references (aliases, I/O modules)
ignore = ["Local:*", "S:*"]

[empty_routines]
enabled = true
severity = "info"
ignore = ["*_Template"]
```

## Output

```
=== project.L5X ===
[info] unused-tag: Controller - Tag 'Spare_01' is defined but never used
[warning] undefined-tag: Program:Main - Tag 'Unknown' is referenced but not defined

Found 2 issue(s) in 1 file(s).
```

## Detected Issues

| Issue | Description | Default Severity |
|-------|-------------|------------------|
| `unused-tag` | Tag defined but never referenced | info |
| `undefined-tag` | Tag referenced but not defined | warning |
| `empty-routine` | Routine with no executable logic | info |

## Disclaimer

This is an independent open-source project and is not affiliated with, endorsed by, or associated with Rockwell Automation, Inc.

"Rockwell Automation", "Allen-Bradley", "Studio 5000", and "Logix Designer" are trademarks of Rockwell Automation, Inc.

## License

MIT License - see [LICENSE](LICENSE) for details.
