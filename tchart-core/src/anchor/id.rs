//! Anchor identifiers — `@{name}` / `@N`.

use crate::anchor::name::AnchorName;

/// An anchor identifier.
///
/// See `docs/spec/types.md` §3.2.x. Named anchors and indexed anchors live in
/// separate namespaces (`@{1}` and `@1` are distinct).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum AnchorId {
    /// `@{edge}` and similar named anchors.
    Named(AnchorName),
    /// `@0`, `@1`, `@2`, ... numeric anchors. Any non-negative `u32` is
    /// accepted; a digit run whose numeric value would exceed `u32::MAX` is
    /// rejected as a parse error rather than being silently truncated.
    Indexed(u32),
}
