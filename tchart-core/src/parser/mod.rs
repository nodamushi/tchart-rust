//! TCML parser entry point.
//!
//! See `docs/spec/types.md` §8 and `docs/spec/tcml-format.md`.
//!
//! # Pipeline at a glance
//!
//! [`parse`] runs three phases:
//!
//! 1. **Line scan** ([`state::Parser::parse_input`]) — walks each source line,
//!    dispatches by leading character (`@` / `%` / `"` / waveform), and
//!    records pending anchors and arrows for the post-pass.
//! 2. **Clock expansion** (in-place during phase 1's tail) — expands every
//!    `@clock`-decorated row out to the chart-wide unit count.
//! 3. **Anchor resolution** ([`state::Parser::finish_into_document`]) —
//!    builds the `AnchorRegistry` from pending anchors, validates every
//!    arrow endpoint against it, and packages the final [`ChartDocument`].
//!
//! Every parser failure is wrapped in a [`ParseError`] carrying a 1-based
//! source location.

mod anchor;
mod arrow;
mod attr;
mod clock;
mod directive;
mod state;
mod text_quote;
mod transition;
mod waveform;

#[cfg(test)]
mod tests;

use crate::document::ChartDocument;
use crate::errors::ParseError;
use state::Parser;
pub(crate) use state::PendingAnchor;

/// Parse a TCML source string into a [`ChartDocument`].
///
/// # Errors
///
/// Returns a [`ParseError`] when the input fails any TCML validation.
pub fn parse(input: &str) -> Result<ChartDocument, ParseError> {
    Parser::parse_input(input)?.finish_into_document(input)
}
