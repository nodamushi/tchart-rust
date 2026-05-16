//! tchart CLI library surface.
//!
//! The crate ships both a binary (`tchart`) and this library so that
//! integration tests under `tests/` can share constants such as the system
//! font candidate list with the production code without duplicating them.
//!
//! Two modules are part of the public library surface:
//!
//! - [`cli`]: `main.rs` calls `Cli::try_parse()` to parse arguments, then
//!   passes the result to [`cli::dispatch`] which routes to the appropriate
//!   subcommand handler.
//! - [`font`]: integration tests import [`font::CANDIDATE_FONTS`] to avoid
//!   duplicating the candidate list.
//!
//! The remaining modules have no external callers and are `pub(crate)`.
//!
//! See `docs/spec/cli.md` for the public contract of the binary.

pub(crate) mod batch;
pub mod cli;
pub(crate) mod error;
pub(crate) mod extract;
pub mod font;
pub(crate) mod parse_error_format;
pub(crate) mod render;
pub(crate) mod validate;
