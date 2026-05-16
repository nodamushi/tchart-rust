//! Helpers for computing waveform-relative y coordinates and chart-wide rects.

use crate::geometry::Point;
use crate::line::SignalLevel;
use crate::units::Px;

/// Y coordinates for the waveform box of a signal row, in chart coordinates.
#[derive(Debug, Clone, Copy)]
pub(super) struct WaveformBoxY {
    /// Top edge of `signal_box` (chart-coord; signal High level for single-rail signals).
    pub(super) top: Px,
    /// Mid-line (HiZ / Bus center).
    pub(super) middle: Px,
    /// Bottom edge of `signal_box` (chart-coord; signal Low level for single-rail signals).
    pub(super) bottom: Px,
}

impl WaveformBoxY {
    /// Compute waveform-box y values from a signal-row bbox origin and local signal_box rect.
    pub(super) fn from_chart(
        bbox_origin: Point,
        signal_local_origin: Point,
        signal_height: Px,
    ) -> Self {
        let top = bbox_origin.y + signal_local_origin.y;
        let bottom = top + signal_height;
        let middle = (top + bottom) * 0.5;
        Self {
            top,
            middle,
            bottom,
        }
    }

    /// Vertical centre of the waveform box (used for text label placement).
    pub(super) fn center(self) -> Px {
        self.middle
    }

    /// Pick the y for a single-rail level.
    pub(super) fn for_single(self, level: SignalLevel) -> Px {
        match level {
            SignalLevel::Low | SignalLevel::DontCareAlongLow => self.bottom,
            SignalLevel::High | SignalLevel::DontCareAlongHigh => self.top,
            SignalLevel::HiZ | SignalLevel::DontCareAlongHiZ => self.middle,
            SignalLevel::Bus | SignalLevel::DontCareAlongBus => self.middle,
        }
    }
}
