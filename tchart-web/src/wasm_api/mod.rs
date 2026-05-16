//! wasm-bindgen entry points (compiled only for the `wasm32` target).
//!
//! Mirrors `docs/spec/web.md` §JavaScript API.

use tchart_core::errors::ParseError;
use wasm_bindgen::prelude::*;

use crate::{RenderError, convert_to_wavejson, extract, png, render_with_metrics};

use self::canvas_metrics::CanvasFontMetrics;

mod canvas_metrics;

/// Render a TCML source string to SVG. The optional `options` argument
/// accepts `{ fontSize?: number }`.
///
/// Returns a `RenderResult` object. On success the result has a single
/// `svg` field carrying the SVG string. On TCML parse failure the result
/// has a single `error` field of shape `{ line, column, length, message }`
/// (1-based line / column, character-unit length, English-fixed message).
/// Non-parse failures (font / layout / invalid font-size argument) are
/// thrown as JS exceptions. See `docs/spec/web.md` §`renderTcml`.
#[wasm_bindgen(js_name = "renderTcml")]
pub fn render_tcml(input: &str, options: Option<js_sys::Object>) -> Result<JsValue, JsValue> {
    let font_size = match options.as_ref() {
        Some(object) => read_font_size(object)?,
        None => None,
    };
    let metrics = CanvasFontMetrics::new()?;
    match render_with_metrics(input, font_size, &metrics) {
        Ok(svg) => {
            // Build `{ svg: <svg string> }`.
            let result = js_sys::Object::new();
            js_sys::Reflect::set(&result, &JsValue::from_str("svg"), &JsValue::from_str(&svg))?;
            Ok(result.into())
        }
        Err(RenderError::Parse(error)) => build_parse_error_result(&error),
        Err(RenderError::Other(message)) => Err(JsValue::from_str(&message)),
    }
}

// Build `{ error: { line, column, length, message } }`.
fn build_parse_error_result(error: &ParseError) -> Result<JsValue, JsValue> {
    let info = js_sys::Object::new();
    js_sys::Reflect::set(
        &info,
        &JsValue::from_str("line"),
        &JsValue::from_f64(f64::from(error.line())),
    )?;
    js_sys::Reflect::set(
        &info,
        &JsValue::from_str("column"),
        &JsValue::from_f64(f64::from(error.column())),
    )?;
    js_sys::Reflect::set(
        &info,
        &JsValue::from_str("length"),
        &JsValue::from_f64(f64::from(error.length())),
    )?;
    js_sys::Reflect::set(
        &info,
        &JsValue::from_str("message"),
        &JsValue::from_str(&error.message()),
    )?;
    let result = js_sys::Object::new();
    js_sys::Reflect::set(&result, &JsValue::from_str("error"), &info)?;
    Ok(result.into())
}

/// Extract the original TCML source embedded in a tchart-generated SVG.
/// Returns `undefined` (Rust `None`) when the marker is missing.
#[wasm_bindgen(js_name = "extractTcmlSource")]
pub fn extract_tcml_source(svg: &str) -> Option<String> {
    extract::extract_tcml_source(svg)
}

/// Extract `tchart-source` iTXt chunk text from a PNG byte buffer. Returns
/// `undefined` when the chunk is missing or the input is not a valid PNG.
/// See `docs/spec/web.md` §`extract_tcml_source_from_png`.
#[wasm_bindgen(js_name = "extractTcmlSourceFromPng")]
pub fn extract_tcml_source_from_png(bytes: &[u8]) -> Option<String> {
    png::extract_tcml_source_from_png(bytes)
}

/// Embed `source` into a `tchart-source` iTXt chunk inside the given PNG.
/// Returns the new PNG byte buffer, or throws when input is not a valid PNG.
/// See `docs/spec/web.md` §`embed_tcml_source_in_png`.
#[wasm_bindgen(js_name = "embedTcmlSourceInPng")]
pub fn embed_tcml_source_in_png(bytes: &[u8], source: &str) -> Result<Vec<u8>, JsValue> {
    png::embed_tcml_source_in_png(bytes, source).map_err(|message| JsValue::from_str(&message))
}

/// Convert TCML to WaveJSON. Returns `{ json: string, warnings: string[] }`
/// or throws when parsing fails. See `docs/spec/web.md` §`to_wavejson`.
#[wasm_bindgen(js_name = "toWaveJson")]
pub fn to_wavejson(input: &str) -> Result<JsValue, JsValue> {
    let (json, warnings) =
        convert_to_wavejson(input).map_err(|message| JsValue::from_str(&message))?;
    let result = js_sys::Object::new();
    js_sys::Reflect::set(
        &result,
        &JsValue::from_str("json"),
        &JsValue::from_str(&json),
    )?;
    let warning_array = js_sys::Array::new();
    for warning in warnings {
        warning_array.push(&JsValue::from_str(&warning.to_string()));
    }
    js_sys::Reflect::set(&result, &JsValue::from_str("warnings"), &warning_array)?;
    Ok(result.into())
}

fn read_font_size(options: &js_sys::Object) -> Result<Option<f32>, JsValue> {
    let value = js_sys::Reflect::get(options.as_ref(), &JsValue::from_str("fontSize"))?;
    if value.is_undefined() || value.is_null() {
        return Ok(None);
    }
    let number = value
        .as_f64()
        .ok_or_else(|| JsValue::from_str("fontSize must be a number"))?;
    Ok(Some(number as f32))
}
