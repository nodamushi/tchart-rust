//! SVG renderer for layout-resolved [`ChartDocument`].
//!
//! See `docs/spec/svg-rendering.md` for the contract. Key design notes:
//!
//! * `WaveformElement::Gap` calls `flush_all` on every accumulator.
//! * x progression uses the layout `LayoutParams::element_width` table;
//!   SVG never recomputes per-element widths.
//! * Every `TransitionKind` is matched exhaustively; shared edges are
//!   rendered as explicit horizontal bridges inside the polyline accumulator.

mod arrows;
mod backgrounds;
mod buf;
mod geometry;
mod labels;
mod overlays;
mod root;
mod rulers;
mod titles;
mod waveform;

#[cfg(test)]
mod tests;

use buf::{SvgBuf, WriteSvgOn};

use crate::document::ChartDocument;
use crate::layout::FontMetrics;

use arrows::ArrowList;
use backgrounds::RowBackgrounds;
use labels::SignalLabels;
use overlays::TextOverlays;
use root::{DontCareHatchDefs, SharedStyle, SourceMetadata};
use rulers::Rulers;
use titles::TitleRows;

/// Render a layout-resolved [`ChartDocument`] into an SVG string.
///
/// `fonts` is used to measure the actual rendered width of signal-name text
/// for the `name_overline` decoration (spec: "実幅 = フォントメトリクスから測ったその行の幅").
pub fn render(document: &ChartDocument, fonts: &dyn FontMetrics) -> String {
    let mut buf = SvgBuf::new();
    // Internal layout dimensions remain at 1.0x; the SVG-root `width`/`height`
    // attributes are the user-visible display size, scaled by `@scale`, and
    // `viewBox` carries the 1.0x logical dimensions so the viewport actually
    // scales. See `docs/spec/svg-rendering.md` §「ルート width / height / viewBox と @scale」.
    let internal_size = root::compute_size(document);
    let display_size = internal_size * document.style.canvas().scale();
    let row_output = waveform::render_rows(&document.lines, &document.style);
    buf.write_svg_root(display_size, internal_size, |body| {
        body.write(&RootBody {
            document,
            fonts,
            row_output: &row_output,
        });
    });
    buf.finish()
}

/// Body of the `<svg>` root: metadata, shared style, then background / waveform / text layers
/// in the order mandated by `docs/spec/svg-rendering.md`.
struct RootBody<'document, 'fonts, 'rows> {
    document: &'document ChartDocument,
    fonts: &'fonts dyn FontMetrics,
    row_output: &'rows waveform::RowOutput,
}

impl WriteSvgOn for RootBody<'_, '_, '_> {
    fn write_svg_on(&self, target: &mut SvgBuf) {
        target.write(&SourceMetadata::new(&self.document.source));
        target.write(&SharedStyle);
        if self.row_output.has_dontcare() {
            target.write(&DontCareHatchDefs::new(self.row_output.dontcare_patterns()));
        }
        // Layer order matches docs/spec/svg-rendering.md §「描画順 (z-order)」:
        // row-backgrounds → rulers → highlights → dontcares → signal-labels
        //   → waveforms → edge-marks → guides → titles → arrows → overlays.
        target.write(&BackgroundLayers {
            document: self.document,
        });
        target.write(&Rulers {
            lines: &self.document.lines,
            style: &self.document.style,
        });
        self.row_output.write_highlights_layer(target);
        self.row_output.write_dontcares_layer(target);
        target.write(&SignalLabelsLayer {
            document: self.document,
            fonts: self.fonts,
        });
        self.row_output.write_waveforms_layer(target);
        self.row_output.write_edge_marks_layer(target);
        self.row_output.write_guides_layer(target);
        target.write(&TopTextLayers {
            document: self.document,
        });
    }
}

/// Background `<g>` group containing one `<rect>` per non-transparent row.
struct BackgroundLayers<'document> {
    document: &'document ChartDocument,
}

impl WriteSvgOn for BackgroundLayers<'_> {
    fn write_svg_on(&self, target: &mut SvgBuf) {
        target.write_layer("row-backgrounds", |layer| {
            layer.write(&RowBackgrounds {
                lines: &self.document.lines,
                style: &self.document.style,
            });
        });
    }
}

/// Signal-labels layer, emitted between `dontcares` and `waveforms`.
struct SignalLabelsLayer<'document, 'fonts> {
    document: &'document ChartDocument,
    fonts: &'fonts dyn FontMetrics,
}

impl WriteSvgOn for SignalLabelsLayer<'_, '_> {
    fn write_svg_on(&self, target: &mut SvgBuf) {
        target.write_layer("signal-labels", |layer| {
            layer.write(&SignalLabels {
                lines: &self.document.lines,
                fonts: self.fonts,
            });
        });
    }
}

/// Top-of-stack layers (`titles`, `arrows`, `overlays`). These follow the
/// waveform/clock/guide stack and are emitted in this order so the arrows
/// layer is rendered exclusively from `Annotations.arrows` — clock markers
/// live in their own `edge-marks` layer drawn earlier.
struct TopTextLayers<'document> {
    document: &'document ChartDocument,
}

impl WriteSvgOn for TopTextLayers<'_> {
    fn write_svg_on(&self, target: &mut SvgBuf) {
        target.write_layer("titles", |layer| {
            layer.write(&TitleRows {
                lines: &self.document.lines,
                page_margin: self.document.style.canvas().page_margin(),
            });
        });
        target.write_layer("arrows", |layer| {
            layer.write(&ArrowList(&self.document.annotations.arrows));
        });
        target.write_layer("overlays", |layer| {
            layer.write(&TextOverlays {
                overlays: &self.document.annotations.overlays,
                canvas: self.document.style.canvas(),
            });
        });
    }
}
