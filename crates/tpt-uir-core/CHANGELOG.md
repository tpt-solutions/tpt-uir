# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this crate adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-08-10

### Added
- Initial release of the TPT-UIR core data structures.
- SSA region/block/operation graph representation.
- Scalar and composite type system.
- Operation name parsing and validation (`dialect.op` format).
- Attributes and metadata.
- `no_std` support via an `alloc` dependency.
- Optional `serde` feature for (de)serialization of IR types.
