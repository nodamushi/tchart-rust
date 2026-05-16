//! Transitions between adjacent signal levels.

use crate::clock::ClockEdge;
use crate::line::waveform::SignalLevel;
use crate::text::UserText;

/// A transition between two adjacent levels.
///
/// See `docs/spec/types.md` §3.2. The optional `label` carries the text from
/// constructs like `X<value>` and is currently surfaced to parser tests; the
/// SVG renderer does not yet draw it.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Transition {
    /// Source level.
    pub(crate) source: SignalLevel,
    /// Destination level.
    pub(crate) target: SignalLevel,
    /// Transition kind.
    pub(crate) kind: TransitionKind,
    /// Optional embedded label from `X<value>` syntax.
    pub(crate) label: Option<UserText>,
}

impl Transition {
    /// Construct a new [`Transition`].
    pub(crate) fn new(
        source: SignalLevel,
        target: SignalLevel,
        kind: TransitionKind,
        label: Option<UserText>,
    ) -> Self {
        Self {
            source,
            target,
            kind,
            label,
        }
    }

    /// Returns `true` when this transition should receive a clock-edge arrow for `edge`.
    pub(crate) fn is_clock_edge_match(&self, edge: ClockEdge) -> bool {
        if self.kind != TransitionKind::SingleEdge {
            return false;
        }
        matches!(
            (edge, self.source, self.target),
            (ClockEdge::Pos, SignalLevel::Low, SignalLevel::High)
                | (ClockEdge::Neg, SignalLevel::High, SignalLevel::Low)
                | (ClockEdge::Both, SignalLevel::Low, SignalLevel::High)
                | (ClockEdge::Both, SignalLevel::High, SignalLevel::Low),
        )
    }
}

/// Transition shape classification.
///
/// See `docs/spec/types.md` §3.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TransitionKind {
    /// Single ↔ Single transition (single slanted edge).
    SingleEdge,
    /// Single → Double — one line opens into two.
    BusOpen,
    /// Double → Single — two lines close into one.
    BusClose,
    /// Double ↔ Double value crossing (`X`).
    BusCross,
}
