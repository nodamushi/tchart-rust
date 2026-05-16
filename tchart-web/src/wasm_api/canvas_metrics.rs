//! `FontMetrics` implementation backed by the browser's Canvas 2D context.
//!
//! Active only on the `wasm32` target.

use tchart_core::layout::FontMetrics;
use tchart_core::text::FontSpec;
use tchart_core::units::Px;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

/// Holds an offscreen 2D context used to measure text advance widths.
pub struct CanvasFontMetrics {
    context: CanvasRenderingContext2d,
}

impl CanvasFontMetrics {
    /// Allocate a hidden `<canvas>` and grab its 2D context.
    pub fn new() -> Result<Self, JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
        let document = window
            .document()
            .ok_or_else(|| JsValue::from_str("no document"))?;
        let element = document.create_element("canvas")?;
        let canvas: HtmlCanvasElement = element
            .dyn_into()
            .map_err(|_| JsValue::from_str("create_element did not return a canvas"))?;
        let context_object = canvas
            .get_context("2d")?
            .ok_or_else(|| JsValue::from_str("no 2d context"))?;
        let context: CanvasRenderingContext2d = context_object
            .dyn_into()
            .map_err(|_| JsValue::from_str("get_context did not return a 2d context"))?;
        Ok(CanvasFontMetrics { context })
    }
}

impl FontMetrics for CanvasFontMetrics {
    fn measure_text_width(&self, text: &str, font: &FontSpec) -> Px {
        let css = font.to_canvas_css();
        self.context.set_font(&css);
        let width = self
            .context
            .measure_text(text)
            .map(|m| m.width())
            .unwrap_or(0.0);
        Px(width as f32)
    }
}
