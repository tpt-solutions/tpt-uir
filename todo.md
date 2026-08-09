# TPT-UIR Implementation Checklist

**Repo:** `tpt-uir` | **Status:** Approved for Implementation  
**Target consumers:** `tpt-gpu`, `tpt-crucible`, `tpt-telos`

---

## Phase 1 — Core Crate (Weeks 1–2)

### Workspace Scaffold
- [ ] Create root `Cargo.toml` as virtual workspace (`[workspace]`, `members = ["crates/*"]`, `resolver = "2"`)
- [ ] Create `crates/tpt-uir-core/` directory structure (`src/`)

### tpt-uir-core `Cargo.toml`
- [ ] Set `#![no_std]` in crate — zero external dependencies
- [ ] Add optional `serde` feature (`serde = { version = "1.0", optional = true, default-features = false, features = ["derive"] }`)

### `src/lib.rs`
- [ ] `#![no_std]` + `extern crate alloc` header
- [ ] Declare modules: `pub mod types`, `pub mod attr`, `pub mod op_name`, `pub mod ir`
- [ ] Re-export all public types at crate root

### `src/types.rs` — Type System
- [ ] `ScalarType` enum: I8, I16, I32, I64, U8, U16, U32, U64, F16, F32, F64, BF16, Bool, Q4_0, Q4_1, Q8_0
- [ ] `ScalarType::size_bytes() -> usize` impl
- [ ] `#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]` on `ScalarType`
- [ ] `ShapeSpec` struct (`dimensions: Vec<Dimension>`)
- [ ] `Dimension` enum: `Fixed(usize)`, `Symbolic(String)`, `Bounded { symbol: String, max_value: usize }`
- [ ] `TensorType` struct (`dtype: ScalarType`, `shape: Option<ShapeSpec>`)
- [ ] `Type` enum: `Scalar(ScalarType)`, `Tensor(TensorType)`, `Index`
- [ ] `serde` derives on all types above

### `src/attr.rs` — Attributes
- [ ] `AttributeValue` enum: `I64(i64)`, `F64(f64)`, `String(String)`, `Type(Type)`, `Shape(ShapeSpec)`, `OpName(OpName)`
- [ ] `Attribute` struct: `key: String`, `value: AttributeValue`
- [ ] Convenience constructors: `Attribute::i64()`, `Attribute::f64()`, `Attribute::string()`, `Attribute::shape()`
- [ ] `serde` derives on `Attribute` and `AttributeValue`

### `src/op_name.rs` — Operation Names
- [ ] `OpName` struct: `dialect: String`, `op: String`
- [ ] `OpName::new(dialect, op)` constructor
- [ ] `OpName::parse(s: &str) -> Result<Self, &'static str>` (splits on `.`)
- [ ] `OpName::to_string(&self) -> String` (formats as `"dialect.op"`)
- [ ] Predefined `&'static str` constants:
  - Core: `core.add`, `core.mul`, `core.load`, `core.store`
  - GPU: `tpt_gpu.launch`, `tpt_gpu.thread_id`, `tpt_gpu.barrier`, `tpt_gpu.alloc`, `tpt_gpu.dealloc`
  - Crucible: `tpt_crucible.map_flash`, `tpt_crucible.route_fpga`, `tpt_crucible.analog_conv`
  - Memory: `tpt_memory.scope_begin`, `tpt_memory.alloc`, `tpt_memory.scope_end`
- [ ] `serde` derives on `OpName`

### `src/ir.rs` — Core IR Structures
- [ ] `pub type ValueId = u32`
- [ ] `pub type OpId = u32`
- [ ] `Operation` struct: `id: OpId`, `op_name: OpName`, `operands: Vec<ValueId>`, `results: Vec<ValueId>`, `regions: Vec<Region>`, `attributes: Vec<Attribute>`
- [ ] `Block` struct: `arguments: Vec<(ValueId, Type)>`, `operations: Vec<Operation>`
- [ ] `Region` struct: `blocks: Vec<Block>`
- [ ] `serde` derives on `Operation`, `Block`, `Region`

### SSA Validation
- [ ] `validate_region(region: &Region) -> Result<(), Vec<String>>` — walks all blocks, checks every `ValueId` in `operands` is defined by a result of a preceding op or block argument
- [ ] Returns all violations (not just the first)

### Tests (Phase 1)
- [ ] `ScalarType::size_bytes()` — assert correct byte sizes for all variants
- [ ] `OpName::parse()` — valid input roundtrip, invalid input returns `Err`
- [ ] SSA validation — valid region passes, undefined-ValueId region returns error
- [ ] `cargo test -p tpt-uir-core` green
- [ ] `cargo clippy -p tpt-uir-core -- -D warnings` clean
- [ ] `cargo build -p tpt-uir-core --no-default-features` succeeds (confirms no_std)

---

## Phase 2 — Dialect Definitions (Weeks 3–4)

### tpt-uir-dialects Scaffold
- [ ] Create `crates/tpt-uir-dialects/` with `src/` and `src/passes/`
- [ ] `Cargo.toml`: depends on `tpt-uir-core` only; no other deps

### `src/gpu.rs` — GPU Dialect
- [ ] Op name constants re-exported for GPU ops
- [ ] `GpuOp` builder: helper to construct `Operation` with correct `op_name` and validate shape is `None` or `Bounded`
- [ ] Doc comment: GPU dialect requires `Dimension::Bounded` or `None` — never `Fixed`

### `src/crucible.rs` — Crucible Dialect
- [ ] Op name constants re-exported for Crucible ops
- [ ] `CrucibleOp` builder: helper to construct `Operation` with correct `op_name`
- [ ] `validate_crucible_region(region: &Region) -> Result<(), Vec<String>>` — errors if any tensor shape contains `Symbolic` or `Bounded` dims
- [ ] Hardware routing attribute helper: `Attribute::target_ip_block(block_name: &str) -> Attribute`

### `src/memory.rs` — Memory Dialect
- [ ] Op name constants for `tpt_memory.*`
- [ ] `MemOp` builders: `scope_begin(lifetime: &str)`, `mem_alloc(size_bytes: ValueId, scope: &str)`, `scope_end(lifetime: &str)`

### `src/passes/liveness.rs` — Liveness Pass
- [ ] `inject_memory_ops(region: &mut Region, scope_name: &str)` — walks ops in each block, wraps alloc-bearing operations with `mem.scope_begin` / `mem.alloc` / `mem.scope_end`

### Dialect Validation Trait
- [ ] `pub trait ValidateDialect` with `fn validate(region: &Region) -> Result<(), Vec<String>>`
- [ ] Implement for `GpuDialect` and `CrucibleDialect` marker types

### Tests (Phase 2)
- [ ] Crucible validator rejects `Symbolic` and `Bounded` dims, accepts `Fixed` only
- [ ] Liveness pass correctly injects memory ops around alloc operations
- [ ] `cargo test -p tpt-uir-dialects` green
- [ ] `cargo test --workspace` green

---

## Phase 2b — Serialization (Weeks 3–4)

### tpt-uir-serde Scaffold
- [ ] Create `crates/tpt-uir-serde/` with `src/`
- [ ] `Cargo.toml`:
  ```toml
  [dependencies]
  postcard = { version = "1.0", default-features = false, features = ["alloc"] }
  serde = { version = "1.0", default-features = false, features = ["derive"] }
  tpt-uir-core = { path = "../tpt-uir-core", features = ["serde"] }
  ```
- [ ] Optional `std` feature for file I/O helpers

### `src/lib.rs`
- [ ] `serialize_op(op: &Operation) -> Result<Vec<u8>, postcard::Error>`
- [ ] `deserialize_op(bytes: &[u8]) -> Result<Operation, postcard::Error>`
- [ ] `serialize_region(region: &Region) -> Result<Vec<u8>, postcard::Error>`
- [ ] `deserialize_region(bytes: &[u8]) -> Result<Region, postcard::Error>`
- [ ] `#[cfg(feature = "std")] write_tptuir(path, region)` — write `.tptuir` file
- [ ] `#[cfg(feature = "std")] read_tptuir(path) -> Result<Region, ...>` — read `.tptuir` file

### Tests (Phase 2b)
- [ ] Round-trip `Operation`: serialize → deserialize → assert eq
- [ ] Round-trip `Region` with nested blocks and multiple ops
- [ ] `cargo test -p tpt-uir-serde` green

---

## Phase 3 — Ingestion Adapters (Weeks 5–6)

> These tasks live in the `tpt-gpu` and `tpt-crucible` repos.

### tpt-gpu Adapter
- [ ] Add `tpt-uir-dialects` as a dependency in `tpt-gpu`
- [ ] Implement TPTIR → TPT-UIR converter: map SSA blocks → `Block`, functions → `Region`
- [ ] Use GPU dialect ops (`tpt_gpu.*`) for all emitted operations
- [ ] Emit shapes as `Dimension::Bounded` or `None`

### tpt-crucible Adapter
- [ ] Add `tpt-uir-dialects` as a dependency in `tpt-crucible`
- [ ] Implement ComputationalGraph → TPT-UIR converter: wrap graph as single `Region` containing one `Block`
- [ ] Use Crucible dialect ops (`tpt_crucible.*`) for all emitted operations
- [ ] Emit shapes as `Dimension::Fixed` only

### Round-Trip Validation
- [ ] Write `test_roundtrip_gpu()`: load Llama-3 block → GPU adapter → TPT-UIR → back → assert lossless
- [ ] Write `test_roundtrip_crucible()`: load Llama-3 block → Crucible adapter → TPT-UIR → back → assert lossless
- [ ] Both round-trip tests pass
- [ ] Fix any lossless failures

---

## Phase 4 — Prover Bridge (Weeks 7–8)

> These tasks live in the `tpt-telos` repo.

### tpt-telos Setup
- [ ] Add `tpt-uir-dialects` and `tpt-uir-serde` as dependencies in `tpt-telos`

### TPT-UIR Consumer
- [ ] Walk a `Region`, extract all `tpt_memory.alloc` ops and their `size_bytes` `ValueId` attributes
- [ ] Extract `Dimension::Symbolic` variables from tensor types across the region

### Z3 Integration
- [ ] Map extracted `Dimension::Symbolic` names → Z3 symbolic integer variables
- [ ] Map `mem.alloc` `size_bytes` → Z3 integer expressions

### Memory Bound Proof
- [ ] Implement proof query: within any `mem.scope`, sum of `mem.alloc` sizes ≤ target hardware physical memory limit
- [ ] Integrate with existing `tpt-telos` prover API (or `z3` crate bindings)
- [ ] Return `ProofResult::Valid` or `ProofResult::Counterexample(model)`

### End-to-End Verification
- [ ] Run first formal memory proof on a simple `tpt-crucible` model (all `Fixed` dims, static shapes)
- [ ] Confirm: valid model → `ProofResult::Valid`
- [ ] Confirm: over-budget model → `ProofResult::Counterexample` with interpretable witness
- [ ] Document the full proof workflow in `tpt-telos` README

---

## CI / Quality Gates

- [ ] `cargo test --workspace` — all tests pass
- [ ] `cargo clippy --workspace -- -D warnings` — zero warnings
- [ ] `cargo build --workspace --no-default-features` — confirms `no_std` integrity
- [ ] `cargo fmt --check` — formatted
- [ ] Add GitHub Actions workflow (or equivalent CI) running the above on every push
