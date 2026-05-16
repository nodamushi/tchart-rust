//! Anchor types: `@{name}` / `@N` markers and the resolution registry.
//!
//! See `docs/spec/types.md` §3.2.x and §5.

mod id;
mod name;
mod registry;

pub(crate) use id::AnchorId;
pub(crate) use name::{AnchorName, AnchorNameError};
pub(crate) use registry::{AnchorRegistry, ResolvedAnchor};

#[cfg(test)]
mod tests;
