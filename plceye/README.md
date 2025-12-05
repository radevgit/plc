# plceye

A static analyzer and code smell detector for PLC files.

## Supported Formats

- **L5X** - Rockwell Automation Studio 5000 Logix Designer
- **PLCopen XML** - IEC 61131-3 standard exchange format

## Features

- **Unused Tags/Variables** (P00001) - Detect tags that are defined but never referenced
- **Undefined Tags** (P00002) - Find tags referenced in code but not declared
- **Empty Routines/POUs** (P00003) - Identify routines with no logic
- **Configurable** - Customize detection via `plceye.toml`

## Installation

```bash
cargo install plceye
```

## Usage

```bash
# Analyze a single file
plceye project.L5X

# Analyze PLCopen XML file
plceye project.xml

# Analyze multiple files
plceye *.L5X

# Use custom configuration
plceye --config plceye.toml project.L5X

# Set minimum severity level
plceye --severity warning project.L5X

# Generate default configuration
plceye init
```

## Configuration

Create a `plceye.toml` file to customize detection:

```toml
[general]
# Minimum severity to report: "info", "warning", "error"
min_severity = "info"

[unused_tags]
enabled = true
# Ignore tags matching these patterns (glob-style)
ignore_patterns = ["_*", "HMI_*"]
# Ignore tags in these scopes
ignore_scopes = []

[undefined_tags]
enabled = true
# Ignore undefined tags matching these patterns
ignore_patterns = ["Local:*"]

[empty_routines]
enabled = true
ignore_patterns = []
```

## Output

```
=== project.L5X ===
[info] P00001: Controller - Tag 'Spare_01' is defined but never used (Spare_01)
[warning] P00002: Program:Main - Tag 'Unknown' is referenced but not defined (Unknown)

Found 2 issue(s) in 1 file(s).
```

## Rule Codes

| Code | Name | Description | Default Severity |
|------|------|-------------|------------------|
| P00001 | unused-tag | Tag/variable defined but never referenced | info |
| P00002 | undefined-tag | Tag referenced but not defined | warning |
| P00003 | empty-block | Routine/POU with no executable logic | info |

## Library Usage

```rust
use plceye::{SmellDetector, LoadedProject};

let project = LoadedProject::from_file("project.L5X")?;
let detector = SmellDetector::new();
let report = detector.analyze(&project)?;

for smell in report.smells() {
    println!("{}", smell);
}
```

## Disclaimer

This is an independent open-source project and is not affiliated with, endorsed by, or associated with Rockwell Automation, Inc.

"Rockwell Automation", "Allen-Bradley", "Studio 5000", and "Logix Designer" are trademarks of Rockwell Automation, Inc.

## License

MIT License - see [LICENSE](LICENSE) for details.
