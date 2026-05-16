//! Period / time-axis normalisation for WaveDrom export.
//!
//! See `docs/spec/wavedrom.md` §period / 時間軸正規化.

use crate::units::Px;

use super::warning::WaveDromWarning;

/// Round a `Px` step to the nearest integer.
///
/// Returns `(rounded, warning)`. When the value changed during rounding,
/// `warning` contains a [`WaveDromWarning::StepRounded`] variant for the
/// caller to surface. `signal_name` is embedded in the variant.
pub(super) fn round_step(step: Px, signal_name: &str) -> (u32, Option<WaveDromWarning>) {
    let raw = step.to_f32();
    let rounded = raw.round();
    let warning = if (raw - rounded).abs() > f32::EPSILON && !signal_name.is_empty() {
        Some(WaveDromWarning::StepRounded {
            signal_name: signal_name.to_owned(),
            original: raw,
            rounded: rounded as u32,
        })
    } else {
        None
    };
    (rounded as u32, warning)
}

/// Compute the GCD of all step integers. Initial value is 0 so that the fold
/// yields the first element when the slice has only one entry, and the overall
/// GCD for longer slices.
///
/// Returns 0 when `steps` is empty or all values are 0.
pub(super) fn compute_divisor(steps: &[u32]) -> u32 {
    steps.iter().copied().fold(0u32, gcd)
}

/// `period` value for a signal with the given `step` integer and `divisor`
/// (result of [`compute_divisor`]). Returns `None` when `period == 1` (omit
/// the field per the WaveDrom spec) or when `divisor == 0`.
pub(super) fn signal_period(step: u32, divisor: u32) -> Option<u32> {
    if divisor == 0 || step == 0 {
        return None;
    }
    let period = step / divisor;
    if period == 1 { None } else { Some(period) }
}

/// Euclidean greatest-common-divisor (iterative).
fn gcd(a: u32, b: u32) -> u32 {
    let mut a = a;
    let mut b = b;
    while b != 0 {
        let remainder = a % b;
        a = b;
        b = remainder;
    }
    a
}
