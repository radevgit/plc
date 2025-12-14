# Changelog

## 0.7.1 (2025-12-14)

### Added
- PLCopen graphical language support: FBD, LD, SFC reference extraction
- PLCopen stats display with `--stats` option
  - Shows POU counts by type (Functions, Function Blocks, Programs)
  - Displays language usage across all 5 IEC 61131-3 languages
  - Variable counts and empty POU detection

### Fixed
- Comment filtering in ST/IL code extraction (prevents false positives)
- IL opcode recognition (prevents opcodes being flagged as undefined variables)
- Improved identifier validation (rejects tokens with punctuation)
- `--stats` now correctly displays PLCopen project statistics

### Changed
- Enhanced variable reference extraction across all IEC 61131-3 languages
- Better unused variable detection for graphical languages

## 0.7.0 (2025-12-11)

### Changed
- **BREAKING**: Migrated from `iecst` to `iec61131` v0.7.0 for ST parsing and analysis
  - Now using standards-compliant IEC 61131-3:2013 parser for Structured Text (ST)
  - All analysis features (CFG, complexity, nesting) ported to new parser
  - Better parse error reporting with source locations
  - Improved ST language support with spec-compliant parsing

### Internal
- Created `iec61131_adapter` module for seamless migration
- Updated all analysis and rule modules to use new AST structure
- Maintained backward compatibility for all rules and detectors

## 0.6.0 (2025-12-06)

### Added
- S0004: Unused AOI detection
- S0005: Unused DataType detection

## 0.5.0 (2025-12-06)

### Added
- M0001: Cyclomatic Complexity detection for ST routines (max: 10)
- M0003: Deep Nesting detection for ST routines (max: 5 levels)
- Enhanced `--stats` output with ST complexity metrics
  - Max/Average cyclomatic complexity
  - Max/Average nesting depth
- Integration with iecst 0.3.0 for CFG and nesting analysis

### Changed
- Stats now display complexity metrics when ST routines are present
- Updated iecst dependency to 0.3.0

## 0.4.0 (2025-12-05)

### Added
- S0004: Unused AOI detection
- S0005: Unused DataType detection

## 0.3.0 (2025-12-05)

### Added
- PLCopen XML format support
- Rule codes (P00001, P00002, P00003) for stable issue identification
- Library API for programmatic use
- Re-exports of `l5x` and `plcopen` crates for extensions

### Changed
- Removed `plcmodel` dependency - analysis is now format-specific
- Updated config format with `ignore_patterns` and `ignore_scopes`
- Improved output format with rule codes

## 0.1.0 (2025-12-04)

- Initial release
- Unused tags detection
- Undefined tags detection
- Empty routines detection
- `--stats` option for file statistics
