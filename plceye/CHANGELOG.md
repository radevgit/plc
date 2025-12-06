# Changelog

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

## 0.4.0 (2024-12-05)

### Added
- S0004: Unused AOI detection
- S0005: Unused DataType detection

## 0.3.0 (2024-12-05)

### Added
- PLCopen XML format support
- Rule codes (P00001, P00002, P00003) for stable issue identification
- Library API for programmatic use
- Re-exports of `l5x` and `plcopen` crates for extensions

### Changed
- Removed `plcmodel` dependency - analysis is now format-specific
- Updated config format with `ignore_patterns` and `ignore_scopes`
- Improved output format with rule codes

## 0.1.0 (2024-12-04)

- Initial release
- Unused tags detection
- Undefined tags detection
- Empty routines detection
- `--stats` option for file statistics
