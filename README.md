# TPT-UIR

**TPT Unified Intermediate Representation (TPT-UIR)** — a compact, `no_std`-friendly
intermediate representation for machine-learning compilers and hardware
accelerators in the TPT stack.

[![CI](https://github.com/tpt-solutions/tpt-uir/actions/workflows/ci.yml/badge.svg)](https://github.com/tpt-solutions/tpt-uir/actions/workflows/ci.yml)

TPT-UIR models SSA region/block/operation graphs with a small, typed attribute
system, and ships first-class serialization in three formats:

- **postcard** — compact, self-describing binary (`tpt-uir-serde`)
- **FlatBuffers** — zero-copy, memory-mappable binary (`tpt-uir-flatbuffers`)
- **text** — a readable, round-trippable textual format (`tpt-uir-text`)

A CLI (`tpt-uir-cli`) and a C FFI (`tpt-uir-ffi`) are provided for inspection and
integration.

## Architecture

```
                 ┌─────────────────────────────────────────────┐
                 │              tpt-uir-core                    │
                 │  types · attr · op_name · ir · builder ·     │
                 │  quant · validate_region (SSA)               │
                 └───────────────┬─────────────────────────────┘
                                 │
        ┌───────────────┬────────┴───────────┬──────────────────┐
        │               │                    │                  │
┌───────▼──────┐ ┌──────▼───────┐   ┌────────▼────────┐  ┌──────▼──────┐
│ tpt-uir-     │ │ tpt-uir-     │   │ tpt-uir-       │  │ tpt-uir-    │
│ serde        │ │ flatbuffers  │   │ text           │  │ ffi         │
│ (postcard)   │ │ (zero-copy)  │   │ (textual)      │  │ (C ABI)     │
└───────┬──────┘ └──────┬───────┘   └────────┬────────┘  └──────┬──────┘
        └───────────────┴─────────┬──────────┴──────────────────┘
                                  │
                       ┌──────────▼──────────┐
                       │   tpt-uir-dialects   │
                       │ GPU · Crucible ·     │
                       │ Memory · ValidateDialect
                       └──────────┬──────────┘
                                  │
                       ┌──────────▼──────────┐
                       │   tpt-uir-cli        │
                       │   tpt-uir-examples   │
                       └──────────────────────┘
```

`dialects` sits on top of `core` and defines the GPU, Crucible, and Memory
dialects plus structural validation (`ValidateDialect`). The serialization,
text, and FFI crates depend only on `core` (and, where relevant, `dialects`).

## Crates

| Crate | Purpose |
| --- | --- |
| `tpt-uir-core` | Core IR data structures, builder API, quantization helpers, SSA validation (`no_std`). |
| `tpt-uir-dialects` | GPU / Crucible / Memory dialects and dialect-level validation. |
| `tpt-uir-serde` | Compact postcard (de)serialization (`no_std`; `std` for file I/O). |
| `tpt-uir-flatbuffers` | Zero-copy FlatBuffers layout + `mmap` access (`no_std`). |
| `tpt-uir-text` | Hand-written textual lexer/parser/printer. |
| `tpt-uir-ffi` | `cdylib`/`rlib`/`staticlib` C ABI for read-only inspection. |
| `tpt-uir-cli` | `validate` / `print` / `convert` command-line tool. |
| `tpt-uir-examples` | Usage examples and end-to-end tests (not published). |

## Dialects & versioning

Each dialect exposes a `current_version()` via the `ValidateDialect` trait.
Producers pin the targeted version with a `dialect_version` (or per-op
`op_dialect_version`) attribute — both stored as `i64`, so they add **no new
wire surface**. The migration story is additive: new capabilities bump the
version, older readers reject or degrade instead of misinterpreting the IR, and
the postcard wire format never renumbers existing enum variants.

## Downstream consumers

- **`tpt-gpu`** — ingests TPTIR (incl. GGUF-loaded Llama-3 blocks) into TPT-UIR
  via `tpt-gpu-uir-adapter`; wired into `tpt-gpu-kernelgen` and `tpt-gpu-runtime`.
- **`tpt-crucible`** — ingests `ComputationalGraph`s into TPT-UIR via
  `tpt-crucible-uir-adapter` (static `Fixed` shapes).
- **`tpt-telos`** — formal memory-bound proofs over TPT-UIR regions via
  `tpt-telos-uir-bridge` (Z3 / Fourier-Motzkin).

## Quick start

```rust
use tpt_uir_core::builder::{BlockBuilder, OpBuilder, RegionBuilder};
use tpt_uir_core::OpName;
use tpt_uir_dialects::{GpuDialect, ValidateDialect};

let region = RegionBuilder::new()
    .block(BlockBuilder::new().op(
        OpBuilder::new(1)
            .name(OpName::parse("tpt_gpu.launch").unwrap())
            .result(2)
            .build(),
    ).build())
    .build();

tpt_uir_core::validate_region(&region).unwrap();
GpuDialect::validate_all(&region).unwrap();
```

See `crates/tpt-uir-examples/examples/` for full postcard / FlatBuffers / text
round-trip examples, and `crates/tpt-uir-cli/README.md` for the CLI.

## Quality gates

`cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace`, `cargo test --doc --workspace`, and
`cargo build --workspace --no-default-features` are all run on every push by the
[CI workflow](.github/workflows/ci.yml).

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at
your option.
