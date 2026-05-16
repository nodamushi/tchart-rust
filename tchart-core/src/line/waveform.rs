//! Waveform element list and the `SignalLevel` enumeration.

use super::TransitionKind;
use crate::anchor::AnchorId;
use crate::clock::ClockPulse;
use crate::line::transition::Transition;
use crate::text::UserText;
use std::ops::Deref;

/// Ordered list of waveform elements parsed from a signal row's body.
///
/// See `docs/spec/types.md` §3.2.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct Waveform {
    /// The ordered list of waveform elements.
    elements: Vec<WaveformElement>,
}

impl From<Vec<WaveformElement>> for Waveform {
    fn from(elements: Vec<WaveformElement>) -> Self {
        Self { elements }
    }
}

impl Waveform {
    /// Append one element to the end of the waveform. Test-only helper.
    #[cfg(test)]
    pub(crate) fn push(&mut self, element: WaveformElement) {
        self.elements.push(element);
    }

    /// Returns `true` when there are no elements.
    pub(crate) const fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Appends one level run generated during clock expansion.
    /// If the last element is a `Level` with a different level, a `SingleEdge`
    /// `Transition` is automatically inserted between them.
    /// `units == 0` is a no-op.
    pub(crate) fn push_clock_run(&mut self, level: SignalLevel, units: u32) {
        if units == 0 {
            return;
        }
        let preceded = if let Some(WaveformElement::Level(prev)) = self.elements.last()
            && prev.level() != level
        {
            let from = prev.level();
            self.elements
                .push(WaveformElement::Transition(Transition::new(
                    from,
                    level,
                    TransitionKind::SingleEdge,
                    None,
                )));
            true
        } else {
            false
        };
        let mut run = LevelRun::new(level, units);
        if preceded {
            run.mark_preceded_by_transition();
        }
        self.elements.push(WaveformElement::Level(run));
    }

    /// Total number of waveform units consumed by the level runs in `waveform`.
    /// Saturating addition prevents `u32` overflow for adversarial inputs.
    /// Shared with [`super::clock`] so the calculation lives in one place.
    pub(crate) fn level_units_total(&self) -> u32 {
        self.elements
            .iter()
            .filter_map(|element| match element {
                WaveformElement::Level(run) => Some(run.units()),
                _ => None,
            })
            .fold(0u32, |sum, units| sum.saturating_add(units))
    }
}

impl Deref for Waveform {
    type Target = [WaveformElement];

    fn deref(&self) -> &Self::Target {
        &self.elements
    }
}

/// One waveform element. See `docs/spec/types.md` §3.2.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum WaveformElement {
    /// A run of one or more identical signal levels.
    Level(LevelRun),
    /// A transition between two adjacent levels.
    Transition(Transition),
    /// `:` — a one-unit wide gap that breaks polyline continuity.
    Gap,
    /// `|` — a vertical guide line.
    Guide,
    /// `[` — start of a highlighted region.
    HighlightStart,
    /// `]` — end of a highlighted region.
    HighlightEnd,
    /// `@{name}` / `@N` — zero-width anchor marker.
    Anchor(AnchorId),
    /// Text to render at the centre of the owning level run's region.
    ///
    /// Width is `Px::ZERO` — text does not advance the waveform x cursor.
    /// See `docs/spec/types.md` §6.4 and `docs/spec/tcml-format.md`
    /// §「レベル文字列中のテキスト文字」.
    Text(UserText),
}

/// One run of identical signal levels.
///
/// All fields are private. Mutation goes through [`LevelRun::extend_units`]
/// and [`LevelRun::mark_preceded_by_transition`]; reads go through
/// [`LevelRun::level`] and [`LevelRun::units`].
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LevelRun {
    level: SignalLevel,
    units: u32,
    /// `true` when this run is immediately preceded by a `Transition::*`
    /// element.  When true the first `slant`-wide slice of the first unit
    /// has already been drawn by that transition, so the effective hold
    /// portion is `units * step - slant` instead of `units * step`.
    ///
    /// See `docs/spec/types.md` §6.4 single-source width rule.
    preceded_by_transition: bool,
}

impl LevelRun {
    /// Construct a new [`LevelRun`] with `preceded_by_transition = false`.
    pub(crate) fn new(level: SignalLevel, units: u32) -> Self {
        Self {
            level,
            units,
            preceded_by_transition: false,
        }
    }

    /// Signal level for this run.
    pub(crate) const fn level(&self) -> SignalLevel {
        self.level
    }

    /// Number of step units the run occupies.
    pub(crate) const fn units(&self) -> u32 {
        self.units
    }

    /// Add `additional` units to this run.
    pub(crate) fn extend_units(&mut self, additional: u32) {
        self.units = self.units.saturating_add(additional);
    }

    /// Mark this run as immediately preceded by a transition element.
    ///
    /// The caller is responsible for ensuring `units >= 1` before calling
    /// this method; the current parser always satisfies that invariant.
    pub(crate) fn mark_preceded_by_transition(&mut self) {
        self.preceded_by_transition = true;
    }

    /// Returns `true` when this run is immediately preceded by a transition element.
    pub(crate) const fn is_preceded_by_transition(&self) -> bool {
        self.preceded_by_transition
    }
}

/// Signal level values used by the waveform.
///
/// See `docs/spec/types.md` §3.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum SignalLevel {
    /// `_` — low.
    Low,
    /// `~` — high.
    High,
    /// `-` — high impedance.
    HiZ,
    /// `=` / `X` / `<label>` — bus.
    Bus,
    /// `?` following `_`.
    DontCareAlongLow,
    /// `?` following `~`.
    DontCareAlongHigh,
    /// `?` following `-`.
    DontCareAlongHiZ,
    /// `?` following `=` / `X`.
    DontCareAlongBus,
}

/// Drawing-shape class used to pick the right transition rendering.
///
/// See `docs/spec/types.md` §3.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LevelShape {
    /// Single line (`_`, `~`, `-`).
    Single,
    /// Two lines forming a bus envelope (`=`).
    Double,
    /// Filled rectangle plus one inner line (`_?`, `~?`, `-?`).
    FillSingle,
    /// Filled rectangle plus a bus envelope (`=?`, `X?`).
    FillDouble,
}

impl SignalLevel {
    /// `true` when this level is one of the `DontCareAlong*` variants
    /// (i.e. emitted by `?` following a base level).
    pub(crate) const fn is_dontcare(self) -> bool {
        matches!(
            self,
            Self::DontCareAlongLow
                | Self::DontCareAlongHigh
                | Self::DontCareAlongHiZ
                | Self::DontCareAlongBus
        )
    }

    /// Map this level to its drawing-shape class.
    pub(crate) fn into_shape(self) -> LevelShape {
        match self {
            Self::Low | Self::High | Self::HiZ => LevelShape::Single,
            Self::Bus => LevelShape::Double,
            Self::DontCareAlongLow | Self::DontCareAlongHigh | Self::DontCareAlongHiZ => {
                LevelShape::FillSingle
            }
            Self::DontCareAlongBus => LevelShape::FillDouble,
        }
    }

    /// Number of units a clock pulse should hold on `level`. Only `High` reads
    /// the high-pulse width; everything else (including `HiZ` / `Bus`, which
    /// cannot legally appear in a clock waveform but are matched defensively)
    /// uses the low-pulse width.
    pub(crate) fn pulse_units_for(self, pulse: ClockPulse) -> u32 {
        match self {
            Self::High => pulse.high_units.get(),
            _ => pulse.low_units.get(),
        }
    }

    /// Toggle a clock level. `Low <-> High`. Anything else (HiZ, Bus, dontcare
    /// variants) is returned unchanged because clock waveforms only ever carry
    /// `Low` / `High` runs after expansion.
    pub(crate) fn toggle(self) -> Self {
        match self {
            Self::Low => Self::High,
            Self::High => Self::Low,
            other => other,
        }
    }

    /// `true` for the two bus-family level variants: `Bus` and `DontCareAlongBus`.
    ///
    /// Used when scanning waveform elements to decide whether an adjacent
    /// transition is a continuation (`BusCross`) or an opening/closing edge.
    pub(crate) const fn is_bus_family(self) -> bool {
        matches!(self, Self::Bus | Self::DontCareAlongBus)
    }

    /// Map this level to the `DontCareAlong*` variant that follows its shape.
    /// Idempotent on `DontCareAlong*` inputs (already resolved).
    pub(crate) fn into_dontcare_along(self) -> Self {
        match self {
            Self::Low => Self::DontCareAlongLow,
            Self::High => Self::DontCareAlongHigh,
            Self::HiZ => Self::DontCareAlongHiZ,
            Self::Bus => Self::DontCareAlongBus,
            Self::DontCareAlongLow
            | Self::DontCareAlongHigh
            | Self::DontCareAlongHiZ
            | Self::DontCareAlongBus => self,
        }
    }
}
