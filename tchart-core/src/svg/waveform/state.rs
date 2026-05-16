//! Bookkeeping state used while iterating waveform elements for one row.

use crate::line::{LevelRun, SignalLevel};
use crate::svg::buf::SvgBuf;
use crate::svg::geometry::WaveformBoxY;
use crate::svg::waveform::dontcare::{
    BusEdge, DontCareBusContext, DontCareBusPolygon, DontCareRect,
};
use crate::svg::waveform::dontcare_pattern::DontcareHatchPatternId;
use crate::svg::waveform::level::LevelDraw;
use crate::svg::waveform::poly::PolyAccum;
use crate::units::Px;

/// Dasharray literal used for HiZ runs and `DontCareAlongHiZ` inner lines.
const HIZ_DASH: &str = "4 2";

/// Arguments for the `DontCareAlongBus` polygon shape, bundled to keep
/// [`RowState::push_level`] within the project argument-count limit.
///
/// `prev_edge`/`next_edge` are resolved by the caller by scanning the waveform
/// element slice; `slant` is the single-edge transition width; `step` is one
/// Bus-unit width used to locate HiZ convergence/divergence points.
pub(super) struct BusPolygonArgs {
    /// Left-edge shape: resolved by scanning backwards for `BusOpen`.
    pub(super) prev_edge: BusEdge,
    /// Right-edge shape: resolved by scanning forwards for `BusClose`.
    pub(super) next_edge: BusEdge,
    /// Single-edge transition slant width.
    pub(super) slant: Px,
    /// One Bus-level step width, for HiZ y_mid convergence placement.
    pub(super) step: Px,
}

/// Trait implemented by waveform pieces that mutate `RowState`.
///
/// The operation lives on `&self` of the source piece (because `RowState`
/// cannot know every kind of piece that mutates it), and `RowState` exposes a
/// single `draw` method as the public entry point. Borrow conflicts vanish
/// naturally because `self` and `target` are independent borrows; per-row
/// scratch values (`waveform_y`, `cursor`) are snapshotted into locals inside
/// `draw_on` before any mutation.
pub(super) trait DrawOn {
    fn draw_on(&self, target: &mut RowState);
}

/// Per-row rendering context tracking polyline accumulators and cursor x.
#[derive(Debug)]
pub(super) struct RowState {
    /// Top-rail / single-line accumulator.
    top: PolyAccum,
    /// Bottom-rail accumulator (Bus, BusOpen/Close, BusCross).
    bottom: PolyAccum,
    /// HiZ accumulator (kept separate because of dasharray style).
    hiz: PolyAccum,
    /// Current x cursor in chart coordinates.
    cursor: Px,
    /// Cached per-row waveform-box y values.
    waveform_y: WaveformBoxY,
    /// X coordinate where the current highlight region began, or `None`.
    highlight_start: Option<Px>,
}

impl RowState {
    /// Build a fresh state at `start_x`.
    pub(super) fn new(start_x: Px, waveform_y: WaveformBoxY) -> Self {
        Self {
            top: PolyAccum::new(),
            bottom: PolyAccum::new(),
            hiz: PolyAccum::new(),
            cursor: start_x,
            waveform_y,
            highlight_start: None,
        }
    }

    /// Apply a `DrawOn` source to this state.
    pub(super) fn draw<T: DrawOn>(&mut self, source: &T) {
        source.draw_on(self);
    }

    pub(super) fn cursor(&self) -> Px {
        self.cursor
    }

    pub(super) fn waveform_y(&self) -> WaveformBoxY {
        self.waveform_y
    }

    /// Flush every accumulator to `buf`.
    pub(super) fn flush_all(&mut self, buf: &mut SvgBuf) {
        self.top.flush(buf, None);
        self.bottom.flush(buf, None);
        self.hiz.flush(buf, Some(HIZ_DASH));
    }

    /// Gap: flush polylines and advance the cursor.
    pub(super) fn handle_gap(&mut self, width: Px, buf: &mut SvgBuf) {
        self.flush_all(buf);
        self.cursor = self.cursor + width;
    }

    /// Process one `LevelRun`: emit the DontCare backing shape into `rects`
    /// (when applicable) and push the level's polyline points.
    ///
    /// For `DontCareAlongBus`, a `<polygon>` is emitted using the pre-computed
    /// `bus_context`; all other DontCare variants emit a `<rect>` spanning the
    /// full `signal_box` height. `bus_context` is ignored for non-Bus DontCare.
    ///
    /// `&mut SvgBuf` is the single secondary `&mut` argument alongside
    /// `&mut self`; it is not threaded any deeper.
    pub(super) fn push_level(
        &mut self,
        run: &LevelRun,
        width: Px,
        pattern_id: Option<DontcareHatchPatternId>,
        bus_context: &BusPolygonArgs,
        rects: &mut SvgBuf,
    ) {
        if run.level() == SignalLevel::DontCareAlongBus {
            let pattern_id = pattern_id.expect("DontCareAlongBus must have a pattern id");
            let context = DontCareBusContext::compute(
                self.cursor,
                width,
                bus_context.slant,
                bus_context.step,
                bus_context.prev_edge,
                bus_context.next_edge,
            );
            rects.write(&DontCareBusPolygon {
                context,
                waveform_y: self.waveform_y,
                pattern_id,
            });
        } else if run.level().is_dontcare() {
            let pattern_id = pattern_id.expect("DontCare run must have a pattern id");
            rects.write(&DontCareRect {
                x: self.cursor,
                width,
                waveform_y: self.waveform_y,
                pattern_id,
            });
        }
        self.draw(&LevelDraw {
            width,
            level: run.level(),
        });
    }

    /// Advance the cursor by `advance_by` and return the new cursor position.
    pub(super) fn advance(&mut self, advance_by: Px) -> Px {
        self.cursor = self.cursor + advance_by;
        self.cursor
    }

    /// Push a point onto the top-rail / single-line polyline.
    pub(super) fn push_top(&mut self, x: Px, y: Px) {
        self.top.push(x, y);
    }

    /// Push a point onto the bottom-rail polyline.
    pub(super) fn push_bottom(&mut self, x: Px, y: Px) {
        self.bottom.push(x, y);
    }

    /// Push a point onto the HiZ polyline.
    pub(super) fn push_hiz(&mut self, x: Px, y: Px) {
        self.hiz.push(x, y);
    }

    /// Swap top and bottom accumulators (used by `BusCross`).
    pub(super) fn swap_top_and_bottom(&mut self) {
        std::mem::swap(&mut self.top, &mut self.bottom);
    }

    /// Mark the start of a highlight region at the current cursor.
    pub(super) fn begin_highlight(&mut self) {
        self.highlight_start = Some(self.cursor);
    }

    /// Take and clear the highlight start position, returning it if set.
    pub(super) fn take_highlight_start(&mut self) -> Option<Px> {
        self.highlight_start.take()
    }
}
