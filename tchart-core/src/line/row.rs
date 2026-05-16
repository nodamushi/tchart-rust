//! Row-content variants: `SignalRow`, `SkipRow`, `TitleRow` and their
//! supporting structures.

use super::{SignalLevel, WaveformElement};
use crate::clock::{ClockMarkStyle, ClockPhase, ClockSpec};
use crate::geometry::{Point, Rect, Size};
use crate::line::waveform::Waveform;
use crate::style::{ChartStyle, LayoutParams, SignalRowStyle, TitleStyle};
use crate::text::{SignalName, UserText};
use crate::units::{Length, Px};

/// A single clock-edge triangle marker attached to a `SignalRow`.
///
/// The triangle is rendered as `<polygon>` inside `<g class="waveforms">`,
/// directly after the polylines for the owning `SignalRow`.
///
/// See `docs/spec/types.md` §3.4.1 and `docs/spec/svg-rendering.md`
/// §「クロックエッジマーカー」.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct EdgeMark {
    /// Start of the transition line.
    pub(crate) line_start: Point,
    /// End of the transition line.
    pub(crate) line_end: Point,
    /// Triangle style.
    pub(crate) mark_style: ClockMarkStyle,
}

impl EdgeMark {
    /// Construct a new [`EdgeMark`].
    pub(crate) fn new(line_start: Point, line_end: Point, mark_style: ClockMarkStyle) -> Self {
        Self {
            line_start,
            line_end,
            mark_style,
        }
    }
}

/// Geometry for a signal row, expressed in `Line.bounding_box` local coordinates.
///
/// See `docs/spec/types.md` §3.2.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(crate) struct SignalGeometry {
    /// Local rectangle for the signal name label.
    pub(crate) label_box: Rect,
    /// Local rectangle for the waveform body.
    pub(crate) signal_box: Rect,
}

impl SignalGeometry {
    /// Construct an explicit [`SignalGeometry`] with both boxes set
    /// (used by the stacking pass and by SVG-render unit tests that
    /// bypass the layout pass).
    pub(crate) fn new(label_box: Rect, signal_box: Rect) -> Self {
        Self {
            label_box,
            signal_box,
        }
    }
}

/// Decoration flags attached to a signal row (clock spec, name overline).
///
/// See `docs/spec/types.md` §3.4.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct SignalDecorations {
    /// Optional clock specification.
    pub(crate) clock: Option<ClockSpec>,
    /// Whether an overline should be drawn above the signal name.
    pub(crate) name_overline: bool,
}

impl SignalDecorations {
    /// Construct a [`SignalDecorations`] with a clock spec and overline flag.
    pub(crate) fn new(clock: Option<ClockSpec>, name_overline: bool) -> Self {
        Self {
            clock,
            name_overline,
        }
    }

    /// Returns `true` when an overline should be drawn above the signal name.
    pub(crate) fn is_name_overline(&self) -> bool {
        self.name_overline
    }
}

/// A signal row carrying its geometry, name, waveform, style, and decorations.
///
/// See `docs/spec/types.md` §3.2.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SignalRow {
    /// Local geometry of the row.
    geometry: SignalGeometry,
    /// Signal name.
    name: SignalName,
    /// Waveform element list.
    waveform: Waveform,
    /// Per-row style.
    style: SignalRowStyle,
    /// Decoration flags.
    decorations: SignalDecorations,
    /// Clock-edge triangle markers. Populated by the clock-expansion pass
    /// (parser or layout). Each entry becomes a `<polygon>` inside
    /// `<g class="waveforms">`. `@->` arrows are **never** mixed in here.
    edge_marks: Vec<EdgeMark>,
    /// Snapshot of the layout parameters active when this row was parsed.
    ///
    /// `@step` and `@slant` are "local parameters" that may be re-specified
    /// between signal rows. Snapshotting at parse time ensures that each row
    /// uses the values in effect at the point it was written, not the final
    /// global value (which would cause all rows to share the last value seen).
    layout_params: LayoutParams,
}

impl SignalRow {
    /// Construct a [`SignalRow`] with the given layout parameter snapshot.
    pub(crate) fn new(
        geometry: SignalGeometry,
        name: SignalName,
        waveform: Waveform,
        style: SignalRowStyle,
        decorations: SignalDecorations,
        layout_params: LayoutParams,
    ) -> Self {
        Self {
            geometry,
            name,
            waveform,
            style,
            decorations,
            edge_marks: Vec::new(),
            layout_params,
        }
    }

    /// Construct a [`SignalRow`] with explicit edge marks and layout params.
    #[cfg(test)]
    pub(crate) fn new_with_edge_marks(
        geometry: SignalGeometry,
        name: SignalName,
        waveform: Waveform,
        style: SignalRowStyle,
        decorations: SignalDecorations,
        edge_marks: Vec<EdgeMark>,
        layout_params: LayoutParams,
    ) -> Self {
        Self {
            geometry,
            name,
            waveform,
            style,
            decorations,
            edge_marks,
            layout_params,
        }
    }

    pub(crate) fn waveform(&self) -> &Waveform {
        &self.waveform
    }

    pub(crate) fn geometry(&self) -> &SignalGeometry {
        &self.geometry
    }

    pub(crate) fn decorations(&self) -> &SignalDecorations {
        &self.decorations
    }

    pub(crate) fn name(&self) -> &SignalName {
        &self.name
    }

    pub(crate) fn edge_marks(&self) -> &[EdgeMark] {
        &self.edge_marks
    }

    pub(crate) fn style(&self) -> &SignalRowStyle {
        &self.style
    }

    /// Layout parameters snapshot captured when this row was parsed.
    pub(crate) fn layout_params(&self) -> &LayoutParams {
        &self.layout_params
    }

    /// Overwrite the layout-parameter snapshot. Test-only helper for layout
    /// unit tests that build a `SignalRow` first and configure
    /// `ChartStyle` afterward; production code captures the snapshot at
    /// parse time and must not mutate it later.
    #[cfg(test)]
    pub(crate) fn set_layout_params(&mut self, layout_params: LayoutParams) {
        self.layout_params = layout_params;
    }

    /// Assign geometry (used by the layout stacking pass after placement).
    pub(crate) fn assign_geometry(&mut self, label_box: Rect, signal_box: Rect) {
        self.geometry = SignalGeometry::new(label_box, signal_box);
    }

    /// Extend the edge-mark list from an iterator.
    pub(crate) fn extend_edge_marks(&mut self, marks: impl IntoIterator<Item = EdgeMark>) {
        self.edge_marks.extend(marks);
    }

    /// Propagate a CLI/WASM `--font-size` override into the per-row style
    /// snapshot. Each row holds its own `SignalRowStyle` snapshot captured at
    /// parse time, so the override is not visible in the SVG text elements
    /// unless every snapshot is updated.
    pub(crate) fn set_font_size(&mut self, size: Px) {
        self.style.set_font_size(size);
    }

    /// 配置パスから呼ばれ、信号行の bounding box を計算しつつ
    /// label box / signal box を確定して `assign_geometry` を内部で行う。
    /// 戻り値は外部スタッカが y カーソルを進めるための bounding box。
    pub(crate) fn place_signal(
        &mut self,
        x: Px,
        y: Px,
        style: &ChartStyle,
        capwidth: Px,
        chart_inner_width: Px,
    ) -> Rect {
        let line_height = style.canvas().line_height();
        // Waveform body height equals the chart-wide line height (see docs/spec/types.md §3.2).
        let body_height = line_height;
        let label_lines = self.name.count_line().max(1) as f32;
        let label_height = line_height * label_lines;
        // Use this row's own layout snapshot so that @step/@slant changes between
        // rows are applied per-row, not globally (last-wins).
        let body_width = self.layout_params.sum_element_widths(&self.waveform);
        let content_height = body_height.max(label_height);
        // bounding_box width uses the chart-wide maximum, not this row's own wave width.
        let bbox = Rect::new(
            x,
            y,
            chart_inner_width,
            content_height + self.layout_params.h_space(),
        );
        let (label_box, signal_box) = Self::compute_signal_geometry_inside(
            bbox.size,
            body_width,
            body_height,
            label_height,
            capwidth,
        );
        self.assign_geometry(label_box, signal_box);

        bbox
    }

    /// Compute the local `(label_box, signal_box)` for a `SignalRow` given the
    /// row's bbox size, the chart-level `capwidth`, and the body / label heights.
    /// Both boxes are vertically centred inside the bbox.
    fn compute_signal_geometry_inside(
        bbox_size: Size,
        waveform_width: Px,
        body_height: Px,
        label_height: Px,
        capwidth: Px,
    ) -> (Rect, Rect) {
        let signal_y = (bbox_size.height - body_height) * 0.5;
        let label_y = (bbox_size.height - label_height) * 0.5;
        let signal_box = Rect {
            origin: Point {
                x: capwidth,
                y: signal_y,
            },
            size: Size {
                width: waveform_width,
                height: body_height,
            },
        };
        let label_box = Rect {
            origin: Point {
                x: Px::ZERO,
                y: label_y,
            },
            size: Size {
                width: capwidth,
                height: label_height,
            },
        };
        (label_box, signal_box)
    }

    /// Expand this row's clock waveform out to `target_units` columns by
    /// appending alternating level runs whose lengths are dictated by
    /// `spec.pulse`. If the row already has user-written content, the
    /// expansion picks up from the last level seen; otherwise it seeds the
    /// row with `spec.start`.
    ///
    /// When `target_units == 0` (all signals are auto) or `target_units` is
    /// less than or equal to the existing unit count, the waveform is left
    /// unchanged.
    ///
    /// Prefer calling [`Self::expand_clock_row`] instead of this method
    /// directly. `expand_clock_row` moves the `ClockSpec` out of
    /// `self.decorations` before calling `expand_row`, which avoids a borrow
    /// conflict (immutable borrow of `self.decorations` via the spec reference
    /// vs. mutable borrow of `self` via `expand_row`).
    pub(crate) fn expand_row(&mut self, spec: &ClockSpec, target_units: u32) {
        if target_units == 0 {
            return;
        }
        let mut filler = ClockFiller::start_from(self, spec);
        filler.seed_when_empty(target_units);
        filler.fill_until(target_units);
    }

    /// Convenience wrapper around [`Self::expand_row`] that pulls the
    /// `ClockSpec` directly out of `self.decorations`. Returns `false` when
    /// no clock is attached. Lets callers avoid cloning the spec just to
    /// satisfy the borrow checker (see parser/state.rs `expand_clock_signals`).
    ///
    /// `target_units` is the per-row target computed by the caller from the
    /// maximum explicit-row pixel width divided by this row's own step.
    pub(crate) fn expand_clock_row(&mut self, target_units: u32) -> bool {
        // Move the spec out so `expand_row` can borrow `self` mutably without
        // aliasing `self.decorations`. The spec is restored before return so
        // the row's external state is unchanged.
        let Some(spec) = self.decorations.clock.take() else {
            return false;
        };
        self.expand_row(&spec, target_units);
        self.decorations.clock = Some(spec);
        true
    }
}

/// Stateful helper that tracks the column count and "current level" while
/// [`SignalRow::expand_row`] appends alternating clock pulses to the row.
struct ClockFiller<'row> {
    row: &'row mut SignalRow,
    pulse: crate::clock::ClockPulse,
    start_level: SignalLevel,
    current: SignalLevel,
    produced: u32,
}

impl<'row> ClockFiller<'row> {
    fn start_from(row: &'row mut SignalRow, spec: &ClockSpec) -> Self {
        let start_level = match spec.start {
            ClockPhase::StartLow => SignalLevel::Low,
            ClockPhase::StartHigh => SignalLevel::High,
        };
        let produced = row.waveform.level_units_total();
        let current = row
            .waveform
            .iter()
            .rev()
            .find_map(|element| match element {
                WaveformElement::Level(run) => Some(run.level()),
                _ => None,
            })
            .unwrap_or(start_level);
        Self {
            row,
            pulse: spec.pulse,
            start_level,
            current,
            produced,
        }
    }

    /// Seed the waveform with the first level run when it is empty.
    ///
    /// `target_units` is passed so the initial run is clamped to the target
    /// (prevents overshooting on a very small target that is shorter than the
    /// first pulse unit count).  No-op when `produced != 0`.
    fn seed_when_empty(&mut self, target_units: u32) {
        if self.produced != 0 {
            return;
        }
        let initial = self
            .start_level
            .pulse_units_for(self.pulse)
            .min(target_units);
        if initial == 0 {
            return;
        }
        self.row.waveform.push_clock_run(self.start_level, initial);
        self.current = self.start_level;
        self.produced = self.produced.saturating_add(initial);
    }

    fn fill_until(&mut self, target_units: u32) {
        while self.produced < target_units {
            self.current = self.current.toggle();
            let unit_count = self
                .current
                .pulse_units_for(self.pulse)
                .min(target_units.saturating_sub(self.produced));
            if unit_count == 0 {
                break;
            }
            self.row.waveform.push_clock_run(self.current, unit_count);
            self.produced = self.produced.saturating_add(unit_count);
        }
    }
}

/// A `@skip` row.
///
/// See `docs/spec/types.md` §3.1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SkipRow {
    /// Vertical skip amount.
    pub(crate) amount: Length,
}

impl SkipRow {
    /// Construct a new [`SkipRow`].
    pub(crate) fn new(amount: Length) -> Self {
        Self { amount }
    }
}

/// A `@title` row.
///
/// See `docs/spec/types.md` §3.3.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TitleRow {
    /// Title text.
    pub(crate) text: UserText,
    /// Title style.
    pub(crate) style: TitleStyle,
}

impl TitleRow {
    /// Construct a new [`TitleRow`].
    pub(crate) fn new(text: UserText, style: TitleStyle) -> Self {
        Self { text, style }
    }

    /// Propagate a CLI/WASM `--font-size` override into the title style
    /// snapshot. Each title row holds its own `TitleStyle` snapshot captured
    /// at parse time.
    pub(crate) fn set_font_size(&mut self, size: Px) {
        self.style.set_font_size(size);
    }
}
