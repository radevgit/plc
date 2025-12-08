# Changelog

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

