# Changelog

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
