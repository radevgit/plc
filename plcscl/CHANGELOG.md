# Changelog

## [0.2.0] - 2025-12-09

### Added
- GOTO and label statements (IEC 61131-3 standard)
- Compound assignment operators (+=, -=, *=, /=) for TIA Portal
- Attribute syntax `{attribute 'name'}` for Beckhoff/TwinCAT compatibility
- VAR_* RETAIN/NON_RETAIN support (VAR_INPUT RETAIN, VAR_OUTPUT RETAIN, VAR_IN_OUT RETAIN, VAR_TEMP RETAIN, VAR_EXTERNAL RETAIN)

### Fixed
- Parser generator now handles optional RETAIN/NON_RETAIN keywords after VAR_INPUT, VAR_OUTPUT, VAR_IN_OUT, VAR_TEMP, VAR_EXTERNAL
- Files that previously failed on VAR_IN_OUT RETAIN (4 files) now parse further into the file

### Improved
- Parser success rate: 37.1% (up from 14.3%, 2.6x improvement)
  - Note: Files with VAR_IN_OUT RETAIN now parse past that construct but may fail on other features
- Grammar now has 109 rules (92 base SCL + 17 TIA extensions)

### Known Issues
- Some attribute block syntaxes cause parse errors
- Type declaration semicolon handling needs improvement

## [0.1.0] - 2025-12-08

### Added
- Initial release: SCL parser with lexer, AST, and EBNF-generated parser
- Siemens TIA Portal extensions: pragmas, absolute addressing, regions, quoted identifiers
- Block types: FUNCTION, FUNCTION_BLOCK, DATA_BLOCK, ORGANIZATION_BLOCK, TYPE, PROGRAM, CLASS, INTERFACE
- Control statements: IF/ELSIF/ELSE, CASE, FOR, WHILE, REPEAT
- Security: Built-in DoS protection in generated parser (recursion depth, complexity, iterations, collection size limits)
- Error messages with source context

### Known Limitations
- ~14% parser success rate on test corpus (improvement ongoing)
- Multi-dimensional arrays not fully supported

