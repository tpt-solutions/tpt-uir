# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this crate adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-08-10

### Added
- Initial release of the TPT-UIR dialect definitions and analysis passes.
- `tpt_gpu` dialect (`GpuDialect`, `GpuOp`) for GPU operations.
- `crucible` dialect (`CrucibleDialect`, `CrucibleOp`).
- `memory` dialect (`MemOp`) for memory operations.
- `ValidateDialect` trait for dialect-level validation.
- Analysis passes, including liveness.
