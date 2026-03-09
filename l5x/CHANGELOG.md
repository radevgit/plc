# Changelog

## 0.6.0 (2026-03-09)

### Added
- Full serialization support: all parsed types now round-trip via `to_string()`
- `to_string()` function for serializing any parsed structure back to L5X XML
- Automatic stripping of Rockwell-encrypted AOI blobs (opaque base64 payloads) before parsing

### Fixed
- Trailing space in `@AutoDiagsEnabled` serde rename attribute
- All `Option` and `Vec` fields now use `skip_serializing_if` to suppress null/empty output
- Mixed-content types rewritten to use individual struct fields + `$text` instead of `Vec<Enum> + $value`, enabling correct serialization
- Output size is ~91% of input (vs 130–270% before) across 525 real L5X files

## 0.5.0 (2025-12-09)
- fixed security module

## 0.4.0 (2025-12-08)

- Added security module with DoS attack protection
- Added `from_str_secure()` for parsing untrusted XML
- Security limits for XML bombs, entity expansion, deep nesting
- Three security profiles: strict, balanced, relaxed
- Added `SecurityLimits` and `SecurityError` types
- Fixed compilation issues with quick-xml integration

## 0.1.0 (2025-12-04)

- Initial release
- L5X file parsing
- RLL (Relay Ladder Logic) parser
- Support for Controller, Program, and AOI export types
