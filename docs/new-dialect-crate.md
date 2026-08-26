# Recipe: Adding a New Dialect Crate

This repo has no `cargo-generate` template yet; until then, follow this recipe
to add a new dialect crate (e.g. an NPU or DSP dialect) in ~30 minutes.

## 1. Scaffold

Create `crates/tpt-uir-<name>/` with this layout:

```
crates/tpt-uir-<name>/
├── Cargo.toml
├── README.md
├── CHANGELOG.md
├── LICENSE-MIT          # copy from any existing crate
├── LICENSE-APACHE       # copy from any existing crate
└── src/
    ├── lib.rs           # crate docs + re-exports
    └── <name>.rs        # dialect ops + validation
```

The virtual workspace picks the crate up automatically (`members = ["crates/*"]`)
— nothing to register in the root `Cargo.toml`.

## 2. `Cargo.toml`

Model it on `crates/tpt-uir-dialects/Cargo.toml`:

```toml
[package]
name = "tpt-uir-<name>"
version = "0.1.0"
edition = "2021"
description = "<Name> dialect for TPT-UIR"
license = "MIT OR Apache-2.0"
authors = ["TPT Solutions"]
repository = "https://github.com/tpt-solutions/tpt-uir"
homepage = "https://github.com/tpt-solutions/tpt-uir"
documentation = "https://docs.rs/tpt-uir-<name>"
keywords = ["uir", "ir", "<name>", "no-std"]
categories = ["compilers", "no-std"]
readme = "README.md"

[dependencies]
tpt-uir-core = { path = "../tpt-uir-core", version = "0.1.0" }
# Only if you need GPU/Crucible validators as references:
tpt-uir-dialects = { path = "../tpt-uir-dialects", version = "0.1.0" }
```

Rules:

- Keep the crate **`#![no_std]`** with `extern crate alloc;` in `lib.rs`.
  CI proves workspace-wide `no_std` integrity.
- Zero external dependencies unless unavoidable.

## 3. Ops and constants (`src/<name>.rs`)

Follow `crates/tpt-uir-dialects/src/gpu.rs`:

1. Add op-name constants to `tpt-uir-core/src/op_name.rs`
   (`pub const TPT_<NAME>_<OP>: &str = "tpt_<name>.<op>";`) — **appending only**;
   the postcard wire format is a contract, never renumber variants.
2. Re-export them from your crate.
3. Provide a marker type `pub struct <Name>Dialect;`.
4. Provide an op builder like `GpuOp::build(...)` that enforces your dialect's
   structural invariants at construction time.
5. Implement `ValidateDialect` for the marker type:

```rust
use tpt_uir_dialects::ValidateDialect;

impl ValidateDialect for MyDialect {
    fn dialect_prefix() -> &'static str {
        "tpt_mydialect"
    }
    fn current_version() -> i64 {
        1
    }
    fn validate(region: &Region) -> Result<(), Vec<String>> {
        // collect ALL violations, not just the first
        validate_my_region(region)
    }
}
```

6. Version attributes reuse `Attribute::dialect_version(v)` /
   `Attribute::op_dialect_version(v)` (stored as `I64`) — **additive versioning,
   no new wire surface**.

## 4. Docs, tests, gates

- `README.md`: purpose, shape/dimension rules, op table, versioning story
  (see `crates/tpt-uir-dialects/README.md` §"Versioning").
- Add `#![doc = include_str!("../README.md")]` so doc-tests exercise the README.
- Unit tests mirroring `gpu.rs`: accept/reject cases for every shape rule and
  builder invariant.
- Run the full gate suite:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test --doc --workspace
cargo build --workspace --no-default-features
```

## 5. Wire it into adoption surfaces

- Add the dialect to `crates/tpt-uir-cli/src/main.rs` `cmd_validate`'s
  `--dialect` match arm.
- Add an example under `crates/out-uir-examples/examples/` (mirror
  `gpu_threaded_add.rs`).
- Update the root `README.md` crate table and this file if anything above
  changes.
