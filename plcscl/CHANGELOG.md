# Changelog

## [0.1.0] - 2025-12-08

### Added
- Initial release: SCL parser with lexer, AST, and EBNF-generated parser
- Siemens TIA Portal extensions: pragmas, absolute addressing, regions, quoted identifiers
- Block types: FUNCTION, FUNCTION_BLOCK, DATA_BLOCK, ORGANIZATION_BLOCK, TYPE, PROGRAM, CLASS, INTERFACE
- Control statements: IF/ELSIF/ELSE, CASE, FOR, WHILE, REPEAT
- Security: `parse_scl_secure()` with input size validation (runtime tracking planned for v0.2.0)
- Error messages with source context

### Known Limitations
- ~14% parser success rate on test corpus (improvement ongoing)
- Multi-dimensional arrays not fully supported
- Security: Only input size validation in v0.1.0; runtime depth/iteration tracking coming in v0.2.0

