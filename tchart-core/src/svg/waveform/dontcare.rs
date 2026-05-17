//! DontCare fill rectangle/polygon output.
//!
//! Every `?` region emits one `<rect>` (when the polygon is a pure rectangle —
//! DC-HiZ always; DC-Low/High/Bus when both adjacent boundaries are vertical)
//! or one `<polygon>` whose left and right edges track the surrounding
//! transition boundaries, per `docs/spec/svg-rendering.md` §「`DontCareAlongLow`
//! / `DontCareAlongHigh` / `DontCareAlongBus` の塗り形状」.
//!
//! The polygon's vertical extent is always `y_high..y_low` (the `signal_box`
//! interior). Adjacent half-slants (HiZ-involved) and BusCross contribute
//! `y_mid` intermediate vertices.

use crate::line::{SignalLevel, Transition, TransitionKind, WaveformElement};
use crate::svg::buf::{SvgBuf, WriteSvgOn};
use crate::svg::geometry::WaveformBoxY;
use crate::svg::waveform::dontcare_pattern::DontcareHatchPatternId;
use crate::units::Px;

/// Kind of single-rail DontCare variant. Drives polygon vs rectangle choice
/// and which y the internal horizontal line is drawn at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DcSingleKind {
    /// DC-Low: internal line at y_l. Polygon edges follow adjacent slants.
    Low,
    /// DC-High: internal line at y_h. Polygon edges follow adjacent slants.
    High,
    /// DC-HiZ: internal dashed line at y_mid. Always a rectangle (cell-grid
    /// span). Adjacent half-slants remain in the waveform polylines.
    HiZ,
}

impl DcSingleKind {
    /// Classify a `DontCareAlong*` (single-rail) level. Returns `None` for
    /// `DontCareAlongBus`, which uses a separate polygon builder.
    pub(super) fn from_level(level: SignalLevel) -> Option<Self> {
        match level {
            SignalLevel::DontCareAlongLow => Some(Self::Low),
            SignalLevel::DontCareAlongHigh => Some(Self::High),
            SignalLevel::DontCareAlongHiZ => Some(Self::HiZ),
            _ => None,
        }
    }
}

/// Adjacency kind for a single-rail DontCare boundary (left or right).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SingleDcAdjacency {
    /// Signal start/end, Gap, or same-level continue: vertical polygon edge.
    Vertical,
    /// SingleEdge / BusOpen / BusClose where the adjacent single-rail level is
    /// Low or High (the full y_h↔y_l slant). The slant direction is fixed by
    /// the DC variant.
    FullSlant,
    /// SingleEdge or BusOpen/BusClose where the adjacent single-rail level is
    /// HiZ (the slant goes between y_mid and the DC's internal line y).
    /// Adds a `y_mid` intermediate vertex to the polygon.
    HalfSlant,
}

impl SingleDcAdjacency {
    /// Resolve the LEFT adjacency for a DC-Low/High/HiZ at `index` in `elements`.
    fn from_prev(elements: &[WaveformElement], index: usize) -> Self {
        let mut scan = index;
        while scan > 0 {
            scan -= 1;
            match &elements[scan] {
                WaveformElement::Transition(transition) => {
                    return Self::from_transition(transition, /* take_target */ false);
                }
                WaveformElement::Anchor(_)
                | WaveformElement::Text(_)
                | WaveformElement::HighlightStart
                | WaveformElement::HighlightEnd
                | WaveformElement::Guide => continue,
                _ => return Self::Vertical,
            }
        }
        Self::Vertical
    }

    /// Resolve the RIGHT adjacency for a DC-Low/High/HiZ at `index` in `elements`.
    fn from_next(elements: &[WaveformElement], index: usize) -> Self {
        let mut scan = index;
        while scan + 1 < elements.len() {
            scan += 1;
            match &elements[scan] {
                WaveformElement::Transition(transition) => {
                    return Self::from_transition(transition, /* take_target */ true);
                }
                WaveformElement::Anchor(_)
                | WaveformElement::Text(_)
                | WaveformElement::HighlightStart
                | WaveformElement::HighlightEnd
                | WaveformElement::Guide => continue,
                _ => return Self::Vertical,
            }
        }
        Self::Vertical
    }

    /// Pick the adjacency kind from a transition that bounds the DC on one
    /// side. When `take_target` is true the DC sits BEFORE the transition (so
    /// the transition's `target` is the outgoing single level); when false the
    /// DC sits AFTER (the transition's `source` is the incoming single level).
    fn from_transition(transition: &Transition, take_target: bool) -> Self {
        let other = if take_target {
            transition.target
        } else {
            transition.source
        };
        match transition.kind {
            TransitionKind::SingleEdge | TransitionKind::BusOpen | TransitionKind::BusClose => {
                if other == SignalLevel::HiZ {
                    Self::HalfSlant
                } else {
                    Self::FullSlant
                }
            }
            TransitionKind::BusCross => Self::Vertical,
        }
    }
}

/// Pre-computed polygon vertex list (CW) plus pattern id, for any DC variant.
///
/// Constructed by [`super::state::RowState::push_level`] and serialised via the
/// [`WriteSvgOn`] trait. Holds up to 6 vertices.
pub(super) struct DontCarePolygon {
    vertices: HeaplessVec,
    pattern_id: DontcareHatchPatternId,
}

/// Bundled inputs for building a DontCare polygon.
///
/// Constructed at the call site in [`super::mod::RowContext::build_dontcare_polygon`]
/// and threaded into the polygon constructors as one parameter so the
/// individual `for_single`/`for_bus` entry points fit within the argument
/// budget. Fields are private; callers go through `new()`.
pub(super) struct DontCarePolygonArgs<'elements> {
    elements: &'elements [WaveformElement],
    index: usize,
    cursor: Px,
    width: Px,
    slant: Px,
    step: Px,
    waveform_y: WaveformBoxY,
    pattern_id: DontcareHatchPatternId,
}

impl<'elements> DontCarePolygonArgs<'elements> {
    /// Build the inputs for one DontCare polygon at `index` in `elements`.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        elements: &'elements [WaveformElement],
        index: usize,
        cursor: Px,
        width: Px,
        slant: Px,
        step: Px,
        waveform_y: WaveformBoxY,
        pattern_id: DontcareHatchPatternId,
    ) -> Self {
        Self {
            elements,
            index,
            cursor,
            width,
            slant,
            step,
            waveform_y,
            pattern_id,
        }
    }
}

impl DontCarePolygon {
    /// Polygon for a single-rail (`DontCareAlongLow/High/HiZ`) run.
    ///
    /// DC-HiZ is always a rectangle spanning the full cell-grid (extends to
    /// `cursor - slant` when preceded). DC-Low/High follow adjacent slants.
    /// The `step` field of `args` is ignored for single-rail DC.
    pub(super) fn for_single(kind: DcSingleKind, args: DontCarePolygonArgs<'_>) -> Self {
        let prev = SingleDcAdjacency::from_prev(args.elements, args.index);
        let next = SingleDcAdjacency::from_next(args.elements, args.index);
        let geometry = SingleDcGeometry {
            cursor: args.cursor,
            width: args.width,
            slant: args.slant,
            waveform_y: args.waveform_y,
        };
        let vertices = match kind {
            DcSingleKind::Low => geometry.build_dc_low(prev, next),
            DcSingleKind::High => geometry.build_dc_high(prev, next),
            DcSingleKind::HiZ => geometry.build_dc_hiz(prev, next),
        };
        Self {
            vertices,
            pattern_id: args.pattern_id,
        }
    }

    /// Polygon for a `DontCareAlongBus` run.
    ///
    /// Reuses the legacy [`BusEdge`] classification (BusOpen/BusClose/BusCross
    /// boundaries on bus rails).
    pub(super) fn for_bus(args: DontCarePolygonArgs<'_>) -> Self {
        let prev_edge = BusEdge::from_prev(args.elements, args.index);
        let next_edge = BusEdge::from_next(args.elements, args.index);
        let context = DontCareBusContext::from_edges(
            args.cursor,
            args.width,
            args.slant,
            args.step,
            prev_edge,
            next_edge,
        );
        let vertices = context.build_polygon(args.waveform_y);
        Self {
            vertices,
            pattern_id: args.pattern_id,
        }
    }
}

impl WriteSvgOn for DontCarePolygon {
    fn write_svg_on(&self, target: &mut SvgBuf) {
        target.write_literal("<polygon points=\"");
        let mut first = true;
        for (x, y) in self.vertices.iter() {
            if !first {
                target.write_char(' ');
            }
            target.write_px(x);
            target.write_char(',');
            target.write_px(y);
            first = false;
        }
        target.write_literal("\"");
        write_dontcare_fill_url(target, self.pattern_id);
        target.write_literal("/>");
    }
}

/// Inputs needed to build a single-rail DC polygon's vertex list.
struct SingleDcGeometry {
    cursor: Px,
    width: Px,
    slant: Px,
    waveform_y: WaveformBoxY,
}

/// Pre-resolved x positions for one side (left or right) of a single-rail DC
/// polygon. Sides that produce a y_mid intermediate vertex include `mid_x`.
struct SideCorners {
    top_x: Px,
    bottom_x: Px,
    mid_x: Option<Px>,
}

impl SideCorners {
    /// Both corners at the same x (vertical edge, no y_mid vertex).
    fn vertical(x: Px) -> Self {
        Self {
            top_x: x,
            bottom_x: x,
            mid_x: None,
        }
    }

    /// Two corners at different x's (full slant, no y_mid vertex).
    fn two_corner(top_x: Px, bottom_x: Px) -> Self {
        Self {
            top_x,
            bottom_x,
            mid_x: None,
        }
    }

    /// Three vertices on this side: top, bottom, plus a y_mid intermediate.
    fn with_mid(top_x: Px, bottom_x: Px, mid_x: Px) -> Self {
        Self {
            top_x,
            bottom_x,
            mid_x: Some(mid_x),
        }
    }
}

impl SingleDcGeometry {
    /// DC-Low LEFT corners: top at grid_start (outer), bot at body_start (inner).
    fn dc_low_left(&self, prev: SingleDcAdjacency) -> SideCorners {
        match prev {
            SingleDcAdjacency::Vertical => SideCorners::vertical(self.cursor),
            SingleDcAdjacency::FullSlant => {
                SideCorners::two_corner(self.cursor - self.slant, self.cursor)
            }
            SingleDcAdjacency::HalfSlant => SideCorners::with_mid(
                self.cursor - self.slant,
                self.cursor,
                self.cursor - self.slant,
            ),
        }
    }

    /// DC-Low RIGHT corners: top at grid_end+s (outer), bot at body_end (inner).
    fn dc_low_right(&self, next: SingleDcAdjacency) -> SideCorners {
        let body_end = self.cursor + self.width;
        match next {
            SingleDcAdjacency::Vertical => SideCorners::vertical(body_end),
            SingleDcAdjacency::FullSlant => {
                SideCorners::two_corner(body_end + self.slant, body_end)
            }
            SingleDcAdjacency::HalfSlant => {
                SideCorners::with_mid(body_end + self.slant, body_end, body_end + self.slant)
            }
        }
    }

    /// DC-High LEFT corners: top at body_start (inner), bot at grid_start (outer).
    /// Half-slant goes y_mid → y_h, so both BL and MID sit at the outer x and
    /// the polygon's left edge below y_mid is vertical.
    fn dc_high_left(&self, prev: SingleDcAdjacency) -> SideCorners {
        match prev {
            SingleDcAdjacency::Vertical => SideCorners::vertical(self.cursor),
            SingleDcAdjacency::FullSlant => {
                SideCorners::two_corner(self.cursor, self.cursor - self.slant)
            }
            SingleDcAdjacency::HalfSlant => SideCorners::with_mid(
                self.cursor,
                self.cursor - self.slant,
                self.cursor - self.slant,
            ),
        }
    }

    /// DC-High RIGHT corners: top at body_end (inner), bot at body_end+s (outer).
    /// Half-slant goes y_h → y_mid only, so both TR and BR stay at inner
    /// body_end and the polygon has a y_mid horn poking out to body_end+s.
    fn dc_high_right(&self, next: SingleDcAdjacency) -> SideCorners {
        let body_end = self.cursor + self.width;
        match next {
            SingleDcAdjacency::Vertical => SideCorners::vertical(body_end),
            SingleDcAdjacency::FullSlant => {
                SideCorners::two_corner(body_end, body_end + self.slant)
            }
            SingleDcAdjacency::HalfSlant => {
                SideCorners::with_mid(body_end, body_end, body_end + self.slant)
            }
        }
    }

    /// Build vertex list for DC-Low.
    fn build_dc_low(&self, prev: SingleDcAdjacency, next: SingleDcAdjacency) -> HeaplessVec {
        self.build_single_polygon(self.dc_low_left(prev), self.dc_low_right(next))
    }

    /// Build vertex list for DC-High (mirror of DC-Low corner placement).
    fn build_dc_high(&self, prev: SingleDcAdjacency, next: SingleDcAdjacency) -> HeaplessVec {
        self.build_single_polygon(self.dc_high_left(prev), self.dc_high_right(next))
    }

    /// Assemble the CW vertex list for any single-rail DC polygon.
    ///
    /// Order: TL, TR, [right MID], BR, BL, [left MID]. Mids appear only when
    /// the adjacent transition is HalfSlant (HiZ-involved).
    fn build_single_polygon(&self, left: SideCorners, right: SideCorners) -> HeaplessVec {
        let y_h = self.waveform_y.top;
        let y_l = self.waveform_y.bottom;
        let y_mid = self.waveform_y.middle;
        let mut vertices = HeaplessVec::default();
        vertices.push(left.top_x, y_h);
        vertices.push(right.top_x, y_h);
        if let Some(x) = right.mid_x {
            vertices.push(x, y_mid);
        }
        vertices.push(right.bottom_x, y_l);
        vertices.push(left.bottom_x, y_l);
        if let Some(x) = left.mid_x {
            vertices.push(x, y_mid);
        }
        vertices
    }

    /// Build vertex list for DC-HiZ — always a cell-grid-spanning rectangle
    /// (no wedges, no mid vertices). When the DC is preceded by a transition,
    /// the polygon's left edge reaches back into the slant area (which
    /// occupies the first `slant` pixels of the DC-HiZ cell). When followed
    /// by a transition the right edge stays at `body_end` because the
    /// following slant belongs to the NEXT cell's grid range.
    fn build_dc_hiz(&self, prev: SingleDcAdjacency, _next: SingleDcAdjacency) -> HeaplessVec {
        let body_end = self.cursor + self.width;
        let y_h = self.waveform_y.top;
        let y_l = self.waveform_y.bottom;
        let x_left = if prev == SingleDcAdjacency::Vertical {
            self.cursor
        } else {
            self.cursor - self.slant
        };
        let x_right = body_end;
        let mut vertices = HeaplessVec::default();
        vertices.push(x_left, y_h);
        vertices.push(x_right, y_h);
        vertices.push(x_right, y_l);
        vertices.push(x_left, y_l);
        vertices
    }
}

/// Edge shape for one side of a `DontCareAlongBus` polygon.
///
/// Determined by the preceding or following waveform element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BusEdge {
    /// Bus continue: the edge is a vertical line.
    Vertical,
    /// Low↔Bus transition: top-y corner is `slant` px ahead of bottom-y corner.
    SlantFromLow,
    /// High↔Bus transition: top-y corner is `slant` px behind bottom-y corner.
    SlantFromHigh,
    /// BusCross (`X`): polygon gains a wedge vertex at `(cross_start + slant/2,
    /// y_mid)`.
    CrossMidpoint,
    /// HiZ↔Bus transition: polygon gains a `y_mid` wedge vertex at the HiZ
    /// rail's x (both top and bottom rails diverge from / converge to that
    /// point).
    SingleFromHiZ {
        /// Total Bus-level run units between BusOpen/BusClose and the DC body.
        bus_run_units: u32,
    },
}

impl BusEdge {
    /// Resolve the left-edge shape for a `DontCareAlongBus` at `index`.
    pub(super) fn from_prev(elements: &[WaveformElement], index: usize) -> Self {
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

    /// Resolve the right-edge shape for a `DontCareAlongBus` at `index`.
    pub(super) fn from_next(elements: &[WaveformElement], index: usize) -> Self {
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

    fn from_source_level(level: SignalLevel, bus_run_units: u32) -> Self {
        match level {
            SignalLevel::Low => Self::SlantFromLow,
            SignalLevel::High => Self::SlantFromHigh,
            SignalLevel::HiZ => Self::SingleFromHiZ { bus_run_units },
            _ => Self::Vertical,
        }
    }
}

/// Bus DontCare polygon context (private to this module).
struct DontCareBusContext {
    left_top_x: Px,
    left_bottom_x: Px,
    right_top_x: Px,
    right_bottom_x: Px,
    left_cross_mid_x: Option<Px>,
    right_cross_mid_x: Option<Px>,
}

impl DontCareBusContext {
    fn from_edges(
        x_start: Px,
        width: Px,
        slant: Px,
        step: Px,
        prev_edge: BusEdge,
        next_edge: BusEdge,
    ) -> Self {
        let x_end = x_start + width;
        let (left_top_x, left_bottom_x, left_cross_mid_x) =
            Self::calc_left_corners(x_start, slant, step, prev_edge);
        let (right_top_x, right_bottom_x, right_cross_mid_x) =
            Self::calc_right_corners(x_end, slant, step, next_edge);
        Self {
            left_top_x,
            left_bottom_x,
            right_top_x,
            right_bottom_x,
            left_cross_mid_x,
            right_cross_mid_x,
        }
    }

    fn calc_left_corners(x_start: Px, slant: Px, step: Px, edge: BusEdge) -> (Px, Px, Option<Px>) {
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

    fn calc_right_corners(x_end: Px, slant: Px, step: Px, edge: BusEdge) -> (Px, Px, Option<Px>) {
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

    /// Emit the CW polygon vertex list, with optional y_mid wedges on each side.
    fn build_polygon(&self, waveform_y: WaveformBoxY) -> HeaplessVec {
        let top = waveform_y.top;
        let bottom = waveform_y.bottom;
        let mid = waveform_y.middle;
        let mut vertices = HeaplessVec::default();
        // Bus-side wedges sit BEFORE TL (left) / between TR and BR (right) /
        // AFTER BL (closing back) — i.e. they stick OUTWARD.
        if let Some(x) = self.left_cross_mid_x {
            vertices.push(x, mid);
        }
        vertices.push(self.left_top_x, top);
        vertices.push(self.right_top_x, top);
        if let Some(x) = self.right_cross_mid_x {
            vertices.push(x, mid);
        }
        vertices.push(self.right_bottom_x, bottom);
        vertices.push(self.left_bottom_x, bottom);
        vertices
    }
}

/// Emit ` fill="url(#dontcare-hatch-N)"` for the given pattern id.
fn write_dontcare_fill_url(target: &mut SvgBuf, pattern_id: DontcareHatchPatternId) {
    target.write_literal(" fill=\"url(#");
    target.write_dontcare_id(pattern_id);
    target.write_literal(")\"");
}

/// Max number of vertices a DC polygon can have (6: TL, TR, RightMid, BR,
/// BL, LeftMid). Statically bounded by the geometry: single-rail DC has at
/// most TL, TR, RightMid, BR, BL, LeftMid (6); bus DC has at most LeftMid,
/// TL, TR, RightMid, BR, BL (6).
const HEAPLESS_VEC_CAPACITY: usize = 6;

/// Inline fixed-capacity vertex list used by the polygon builders. Avoids
/// pulling in a `Vec` allocation for what is at most 6 vertices.
pub(super) struct HeaplessVec {
    data: [(Px, Px); HEAPLESS_VEC_CAPACITY],
    length: usize,
}

impl Default for HeaplessVec {
    fn default() -> Self {
        Self {
            data: [(Px::ZERO, Px::ZERO); HEAPLESS_VEC_CAPACITY],
            length: 0,
        }
    }
}

impl HeaplessVec {
    pub(super) fn push(&mut self, x: Px, y: Px) {
        // The polygon builders in this file push at most 6 vertices: see
        // `build_single_polygon` (TL, TR, optional RightMid, BR, BL, optional
        // LeftMid = up to 6), `build_dc_hiz` (4 fixed), and
        // `DontCareBusContext::build_polygon` (optional LeftMid, TL, TR,
        // optional RightMid, BR, BL = up to 6). `debug_assert!` catches a
        // future bug in those builders during tests; release builds rely on
        // the static guarantee.
        debug_assert!(
            self.length < HEAPLESS_VEC_CAPACITY,
            "DontCare polygon exceeded {HEAPLESS_VEC_CAPACITY} vertices"
        );
        self.data[self.length] = (x, y);
        self.length += 1;
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = (Px, Px)> + '_ {
        self.data[..self.length].iter().copied()
    }
}
