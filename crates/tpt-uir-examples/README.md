# tpt-uir-examples

Runnable examples and end-to-end tests for **TPT-UIR**. This crate is not
published (`publish = false`); it exists to exercise the public API of every
other crate in the workspace together, the way a real consumer would.

## Examples

Run with `cargo run --example <name> -p tpt-uir-examples`:

| Example | Demonstrates |
| --- | --- |
| [`quickstart`](examples/quickstart.rs) | Building a region with the fluent builders, attaching quantization and dialect-version metadata, GPU dialect validation, and round-tripping through postcard, FlatBuffers, and text. |
| [`crucible_static_shapes`](examples/crucible_static_shapes.rs) | The Crucible dialect's static-shape requirement (`Dimension::Fixed`), contrasted with the GPU dialect's `Bounded`/`None` shapes, plus hardware routing attributes. |
| [`gpu_threaded_add`](examples/gpu_threaded_add.rs) | The GPU dialect in action: `Bounded` shapes, a nested kernel region with `tpt_gpu.thread_id`/`tpt_gpu.barrier`, grid attributes, and text round-trip. |
| [`cookbook_mlp`](examples/cookbook_mlp.rs) | Cookbook: a larger, realistic graph — fused block-quantized GEMM with a nested tile loop, memory-arena scope ops, GPU launch, and round-trips through all three wire formats. |

```sh
cargo run --example quickstart -p tpt-uir-examples
```

## Tests

[`tests/end_to_end.rs`](tests/end_to_end.rs) covers cross-crate round-trips
(build → validate → serialize → deserialize → validate again) across all three
wire formats.

```sh
cargo test -p tpt-uir-examples
```

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at
your option.
