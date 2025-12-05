# Changelog

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
