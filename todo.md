# TPT-UIR Implementation Checklist

**Repo:** `tpt-uir` | **Status:** Approved for Implementation
**Target consumers:** `tpt-gpu`, `tpt-crucible`, `tpt-telos`
**Last updated:** 2026-08-26 — Phases 1–11 complete; longer-horizon backlog below.

---

## Phase 1 — Core Crate (Weeks 1–2) ✅ DONE (in-repo)

> Implemented in `crates/tpt-uir-core`. All items below complete and verified by `cargo test -p tpt-uir-core`.

### Workspace Scaffold
- [x] Create root `Cargo.toml` as virtual workspace (`[workspace]`, `members = ["crates/*"]`, `resolver = "2"`)
- [x] Create `crates/tpt-uir-core/` directory structure (`src/`)

### tpt-uir-core `Cargo.toml`
- [x] Set `#![no_std]` in crate — zero external dependencies
- [x] Add optional `serde` feature (`serde = { version = "1.0", optional = true, default-features = false, features = ["derive", "alloc"] }`)

### `src/lib.rs`
- [x] `#![no_std]` + `extern crate alloc` header
- [x] Declare modules: `pub mod types`, `pub mod attr`, `pub mod op_name`, `pub mod ir`
- [x] Re-export all public types at crate root

### `src/types.rs` — Type System
- [x] `ScalarType` enum: I8, I16, I32, I64, U8, U16, U32, U64, F16, F32, F64, BF16, Bool, Q4_0, Q4_1, Q8_0
- [x] `ScalarType::size_bytes() -> usize` impl
- [x] `#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]` on `ScalarType`
- [x] `ShapeSpec` struct (`dimensions: Vec<Dimension>`)
- [x] `Dimension` enum: `Fixed(usize)`, `Symbolic(String)`, `Bounded { symbol: String, max_value: usize }`
- [x] `TensorType` struct (`dtype: ScalarType`, `shape: Option<ShapeSpec>`)
- [x] `Type` enum: `Scalar(ScalarType)`, `Tensor(TensorType)`, `Index`
- [x] `serde` derives on all types above

### `src/attr.rs` — Attributes
- [x] `AttributeValue` enum: `I64(i64)`, `F64(f64)`, `String(String)`, `Type(Type)`, `Shape(ShapeSpec)`, `OpName(OpName)`
- [x] `Attribute` struct: `key: String`, `value: AttributeValue`
- [x] Convenience constructors: `Attribute::i64()`, `Attribute::f64()`, `Attribute::string()`, `Attribute::shape()`
- [x] `serde` derives on `Attribute` and `AttributeValue`

### `src/op_name.rs` — Operation Names
- [x] `OpName` struct: `dialect: String`, `op: String`
- [x] `OpName::new(dialect, op)` constructor
- [x] `OpName::parse(s: &str) -> Result<Self, &'static str>` (splits on `.`)
- [x] `OpName::to_string(&self) -> String` (formats as `"dialect.op"`)
- [x] Predefined `&'static str` constants:
  - Core: `core.add`, `core.mul`, `core.load`, `core.store`
  - GPU: `tpt_gpu.launch`, `tpt_gpu.thread_id`, `tpt_gpu.barrier`, `tpt_gpu.alloc`, `tpt_gpu.dealloc`
  - Crucible: `tpt_crucible.map_flash`, `tpt_crucible.route_fpga`, `tpt_crucible.analog_conv`
  - Memory: `tpt_memory.scope_begin`, `tpt_memory.alloc`, `tpt_memory.scope_end`
- [x] `serde` derives on `OpName`

### `src/ir.rs` — Core IR Structures
- [x] `pub type ValueId = u32`
- [x] `pub type OpId = u32`
- [x] `Operation` struct: `id: OpId`, `op_name: OpName`, `operands: Vec<ValueId>`, `results: Vec<ValueId>`, `regions: Vec<Region>`, `attributes: Vec<Attribute>`
- [x] `Block` struct: `arguments: Vec<(ValueId, Type)>`, `operations: Vec<Operation>`
- [x] `Region` struct: `blocks: Vec<Block>`
- [x] `serde` derives on `Operation`, `Block`, `Region`

### SSA Validation
- [x] `validate_region(region: &Region) -> Result<(), Vec<String>>` — walks all blocks, checks every `ValueId` in `operands` is defined by a result of a preceding op or block argument
- [x] Returns all violations (not just the first)

### Tests (Phase 1)
- [x] `ScalarType::size_bytes()` — assert correct byte sizes for all variants
- [x] `OpName::parse()` — valid input roundtrip, invalid input returns `Err`
- [x] SSA validation — valid region passes, undefined-ValueId region returns error
- [x] `cargo test -p tpt-uir-core` green
- [x] `cargo clippy -p tpt-uir-core -- -D warnings` clean
- [x] `cargo build -p tpt-uir-core --no-default-features` succeeds (confirms no_std)

---

## Phase 2 — Dialect Definitions (Weeks 3–4) ✅ DONE (in-repo)

> Implemented in `crates/tpt-uir-dialects`. All items below complete and verified by `cargo test -p tpt-uir-dialects`.

### tpt-uir-dialects Scaffold
- [x] Create `crates/tpt-uir-dialects/` with `src/` and `src/passes/`
- [x] `Cargo.toml`: depends on `tpt-uir-core` only; no other deps

### `src/gpu.rs` — GPU Dialect
- [x] Op name constants re-exported for GPU ops
- [x] `GpuOp` builder: helper to construct `Operation` with correct `op_name` and validate shape is `None` or `Bounded`
- [x] Doc comment: GPU dialect requires `Dimension::Bounded` or `None` — never `Fixed`

### `src/crucible.rs` — Crucible Dialect
- [x] Op name constants re-exported for Crucible ops
- [x] `CrucibleOp` builder: helper to construct `Operation` with correct `op_name`
- [x] `validate_crucible_region(region: &Region) -> Result<(), Vec<String>>` — errors if any tensor shape contains `Symbolic` or `Bounded` dims
- [x] Hardware routing attribute helper: `crucible::target_ip_block(block_name: &str) -> Attribute`

### `src/memory.rs` — Memory Dialect
- [x] Op name constants for `tpt_memory.*`
- [x] `MemOp` builders: `scope_begin(lifetime: &str)`, `mem_alloc(size_bytes: ValueId, scope: &str)`, `scope_end(lifetime: &str)`

### `src/passes/liveness.rs` — Liveness Pass
- [x] `inject_memory_ops(region: &mut Region, scope_name: &str)` — walks ops in each block, wraps alloc-bearing operations with `mem.scope_begin` / `mem.alloc` / `mem.scope_end`

### Dialect Validation Trait
- [x] `pub trait ValidateDialect` with `fn validate(region: &Region) -> Result<(), Vec<String>>`
- [x] Implement for `GpuDialect` and `CrucibleDialect` marker types

### Tests (Phase 2)
- [x] Crucible validator rejects `Symbolic` and `Bounded` dims, accepts `Fixed` only
- [x] Liveness pass correctly injects memory ops around alloc operations
- [x] `cargo test -p tpt-uir-dialects` green
- [x] `cargo test --workspace` green

---

## Phase 2b — Serialization (Weeks 3–4) ✅ DONE (in-repo)

> Implemented in `crates/tpt-uir-serde`. All items below complete and verified by `cargo test -p tpt-uir-serde` (file I/O verified with `--features std`).

### tpt-uir-serde Scaffold
- [x] Create `crates/tpt-uir-serde/` with `src/`
- [x] `Cargo.toml`:
  ```toml
  [dependencies]
  postcard = { version = "1.0", default-features = false, features = ["alloc"] }
  serde = { version = "1.0", default-features = false, features = ["derive"] }
  tpt-uir-core = { path = "../tpt-uir-core", features = ["serde"] }
  ```
- [x] Optional `std` feature for file I/O helpers

### `src/lib.rs`
- [x] `serialize_op(op: &Operation) -> Result<Vec<u8>, postcard::Error>`
- [x] `deserialize_op(bytes: &[u8]) -> Result<Operation, postcard::Error>`
- [x] `serialize_region(region: &Region) -> Result<Vec<u8>, postcard::Error>`
- [x] `deserialize_region(bytes: &[u8]) -> Result<Region, postcard::Error>`
- [x] `#[cfg(feature = "std")] write_tptuir(path, region)` — write `.tptuir` file
- [x] `#[cfg(feature = "std")] read_tptuir(path) -> Result<Region, ...>` — read `.tptuir` file

### Tests (Phase 2b)
- [x] Round-trip `Operation`: serialize → deserialize → assert eq
- [x] Round-trip `Region` with nested blocks and multiple ops
- [x] `cargo test -p tpt-uir-serde` green

---

## Phase 3 — Ingestion Adapters (Weeks 5–6) ⛔ OUT OF REPO

> These tasks live in the `tpt-gpu` and `tpt-crucible` repos — NOT in this `tpt-uir` repo. They are listed here for completeness but cannot be implemented here. The `tpt-uir` side (dialect ops, validation, serialization) that they depend on is complete.

> **Status (2026-08-10):** the `tpt-gpu` adapter is implemented in `tpt-gpu/crates/tpt-gpu-uir-adapter` (see `tpt-gpu/todo.md` Phase 16 for full tracking). The `tpt-crucible` adapter is now implemented in `tpt-crucible/crates/tpt-crucible-uir-adapter`.

### tpt-gpu Adapter
- [x] Add `tpt-uir-dialects` as a dependency in `tpt-gpu` (via `crates/tpt-gpu-uir-adapter`, path deps to the sibling `tpt-uir` workspace)
- [x] Implement TPTIR → TPT-UIR converter: map SSA blocks → `Block`, functions → `Region` (`from_tptir` in `tpt-gpu-uir-adapter`)
- [x] Use GPU dialect ops (`tpt_gpu.*` / `core.*`) for all emitted operations
- [x] Emit shapes as `Dimension::Bounded` or `None` (GPU invariant enforced via `GpuOp::build` + `GpuDialect::validate`)
- [x] Reverse converter `to_tptir` + GGUF→TPTIR path (`gguf` module: `parse_gguf_bytes` + `gguf_to_tptir`) for a real Llama-3 block
- [x] Wired into `tpt-gpu-kernelgen` (`generate --output-tptuir`) and `tpt-gpu-runtime` (`Device::load_module_tptuir`) — see `tpt-gpu/todo.md` Phase 16

### tpt-crucible Adapter
- [x] Add `tpt-uir-dialects` as a dependency in `tpt-crucible`
- [x] Implement ComputationalGraph → TPT-UIR converter: wrap graph as single `Region` containing one `Block`
- [x] Use Crucible dialect ops (`tpt_crucible.*`) for all emitted operations
- [x] Emit shapes as `Dimension::Fixed` only

### Round-Trip Validation
- [x] Write `test_roundtrip_gpu()`: load Llama-3 block (GGUF fixture + hand-authored multi-block/rich kernels) → GPU adapter → TPT-UIR → back → assert lossless (8 adapter tests pass; kernelgen/runtime integration tests pass)
- [x] Write `test_roundtrip_crucible()`: load Llama-3 block → Crucible adapter → TPT-UIR → back → assert lossless (implemented in `tpt-crucible/crates/tpt-crucible-uir-adapter`; 9 adapter tests pass incl. lossless graph round-trip, postcard serialization, and `.tptuir` file round-trip)
- [x] Both round-trip tests pass — GPU and Crucible adapters implemented
- [x] Fix any lossless failures (GPU path is lossless; no fixes required)

---

## Phase 4 — Prover Bridge (Weeks 7–8) ✅ DONE (in tpt-telos)

> Implemented in `tpt-telos/crates/tpt-telos-uir-bridge` (see that repo's `crates/tpt-telos-uir-bridge/README.md`
> and the "TPT-UIR Prover Bridge" section of the `tpt-telos` README). Verified by
> `cargo test -p tpt-telos-uir-bridge --features uir` (8 unit + 3 CLI + 1 doctest, all green).
> The `tpt-uir` side it consumes (memory dialect ops, `Dimension::Symbolic` extraction,
> postcard serialization) is complete.

### tpt-telos Setup
- [x] Add `tpt-uir-dialects` and `tpt-uir-serde` as dependencies in `tpt-telos` (path deps in `crates/tpt-telos-uir-bridge`)

### TPT-UIR Consumer
- [x] Walk a `Region`, extract all `tpt_memory.alloc` ops and their `size_bytes` `ValueId` attributes (`extract_allocs`)
- [x] Extract `Dimension::Symbolic` variables from tensor types across the region (`extract_symbolic_dims`)

### Z3 Integration
- [x] Map extracted `Dimension::Symbolic` names → Z3 symbolic integer variables (FM `Linear`/`Int` vars; `z3` feature builds Z3 `Int` ASTs)
- [x] Map `mem.alloc` `size_bytes` → Z3 integer expressions (`tensor_byte_expr` → `SizeExpr`)

### Memory Bound Proof
- [x] Implement proof query: within any `mem.scope`, sum of `mem.alloc` sizes ≤ target hardware physical memory limit (`prove_memory_bounds`)
- [x] Integrate with existing `tpt-telos` prover API (Fourier-Motzkin `model`/`unsat`, plus `z3` feature for nonlinear)
- [x] Return `ProofResult::Valid` or `ProofResult::Counterexample(model)`

### End-to-End Verification
- [x] Run first formal memory proof on a simple `tpt-crucible` model (all `Fixed` dims, static shapes)
- [x] Confirm: valid model → `ProofResult::Valid`
- [x] Confirm: over-budget model → `ProofResult::Counterexample` with interpretable witness
- [x] Document the full proof workflow in `tpt-telos` README

---

## CI / Quality Gates ✅ DONE

- [x] `cargo test --workspace` — all tests pass
- [x] `cargo clippy --workspace -- -D warnings` — zero warnings
- [x] `cargo build --workspace --no-default-features` — confirms `no_std` integrity
- [x] `cargo fmt --check` — formatted
- [x] Add GitHub Actions workflow (or equivalent CI) running the above on every push

---

## Phase 5 — Spec-Completion: Quick-Win Doc/Adoption Fixes ✅ DONE

> Full design: `C:\Users\phill\.claude\plans\review-platform-for-bugs-atomic-elephant.md`.
> Follow-up from a platform review: closes the gap between `spec.txt` and what's
> actually implemented, without breaking the `OpName` constants or postcard wire format
> that `tpt-gpu`/`tpt-crucible`/`tpt-telos` already consume.

- [x] Fix broken example in `crates/tpt-uir-dialects/README.md` (calls nonexistent `GpuDialect::op(GpuOp::Launch)`)
- [x] Reword `crates/tpt-uir-serde/README.md` + its `Cargo.toml` `description`: "zero-copy" → "compact binary (de)serialization" (postcard isn't zero-copy; point at `tpt-uir-flatbuffers` for that)
- [x] Copy `LICENSE-MIT`/`LICENSE-APACHE` from workspace root into each `crates/*/` directory (existing README links are dead)
- [x] Add `#![doc = include_str!("../README.md")]` to each library crate's `src/lib.rs` so `cargo test --doc` exercises README samples
- [x] Update each touched crate's `CHANGELOG.md` (amend existing `0.1.0` entry — not yet published)

## Phase 6 — Core IR Additive Changes (`tpt-uir-core`, `tpt-uir-dialects`) ✅ DONE

- [x] `RegionBuilder`/`BlockBuilder`/`OpBuilder` fluent construction API — `crates/tpt-uir-core/src/builder.rs`
- [x] `QuantizationParams` + `tensor_byte_size()` helper (checked arithmetic, GGUF-correct block sizes for Q4_0/Q4_1/Q8_0) — `crates/tpt-uir-core/src/quant.rs`
- [x] Append `AttributeValue::Quantization(QuantizationParams)` as the **last** enum variant (postcard wire-compat) + `Attribute::quantization()` ctor — `crates/tpt-uir-core/src/attr.rs`
- [x] `Attribute::dialect_version()` / `op_dialect_version()` (reuses `AttributeValue::I64`, no new wire surface) — `crates/tpt-uir-core/src/attr.rs`
- [x] Extend `ValidateDialect` trait with `dialect_prefix()`, `current_version()`, `validate_versions()`, `validate_all()` default methods; implement `dialect_prefix()` for `GpuDialect`/`CrucibleDialect` — `crates/tpt-uir-dialects/src/validate.rs`
- [x] Document dialect-versioning migration story in `crates/tpt-uir-dialects/README.md`
- [x] Postcard wire-compatibility regression test: pre-change byte fixture still decodes after the `Quantization` variant is appended

## Phase 7 — FlatBuffers Zero-Copy Format + FFI (spec.txt §6) ✅ DONE

> Depends on Phase 6 (`AttributeValue::Quantization` must exist for the schema).

- [x] Author `schema/tpt_uir.fbs` mirroring `Operation`/`Block`/`Region`/`Type`/`Attribute`/`OpName`/`Dimension`/`QuantizationParams` (wrapper-table + union pattern) — `crates/tpt-uir-flatbuffers/`
- [x] Check in `flatc`-generated Rust code under `src/generated/`; pin exact `flatc` version in `schema/README.md` (no `flatc` in CI — codegen is manual, not build.rs)
- [x] `to_flatbuffer()`/`from_flatbuffer()` conversion + `root_as_region()` zero-copy accessor — `src/convert.rs`
- [x] Feature-gated `mmap` module: `open_mmap()` → plain `memmap2::Mmap`, read via `root_as_region(&mmap[..])` — `src/mmap.rs`
- [x] `tpt-uir-flatbuffers` stays `no_std` by default (`flatbuffers` crate with `default-features = false`)
- [x] New `tpt-uir-ffi` crate (`crate-type = ["cdylib", "rlib", "staticlib"]`): opaque-handle C API — `tpt_uir_load`, `tpt_uir_free`, `tpt_uir_block_count`, `tpt_uir_op_count`, `tpt_uir_op_dialect`/`tpt_uir_op_op`, `tpt_uir_op_operand(_count)`, `tpt_uir_validate_ssa`, `tpt_uir_validate_dialect` (read-only for this pass)
- [x] `build.rs` uses the `cbindgen` library crate (not the CLI binary) to emit `include/tpt_uir_ffi.h`

## Phase 8 — Textual IR Format (`crates/tpt-uir-text`) ✅ DONE

> Depends on Phase 6 (`quant<...>` grammar production needs `QuantizationParams`).

- [x] Hand-written lexer + recursive-descent parser (no `nom`/`pest`) — `src/lex.rs`, `src/parse.rs`
- [x] Grammar: `^bb`-blocks, `#id: %res = "op"(%operands) {attrs} [regions]`, `tensor<...>`, `shape<...>`, `quant<block_size,bytes_per_block[,scales]>`
- [x] `to_text(region) -> String`, `parse_text(src) -> Result<Region, ParseError>`, `Pretty<'a>(&'a Region): Display` — `src/print.rs`, `src/lib.rs`
- [x] Round-trip tests: nested regions, `Quantization` attribute, `dialect_version` attribute, empty block/region edge cases

## Phase 9 — CLI Tool (`crates/tpt-uir-cli`) ✅ DONE

> Depends on Phases 6–8.

- [x] `validate <file> [--dialect gpu|crucible]` subcommand (SSA always; `validate_all` when `--dialect` given)
- [x] `print <file>` subcommand (pretty-print via `tpt-uir-text`)
- [x] `convert --in <file> --out <file> [--from-format ...] [--to-format postcard|text|flatbuffers]` subcommand
- [x] CLI integration tests (`tests/cli.rs`) against fixtures

## Phase 10 — Examples Crate + Root README (adoption) ✅ DONE

> Depends on Phases 6–9; written last to demonstrate the final feature set.

- [x] New `crates/out-uir-examples/` (`publish = false`), depends on every other workspace crate except `tpt-uir-cli`
- [x] `examples/quickstart.rs` — `RegionBuilder` → `validate_all` → `Quantization`/`dialect_version` attrs → postcard + FlatBuffers serialization → text pretty-print
- [x] `examples/crucible_static_shapes.rs` — Crucible/`Fixed`-shape contrast case
- [x] `tests/end_to_end.rs` — full pipeline as assertions (build → validate → serialize → deserialize → print → parse → equality)
- [x] Root `README.md` — architecture summary, crate table, downstream consumers, CI badge, license section

## Spec-Completion Quality Gates (Phases 5–10) ✅ DONE

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace`
- [x] `cargo test --doc --workspace`
- [x] `cargo build --workspace --no-default-features` (confirms `no_std` integrity, incl. `tpt-uir-flatbuffers`)
- [x] `cargo run -p tpt-uir-cli -- validate/print/convert` against example fixtures
- [x] `cargo package -p <crate> --list` for every publishable crate (confirms `LICENSE-*` files included)

### Explicitly out of scope (backlog, not tracked as phases here)
- Dynamic control-flow / worst-case memory-scope-overlap handling for Crucible (`spec.txt` §8) — lives in sibling `tpt-telos` repo, not this one
- Prebuilt cross-platform `tpt-uir-ffi` binary release pipeline (GitHub Actions release matrix) — release-engineering scope, separate from the library/FFI code
- FFI/CLI write-and-construct-IR capability (read/validate/inspect only for now)
- `cargo semver-checks` in CI — add once there's a released baseline to check against

## Phase 11 — Review Follow-ups ✅ DONE

> From the 2026-08-26 platform review (bugs, gaps, adoption). See
> `C:\Users\phill\.claude\plans\review-platform-for-bugs-vivid-turing.md` for full detail.

### Bugs
- [x] `tpt-uir-flatbuffers/src/convert.rs`: replace `.expect()`/`panic!`/`unreachable!()` in the read path (`read_type`, `read_attr_value`, union/discriminant handling) with `Result`-returning error handling, matching `tpt-uir-serde`/`tpt-uir-text` (`from_flatbuffer` now returns `Result<Region, ConvertError>`; callers updated)
- [x] `tpt-uir-ffi/src/lib.rs`: replace `CString::new(...).expect(...)` in `tpt_uir_op_dialect` / `tpt_uir_op_op` with the crate's existing `c_int` error-code convention (avoid panicking across the C ABI boundary) (new code `3`: interior NUL)
- [x] Fix CI/README doc-test drift — added an explicit `cargo test --doc --workspace` step to `.github/workflows/ci.yml`
- [x] Fix malformed `tpt-uir-core/CHANGELOG.md` (two stacked `### Added` headers under `[0.1.0]`)

### Adoption
- [x] Add a short issues-only `CONTRIBUTING.md` (point at `AGENTS.md`'s quality-gate commands)
- [x] Add a bug-report issue template under `.github/ISSUE_TEMPLATE/`
- [x] Add a GPU-dialect-focused example in `crates/out-uir-examples/` (mirroring `crucible_static_shapes.rs`) — `examples/gpu_threaded_add.rs`
- [x] Add a minimal C example calling `tpt-uir-ffi` directly, exercising `include/tpt_uir_ffi.h` — `crates/tpt-uir-ffi/examples/basic.c`
- [x] Expand `crates/out-uir-examples/README.md` with a "cookbook" example showing a larger, more realistic graph — `examples/cookbook_mlp.rs`

### Automation
- [x] Add `cargo-deny` or `cargo-audit` CI step for license/advisory scanning (cargo-deny + root `deny.toml`, separate `supply-chain` CI job)
- [x] Add Dependabot/Renovate config for crates with external dependencies (`.github/dependabot.yml`: cargo weekly + github-actions weekly)

## Phase 12 — Longer-Horizon Progress

### Done (2026-08-26, batch 1)
- [x] CLI `diff` subcommand (`crates/tpt-uir-cli`) — structural comparison of two
  regions in any format mix via `--format-a`/`--format-b`; exit `0` when identical,
  exit `1` with `-`/`+` op lines otherwise. Covered by 2 integration tests.
- [x] CLI `fmt` subcommand (`crates/tpt-uir-cli`) — canonicalizes textual files;
  `--check` exits non-zero without writing when the file is not canonical.
  Covered by 2 integration tests.
- [x] CLI README + CHANGELOG updated with both subcommands.
- [x] Full quality gates re-run green after the change: `cargo fmt --check`,
  clippy (default + no-default-features, `-D warnings`),
  `cargo test --workspace`, `cargo build --workspace --no-default-features`.

### Done (2026-08-26, batch 2)
- [x] Text-format gap fixed: shape attributes now round-trip losslessly.
  Root cause was in the parser (`parse_attr_value` consumed the `shape`
  keyword, then `parse_shape` expected it again); it now parses only the
  `<...>` body. Regression test `roundtrip_shape_attribute` added to
  `tpt-uir-text`; `gpu_threaded_add.rs` example restored to a real
  `AttributeValue::Shape` grid attribute.
- [x] Release pipeline for `tpt-uir-cli` / `tpt-uir-ffi` binaries — hand-rolled
  `.github/workflows/release.yml`: tag-triggered (`v*`), cross-platform matrix
  (linux x86_64/aarch64, macOS x86_64/aarch64, Windows x86_64) building the CLI
  binary plus FFI cdylib/staticlib + C header, tar.gz/zip archives with SHA256
  checksums, attached to an auto-generated GitHub release.
- [x] Python bindings over the C ABI — pure-ctypes package under
  `bindings/python/` (`tpt_uir.Region`, `TptUirError`, auto-discovery of
  `target/release` libs or via `TPT_UIR_LIBRARY`), with README and a verified
  smoke test exercising every accessor incl. error paths.
- [x] Editor support — TextMate grammar `editors/textmate/tpt-uir-text.tmLanguage.json`
  (`source.tpt-uir-text`) highlighting blocks, op ids, value refs, op names,
  types, attributes; VS Code embedding recipe in `editors/textmate/README.md`.
  A full tree-sitter grammar remains future work.
- [x] WASM playground — new `crates/out-uir-wasm` crate (wasm-bindgen:
  `format_text`/`validate_text`) plus `playground/index.html` editor UI;
  build instructions in `playground/README.md`. Compiles natively under all
  workspace CI gates.
- [x] Documented recipe for new dialect/adapter crates — `docs/new-dialect-crate.md`
  (scaffold, no_std rules, op-name constants, ValidateDialect impl, additive
  versioning, adoption surfaces).

### Remaining (not yet scheduled)
- [ ] Publish crates to crates.io once API is considered stable enough
