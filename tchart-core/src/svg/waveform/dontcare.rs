//! DontCare fill rectangle/polygon output (the inner horizontal line is handled by
//! [`super::level`] which pushes points into the polyline accumulators).
//!
//! Single-rail variants (`DontCareAlongLow/High/HiZ`) emit a `<rect>` spanning the full
//! `signal_box` height. The bus variant (`DontCareAlongBus`) emits a `<polygon>` whose
//! left and right edges track the transition boundaries of the surrounding waveform, per
//! `docs/spec/svg-rendering.md` §「`DontCareAlongBus` の塗り形状」.

use crate::line::SignalLevel;
use crate::svg::buf::{SvgBuf, WriteSvgOn};
use crate::svg::geometry::WaveformBoxY;
use crate::svg::waveform::dontcare_pattern::DontcareHatchPatternId;
use crate::units::Px;

/// `<rect>` for one `DontCareAlongLow/High/HiZ` run.
///
/// Constructed by [`super::state::RowState::push_level`] from the current
/// cursor / waveform-y snapshot before the matching `LevelDraw` advances the
/// cursor; serialized via the [`WriteSvgOn`] trait so the sub-buffer's public
/// surface remains a single `&mut self` write entry.
pub(super) struct DontCareRect {
    pub(super) x: Px,
    pub(super) width: Px,
    pub(super) waveform_y: WaveformBoxY,
    pub(super) pattern_id: DontcareHatchPatternId,
}

impl WriteSvgOn for DontCareRect {
    fn write_svg_on(&self, target: &mut SvgBuf) {
        let height = self.waveform_y.bottom - self.waveform_y.top;
        target.write_literal("<rect");
        target.write_px_attribute("x", self.x);
        target.write_px_attribute("y", self.waveform_y.top);
        target.write_px_attribute("width", self.width);
        target.write_px_attribute("height", height);
        write_dontcare_fill_url(target, self.pattern_id);
        target.write_literal("/>");
    }
}

/// Context describing the left and right edge shapes of a `DontCareAlongBus` polygon.
///
/// The corner x coordinates are derived from the surrounding transitions, per
/// `docs/spec/svg-rendering.md` §「`DontCareAlongBus` の塗り形状」.
///
/// When `left_cross_mid_x` or `right_cross_mid_x` is `Some`, the polygon gains an extra
/// vertex at `(cross_mid_x, y_mid)` — producing a 5-point (one cross) or 6-point
/// (both crosses) polygon.
///
/// All combinations of (prev × next) edges are covered explicitly via [`BusEdge`].
pub(super) struct DontCareBusContext {
    /// Left-top x coordinate (y = `waveform_y.top`).
    left_top_x: Px,
    /// Left-bottom x coordinate (y = `waveform_y.bottom`).
    left_bottom_x: Px,
    /// Right-top x coordinate (y = `waveform_y.top`).
    right_top_x: Px,
    /// Right-bottom x coordinate (y = `waveform_y.bottom`).
    right_bottom_x: Px,
    /// Cross midpoint x for the left edge: `Some` when preceded by `BusCross`.
    /// Polygon vertex at `(left_cross_mid_x, y_mid)`.
    left_cross_mid_x: Option<Px>,
    /// Cross midpoint x for the right edge: `Some` when followed by `BusCross`.
    /// Polygon vertex at `(right_cross_mid_x, y_mid)`.
    right_cross_mid_x: Option<Px>,
}

/// The slant-shape of one edge (left or right) of a `DontCareAlongBus` polygon.
///
/// Determined by the preceding or following waveform element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BusEdge {
    /// Bus continue: the edge is a vertical line.
    Vertical,
    /// Low↔Bus transition: the edge tilts so that the y_high corner is `slant` px ahead
    /// of the y_low corner (a `/` shape on the left, `\` on the right).
    SlantFromLow,
    /// High↔Bus transition: the edge tilts so that the y_high corner is `slant` px behind
    /// the y_low corner (a `\` shape on the left, `/` on the right).
    SlantFromHigh,
    /// BusCross (`X`) transition: the polygon gains a vertex at `(x_cross_start + slant/2, y_mid)`,
    /// which is the centre of the X crossing.
    CrossMidpoint,
    /// HiZ↔Bus transition: the polygon gains a vertex at `y_mid` (the single HiZ rail),
    /// with both top and bottom rails diverging from / converging to that one point.
    ///
    /// `bus_run_units` is the total count of Bus/DontCareAlongBus level-run units
    /// between the `BusOpen`/`BusClose` and the `DontCareAlongBus` body, used to
    /// compute the absolute x of the convergence/divergence point.
    SingleFromHiZ {
        /// Total Bus-level run units between BusOpen/BusClose and the DontCare body.
        bus_run_units: u32,
    },
}

impl BusEdge {
    /// Resolve the left-edge shape for a `DontCareAlongBus` at `index` in `elements`.
    ///
    /// Walks backwards through Bus/DontCareAlongBus `Level` runs until it finds the
    /// boundary transition. Returns `CrossMidpoint` for `BusCross`, a shape derived
    /// from the source level for `BusOpen`, and `Vertical` for end-of-waveform or any
    /// other element. Counts Bus-run units between the `BusOpen` and the DontCare body
    /// so `SingleFromHiZ` can place its convergence point correctly.
    pub(super) fn from_prev(elements: &[crate::line::WaveformElement], index: usize) -> Self {
        use crate::line::{Transition, TransitionKind, WaveformElement};
        let mut scan = index;
        let mut bus_run_units = 0u32;
        while scan > 0 {
            scan -= 1;
            match &elements[scan] {
                WaveformElement::Level(run) if run.level().is_bus_family() => {
                    bus_run_units = bus_run_units.saturating_add(run.units());
                }
                WaveformElement::Transition(Transition {
                    kind: TransitionKind::BusOpen,
                    source,
                    ..
                }) => return Self::from_source_level(*source, bus_run_units),
                WaveformElement::Transition(Transition {
                    kind: TransitionKind::BusCross,
                    ..
                }) => return Self::CrossMidpoint,
                _ => return Self::Vertical,
            }
        }
        Self::Vertical
    }

    /// Resolve the right-edge shape for a `DontCareAlongBus` at `index` in `elements`.
    ///
    /// Walks forwards through Bus/DontCareAlongBus `Level` runs until it finds the
    /// boundary transition. Returns `CrossMidpoint` for `BusCross`, a shape derived
    /// from the target level for `BusClose`, and `Vertical` for end-of-waveform or any
    /// other element. Counts Bus-run units between the DontCare body and `BusClose`
    /// so `SingleFromHiZ` can place its convergence point correctly.
    pub(super) fn from_next(elements: &[crate::line::WaveformElement], index: usize) -> Self {
        use crate::line::{Transition, TransitionKind, WaveformElement};
        let mut scan = index;
        let mut bus_run_units = 0u32;
        while scan + 1 < elements.len() {
            scan += 1;
            match &elements[scan] {
                WaveformElement::Level(run) if run.level().is_bus_family() => {
                    bus_run_units = bus_run_units.saturating_add(run.units());
                }
                WaveformElement::Transition(Transition {
                    kind: TransitionKind::BusClose,
                    target,
                    ..
                }) => return Self::from_source_level(*target, bus_run_units),
                WaveformElement::Transition(Transition {
                    kind: TransitionKind::BusCross,
                    ..
                }) => return Self::CrossMidpoint,
                _ => return Self::Vertical,
            }
        }
        Self::Vertical
    }

    /// Classify a source/target level as an edge shape.
    ///
    /// Low → `SlantFromLow`, High → `SlantFromHigh`,
    /// HiZ → `SingleFromHiZ` (y_mid convergence with `bus_run_units`),
    /// Bus-family → `Vertical`.
    fn from_source_level(level: SignalLevel, bus_run_units: u32) -> Self {
        match level {
            SignalLevel::Low => Self::SlantFromLow,
            SignalLevel::High => Self::SlantFromHigh,
            SignalLevel::HiZ => Self::SingleFromHiZ { bus_run_units },
            _ => Self::Vertical,
        }
    }
}

impl DontCareBusContext {
    /// Compute the polygon x-coordinates from the edge shapes and dimensions.
    ///
    /// `x_start` is the cursor at the beginning of the `?` run; `x_end = x_start + width`.
    /// `slant` is the single-edge transition width; `step` is one Bus-level unit width
    /// (used to locate the `BusOpen`/`BusClose` start when the source/target is HiZ).
    pub(super) fn compute(
        x_start: Px,
        width: Px,
        slant: Px,
        step: Px,
        prev_edge: BusEdge,
        next_edge: BusEdge,
    ) -> Self {
        let x_end = x_start + width;
        let (left_top_x, left_bottom_x, left_cross_mid_x) =
            Self::left_corners(x_start, slant, step, prev_edge);
        let (right_top_x, right_bottom_x, right_cross_mid_x) =
            Self::right_corners(x_end, slant, step, next_edge);
        Self {
            left_top_x,
            left_bottom_x,
            right_top_x,
            right_bottom_x,
            left_cross_mid_x,
            right_cross_mid_x,
        }
    }

    /// Compute left-edge corner x coordinates and optional midpoint from `prev_edge`.
    ///
    /// Returns `(top_x, bottom_x, midpoint_x)`.
    fn left_corners(x_start: Px, slant: Px, step: Px, edge: BusEdge) -> (Px, Px, Option<Px>) {
        match edge {
            BusEdge::Vertical => (x_start, x_start, None),
            BusEdge::SlantFromLow => (x_start, x_start - slant, None),
            BusEdge::SlantFromHigh => (x_start - slant, x_start, None),
            BusEdge::CrossMidpoint => (x_start, x_start, Some(x_start - slant * 0.5)),
            BusEdge::SingleFromHiZ { bus_run_units } => {
                let hiz_x = x_start - step * (bus_run_units as f32) - slant;
                (x_start, x_start, Some(hiz_x))
            }
        }
    }

    /// Compute right-edge corner x coordinates and optional midpoint from `next_edge`.
    ///
    /// Returns `(top_x, bottom_x, midpoint_x)`.
    fn right_corners(x_end: Px, slant: Px, step: Px, edge: BusEdge) -> (Px, Px, Option<Px>) {
        match edge {
            BusEdge::Vertical => (x_end, x_end, None),
            BusEdge::SlantFromLow => (x_end, x_end + slant, None),
            BusEdge::SlantFromHigh => (x_end + slant, x_end, None),
            BusEdge::CrossMidpoint => (x_end, x_end, Some(x_end + slant * 0.5)),
            BusEdge::SingleFromHiZ { bus_run_units } => {
                let hiz_x = x_end + step * (bus_run_units as f32) + slant;
                (x_end, x_end, Some(hiz_x))
            }
        }
    }
}

/// `<polygon>` for one `DontCareAlongBus` run.
///
/// Emits a 4-to-6-point polygon whose left/right edges track the surrounding transition
/// boundaries, per `docs/spec/svg-rendering.md` §「`DontCareAlongBus` の塗り形状」.
/// The polygon is always bounded by `y_top`/`y_bottom` (the `signal_box` height) and
/// never overflows vertically.
pub(super) struct DontCareBusPolygon {
    pub(super) context: DontCareBusContext,
    pub(super) waveform_y: WaveformBoxY,
    pub(super) pattern_id: DontcareHatchPatternId,
}

impl WriteSvgOn for DontCareBusPolygon {
    fn write_svg_on(&self, target: &mut SvgBuf) {
        let top = self.waveform_y.top;
        let bottom = self.waveform_y.bottom;
        let mid = self.waveform_y.middle;
        let context = &self.context;
        // Build clockwise point list; cross midpoints expand the polygon to 5 or 6 vertices.
        // The first vertex written (left midpoint when present, otherwise left_top) must
        // use first=true so the points attribute does not start with a leading space.
        target.write_literal("<polygon points=\"");
        if let Some(x) = context.left_cross_mid_x {
            write_sep_point(target, x, mid, true);
            write_sep_point(target, context.left_top_x, top, false);
        } else {
            write_sep_point(target, context.left_top_x, top, true);
        }
        write_sep_point(target, context.right_top_x, top, false);
        if let Some(x) = context.right_cross_mid_x {
            write_sep_point(target, x, mid, false);
        }
        write_sep_point(target, context.right_bottom_x, bottom, false);
        write_sep_point(target, context.left_bottom_x, bottom, false);
        target.write_literal("\"");
        write_dontcare_fill_url(target, self.pattern_id);
        target.write_literal("/>");
    }
}

/// Emit ` fill="url(#dontcare-hatch-N)"` for the given pattern id.
fn write_dontcare_fill_url(target: &mut SvgBuf, pattern_id: DontcareHatchPatternId) {
    target.write_literal(" fill=\"url(#");
    target.write_dontcare_id(pattern_id);
    target.write_literal(")\"");
}

/// Write one `x,y` point, optionally omitting the leading space separator.
///
/// `first` suppresses the space so the first point in a `<polygon points="…">`
/// does not start with a space.
fn write_sep_point(target: &mut SvgBuf, x: Px, y: Px, first: bool) {
    if !first {
        target.write_char(' ');
    }
    target.write_px(x);
    target.write_char(',');
    target.write_px(y);
}
