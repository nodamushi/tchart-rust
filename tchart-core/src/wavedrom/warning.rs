//! Typed warning variants for the WaveDrom export.
//!
//! See `docs/spec/wavedrom.md` §警告 for message format requirements.

use std::fmt;

/// A warning produced during WaveDrom JSON export.
///
/// Warnings do not affect the validity of the output; they signal lossy or
/// approximate conversions that the caller may want to surface to users.
///
/// `Debug` is hand-rolled so that diagnostic output reproduces the same
/// canonical message as `Display` (per `docs/spec/wavedrom.md` §警告) rather
/// than a raw enum dump.
#[derive(Clone, PartialEq)]
pub enum WaveDromWarning {
    /// A signal's `step` value changed when rounded to the nearest integer.
    StepRounded {
        /// Signal name (newlines already flattened to spaces).
        signal_name: String,
        /// Original `step` value before rounding.
        original: f32,
        /// Rounded integer value used in period calculation.
        rounded: u32,
    },
    /// More than 52 anchors were referenced; excess edges are dropped.
    ///
    /// The spec-defined message (`docs/spec/wavedrom.md` §警告) does not
    /// include the count, so this variant carries no payload.
    TooManyAnchors,
    /// More than one `@title` row was present; only the first is kept and the
    /// rest are dropped because WaveDrom's `head.text` is a single string.
    AdditionalTitlesDropped {
        /// Number of `@title` rows after the first (i.e. how many were dropped).
        dropped_count: usize,
    },
}

impl fmt::Display for WaveDromWarning {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WaveDromWarning::StepRounded {
                signal_name,
                original,
                rounded,
            } => write!(
                formatter,
                "warning: signal \"{signal_name}\" step rounded: {original} -> {rounded}"
            ),
            WaveDromWarning::TooManyAnchors => write!(
                formatter,
                "warning: more than 52 anchors; edges referencing extra anchors are dropped"
            ),
            WaveDromWarning::AdditionalTitlesDropped { dropped_count } => write!(
                formatter,
                "warning: only the first @title is kept; {dropped_count} additional @title row(s) dropped"
            ),
        }
    }
}

impl fmt::Debug for WaveDromWarning {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}
