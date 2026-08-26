# TPT-UIR WASM Playground

A minimal browser playground for the textual TPT-UIR format, powered by the
`tpt-uir-wasm` crate (`format_text` / `validate_text` over
[wasm-bindgen](https://rustwasm.github.io/wasm-pack/)).

## Build & run

```sh
# one-time: install wasm-pack (https://rustwasm.github.io/wasm-pack/)
wasm-pack build crates/tpt-uir-wasm --target web --out-dir ../../playground/pkg

# serve the repo root from any static file server:
python -m http.server 8000
# open http://localhost:8000/playground/
```

Paste a textual region on the left, click **Format & validate**, and see the
canonical printer output plus SSA status on the right.

The `pkg/` directory is generated and not committed. The workspace CI gates do
not build WASM; the crate is still kept compiling natively via the standard
`cargo clippy/test` runs.
