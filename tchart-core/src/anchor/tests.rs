//! Unit tests for `anchor`.

use std::num::NonZeroU32;

use super::{AnchorId, AnchorName, AnchorNameError, ResolvedAnchor};
use crate::geometry::Point;

#[test]
fn anchor_name_accepts_basic() {
    let name = AnchorName::parse("edge").expect("ok");
    assert_eq!(name.as_str(), "edge");
}

#[test]
fn anchor_name_accepts_underscore_leading_and_dash() {
    AnchorName::parse("_foo-bar_42").expect("ok");
}

#[test]
fn anchor_name_rejects_empty() {
    assert_eq!(AnchorName::parse(""), Err(AnchorNameError::Empty));
}

#[test]
fn anchor_name_accepts_leading_digit() {
    // Per `docs/spec/types.md` §2.1 / `docs/spec/tcml-format.md`
    // §「アンカー」: leading digit is allowed; pure numeric names like
    // `1edge` or `1` are valid named anchors. The `{ }` distinguishes
    // named vs numbered, not the first character.
    AnchorName::parse("1edge").expect("digit leading char is valid");
    AnchorName::parse("1").expect("pure numeric named anchor is valid");
}

#[test]
fn anchor_name_rejects_leading_dash() {
    assert_eq!(
        AnchorName::parse("-edge"),
        Err(AnchorNameError::InvalidLeadingChar)
    );
}

#[test]
fn anchor_name_rejects_invalid_tail() {
    // "edge!" — the `!` is at char offset 4.
    assert_eq!(
        AnchorName::parse("edge!"),
        Err(AnchorNameError::InvalidChar { char_offset: 4 })
    );
}

#[test]
fn anchor_id_named_and_indexed_are_distinct() {
    let named = AnchorId::Named(AnchorName::parse("a").expect("ok"));
    let indexed = AnchorId::Indexed(NonZeroU32::new(1).expect("nz"));
    assert_ne!(named, indexed);
}

#[test]
fn resolved_anchor_stores_inline_indices() {
    let resolved = ResolvedAnchor::new(Point::ZERO, 2, 5);
    assert_eq!(resolved.signal_index, 2);
    assert_eq!(resolved.element_index, 5);
}
