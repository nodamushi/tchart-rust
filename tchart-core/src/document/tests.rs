//! Unit tests for `document`.

use super::{Annotations, ChartDocument, TcmlSource, TextOverlay};
use crate::anchor::AnchorRegistry;
use crate::geometry::Point;
use crate::style::ChartStyle;
use crate::text::UserText;
use crate::units::Px;

#[test]
fn tcml_source_roundtrip() {
    let source = TcmlSource::new("@title hello");
    assert_eq!(source.as_str(), "@title hello");
    assert!(!source.is_empty());
}

#[test]
fn tcml_source_default_is_empty() {
    let source = TcmlSource::default();
    assert!(source.is_empty());
    assert_eq!(source.as_str(), "");
}

#[test]
fn text_overlay_holds_position_and_text() {
    let overlay = TextOverlay::new(
        Point {
            x: Px(1.0),
            y: Px(2.0),
        },
        UserText::parse("hi").expect("text"),
    );
    assert_eq!(overlay.at.x, Px(1.0));
    assert_eq!(overlay.text.as_str(), "hi");
}

#[test]
fn chart_document_constructs() {
    let document = ChartDocument::new(
        ChartStyle::default(),
        Vec::new(),
        Annotations::default(),
        TcmlSource::default(),
    );
    assert!(document.lines.is_empty());
    assert!(document.annotations.overlays.is_empty());
}

#[test]
fn annotations_default_empty() {
    let annotations = Annotations::default();
    assert!(annotations.overlays.is_empty());
    assert!(annotations.arrows.is_empty());
    assert!(annotations.anchors.is_empty());
}

#[test]
fn annotations_new_holds_components() {
    let annotations = Annotations::new(Vec::new(), Vec::new(), AnchorRegistry::default());
    assert!(annotations.overlays.is_empty());
    assert!(annotations.arrows.is_empty());
    assert!(annotations.anchors.is_empty());
}
