//! Transition rendering. Each `TransitionKind` is matched exhaustively
//! (no `_ =>`), and shared horizontal edges are drawn explicitly.
//!
//! See `docs/spec/svg-rendering.md` "Transition 描画契約" and
//! `docs/spec/types.md` §11.3.

use crate::line::{SignalLevel, Transition, TransitionKind};
use crate::svg::geometry::WaveformBoxY;
use crate::svg::waveform::state::{DrawOn, RowState};
use crate::units::Px;

/// Owned snapshot of a `Transition` plus its width, used as the `DrawOn` source.
///
/// Constructed by `RowOutput::process_element` and immediately handed to
/// `RowState::draw` — there is no separate `render_transition` wrapper since
/// the construction itself fully describes the operation.
pub(super) struct TransitionDraw {
    pub(super) width: Px,
    /// Width of the cross region inside `BusCross` (the slant width).
    pub(super) slant_width: Px,
    pub(super) kind: TransitionKind,
    pub(super) source: SignalLevel,
    pub(super) target: SignalLevel,
}

impl TransitionDraw {
    /// Construct from a parsed [`Transition`], its rendered width, and the
    /// cross-region slant width for `BusCross` rendering.
    pub(super) fn new(transition: &Transition, width: Px, slant_width: Px) -> Self {
        Self {
            width,
            slant_width,
            kind: transition.kind,
            source: transition.source,
            target: transition.target,
        }
    }
}

impl DrawOn for TransitionDraw {
    fn draw_on(&self, target_state: &mut RowState) {
        let waveform_y = target_state.waveform_y();
        let start_x = target_state.cursor();
        let end_x = target_state.advance(self.width);
        match self.kind {
            TransitionKind::SingleEdge => {
                self.draw_single_edge(target_state, waveform_y, start_x, end_x);
            }
            TransitionKind::BusOpen => {
                self.draw_bus_open(target_state, waveform_y, start_x, end_x);
            }
            TransitionKind::BusClose => {
                self.draw_bus_close(target_state, waveform_y, start_x, end_x);
            }
            TransitionKind::BusCross => {
                self.draw_bus_cross(target_state, waveform_y, start_x, end_x);
            }
        }
    }
}

impl TransitionDraw {
    fn draw_single_edge(
        &self,
        state: &mut RowState,
        waveform_y: WaveformBoxY,
        start_x: Px,
        end_x: Px,
    ) {
        let y_from = waveform_y.for_single(self.source);
        let y_to = waveform_y.for_single(self.target);
        let use_hiz = self.source == SignalLevel::HiZ || self.target == SignalLevel::HiZ;
        if use_hiz {
            state.push_hiz(start_x, y_from);
            state.push_hiz(end_x, y_to);
        } else {
            state.push_top(start_x, y_from);
            state.push_top(end_x, y_to);
        }
    }

    fn draw_bus_open(
        &self,
        state: &mut RowState,
        waveform_y: WaveformBoxY,
        start_x: Px,
        end_x: Px,
    ) {
        let y_from = waveform_y.for_single(self.source);
        state.push_top(start_x, y_from);
        state.push_top(end_x, waveform_y.top);
        state.push_bottom(start_x, y_from);
        state.push_bottom(end_x, waveform_y.bottom);
    }

    fn draw_bus_close(
        &self,
        state: &mut RowState,
        waveform_y: WaveformBoxY,
        start_x: Px,
        end_x: Px,
    ) {
        let y_to = waveform_y.for_single(self.target);
        state.push_top(start_x, waveform_y.top);
        state.push_top(end_x, y_to);
        state.push_bottom(start_x, waveform_y.bottom);
        state.push_bottom(end_x, y_to);
    }

    fn draw_bus_cross(
        &self,
        state: &mut RowState,
        waveform_y: WaveformBoxY,
        start_x: Px,
        end_x: Px,
    ) {
        if self.source.is_bus_family() {
            self.draw_bus_cross_with_prior_bus(state, waveform_y, start_x, end_x);
        } else {
            self.draw_bus_open_full(state, waveform_y, start_x, end_x);
        }
    }

    /// `X` has a preceding bus: draw the cross region `[start_x, x_c]` and then
    /// horizontal bus rails `[x_c, end_x]`.
    fn draw_bus_cross_with_prior_bus(
        &self,
        state: &mut RowState,
        waveform_y: WaveformBoxY,
        start_x: Px,
        end_x: Px,
    ) {
        let x_c = start_x + self.slant_width;
        // Cross region: line A (top→bottom) and line B (bottom→top).
        state.push_top(start_x, waveform_y.top);
        state.push_top(x_c, waveform_y.bottom);
        state.push_bottom(start_x, waveform_y.bottom);
        state.push_bottom(x_c, waveform_y.top);
        // Rails swap due to crossing.
        state.swap_top_and_bottom();
        // Bus continuation after cross: horizontal rails to end_x.
        if x_c < end_x {
            state.push_top(end_x, waveform_y.top);
            state.push_bottom(end_x, waveform_y.bottom);
        }
    }

    /// `X` has no preceding bus: open the bus region without drawing a cross.
    ///
    /// The source level fan-out forms the opening (BusOpen-equivalent geometry).
    fn draw_bus_open_full(
        &self,
        state: &mut RowState,
        waveform_y: WaveformBoxY,
        start_x: Px,
        end_x: Px,
    ) {
        let y_from = waveform_y.for_single(self.source);
        state.push_top(start_x, y_from);
        state.push_top(end_x, waveform_y.top);
        state.push_bottom(start_x, y_from);
        state.push_bottom(end_x, waveform_y.bottom);
    }
}
