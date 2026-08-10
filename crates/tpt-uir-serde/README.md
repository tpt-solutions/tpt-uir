# tpt-uir-serde

Zero-copy (de)serialization for the **TPT Unified Intermediate Representation
(TPT-UIR)** using [postcard](https://docs.rs/postcard).

This crate builds on [`tpt-uir-core`](../tpt-uir-core) (with its `serde`
feature enabled) and provides:

- `serialize_op` / `deserialize_op` for single operations
- `serialize_region` / `deserialize_region` for whole regions
- `write_tptuir` / `read_tptuir` for `.tptuir` files (requires the `std` feature)

It is `no_std` by default; enable the `std` feature for file I/O helpers.

## Example

```rust
use tpt_uir_serde::{serialize_region, deserialize_region};
use tpt_uir_core::{Region, Block, Operation, OpName, Type};

let region = Region {
    blocks: vec![Block {
        arguments: vec![(0u32, Type::Index)],
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

let bytes = serialize_region(&region).unwrap();
let back = deserialize_region(&bytes).unwrap();
assert_eq!(region, back);
```

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at
your option.
