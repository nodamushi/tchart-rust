//! Unit tests for `clock`.

use std::num::NonZeroU32;

use super::{ClockEdge, ClockMarkStyle, ClockPhase, ClockPulse, ClockSpec};

fn make_nonzero(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value).expect("value must be non-zero")
}

#[test]
fn clock_spec_constructs() {
    let spec = ClockSpec::new(
        ClockEdge::Pos,
        ClockPulse::new(make_nonzero(2), make_nonzero(3)),
        ClockPhase::StartLow,
        ClockMarkStyle::default(),
    );
    assert_eq!(spec.edge, ClockEdge::Pos);
    assert_eq!(spec.pulse.low_units.get(), 2);
    assert_eq!(spec.pulse.high_units.get(), 3);
    assert_eq!(spec.start, ClockPhase::StartLow);
}

#[test]
fn clock_edge_variants_distinct() {
    assert_ne!(ClockEdge::Pos, ClockEdge::Neg);
    assert_ne!(ClockEdge::Both, ClockEdge::None);
}

#[test]
fn clock_phase_variants_distinct() {
    assert_ne!(ClockPhase::StartLow, ClockPhase::StartHigh);
}
