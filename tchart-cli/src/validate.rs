//! CLI argument validators.
//!
//! Houses the small checks that run on parsed clap arguments before they are
//! handed off to the rest of the pipeline. Kept separate from `error.rs`
//! because these are input-validation entry points, not error-type construction
//! helpers — `error.rs` only defines `CliError` and its accessors.
//!
//! The numeric value-domain check lives in `tchart_core::units::Px` so that
//! the wasm front-end uses the same kernel. This module exists only to map
//! the rejection into [`CliError::InvalidFontSize`].

use tchart_core::units::Px;

use crate::error::CliError;

/// Validate a `--font-size` argument value: when supplied, the value must be a
/// strictly positive finite `f32`. Returns the value unchanged on success.
///
/// The numeric check is delegated to [`Px::try_from_positive_finite`] so the
/// CLI and wasm front-ends share a single rejection rule (see
/// `docs/spec/cli.md` / `docs/spec/web.md` — `--font-size` / `fontSize` must
/// be a strictly positive finite value).
pub(crate) fn validate_font_size(size: Option<f32>) -> Result<Option<f32>, CliError> {
    match size {
        Some(value) => match Px::try_from_positive_finite(value) {
            Ok(_) => Ok(Some(value)),
            Err(rejected) => Err(CliError::InvalidFontSize(rejected)),
        },
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests;
