# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this crate adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-08-10

### Added
- Initial release of the TPT-UIR examples and end-to-end tests crate.
- `examples/quickstart.rs`: builds a region with the fluent builders, validates
  it, attaches `Quantization` / `dialect_version` attributes, and round-trips it
  through postcard, FlatBuffers, and textual formats.
- `examples/crucible_static_shapes.rs`: Crucible `Fixed`-shape contrast case.
- `tests/end_to_end.rs`: full pipeline assertions (build -> validate ->
  serialize -> deserialize -> print -> parse -> equality).
