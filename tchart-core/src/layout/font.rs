//! Font metrics abstraction injected into the layout engine.
//!
//! `tchart-core` does not depend on any specific font implementation. The
//! layout engine measures text widths through this trait so that each host
//! (CLI uses `fontdue`/`ab_glyph`, Web uses `Canvas.measureText`) can supply
//! its own metric source.
//!
//! See `docs/spec/architecture.md` "FontMetrics の抽象化".

use crate::text::FontSpec;
use crate::units::Px;

/// Source of text-width measurements used by layout.
pub trait FontMetrics {
    /// Returns the rendered advance width of `text` in `font` (in pixels).
    fn measure_text_width(&self, text: &str, font: &FontSpec) -> Px;
}
