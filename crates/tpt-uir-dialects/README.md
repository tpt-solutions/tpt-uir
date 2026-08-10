# tpt-uir-dialects

Dialect definitions and analysis passes for the **TPT Unified Intermediate
Representation (TPT-UIR)**.

This crate builds on [`tpt-uir-core`](../tpt-uir-core) and provides:

- `tpt_gpu` dialect (`GpuDialect`, `GpuOp`) for GPU operations
- `crucible` dialect (`CrucibleDialect`, `CrucibleOp`)
- `memory` dialect (`MemOp`) for memory operations
- A `ValidateDialect` trait for dialect-level validation
- Analysis passes (e.g. liveness)

## Example

```rust
use tpt_uir_dialects::{GpuDialect, GpuOp};

let op = GpuDialect::op(GpuOp::Launch);
println!("{}", op);
```

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at
your option.
