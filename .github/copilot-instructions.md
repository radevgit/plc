my name is `zoo`

## Project Overview

This is a Rust workspace for parsing and analyzing PCL programming files.

**Primary goal:** Build tools for PLC code analysis, starting with a **Code Smell Detector / Static Analyzer**.

### Error Handling
- Permissive parsing: continue on non-critical errors
- Log warnings for unexpected content
- Use enum-based errors (not `String` in `Err`)
- dont use `panic!` in code, only in tests

## GIT
- do not commit in git, except explicitely asked

## When Helping with This Project

1. **Check existing docs first** - Many decisions are documented in `docs/`
2. **Use existing patterns** - Follow the XSD parser/codegen structure
3. **Test with real files** - Use files in `~/devpublic/dataplc/` directory
4. **Specificantion documents** - `~/devpublic/specs/`



