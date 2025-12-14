# Changelog

## 0.3.1 (2025-12-14)

### Added
- Full support for graphical language elements (FBD, LD, SFC)
- Text field in `FormattedText` to capture ST/IL code content
- Multiple group reference support in XSD codegen

### Fixed
- Body structures now fully populated with language-specific elements:
  - `Body_FBD_Inline`: 13 fields (blocks, variables, labels, jumps)
  - `Body_LD_Inline`: 15 fields (contacts, coils, power rails, blocks)
  - `Body_SFC_Inline`: 21 fields (steps, transitions, actions)
- Namespace prefix handling in group references
- Recursive type detection for group-expanded elements

### Changed
- XSD parser now collects multiple `<group ref>` elements
- Codegen applies `Box<>` wrapper for recursive types in groups
- Enhanced `xs:any` handling for mixed content extraction

## 0.3.0 (2025-12-11)
- moved to iec61131 for ST parsing

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
- Type-safe parsing using quick-xml and serde
- Generated types from official PLCopen TC6 XML schema (v2.01)
