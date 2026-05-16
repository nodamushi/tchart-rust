//! `Line` and `LineContent` — the row container.

use crate::clock::ClockEdge;
use crate::color::Color;
use crate::geometry::{Point, Rect};
use crate::line::row::{EdgeMark, SignalRow, SkipRow, TitleRow};
use crate::line::ruler::RulerContribution;
use crate::line::transition::Transition;
use crate::line::waveform::{SignalLevel, WaveformElement};

use crate::layout::ChartDimensions;
use crate::style::{ChartStyle, LayoutParams};
use crate::units::Px;

/// One row in the chart.
///
/// See `docs/spec/types.md` §3.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Line {
    /// Global bounding rectangle for the row.
    pub(crate) bounding_box: Rect,
    /// Local row background (`@bg`). Overrides `@bgcolor0/1` for this row when
    /// `Some`. `None` means fall back to the alternating bgcolor stripes.
    pub(crate) background: Option<Color>,
    /// `@ruler` background-guide donations contributed by this row when it
    /// was committed under `@ruler on` (parser-time snapshot). Empty for
    /// title rows and for rows committed while `@ruler off`. See
    /// `docs/spec/tcml-format.md` §「`@ruler` の詳細」.
    pub(crate) ruler_contributions: Vec<RulerContribution>,
    /// Row payload.
    pub(crate) content: LineContent,
}

impl Line {
    /// Run the stacking pass mutating every `Line.bounding_box` and `SignalRow.geometry`.
    pub(crate) fn stack_lines(
        lines: &mut [Self],
        style: &ChartStyle,
        capwidth: Px,
    ) -> StackingResult {
        // Empty chart: ignore the passed-in capwidth and use zero.
        let effective_capwidth = if lines.is_empty() { Px::ZERO } else { capwidth };
        let max_waveform_width = Self::compute_max_waveform_width(lines);
        let chart_content_width = effective_capwidth + max_waveform_width;
        let origin_x = style.canvas().page_margin();
        let mut cursor_y = style.canvas().page_margin();
        for line in lines.iter_mut() {
            line.place_line(
                style,
                effective_capwidth,
                chart_content_width,
                origin_x,
                cursor_y,
            );
            cursor_y = cursor_y + line.bounding_box.size.height;
        }
        StackingResult {
            capwidth: effective_capwidth,
            max_waveform_width,
            stacking_end_y: cursor_y,
        }
    }

    /// Construct a new [`Line`] with an empty (default) bounding box.
    pub(crate) fn new(content: LineContent, background: Option<Color>) -> Self {
        Self {
            bounding_box: Rect::default(),
            background,
            ruler_contributions: Vec::new(),
            content,
        }
    }

    /// Construct a new [`Line`] with explicit ruler contributions and an
    /// empty bounding box. Used by the parser when a row is committed under
    /// `@ruler on` and the donation set has already been computed.
    pub(crate) fn new_with_ruler_contributions(
        content: LineContent,
        background: Option<Color>,
        ruler_contributions: Vec<RulerContribution>,
    ) -> Self {
        Self {
            bounding_box: Rect::default(),
            background,
            ruler_contributions,
            content,
        }
    }

    #[cfg(test)]
    /// Construct a new [`Line`] with an empty (default) bounding box.
    pub(crate) fn new_with_bounding_box(
        content: LineContent,
        background: Option<Color>,
        bounding_box: Rect,
    ) -> Self {
        Self {
            bounding_box,
            background,
            ruler_contributions: Vec::new(),
            content,
        }
    }

    fn compute_max_waveform_width(lines: &[Self]) -> Px {
        lines
            .iter()
            .filter_map(|line| {
                if let LineContent::Signal(row) = &line.content {
                    Some(row)
                } else {
                    None
                }
            })
            // Each row uses its own layout snapshot so that per-row @step/@slant
            // changes produce correct per-row widths when computing the chart maximum.
            .map(|row| row.layout_params().sum_element_widths(row.waveform()))
            .fold(Px::ZERO, Px::max)
    }

    /// Propagate a CLI/WASM `--font-size` override into per-row style
    /// snapshots captured at parse time. `SkipRow` has no font, so it is a
    /// no-op for that variant.
    pub(crate) fn set_font_size(&mut self, size: Px) {
        match &mut self.content {
            LineContent::Signal(row) => row.set_font_size(size),
            LineContent::Title(row) => row.set_font_size(size),
            LineContent::Skip(_) => {}
        }
    }

    /// Populate `SignalRow.edge_marks` with one [`EdgeMark`] per matching
    /// clock-edge transition. No-op for non-Signal rows or rows whose
    /// `decorations.clock` is `None` / `ClockEdge::None`.
    pub(crate) fn fill_clock_edge_marks(&mut self) {
        // Collect all data while holding only an immutable borrow, then drop it.
        let marks = {
            let LineContent::Signal(row) = &self.content else {
                return;
            };
            let Some(spec) = row.decorations().clock.clone() else {
                return;
            };
            if spec.is_edge_none() || row.waveform().is_empty() {
                return;
            }
            let signal_box = row.geometry().signal_box;
            let signal_origin = self.bounding_box.origin + signal_box.origin;
            // Use this row's own layout snapshot for per-row @step/@slant correctness.
            let layout = row.layout_params();
            collect_edge_marks(
                row.waveform(),
                spec.edge,
                signal_origin,
                signal_box.size.height,
                layout.slant(),
                spec.mark_style.clone(),
                layout,
            )
        };

        let LineContent::Signal(row) = &mut self.content else {
            return;
        };
        row.extend_edge_marks(marks);
    }

    fn place_line(
        &mut self,
        style: &ChartStyle,
        capwidth: Px,
        chart_content_width: Px,
        x: Px,
        y: Px,
    ) {
        let computed_rect = match &mut self.content {
            LineContent::Signal(row) => {
                // Pass chart_content_width so all Signal rows get uniform bounding_box width.
                row.place_signal(x, y, style, capwidth, chart_content_width)
            }
            LineContent::Skip(skip) => {
                let height = skip.amount.resolve(style.canvas().line_height());
                Rect::new(x, y, chart_content_width, height)
            }
            LineContent::Title(title) => {
                let count = title.text.count_line().max(1) as f32;
                let height = style.canvas().line_height() * count;
                Rect::new(x, y, chart_content_width, height)
            }
        };
        self.bounding_box = computed_rect;
    }
}

use crate::clock::ClockMarkStyle;

fn collect_edge_marks(
    elements: &[WaveformElement],
    edge: ClockEdge,
    signal_origin: Point,
    signal_height: Px,
    slant: Px,
    mark_style: ClockMarkStyle,
    layout: &LayoutParams,
) -> Vec<EdgeMark> {
    elements
        .iter()
        .scan(Px::ZERO, |cursor_x, element| {
            let element_width = layout.element_width(element);
            let mark = match element {
                WaveformElement::Transition(transition) if transition.is_clock_edge_match(edge) => {
                    build_edge_mark(
                        transition,
                        signal_origin,
                        *cursor_x,
                        signal_height,
                        slant,
                        mark_style.clone(),
                    )
                }
                _ => None,
            };
            *cursor_x = *cursor_x + element_width;
            Some(mark)
        })
        .flatten()
        .collect()
}

/// Build one `EdgeMark` for a matching clock transition.
///
/// Coordinates follow `docs/spec/types.md` §3.4.2 step 5:
/// - `Pos` (`Low → High`): `line_start = (x, y_low)`, `line_end = (x + slant, y_high)`.
/// - `Neg` (`High → Low`): `line_start = (x, y_high)`, `line_end = (x + slant, y_low)`.
fn build_edge_mark(
    transition: &Transition,
    signal_origin: Point,
    cursor_x: Px,
    signal_height: Px,
    slant: Px,
    mark_style: ClockMarkStyle,
) -> Option<EdgeMark> {
    let x = signal_origin.x + cursor_x;
    let y_high = signal_origin.y;
    let y_low = signal_origin.y + signal_height;
    let (line_start, line_end) = match (transition.source, transition.target) {
        (SignalLevel::Low, SignalLevel::High) => {
            (Point::new(x, y_low), Point::new(x + slant, y_high))
        }
        (SignalLevel::High, SignalLevel::Low) => {
            (Point::new(x, y_high), Point::new(x + slant, y_low))
        }
        _ => return None,
    };

    Some(EdgeMark::new(line_start, line_end, mark_style))
}

/// Variants of [`Line`] payload.
///
/// See `docs/spec/types.md` §3.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum LineContent {
    /// A signal row. Boxed because `SignalRow` is the largest variant.
    Signal(Box<SignalRow>),
    /// A `@skip` row.
    Skip(SkipRow),
    /// A `@title` row.
    Title(TitleRow),
}

impl LineContent {
    /// Returns `true` if this line carries a `@title` row.
    pub(crate) fn is_title(&self) -> bool {
        matches!(self, Self::Title(_))
    }
}

/// Result of stacking — per-row x/y placement plus chart-wide geometry.
///
/// One-shot result struct: values are produced by [`stack_lines`] and
/// consumed by [`Self::into_chart_dimensions`] in the same module. Fields
/// are `pub(super)` so the consumer reads them directly instead of routing
/// through noun-only accessors.
#[derive(Debug, Clone, Copy)]
pub(crate) struct StackingResult {
    /// Resolved cap-column width.
    pub(super) capwidth: Px,
    /// Maximum waveform width across signal rows (Px::ZERO if no signal rows).
    pub(super) max_waveform_width: Px,
    /// Y at which the next line would start (bottom of the last bounding_box).
    pub(super) stacking_end_y: Px,
}

impl StackingResult {
    /// Convert this stacking result into the chart-wide [`ChartDimensions`].
    pub(crate) fn into_chart_dimensions(self, style: &ChartStyle) -> ChartDimensions {
        let page_margin = style.canvas().page_margin();
        let content_width = self.capwidth + self.max_waveform_width;
        let total_width = content_width + (page_margin * 2.0);
        let total_height = self.stacking_end_y + page_margin;
        ChartDimensions {
            width: total_width,
            height: total_height,
        }
    }
}
