//! Clock specification types — `@clock(...)` declarations.
//!
//! See `docs/spec/types.md` §3.4.1.

use std::num::NonZeroU32;

use crate::color::Color;
use crate::defaults::{
    DEFAULT_CLOCKMARK_HEIGHT_PX, DEFAULT_CLOCKMARK_POSITION, DEFAULT_CLOCKMARK_WIDTH_PX,
};
use crate::units::Px;

/// Style for the triangular edge marker drawn on a clock transition.
///
/// See `docs/spec/types.md` §3.4.1.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ClockMarkStyle {
    /// Position of the triangle apex along the transition line, in `[0.0, 1.0]`.
    pub(crate) position: f32,
    /// Height of the triangle in the line direction (px).
    pub(crate) height: Px,
    /// Width of the triangle base perpendicular to the line (px).
    pub(crate) width: Px,
    /// Fill color (`stroke` is always `none`).
    pub(crate) color: Color,
}

impl ClockMarkStyle {
    /// Construct a new marker style.
    pub(crate) fn new(position: f32, height: Px, width: Px, color: Color) -> Self {
        Self {
            position,
            height,
            width,
            color,
        }
    }
}

/// Full `@clock(...)` specification attached to a signal row.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ClockSpec {
    /// Edge directive (which transitions get arrows, plus implied phase).
    pub(crate) edge: ClockEdge,
    /// Pulse durations (low / high run lengths).
    pub(crate) pulse: ClockPulse,
    /// Initial waveform phase.
    pub(crate) start: ClockPhase,
    /// Triangle marker style for edge markers.
    pub(crate) mark_style: ClockMarkStyle,
}

impl ClockSpec {
    /// Construct a full clock specification.
    pub(crate) fn new(
        edge: ClockEdge,
        pulse: ClockPulse,
        start: ClockPhase,
        mark_style: ClockMarkStyle,
    ) -> Self {
        Self {
            edge,
            pulse,
            start,
            mark_style,
        }
    }

    /// Return `true` if no edge markers should be drawn (`ClockEdge::None`).
    pub(crate) fn is_edge_none(&self) -> bool {
        self.edge == ClockEdge::None
    }
}

/// Which clock edges receive triangle markers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ClockEdge {
    /// `@clock(pos)` — rising edge only.
    Pos,
    /// `@clock(neg)` — falling edge only.
    Neg,
    /// `@clock(both)` — both edges.
    Both,
    /// `@clock(none)` — clock waveform without arrows.
    None,
}

/// Pulse widths for a clock specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ClockPulse {
    /// Number of step units in a low pulse (`_=N`).
    pub(crate) low_units: NonZeroU32,
    /// Number of step units in a high pulse (`~=M`).
    pub(crate) high_units: NonZeroU32,
}

impl ClockPulse {
    /// Construct a clock pulse with given low and high unit counts.
    pub(crate) fn new(low_units: NonZeroU32, high_units: NonZeroU32) -> Self {
        Self {
            low_units,
            high_units,
        }
    }
}

/// Initial phase of an auto-expanded clock waveform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ClockPhase {
    /// Waveform begins at low.
    StartLow,
    /// Waveform begins at high.
    StartHigh,
}

impl Default for ClockMarkStyle {
    /// Default marker style: `DEFAULT_CLOCKMARK_*` constants, color = [`Color::BLACK`].
    fn default() -> Self {
        Self::new(
            DEFAULT_CLOCKMARK_POSITION,
            DEFAULT_CLOCKMARK_HEIGHT_PX,
            DEFAULT_CLOCKMARK_WIDTH_PX,
            Color::BLACK,
        )
    }
}

impl ClockEdge {
    /// Spec (`docs/spec/tcml-format.md` §「@clock」) says the edge keyword
    /// is one of `pos` / `neg` / `both` / `none`, and the attribute keys are
    /// ASCII case-insensitive. The keyword falls under the same case rule
    /// (positional attribute), so `POS`, `Pos`, `pos` are equivalent.
    pub(crate) fn from_keyword(token: &str) -> Option<Self> {
        match token.to_ascii_lowercase().as_str() {
            "pos" => Some(Self::Pos),
            "neg" => Some(Self::Neg),
            "both" => Some(Self::Both),
            "none" => Some(Self::None),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests;
