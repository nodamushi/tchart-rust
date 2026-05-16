//! `LevelRun` polyline-point pusher (the DontCare backing rect is handled by
//! [`super::dontcare::DontCareRect`]).

use crate::line::SignalLevel;
use crate::svg::waveform::state::{DrawOn, RowState};
use crate::units::Px;

/// Polyline portion of a `LevelRun` — produces top/bottom/hiz points.
///
/// Constructed by [`super::state::RowState::push_level`] and applied through
/// the [`DrawOn`] trait so that the per-row `&mut RowState` exposes a single
/// `draw` entry.
pub(super) struct LevelDraw {
    pub(super) width: Px,
    pub(super) level: SignalLevel,
}

impl DrawOn for LevelDraw {
    fn draw_on(&self, target: &mut RowState) {
        let waveform_y = target.waveform_y();
        let x0 = target.cursor();
        let x1 = target.advance(self.width);
        match self.level {
            SignalLevel::Low
            | SignalLevel::High
            | SignalLevel::DontCareAlongLow
            | SignalLevel::DontCareAlongHigh => {
                let y = waveform_y.for_single(self.level);
                target.push_top(x0, y);
                target.push_top(x1, y);
            }
            SignalLevel::HiZ | SignalLevel::DontCareAlongHiZ => {
                target.push_hiz(x0, waveform_y.middle);
                target.push_hiz(x1, waveform_y.middle);
            }
            SignalLevel::Bus | SignalLevel::DontCareAlongBus => {
                target.push_top(x0, waveform_y.top);
                target.push_top(x1, waveform_y.top);
                target.push_bottom(x0, waveform_y.bottom);
                target.push_bottom(x1, waveform_y.bottom);
            }
        }
    }
}
