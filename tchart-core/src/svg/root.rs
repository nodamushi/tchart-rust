//! SVG root element helpers: `<metadata>`, `<style>`, and chart-size resolution.
//!
//! The `<svg>` opening / closing tags themselves are emitted by
//! [`SvgBuf::write_svg_root`], a bracket API that owns the close tag so that
//! callers do not have to thread `&mut SvgBuf` through `open_*` / `close_*`
//! free functions.

use crate::defaults::{DEFAULT_DONTCARE_HATCH_STROKE_WIDTH_PX, DEFAULT_DONTCARE_HATCH_TILE_PX};
use crate::document::{ChartDocument, TcmlSource};
use crate::geometry::Size;
use crate::line::Line;
use crate::style::ChartStyle;
use crate::svg::buf::{SvgBuf, WriteSvgOn};
use crate::svg::waveform::DontcareHatchPatternTable;
use crate::units::Px;

/// CSS rules embedded inside `<style>`.
const SHARED_CSS: &str = ".waveforms polyline { fill: none; stroke: black; }\n\
.guides line { stroke: red; }\n\
.highlights rect { fill: #ff8; }";

/// Compute the chart's outer size from `document.lines` and `style.canvas().page_margin()`.
pub(super) fn compute_size(document: &ChartDocument) -> Size {
    let style = &document.style;
    let height = calc_stack_total_height(&document.lines, style.canvas().page_margin());
    let width = calc_chart_total_width(&document.lines, style);
    Size { width, height }
}

fn calc_stack_total_height(lines: &[Line], page_margin: Px) -> Px {
    let last_bottom = lines
        .last()
        .map(|line| line.bounding_box.origin.y + line.bounding_box.size.height)
        .unwrap_or(page_margin);
    last_bottom + page_margin
}

fn calc_chart_total_width(lines: &[Line], style: &ChartStyle) -> Px {
    // All lines share the same bounding_box.size.width after the layout pass.
    // Use the first line's value (+ its origin.x for page_margin) plus one
    // trailing page_margin.  For an empty chart, return 2 * page_margin.
    match lines.first() {
        Some(first) => {
            first.bounding_box.origin.x
                + first.bounding_box.size.width
                + style.canvas().page_margin()
        }
        None => style.canvas().page_margin() * 2.0,
    }
}

/// `<metadata><tchart:source>...</tchart:source></metadata>`.
pub(super) struct SourceMetadata<'source>(&'source TcmlSource);

impl<'source> SourceMetadata<'source> {
    /// Construct a `SourceMetadata` wrapping the given source.
    pub(super) fn new(source: &'source TcmlSource) -> Self {
        Self(source)
    }
}

impl WriteSvgOn for SourceMetadata<'_> {
    fn write_svg_on(&self, target: &mut SvgBuf) {
        target.write_literal("<metadata><tchart:source>");
        target.write_escaped_str(self.0.as_str());
        target.write_literal("</tchart:source></metadata>");
    }
}

/// Shared `<style>` block embedded at the top of the SVG document.
pub(super) struct SharedStyle;

impl WriteSvgOn for SharedStyle {
    fn write_svg_on(&self, target: &mut SvgBuf) {
        target.write_literal("<style>");
        target.write_literal(SHARED_CSS);
        target.write_literal("</style>");
    }
}

/// `<defs>` block containing one `<pattern id="dontcare-hatch-N">` per unique
/// hatch line color used by the chart.
///
/// Emitted only when the chart contains at least one DontCare (`?`) element
/// (i.e. when the table is non-empty).
/// See `docs/spec/svg-rendering.md` §「`<defs>` (パターン定義)」.
pub(super) struct DontCareHatchDefs<'table> {
    table: &'table DontcareHatchPatternTable,
}

impl<'table> DontCareHatchDefs<'table> {
    pub(super) fn new(table: &'table DontcareHatchPatternTable) -> Self {
        Self { table }
    }
}

impl WriteSvgOn for DontCareHatchDefs<'_> {
    fn write_svg_on(&self, target: &mut SvgBuf) {
        target.write_literal("<defs>");
        for pattern in self.table.as_slice() {
            target.write_literal("<pattern id=\"");
            target.write_dontcare_id(pattern.id());
            target.write_char('"');
            target.write_static_attribute("patternUnits", "userSpaceOnUse");
            target.write_px_attribute("width", DEFAULT_DONTCARE_HATCH_TILE_PX);
            target.write_px_attribute("height", DEFAULT_DONTCARE_HATCH_TILE_PX);
            target.write_static_attribute("patternTransform", "rotate(45)");
            target.write_literal(">");
            target.write_literal("<line");
            target.write_static_attribute("x1", "0");
            target.write_static_attribute("y1", "0");
            target.write_static_attribute("x2", "0");
            target.write_px_attribute("y2", DEFAULT_DONTCARE_HATCH_TILE_PX);
            target.write_user_attribute("stroke", &pattern.stroke_color());
            target.write_px_attribute("stroke-width", DEFAULT_DONTCARE_HATCH_STROKE_WIDTH_PX);
            target.write_literal("/>");
            target.write_literal("</pattern>");
        }
        target.write_literal("</defs>");
    }
}
