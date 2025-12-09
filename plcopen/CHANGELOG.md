# Changelog

## 0.2.0 (2025-12-09)

- Added security module with DoS attack protection
- Added `from_str_secure()` for parsing untrusted XML
- Security limits for XML bombs, entity expansion, deep nesting
- Three security profiles: strict, balanced, relaxed
- Added `SecurityLimits` and `SecurityError` types
- ST (Structured Text) extraction and parsing via `iecst` 0.5

## 0.1.0 (2025-12-04)

- Initial release
- PLCopen TC6 XML parser
- Support for all IEC 61131-3 languages (ST, IL, LD, FBD, SFC)
- Type-safe parsing using quick-xml and serde
- Generated types from official PLCopen TC6 XML schema (v2.01)
