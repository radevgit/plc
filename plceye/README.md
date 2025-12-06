# plceye

A static analyzer and code smell detector for PLC files.

## Supported Formats

- **L5X** - Rockwell Automation Studio 5000 Logix Designer
- **PLCopen XML** - IEC 61131-3 standard exchange format

## Features

- **Unused Tags/Variables** (S0001) - Detect tags that are defined but never referenced
- **Undefined Tags** (S0002) - Find tags referenced in code but not declared
- **Empty Routines/POUs** (S0003) - Identify routines with no logic
- **Unused AOIs** (S0004) - Detect AOIs that are never called
- **Unused DataTypes** (S0005) - Find user-defined types that are never used
- **Cyclomatic Complexity** (M0001) - Detect overly complex ST routines (>10)
- **Deep Nesting** (M0003) - Find deeply nested control structures (>5 levels)
- **Statistics** - View file metrics including complexity analysis
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

# Show file statistics (no rule detection)
plceye --stats project.L5X

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

[unused_aois]
enabled = true

[unused_datatypes]
enabled = true

[complexity]
enabled = true
max_complexity = 10

[nesting]
enabled = true
max_depth = 5
```

## Output

```
=== project.L5X ===
[info] S0001: Controller - Tag 'Spare_01' is defined but never used (Spare_01)
[warning] S0002: Program:Main - Tag 'Unknown' is referenced but not defined (Unknown)
[info] M0001: Program:Main - Routine 'ComplexLogic' has cyclomatic complexity of 15 (max: 10) (ComplexLogic)

Found 3 issue(s) in 1 file(s).
```

## Rule Codes

| Code | Name | Description | Default Severity |
|------|------|-------------|------------------|
| S0001 | unused-tag | Tag/variable defined but never referenced | info |
| S0002 | undefined-tag | Tag referenced but not defined | warning |
| S0003 | empty-block | Routine/POU with no executable logic | info |
| S0004 | unused-aoi | AOI defined but never called | info |
| S0005 | unused-datatype | User-defined type never used | info |
| M0001 | cyclomatic-complexity | ST routine complexity exceeds threshold | info |
| M0003 | deep-nesting | Control structure nesting too deep | info |

## Library Usage

```rust
use plceye::{RuleDetector, LoadedProject};

let project = LoadedProject::from_file("project.L5X")?;
let detector = RuleDetector::new();
let report = detector.analyze(&project)?;

for rule in report.rules() {
    println!("{}", rule);
}

// Get statistics
let stats = detector.get_stats(&project)?;
println!("ST Routines: {}", stats.st_routines);
println!("Max Complexity: {}", stats.st_max_complexity);
```

## Disclaimer

This is an independent open-source project and is not affiliated with, endorsed by, or associated with Rockwell Automation, Inc.

"Rockwell Automation", "Allen-Bradley", "Studio 5000", and "Logix Designer" are trademarks of Rockwell Automation, Inc.

## License

MIT License - see [LICENSE](LICENSE) for details.
