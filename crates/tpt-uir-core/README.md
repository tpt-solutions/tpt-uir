# tpt-uir-core

Core data structures for the **TPT Unified Intermediate Representation (TPT-UIR)**.

This crate provides the foundational, `no_std`-compatible building blocks of the IR:

- SSA region/block/operation graphs
- Scalar and composite type system
- Operation names (`dialect.op`) with parsing and validation
- Attributes and metadata

It has no dependencies on a specific dialect or serialization format, making it
suitable for bare-metal and embedded targets as well as hosted environments.

## Features

- `serde` (optional): derive `Serialize`/`Deserialize` for the IR types via
  `serde`.

## Example

```rust
use tpt_uir_core::{OpName, Region, Block, Operation, validate_region};

let region = Region {
    blocks: vec![Block {
        arguments: vec![(0u32, tpt_uir_core::Type::Index)],
        operations: vec![Operation {
            id: 1,
            op_name: OpName::parse("core.add").unwrap(),
            operands: vec![0],
            results: vec![1],
            regions: vec![],
            attributes: vec![],
        }],
    }],
};

assert!(validate_region(&region).is_ok());
```

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at
your option.
