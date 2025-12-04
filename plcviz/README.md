# plcviz

PLC code visualization library for Rust.

Generate SVG diagrams from L5X files:
- Call graphs (routine → routine)
- Tag dataflow
- AOI dependencies
- Program structure

## Usage

```rust
use l5x::L5xFile;
use plcviz::{CallGraph, SvgRenderer};

let project = L5xFile::parse_file("project.l5x")?;
let graph = CallGraph::from_project(&project);
let svg = SvgRenderer::new().render(&graph);

std::fs::write("call_graph.svg", svg)?;
```

## License

MIT
