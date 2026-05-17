//! Waveform-layer rendering — one row at a time.

mod dontcare;
mod dontcare_pattern;
mod level;
mod poly;
mod state;
mod transition;

use crate::geometry::Point;
use crate::line::{
    EdgeMark, LevelRun, Line, LineContent, SignalLevel, SignalRow, WaveformElement,
};
use crate::style::{ChartStyle, GuideStyle, LayoutParams, SvgAttrList};
use crate::svg::buf::{SvgBuf, WriteSvgOn};
use crate::svg::geometry::WaveformBoxY;
use crate::text::{UnsafeLineText, UserText};
use crate::units::Px;

use crate::svg::waveform::dontcare::{DcSingleKind, DontCarePolygon, DontCarePolygonArgs};
pub(crate) use dontcare_pattern::{DontcareHatchPatternId, DontcareHatchPatternTable};
use state::RowState;

/// Per-row outputs collected by the waveform pass.
///
/// The buffers are accumulated as one row is processed, then rendered as four
/// distinct SVG layer groups (`<g class="...">`) by the four `write_*_layer`
/// methods. Layer ordering is enforced by the caller (`svg/mod.rs`).
#[derive(Debug, Default)]
pub(super) struct RowOutput {
    /// `<polyline>` and waveform-layer `<text>` elements only.
    ///
    /// Concatenated into `<g class="waveforms">`. Edge markers go to a
    /// separate `edge_marks` buffer so they form their own layer.
    polylines: SvgBuf,
    /// `<polygon>` elements for clock edge markers. Emitted as
    /// `<g class="edge-marks">`, between the waveforms and guides layers.
    edge_marks: SvgBuf,
    /// `<rect>` and `<polygon>` elements for DontCare fills (dontcares layer).
    dontcare_rects: SvgBuf,
    /// `<rect>` elements for highlight regions.
    highlight_rects: SvgBuf,
    /// `<line>` elements for guides.
    guide_lines: SvgBuf,
    /// Hatch patterns interned during rendering. One entry per unique
    /// `@dontcare_color` value (deduped by color). The `<defs>` writer
    /// reads this table to emit `<pattern id="dontcare-hatch-N">` per entry;
    /// the table is empty iff no DontCare elements were rendered.
    dontcare_patterns: DontcareHatchPatternTable,
}

impl RowOutput {
    /// Render `<g class="highlights">…</g>` into `target`.
    pub(super) fn write_highlights_layer(&self, target: &mut SvgBuf) {
        target.write_layer_buffer("highlights", &self.highlight_rects);
    }

    /// Render `<g class="dontcares">…</g>` into `target`.
    pub(super) fn write_dontcares_layer(&self, target: &mut SvgBuf) {
        target.write_layer_buffer("dontcares", &self.dontcare_rects);
    }

    /// Whether at least one DontCare element was rendered.
    ///
    /// When `true`, the caller must emit a `<defs>` block containing the
    /// `dontcare-hatch-N` patterns before any layer content.
    pub(super) fn has_dontcare(&self) -> bool {
        !self.dontcare_patterns.is_empty()
    }

    /// Hatch patterns table consumed by the `<defs>` writer.
    pub(super) fn dontcare_patterns(&self) -> &DontcareHatchPatternTable {
        &self.dontcare_patterns
    }

    /// Render `<g class="waveforms">…polylines…</g>` into `target`.
    pub(super) fn write_waveforms_layer(&self, target: &mut SvgBuf) {
        target.write_layer_buffer("waveforms", &self.polylines);
    }

    /// Render `<g class="edge-marks">…edge polygons…</g>` into `target`.
    pub(super) fn write_edge_marks_layer(&self, target: &mut SvgBuf) {
        target.write_layer_buffer("edge-marks", &self.edge_marks);
    }

    /// Render `<g class="guides">…</g>` into `target`.
    pub(super) fn write_guides_layer(&self, target: &mut SvgBuf) {
        target.write_layer_buffer("guides", &self.guide_lines);
    }

    /// Append one signal row's waveform output (polylines, dontcare/highlight
    /// rects, guides, edge markers) into the appropriate sub-buffers.
    fn append_signal_row(
        &mut self,
        origin: Point,
        row: &SignalRow,
        style: &ChartStyle,
        title_boundaries: &TitleBoundaries,
    ) {
        let signal_box = row.geometry().signal_box;
        let ys = WaveformBoxY::from_chart(origin, signal_box.origin, signal_box.size.height);
        let start_x = origin.x + signal_box.origin.x;
        let mut state = RowState::new(start_x, ys);
        let context = RowContext {
            row,
            style,
            origin_y: origin.y,
            title_boundaries,
        };
        let waveform = row.waveform();
        let elements: &[WaveformElement] = waveform;
        for (index, element) in elements.iter().enumerate() {
            self.append_element(element, elements, index, &context, &mut state);
        }
        state.flush_all(&mut self.polylines);
        // Clock-edge triangle markers go to their own `edge-marks` layer
        // (drawn between `waveforms` and `guides`). The arrows layer remains
        // reserved exclusively for `@->` user arrows.
        for edge_mark in row.edge_marks() {
            self.edge_marks.write(edge_mark);
        }
    }

    /// Dispatch one waveform element to the appropriate state/buffer handler.
    fn append_element(
        &mut self,
        element: &WaveformElement,
        elements: &[WaveformElement],
        index: usize,
        context: &RowContext<'_>,
        state: &mut RowState,
    ) {
        // Use this row's own layout snapshot for per-row @step/@slant correctness.
        let layout = context.row.layout_params();
        let width = layout.element_width(element);
        match element {
            WaveformElement::Level(run) => {
                self.append_level(run, elements, index, width, context, state);
            }
            WaveformElement::Transition(transition) => {
                let slant_width = layout.slant();
                state.draw(&transition::TransitionDraw::new(
                    transition,
                    width,
                    slant_width,
                ));
            }
            WaveformElement::Gap => state.handle_gap(width, &mut self.polylines),
            WaveformElement::Guide => self.append_guide(state, context),
            WaveformElement::HighlightStart => state.begin_highlight(),
            WaveformElement::HighlightEnd => self.append_highlight_end(state, context),
            WaveformElement::Anchor(_) => {}
            WaveformElement::Text(text) => {
                self.append_waveform_text(text, elements, index, context, state);
            }
        }
    }

    /// Handle a `Level` run: build the DontCare backing polygon (when this
    /// level is a `DontCareAlong*`) and push the level's polyline points.
    fn append_level(
        &mut self,
        run: &LevelRun,
        elements: &[WaveformElement],
        index: usize,
        width: Px,
        context: &RowContext<'_>,
        state: &mut RowState,
    ) {
        let polygon = run.level().is_dontcare().then(|| {
            let pattern_id = self
                .dontcare_patterns
                .insert_color(context.row.style().signal().dontcare_color());
            let layout = context.row.layout_params();
            let args = DontCarePolygonArgs::new(
                elements,
                index,
                state.cursor(),
                width,
                layout.slant(),
                layout.step(),
                state.waveform_y(),
                pattern_id,
            );
            context.build_dontcare_polygon(run, args)
        });
        state.push_level(run, width, polygon, &mut self.dontcare_rects);
    }

    /// Emit a `<text>` element for a waveform text label.
    ///
    /// The text is centred at the mid-point of the owning level run, which is
    /// immediately before this `Text` element in `elements`. The cursor is not
    /// advanced (width = `Px::ZERO`).
    ///
    /// An empty label (`""`) produces no element at all; a whitespace-only
    /// label (e.g. `" "`) is emitted normally.
    fn append_waveform_text(
        &mut self,
        text: &UserText,
        elements: &[WaveformElement],
        index: usize,
        context: &RowContext<'_>,
        state: &RowState,
    ) {
        if text.is_empty() {
            return;
        }
        let run_width = owning_level_run_width(elements, index, context.row.layout_params());
        let font = context.row.style().label().font();
        self.polylines.write(&WaveformTextDraw {
            x_center: state.cursor() - run_width * 0.5,
            y_center: state.waveform_y().center(),
            font_size: font.size(),
            font_family: font.family().as_unsafe_line(),
            color: context.row.style().signal().color(),
            text,
        });
    }

    /// Append one guide vertical line spanning Title-bounded or chart-bounded y range.
    fn append_guide(&mut self, state: &RowState, context: &RowContext<'_>) {
        let guide_y = context.title_boundaries.calc_guide_y(context.origin_y);
        self.guide_lines.write(&GuideLine {
            x: state.cursor(),
            y1: guide_y.top,
            y2: guide_y.bottom,
            style: context.style.default_guide_style(),
        });
    }

    /// Append a highlight `<rect>` that closes the active highlight region.
    ///
    /// Takes the buffered start x from `state`, uses the current cursor as the
    /// end x, and spans the Title-bounded or chart-bounded y range.
    fn append_highlight_end(&mut self, state: &mut RowState, context: &RowContext<'_>) {
        let Some(x_start) = state.take_highlight_start() else {
            return;
        };
        let guide_y = context.title_boundaries.calc_guide_y(context.origin_y);
        self.highlight_rects.write(&HighlightRect {
            x_start,
            x_end: state.cursor(),
            y1: guide_y.top,
            y2: guide_y.bottom,
            attrs: context.row.style().signal().highlight_attrs(),
        });
    }
}

/// Render every signal row's waveform, accumulating per-layer SVG output.
pub(super) fn render_rows(lines: &[Line], style: &ChartStyle) -> RowOutput {
    let title_boundaries = TitleBoundaries::new(lines, style);
    let mut output = RowOutput::default();
    for line in lines {
        if let LineContent::Signal(row) = &line.content {
            output.append_signal_row(line.bounding_box.origin, row, style, &title_boundaries);
        }
    }
    output
}

/// Drawing context for one signal row, passed to `process_element`.
struct RowContext<'a> {
    /// The signal row being rendered.
    row: &'a SignalRow,
    /// Chart-wide style.
    style: &'a ChartStyle,
    /// Top y of this row's bounding box (chart coordinates).
    origin_y: Px,
    /// Pre-computed Title boundaries for guide/highlight clipping.
    title_boundaries: &'a TitleBoundaries,
}

impl<'a> RowContext<'a> {
    /// Build the DontCare backing polygon for a DC level at `index`.
    ///
    /// Uses this row's own layout parameter snapshot so per-row `@step`/
    /// `@slant` changes produce correct polygon coordinates. The polygon's
    /// shape depends on the DC variant (Low/High/HiZ/Bus) and the adjacent
    /// transitions around `index`.
    fn build_dontcare_polygon(
        &self,
        run: &LevelRun,
        args: DontCarePolygonArgs<'_>,
    ) -> DontCarePolygon {
        if run.level() == SignalLevel::DontCareAlongBus {
            DontCarePolygon::for_bus(args)
        } else {
            let kind = DcSingleKind::from_level(run.level())
                .expect("non-bus DontCare run must be DC-Low/High/HiZ");
            DontCarePolygon::for_single(kind, args)
        }
    }
}

/// Vertical y extent for a guide line or highlight rectangle.
struct GuideY {
    /// Top edge (chart coordinates).
    top: Px,
    /// Bottom edge (chart coordinates).
    bottom: Px,
}

/// Bounding box of one `@title` row, used to clip guide lines and highlight rects.
struct TitleRange {
    /// Top edge of the title row bounding box (chart coordinates).
    top: Px,
    /// Bottom edge of the title row bounding box (chart coordinates).
    bottom: Px,
}

/// Pre-computed Title row boundaries used to clip guide lines and highlight rects.
///
/// Spec: `docs/spec/svg-rendering.md` §「Guide」, §「HighlightStart / HighlightEnd」.
struct TitleBoundaries {
    /// Default top: `first_row.bbox.origin.y - page_margin/2`.
    chart_top: Px,
    /// Default bottom: `last_row.bbox.bottom + page_margin/2`.
    chart_bottom: Px,
    /// Bounding boxes of all `@title` rows, in document order.
    ///
    /// The order is not necessarily sorted by `top`; `calc_guide_y` scans all
    /// entries with `filter` + `fold` and does not require sorted input.
    title_ranges: Vec<TitleRange>,
}

impl TitleBoundaries {
    /// Build `TitleBoundaries` from all rows.
    fn new(lines: &[Line], style: &ChartStyle) -> Self {
        let half_margin = style.page_margin_half();
        let chart_top = lines
            .first()
            .map_or(Px::ZERO, |line| line.bounding_box.origin.y - half_margin);
        let chart_bottom = lines.last().map_or(Px::ZERO, |line| {
            line.bounding_box.origin.y + line.bounding_box.size.height + half_margin
        });
        let title_ranges = lines
            .iter()
            .filter(|line| line.content.is_title())
            .map(|line| {
                let top = line.bounding_box.origin.y;
                let bottom = top + line.bounding_box.size.height;
                TitleRange { top, bottom }
            })
            .collect();
        Self {
            chart_top,
            chart_bottom,
            title_ranges,
        }
    }

    /// Compute the guide/highlight y extent for a source row whose bbox starts at `row_origin_y`.
    ///
    /// Upper boundary: the `bbox_bottom` of the nearest Title row strictly above `row_origin_y`,
    /// or `chart_top` if none exists.
    ///
    /// Lower boundary: the `bbox_top` of the nearest Title row strictly below `row_origin_y`,
    /// or `chart_bottom` if none exists.
    fn calc_guide_y(&self, row_origin_y: Px) -> GuideY {
        let upper = self
            .title_ranges
            .iter()
            .filter(|title| title.top < row_origin_y)
            .map(|title| title.bottom)
            .fold(self.chart_top, Px::max);
        let lower = self
            .title_ranges
            .iter()
            .filter(|title| title.bottom > row_origin_y)
            .map(|title| title.top)
            .fold(self.chart_bottom, Px::min);
        GuideY {
            top: upper,
            bottom: lower,
        }
    }
}

/// `EdgeMark` writes itself as a `<polygon>`.
///
/// Geometry formula follows `docs/spec/svg-rendering.md` §「クロックエッジマーカー」.
impl WriteSvgOn for EdgeMark {
    fn write_svg_on(&self, target: &mut SvgBuf) {
        let triangle = EdgeMarkTriangle::compute(self);
        target.write_literal("<polygon points=\"");
        target.write_px(triangle.apex.x);
        target.write_char(',');
        target.write_px(triangle.apex.y);
        target.write_char(' ');
        target.write_px(triangle.base_left.x);
        target.write_char(',');
        target.write_px(triangle.base_left.y);
        target.write_char(' ');
        target.write_px(triangle.base_right.x);
        target.write_char(',');
        target.write_px(triangle.base_right.y);
        target.write_literal("\"");
        target.write_user_attribute("fill", &self.mark_style.color);
        target.write_static_attribute("stroke", "none");
        target.write_literal("/>");
    }
}

/// Three computed vertices for one clock-edge triangle.
struct EdgeMarkTriangle {
    apex: Point,
    base_left: Point,
    base_right: Point,
}

impl EdgeMarkTriangle {
    /// Compute the three vertices from an `EdgeMark`.
    ///
    /// All variable names and formulae follow `docs/spec/svg-rendering.md`
    /// §「1 つの三角形の幾何」 exactly. Abbreviated identifiers (`perp`,
    /// `ux`, etc.) are forbidden in this codebase.
    fn compute(edge_mark: &EdgeMark) -> Self {
        let position = edge_mark.mark_style.position;
        let height = edge_mark.mark_style.height;
        let width = edge_mark.mark_style.width;

        let delta = edge_mark.line_end - edge_mark.line_start;
        let (line_length, line_direction) = delta.normal();
        let effective_height = height.to_f32().min(line_length);

        // Perpendicular unit vector (clockwise 90° rotation of the line direction).
        let perpendicular_unit = line_direction.perpendicular_clockwise();

        let apex_distance = (line_length - effective_height) * position + effective_height;
        let base_center_distance = (line_length - effective_height) * position;

        let start = edge_mark.line_start;
        let apex = start + line_direction * apex_distance;
        let base_center = start + line_direction * base_center_distance;
        let half_width = width.to_f32() * 0.5;
        let base_left = base_center + perpendicular_unit * half_width;
        let base_right = base_center - perpendicular_unit * half_width;

        Self {
            apex,
            base_left,
            base_right,
        }
    }
}

/// Vertical guide line `<line>` at column `x`, spanning `y1`..`y2`.
struct GuideLine<'style> {
    x: Px,
    y1: Px,
    y2: Px,
    style: &'style GuideStyle,
}

impl WriteSvgOn for GuideLine<'_> {
    fn write_svg_on(&self, target: &mut SvgBuf) {
        target.write_literal("<line");
        target.write_px_attribute("x1", self.x);
        target.write_px_attribute("y1", self.y1);
        target.write_px_attribute("x2", self.x);
        target.write_px_attribute("y2", self.y2);
        target.write_user_attribute("stroke", &self.style.color());
        target.write_px_attribute("stroke-width", self.style.width());
        target.write_literal("/>");
    }
}

/// Look backward from `index` in `elements` to find the most recent `LevelRun`
/// and return its layout width under `layout`. Returns `Px::ZERO` when no
/// preceding level is found.
fn owning_level_run_width(elements: &[WaveformElement], index: usize, layout: &LayoutParams) -> Px {
    elements[..index]
        .iter()
        .rev()
        .find_map(|element| match element {
            WaveformElement::Level(_) => Some(layout.element_width(element)),
            _ => None,
        })
        .unwrap_or(Px::ZERO)
}

/// One waveform-layer `<text>` element for a level-run text label.
///
/// Rendered into `<g class="waveforms">` at the centre of the owning level
/// run. Does not clip; the text may extend beyond the run boundaries.
/// See `docs/spec/svg-rendering.md` §「`Text` — レベル文字列中のテキスト文字」.
struct WaveformTextDraw<'a> {
    x_center: Px,
    y_center: Px,
    font_size: Px,
    font_family: UnsafeLineText<'a>,
    color: crate::color::Color,
    text: &'a UserText,
}

impl WriteSvgOn for WaveformTextDraw<'_> {
    fn write_svg_on(&self, target: &mut SvgBuf) {
        target.write_literal("<text");
        target.write_px_attribute("x", self.x_center);
        target.write_px_attribute("y", self.y_center);
        target.write_user_attribute("font-family", &self.font_family);
        target.write_px_attribute("font-size", self.font_size);
        target.write_user_attribute("fill", &self.color);
        target.write_static_attribute("text-anchor", "middle");
        target.write_static_attribute("dominant-baseline", "middle");
        target.write_char('>');
        target.write_escaped(self.text);
        target.write_literal("</text>");
    }
}

/// Highlight `<rect>` spanning `x_start`..`x_end` and the row's vertical extent.
///
/// The vertical span (`y1`..`y2`) is determined by `TitleBoundaries::calc_guide_y`
/// via `RowContext`: it reaches from the nearest title row boundary above (or
/// `chart_top`) to the nearest title row boundary below (or `chart_bottom`).
struct HighlightRect<'attrs> {
    x_start: Px,
    x_end: Px,
    y1: Px,
    y2: Px,
    attrs: &'attrs SvgAttrList,
}

impl WriteSvgOn for HighlightRect<'_> {
    fn write_svg_on(&self, target: &mut SvgBuf) {
        target.write_literal("<rect");
        target.write_px_attribute("x", self.x_start);
        target.write_px_attribute("y", self.y1);
        target.write_px_attribute("width", self.x_end - self.x_start);
        target.write_px_attribute("height", self.y2 - self.y1);
        for (key, value) in self.attrs.safe_pairs() {
            target.write_char(' ');
            target.write_escaped_str(key);
            target.write_literal("=\"");
            target.write_escaped(value);
            target.write_char('"');
        }
        target.write_literal("/>");
    }
}

