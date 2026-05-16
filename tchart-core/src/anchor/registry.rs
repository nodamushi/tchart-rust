//! Resolved anchor positions accumulated during the parser's resolution pass.

use std::collections::HashMap;

use crate::anchor::id::AnchorId;
use crate::errors::ParseError;
use crate::geometry::Point;
use crate::parser::PendingAnchor;

/// Mapping from each declared anchor to its resolved position.
///
/// See `docs/spec/types.md` §5.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct AnchorRegistry {
    by_id: HashMap<AnchorId, ResolvedAnchor>,
}

impl AnchorRegistry {
    /// Returns `true` when no anchors have been registered. Used by document
    /// unit tests to verify that the default registry is empty.
    #[cfg(test)]
    pub(crate) fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    /// Returns the number of registered anchors. Used by parser unit tests to
    /// verify that anchor declarations were collected as expected.
    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.by_id.len()
    }

    #[cfg(test)]
    pub(crate) fn insert(&mut self, id: AnchorId, anchor: ResolvedAnchor) {
        self.by_id.insert(id, anchor);
    }

    pub(crate) fn build(pending_anchors: &[PendingAnchor]) -> Result<Self, ParseError> {
        let mut registry = Self::default();
        for anchor in pending_anchors {
            anchor.register_into(&mut registry)?;
        }
        Ok(registry)
    }

    /// Insert `(id, anchor)` if `id` is not already registered. Returns
    /// `Err(())` to signal a duplicate; the caller wraps that into the
    /// concrete `ParseError` carrying the source location.
    pub(crate) fn try_insert_unique(
        &mut self,
        id: AnchorId,
        anchor: ResolvedAnchor,
    ) -> Result<(), ()> {
        use std::collections::hash_map::Entry;
        match self.by_id.entry(id) {
            Entry::Occupied(_) => Err(()),
            Entry::Vacant(slot) => {
                slot.insert(anchor);
                Ok(())
            }
        }
    }

    /// Return `true` if an anchor with the given id is already registered.
    pub(crate) fn contains(&self, id: &AnchorId) -> bool {
        self.by_id.contains_key(id)
    }

    /// Look up an anchor by id and return its resolved position, or `None`.
    pub(crate) fn lookup_position(&self, id: &AnchorId) -> Option<Point> {
        self.by_id.get(id).map(|anchor| anchor.at)
    }

    /// Iterate mutably over all registered anchors.
    pub(crate) fn iter_resolved_mut(&mut self) -> impl Iterator<Item = &mut ResolvedAnchor> {
        self.by_id.values_mut()
    }
}

/// A resolved anchor — its position plus the inline source coordinates that
/// the layout engine uses to recompute the position after stacking.
///
/// Inline source coordinates (`signal_index` / `element_index`) are exposed
/// as `pub(crate)` because both are independent index scalars with no
/// cross-field invariant. All anchors currently originate inline inside a
/// signal row's waveform string; the layout engine uses these indices to
/// compute the absolute position once row geometry is resolved.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ResolvedAnchor {
    at: Point,
    /// 0-based index into [`crate::document::ChartDocument::lines`].
    pub(crate) signal_index: usize,
    /// 0-based index into the row's waveform element list.
    pub(crate) element_index: usize,
}

impl ResolvedAnchor {
    /// Construct a new resolved anchor from its position and inline source
    /// indices.
    pub(crate) fn new(at: Point, signal_index: usize, element_index: usize) -> Self {
        Self {
            at,
            signal_index,
            element_index,
        }
    }

    /// Update the resolved position (called by the layout engine).
    pub(crate) fn set_position(&mut self, point: Point) {
        self.at = point;
    }
}
