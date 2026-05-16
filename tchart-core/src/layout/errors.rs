//! Errors returned by the layout engine.

/// Reason a layout pass failed.
///
/// Layout currently has only one failure mode: an `Arrow` referenced an
/// `AnchorEnd::Anchor` that did not appear in the resolved
/// [`crate::anchor::AnchorRegistry`]. Parser-level validation should already
/// catch undefined anchors, but the layout engine guards against drift
/// between parser and layout.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LayoutError {
    /// An arrow endpoint referenced an anchor that was not in the registry.
    #[error("unresolved anchor reference")]
    UnresolvedAnchor,
}
