//! Row model — `Line`, `LineContent`, `SignalRow`, `SkipRow`, `TitleRow`,
//! waveform elements and transitions.
//!
//! See `docs/spec/types.md` §3.

mod content;
mod row;
mod ruler;
mod transition;
mod waveform;

pub(crate) use content::{Line, LineContent};
pub(crate) use row::{EdgeMark, SignalDecorations, SignalGeometry, SignalRow, SkipRow, TitleRow};
pub(crate) use ruler::RulerContribution;
pub(crate) use transition::{Transition, TransitionKind};
pub(crate) use waveform::{LevelRun, LevelShape, SignalLevel, Waveform, WaveformElement};

#[cfg(test)]
mod tests;
