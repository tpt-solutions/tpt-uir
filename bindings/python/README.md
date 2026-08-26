# Python bindings for TPT-UIR (`tpt_uir`)

Pure-[ctypes](https://docs.python.org/3/library/ctypes.html) bindings over the
`tpt-uir-ffi` C ABI. No compiler, PyO3, or pip install required — just a
built native library on your `PATH`.

## Setup

```sh
cargo build --release -p tpt-uir-ffi   # produces target/release/{lib}tpt_uir_ffi.{dll,so,dylib}
```

Then either run Python from the repo root (the loader checks
`target/release/` automatically), or point it at the library explicitly:

```sh
export TPT_UIR_LIBRARY=/path/to/libtpt_uir_ffi.so
export PYTHONPATH=bindings/python
```

## Usage

```python
import tpt_uir

with tpt_uir.Region.load_bytes(open("model.tptuir", "rb").read()) as region:
    print(region.block_count())          # 1
    print(region.op_count(0))            # ops in block 0
    print(region.op_name(0, 0))          # e.g. "tpt_gpu.launch"
    print(region.operand_count(0))       # operands of op #0
    print(region.validate_ssa())         # True
    print(region.validate_dialect("tpt_gpu"))
```

The handle is freed automatically via context manager / finalizer; string
pointers returned by the C API stay valid until `close()`.

## Error model

Mirrors the C contract — any non-zero code raises
`tpt_uir.TptUirError(function, code)`:

| code | meaning |
| --- | --- |
| 1 | bad pointer, handle, or index |
| 2 | postcard decode error |
| 3 | interior NUL byte in an op-name part |

## Smoke test

```sh
python bindings/python/smoke_test.py
```

Builds the fixture region bytes inline (same fixture as the Rust/C tests) and
exercises every accessor.

## Limitations

Read-only inspection, exactly like the C ABI: no IR construction, and no
textual-format parsing in Python — use `tpt-uir-cli convert` to produce
postcard files first.
