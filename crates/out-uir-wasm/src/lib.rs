//! WASM bindings for the TPT-UIR text format, powering the browser playground.
//!
//! Build for the web with:
//!
//! ```sh
//! wasm-pack build crates/out-uir-wasm --target web --out-dir ../../playground/pkg
//! ```
//!
//! Then serve the repo root statically (`python -m http.server`) and open
//! `playground/index.html`.

use tpt_uir_core::validate_region;
use wasm_bindgen::prelude::*;

/// Pretty-print (canonicalize) a textual TPT-UIR region.
///
/// Returns an error string when parsing fails.
#[wasm_bindgen]
pub fn format_text(src: &str) -> Result<String, JsValue> {
    let region = tpt_uir_text::parse_text(src).map_err(|e| JsValue::from_str(&e.0))?;
    Ok(tpt_uir_text::to_text(&region))
}

/// Validate SSA well-formedness of a textual region.
///
/// Returns `Ok(true)` when valid, `Ok(false)` when invalid, and `Err` when the
/// input cannot be parsed.
#[wasm_bindgen]
pub fn validate_text(src: &str) -> Result<bool, JsValue> {
    let region = tpt_uir_text::parse_text(src).map_err(|e| JsValue::from_str(&e.0))?;
    Ok(validate_region(&region).is_ok())
}
