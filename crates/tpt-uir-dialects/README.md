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
use tpt_uir_core::op_name::TPT_GPU_LAUNCH;
use tpt_uir_core::OpName;
use tpt_uir_dialects::GpuOp;

let op = GpuOp::build(
    1,
    OpName::parse(TPT_GPU_LAUNCH).unwrap(),
    vec![],
    vec![],
    vec![],
    vec![],
)
.unwrap();

println!("{}", op.op_name);
```

## Dialect versioning

Every dialect exposes a `current_version()` (starting at `1`) via the
[`ValidateDialect`] trait. Producers pin the dialect version they target by
attaching one of two attributes to the relevant operations:

- `dialect_version` (dialect-wide) — set once on a representative op, e.g. a
  function/module entry op.
- `op_dialect_version` (per-op override) — set when a single op uses a newer
  capability than the surrounding dialect version.

Both attributes reuse the existing `i64` wire representation, so they introduce
**no new serde/postcard surface**. Consumers should record the dialect version
when serializing and check it on load. The migration story is additive:

1. A dialect adds a new capability in version `N`.
2. Producers emit `dialect_version = N` (or `op_dialect_version = N` for the
   specific ops that need it).
3. Older consumers that only understand `N-1` read the attribute, see a version
   they do not recognise, and can reject or degrade gracefully instead of
   misinterpreting the IR.
4. The wire format never renumbers existing enum variants, so a serialized IR
   remains decodable by both old and new readers.

Validate with [`ValidateDialect::validate_all`], which runs the dialect's
structural checks *and* prefix/version checks together and reports every
violation:

```rust
use tpt_uir_core::op_name::TPT_GPU_LAUNCH;
use tpt_uir_core::{attr::Attribute, op_name::OpName, Operation};
use tpt_uir_dialects::GpuDialect;
use tpt_uir_dialects::ValidateDialect;

let op = Operation {
    id: 1,
    op_name: OpName::parse(TPT_GPU_LAUNCH).unwrap(),
    operands: vec![],
    results: vec![],
    regions: vec![],
    attributes: vec![Attribute::dialect_version(1)],
};

let region = tpt_uir_core::Region {
    blocks: vec![tpt_uir_core::Block {
        arguments: vec![],
        operations: vec![op],
    }],
};

assert!(GpuDialect::validate_all(&region).is_ok());
```

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at
your option.
