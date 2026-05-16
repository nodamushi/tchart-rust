//! Layout-related parameters carried by the chart style.
//!
//! These values steer the layout engine: the per-unit step width,
//! the slant width for transitions, the symmetric inter-row gap, and the
//! optional explicit `capwidth`. They are populated by the parser from
//! `@step` / `@slant` / `@h_space` / `@capwidth` directives and consumed
//! by [`crate::layout::layout`].
//!
//! See `docs/spec/types.md` §6.4 (single-source width rule).

use crate::defaults::{DEFAULT_H_SPACE_PX, DEFAULT_SLANT_PX, DEFAULT_STEP_PX};
use crate::line::{TransitionKind, WaveformElement};
use crate::units::Px;

/// Layout-time parameters applied uniformly to the chart.
///
/// `slant_explicit` records whether the active `slant` value came from a
/// user-supplied `@slant` directive (`true`) or is still the implicit default
/// (`false`). The flag exists so [`Self::set_step`] can shrink the slant when
/// a small `@step` would otherwise make `step <= slant` (a configuration that
/// cannot host any level hold portion). When the user has explicitly chosen a
/// slant value, that choice is respected and the resulting `step <= slant`
/// configuration is reported as a parse error instead.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct LayoutParams {
    step: Px,
    slant: Px,
    slant_explicit: bool,
    h_space: Px,
    capwidth: Option<Px>,
}

impl LayoutParams {
    /// Vertical inter-row gap, distributed symmetrically as `gap/2`.
    pub(crate) fn h_space(&self) -> Px {
        self.h_space
    }

    /// Explicit cap (signal-name) column width. `None` triggers auto layout.
    pub(crate) fn capwidth(&self) -> Option<Px> {
        self.capwidth
    }

    /// Step width per time unit: the x-advance for one level character or
    /// one `Gap` element.
    pub(crate) fn step(&self) -> Px {
        self.step
    }

    /// Slant width for all transition kinds: `SingleEdge`, `BusOpen`,
    /// `BusClose`, and `BusCross` (cross region only).
    pub(crate) fn slant(&self) -> Px {
        self.slant
    }

    /// Resolved layout width of `element` under these parameters.
    ///
    /// For a `LevelRun` preceded by a `Transition`, the width is
    /// `units * step - slant` because the first slant-wide slice is already
    /// consumed by that transition.  Otherwise the width is `units * step`.
    ///
    /// See `docs/spec/types.md` §6.4 (single-source width rule).
    pub(crate) fn element_width(&self, element: &WaveformElement) -> Px {
        match element {
            WaveformElement::Level(run) => {
                let full = self.step * (run.units() as f32);
                if run.is_preceded_by_transition() {
                    full - self.slant
                } else {
                    full
                }
            }
            WaveformElement::Transition(transition) => self.transition_width(transition.kind),
            WaveformElement::Gap => self.step,
            WaveformElement::Guide
            | WaveformElement::HighlightStart
            | WaveformElement::HighlightEnd
            | WaveformElement::Anchor(_)
            | WaveformElement::Text(_) => Px::ZERO,
        }
    }

    /// Width of a transition of the given `kind`.
    fn transition_width(&self, kind: TransitionKind) -> Px {
        match kind {
            TransitionKind::SingleEdge
            | TransitionKind::BusOpen
            | TransitionKind::BusClose
            | TransitionKind::BusCross => self.slant,
        }
    }

    /// Sum of [`Self::element_width`] over `elements`.
    pub(crate) fn sum_element_widths(&self, elements: &[WaveformElement]) -> Px {
        elements
            .iter()
            .map(|element| self.element_width(element))
            .fold(Px::ZERO, |total, width| total + width)
    }

    /// Set the step width per time unit.
    ///
    /// When the new step is smaller than the current slant and the slant has
    /// not been explicitly set by the user (`@slant`), the slant is silently
    /// clamped to `step / 2` (rounded down to half a pixel). This keeps small
    /// `@step` values such as `@step 2` usable with the implicit default slant
    /// (5 px) without surfacing `InvalidStepSlant`. An explicit slant is
    /// preserved as-is; the existing `step <= slant` check then surfaces the
    /// configuration error to the user as before.
    pub(super) fn set_step(&mut self, step: Px) {
        self.step = step;
        if !self.slant_explicit && self.slant >= step {
            let half = step.to_f32() / 2.0;
            self.slant = Px(half.max(0.0));
        }
    }

    /// Set the slant width for all transitions and mark the value as explicit
    /// so subsequent `@step` directives no longer auto-clamp it.
    pub(super) fn set_slant(&mut self, slant: Px) {
        self.slant = slant;
        self.slant_explicit = true;
    }

    /// Set the inter-row gap.
    pub(super) fn set_h_space(&mut self, h_space: Px) {
        self.h_space = h_space;
    }

    /// Set the explicit cap width.
    pub(super) fn set_capwidth(&mut self, capwidth: Option<Px>) {
        self.capwidth = capwidth;
    }
}

impl Default for LayoutParams {
    fn default() -> Self {
        Self {
            step: DEFAULT_STEP_PX,
            slant: DEFAULT_SLANT_PX,
            slant_explicit: false,
            h_space: DEFAULT_H_SPACE_PX,
            capwidth: None,
        }
    }
}
