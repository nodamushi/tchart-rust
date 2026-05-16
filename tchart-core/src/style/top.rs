//! Top-level chart style aggregating canvas / background / row defaults.

use crate::color::Color;
use crate::style::canvas::{BackgroundStyle, CanvasStyle};
use crate::style::label::LabelStyle;
use crate::style::layout::LayoutParams;
use crate::style::row::DefaultRowStyles;
use crate::style::signal::{GuideStyle, SignalStyle};
use crate::style::svg_attrs::SvgAttrList;
use crate::text::FontFamily;
use crate::units::Px;

/// Complete chart style: canvas + background + row defaults.
///
/// See `docs/spec/types.md` §4.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ChartStyle {
    canvas: CanvasStyle,
    background: BackgroundStyle,
    defaults: DefaultRowStyles,
    layout: LayoutParams,
}

impl ChartStyle {
    /// Canvas-wide style (read-only).
    pub fn canvas(&self) -> &CanvasStyle {
        &self.canvas
    }

    /// Update font size and recalculate line height atomically.
    ///
    /// This is the public API for external crates to override the font size.
    pub fn set_font_size(&mut self, size: Px) {
        self.canvas.set_font_size(size);
    }

    // -- Tell, don't ask: setters used by the parser and tests --------------

    /// Replace the line-height ratio applied to the current font size.
    pub(crate) fn set_line_height_ratio(&mut self, ratio: f32) {
        self.canvas.set_line_height_ratio(ratio);
    }

    /// Override the resolved line height directly. Test-only.
    #[cfg(test)]
    pub(crate) fn set_line_height(&mut self, height: Px) {
        self.canvas.set_line_height(height);
    }

    /// Override the page margin.
    pub(crate) fn set_page_margin(&mut self, margin: Px) {
        self.canvas.set_page_margin(margin);
    }

    /// Override the SVG-root display scale.
    pub(crate) fn set_scale(&mut self, scale: f32) {
        self.canvas.set_scale(scale);
    }

    /// Update the canvas font family and propagate it to the label default.
    pub(crate) fn set_font_family(&mut self, family: FontFamily) {
        self.canvas.set_font_family(family.clone());
        self.defaults.set_label_font_family(family);
    }

    /// Set the default signal stroke color.
    ///
    /// The label text color is propagated alongside the signal stroke. TCML
    /// has no dedicated `@label_color` directive, so label text and the
    /// signal-name overline (which inherits `LabelStyle.color`) share the
    /// `@signal_color` value.
    pub(crate) fn set_signal_color(&mut self, color: Color) {
        self.defaults.signal_mut().set_color(color);
        self.defaults.label_mut().set_color(color);
    }

    /// Set the default signal stroke width.
    pub(crate) fn set_signal_width(&mut self, width: Px) {
        self.defaults.signal_mut().set_width(width);
    }

    /// Set the default guide color.
    pub(crate) fn set_guide_color(&mut self, color: Color) {
        self.defaults.signal_mut().set_guide_color(color);
    }

    /// Set the default guide width.
    pub(crate) fn set_guide_width(&mut self, width: Px) {
        self.defaults.signal_mut().set_guide_width(width);
    }

    /// Replace the bgcolor0 (even-row) stripe color.
    pub(crate) fn set_bgcolor0(&mut self, color: Color) {
        self.background.set_bgcolor0(color);
    }

    /// Replace the bgcolor1 (odd-row) stripe color.
    pub(crate) fn set_bgcolor1(&mut self, color: Color) {
        self.background.set_bgcolor1(color);
    }

    /// Replace the highlight (`[...]`) attribute list.
    pub(crate) fn set_highlight_attrs(&mut self, attrs: SvgAttrList) {
        self.defaults.signal_mut().set_highlight_attrs(attrs);
    }

    /// Set the don't-care (`?`) hatch line color.
    pub(crate) fn set_dontcare_color(&mut self, color: Color) {
        self.defaults.signal_mut().set_dontcare_color(color);
    }

    /// Set the label padding.
    pub(crate) fn set_name_padding(&mut self, padding: Px) {
        self.defaults.label_mut().set_padding(padding);
    }

    /// Set the overline gap.
    pub(crate) fn set_overline_gap(&mut self, gap: Px) {
        self.defaults.label_mut().set_overline_gap(gap);
    }

    /// Set the overline thickness.
    pub(crate) fn set_overline_thickness(&mut self, thickness: Px) {
        self.defaults.label_mut().set_overline_thickness(thickness);
    }

    /// Set the explicit cap (signal-name) column width.
    pub(crate) fn set_capwidth(&mut self, width: Option<Px>) {
        self.layout.set_capwidth(width);
    }

    /// Set the step width per time unit.
    pub(crate) fn set_step(&mut self, step: Px) {
        self.layout.set_step(step);
    }

    /// Set the slant width for all transitions.
    pub(crate) fn set_slant(&mut self, slant: Px) {
        self.layout.set_slant(slant);
    }

    /// Set the symmetric inter-row gap (`h_space`).
    pub(crate) fn set_h_space(&mut self, h_space: Px) {
        self.layout.set_h_space(h_space);
    }

    // -- Read-only views consumed by layout / SVG ---------------------------

    /// Default label style — the parser snapshots this when constructing rows.
    pub(crate) fn default_label_style(&self) -> &LabelStyle {
        self.defaults.label()
    }

    /// Default signal style — the parser snapshots this when constructing rows.
    pub(crate) fn default_signal_style(&self) -> &SignalStyle {
        self.defaults.signal()
    }

    /// Default guide style consumed by the SVG waveform layer.
    pub(crate) fn default_guide_style(&self) -> &GuideStyle {
        self.defaults.signal().guide()
    }

    /// Layout-time parameters consumed by the layout engine and SVG renderer.
    pub(crate) fn layout(&self) -> &LayoutParams {
        &self.layout
    }

    /// Pick the row-stripe background color for the given 0-based signal-row index.
    pub(crate) fn stripe_for_signal_index(&self, index: u32) -> Color {
        self.background.stripe_for_index(index)
    }

    // -- SVG-level helpers expressed on ChartStyle (Tell, don't ask) -------

    /// Half of `page_margin` — used by the SVG guide-line vertical extension.
    pub(crate) fn page_margin_half(&self) -> Px {
        self.canvas.page_margin() * 0.5
    }
}
