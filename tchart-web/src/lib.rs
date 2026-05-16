//! tchart-web — WebAssembly bindings.
//!
//! See `docs/spec/web.md` for the public surface.
//!
//! The wasm-bindgen-annotated public entry points compile only for the
//! `wasm32` target. Pure logic (parser → layout → SVG, plus extract) is
//! exposed as a target-agnostic API for unit testing.

use tchart_core::errors::ParseError;
use tchart_core::layout::{FontMetrics, layout};
use tchart_core::parser::parse;
use tchart_core::svg::render;
use tchart_core::units::Px;
use tchart_core::wavedrom::{WaveDromWarning, to_wavejson};

pub mod extract;
pub mod png;

#[cfg(target_arch = "wasm32")]
mod wasm_api;

/// Failure surface for [`render_with_metrics`].
///
/// The wasm wrapper splits these two cases: `Parse` becomes a structured
/// `RenderResult.error` object (1-based line / column, character-unit length,
/// English-fixed message) and `Other` is thrown as a JS exception. See
/// `docs/spec/web.md` §`renderTcml`.
#[derive(Debug)]
pub enum RenderError {
    /// TCML parse failure carrying source position metadata.
    Parse(ParseError),
    /// Any other failure (invalid font-size argument, layout failure, etc.).
    /// Treated as an exception by the wasm wrapper because the source-position
    /// model does not apply.
    Other(String),
}

// `Display` + `Error` are intentionally kept on the public API surface so
// embedders that wrap `render_with_metrics` (outside the wasm path) can log
// or chain `RenderError` through standard error traits. The wasm wrapper in
// `wasm_api/mod.rs` does not use `Display` — it splits the variants directly
// to build a structured `RenderResult.error` per `docs/spec/web.md`.
impl std::fmt::Display for RenderError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(error) => write!(formatter, "parse error: {}", error.message_display()),
            Self::Other(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for RenderError {}

/// Run parser → layout → SVG with the supplied font metrics. Returns the
/// SVG string or a [`RenderError`] that distinguishes a structured parse
/// failure from other (layout / font / config) failures.
///
/// `font_size` must be a strictly positive finite value when supplied; zero,
/// negative, and non-finite (`NaN` / infinite) values are rejected with an
/// "invalid font size" error per `docs/spec/web.md` §`RenderOptions`.
pub fn render_with_metrics(
    input: &str,
    font_size: Option<f32>,
    metrics: &dyn FontMetrics,
) -> Result<String, RenderError> {
    let font_size_px = font_size
        .map(validate_font_size)
        .transpose()
        .map_err(RenderError::Other)?;
    let mut document = parse(input).map_err(RenderError::Parse)?;
    if let Some(size) = font_size_px {
        document.set_font_size(size);
    }
    layout(&mut document, metrics)
        .map_err(|error| RenderError::Other(format!("layout error: {error:?}")))?;
    Ok(render(&document, metrics))
}

/// Validate a user-supplied font-size value. The value must be a finite,
/// strictly positive number; zero, negative, `NaN`, and infinite values are
/// rejected with an "invalid font size" error string.
///
/// The numeric check is delegated to [`Px::try_from_positive_finite`] so the
/// wasm and CLI front-ends share a single rejection rule (see
/// `docs/spec/web.md` / `docs/spec/cli.md`).
fn validate_font_size(size: f32) -> Result<Px, String> {
    Px::try_from_positive_finite(size).map_err(|rejected| format!("invalid font size: {rejected}"))
}

/// Convert TCML to WaveJSON. Returns `(json, warnings)` or an error message
/// when parsing fails.
pub fn convert_to_wavejson(input: &str) -> Result<(String, Vec<WaveDromWarning>), String> {
    let document = parse(input).map_err(|error| format!("parse error: {error:?}"))?;
    Ok(to_wavejson(&document))
}

#[cfg(test)]
mod tests;
