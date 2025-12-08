# plcscl v0.1.0 Release Notes

## Summary

First public release of plcscl - an SCL (Siemens Structured Control Language) parser for Rust.

## What's Included

### Core Functionality
- **Lexer**: Full tokenization of SCL source code
- **Parser**: Recursive descent parser with operator precedence climbing
- **AST**: Complete abstract syntax tree representation
- **Error Handling**: Detailed error messages with source context

### Language Support
- **Block Types**: FUNCTION, FUNCTION_BLOCK, DATA_BLOCK, ORGANIZATION_BLOCK, TYPE, PROGRAM, CLASS, INTERFACE
- **Control Flow**: IF/ELSIF/ELSE, CASE, FOR, WHILE, REPEAT, CONTINUE, EXIT, RETURN, GOTO
- **Data Types**: All SCL elementary types, strings, time types, arrays, structs, pointers
- **TIA Portal Extensions**: Pragmas, absolute addressing, regions, quoted identifiers

### Security Features
**v0.1.0 Implementation**: Input size validation only
- `parse_scl_secure()` function checks input size before parsing
- Three preset limits: strict(), balanced(), relaxed()
- Prevents memory exhaustion from excessively large inputs

**Planned for v0.2.0**: Runtime tracking
- Depth tracking (nesting levels)
- Iteration counting (loop iterations)
- Statement counting
- Full DoS protection

### API
```rust
// Standard parsing
pub fn parse_scl(input: &str) -> Result<Program, ParseError>

// Secure parsing (v0.1.0: input size validation only)
pub fn parse_scl_secure(input: &str, limits: ParserLimits) 
    -> Result<Program, SecureParseError>
```

### Documentation
- Comprehensive README.md with examples
- API documentation in lib.rs
- Examples in `examples/`:
  - `parse_file.rs` - Parse SCL file from disk
  - `parser_tests.rs` - Various SCL syntax tests
  - `security_test.rs` - Security limit demonstrations
  - `test_corpus.rs` - Test against corpus of real files

## Known Limitations

### Parser Success Rate
- **Current**: ~14% on test corpus of real Siemens SCL files
- **Reason**: Real-world SCL uses many undocumented TIA Portal features
- **Ongoing**: Parser improvements in progress

### Security Implementation
- **v0.1.0**: Only input size validation
- **Missing**: Runtime depth/iteration/statement tracking
- **Why**: Parser is generated from EBNF grammar; runtime tracking requires generator changes
- **Timeline**: Full integration planned for v0.2.0

### Language Features
- Multi-dimensional arrays not fully supported
- Some TIA Portal proprietary syntax may not parse

## Honest Assessment

This is a **foundational release**:
- ✅ Core parsing infrastructure works
- ✅ API is stable and usable
- ✅ Security module exists with proper types
- ⚠️ Parser success rate needs improvement
- ⚠️ Security is minimal (input size only)

**Use v0.1.0 for**:
- Experimentation with SCL parsing
- Building SCL tooling infrastructure
- Parsing simple/standard SCL code
- Contributing to parser improvements

**Not ready for**:
- Production parsing of complex TIA Portal code
- High-security environments (limited DoS protection)
- Mission-critical applications

## Roadmap

### v0.2.0 (Planned)
- Full security integration (runtime tracking)
- Parser improvements (target >50% success rate)
- Better error messages
- Multi-dimensional array support

### v0.3.0+ (Future)
- TIA Portal proprietary syntax improvements
- Parser success rate >80%
- Performance optimizations
- Semantic analysis tools

## Contributing

Parser improvements welcome! Test files in `~/devpublic/dataplc/` show real-world cases that need work.

## License

Dual-licensed: MIT OR Apache-2.0
