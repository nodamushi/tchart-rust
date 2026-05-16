//! `RulerContribution` — one (x, color) pair a row contributes to the
//! `@ruler` background guide layer.
//!
//! See `docs/spec/tcml-format.md` §「`@ruler` の詳細」for the donation model.

use crate::color::Color;
use crate::units::Px;

/// One ruler-line contribution donated by a signal or `@skip` row that was
/// committed while `@ruler on` was active.
///
/// Each contribution carries the x position (snapshot of `i × step` for some
/// `0 ≤ i ≤ units`) and the color that was active at commit time
/// (`@ruler_color` snapshot). Renderers walk every row, merge contributions
/// by x position (last-wins), and draw one `<line>` per surviving x.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct RulerContribution {
    /// x position in chart-inner-local coordinates (= waveform-area-local x).
    pub(crate) x: Px,
    /// Snapshotted ruler color active at the donating row's commit time.
    pub(crate) color: Color,
}

impl RulerContribution {
    /// Construct a contribution from an explicit x and color.
    pub(crate) fn new(x: Px, color: Color) -> Self {
        Self { x, color }
    }

    /// Generate `units + 1` contributions at `0, step, 2*step, ..., units*step`
    /// using `color`. The returned iterator is empty only for `units == 0`
    /// → single contribution at `x = 0`.
    pub(crate) fn donations(
        step: Px,
        units: u32,
        color: Color,
    ) -> impl Iterator<Item = RulerContribution> {
        (0..=units).map(move |index| RulerContribution::new(step * (index as f32), color))
    }
}
