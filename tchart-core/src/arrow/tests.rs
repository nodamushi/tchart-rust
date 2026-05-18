//! Unit tests for `arrow`.

use super::{Arrow, ArrowEnd, ArrowHead, ArrowStyle, LineDashStyle};
use crate::anchor::AnchorId;
use crate::color::Color;
use crate::geometry::Point;
use crate::text::{FontSpec, UserText};
use crate::units::Px;

#[test]
fn arrow_constructs_with_anchor_endpoints() {
    let from = ArrowEnd::Anchor(AnchorId::Indexed(1));
    let to = ArrowEnd::Anchor(AnchorId::Indexed(2));
    let arrow = Arrow::new(
        from.clone(),
        to.clone(),
        ArrowStyle::new(
            Color::NONE,
            Px(1.0),
            LineDashStyle::Solid,
            ArrowHead::EndOnly,
        ),
        None,
        FontSpec::default(),
    );
    assert_eq!(arrow.from, from);
    assert_eq!(arrow.to, to);
}

#[test]
fn arrow_end_absolute_holds_point() {
    let endpoint = ArrowEnd::Absolute(Point {
        x: Px(5.0),
        y: Px(10.0),
    });
    match endpoint {
        ArrowEnd::Absolute(point) => assert_eq!(point.x, Px(5.0)),
        ArrowEnd::Anchor(_) => panic!("expected absolute"),
    }
}

#[test]
fn line_dash_style_variants_distinct() {
    assert_ne!(LineDashStyle::Solid, LineDashStyle::Dashed);
    assert_ne!(LineDashStyle::Dotted, LineDashStyle::Solid);
}

#[test]
fn arrow_head_variants_distinct() {
    assert_ne!(ArrowHead::EndOnly, ArrowHead::BothEnds);
    assert_ne!(ArrowHead::None, ArrowHead::EndOnly);
}

#[test]
fn arrow_label_optional() {
    let label = UserText::parse("setup").expect("text");
    let arrow = Arrow::new(
        ArrowEnd::Absolute(Point::ZERO),
        ArrowEnd::Absolute(Point::ZERO),
        ArrowStyle::new(
            Color::NONE,
            Px(1.0),
            LineDashStyle::Solid,
            ArrowHead::EndOnly,
        ),
        Some(label.clone()),
        FontSpec::default(),
    );
    assert_eq!(arrow.label.as_ref(), Some(&label));
}
