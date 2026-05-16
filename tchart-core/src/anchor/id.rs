//! Anchor identifiers — `@{name}` / `@N`.

use std::num::NonZeroU32;

use crate::anchor::name::AnchorName;

/// An anchor identifier.
///
/// See `docs/spec/types.md` §3.2.x. Named anchors and indexed anchors live in
/// separate namespaces (`@{1}` and `@1` are distinct).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum AnchorId {
    /// `@{edge}` and similar named anchors.
    Named(AnchorName),
    /// `@1`, `@2`, ... numeric anchors. The number must be at least 1.
    Indexed(NonZeroU32),
}
