use super::{Point, Rect, Size};
use crate::units::Px;

#[test]
fn test_point_add() {
    let a = Point {
        x: Px(1.0),
        y: Px(2.0),
    };
    let b = Point {
        x: Px(3.0),
        y: Px(4.0),
    };
    assert_eq!(
        a + b,
        Point {
            x: Px(4.0),
            y: Px(6.0)
        }
    );
}

#[test]
fn rect_zero_is_all_zero() {
    assert_eq!(Rect::ZERO.origin, Point::ZERO);
    assert_eq!(Rect::ZERO.size, Size::ZERO);
}

#[test]
fn test_point_construction() {
    let point = Point {
        x: Px(1.0),
        y: Px(2.0),
    };
    assert_eq!(point.x, Px(1.0));
    assert_eq!(point.y, Px(2.0));
}

#[test]
fn test_size_construction() {
    let size = Size {
        width: Px(3.0),
        height: Px(4.0),
    };
    assert_eq!(size.width, Px(3.0));
    assert_eq!(size.height, Px(4.0));
}

#[test]
fn test_rect_construction() {
    let rect = Rect {
        origin: Point {
            x: Px(1.0),
            y: Px(2.0),
        },
        size: Size {
            width: Px(3.0),
            height: Px(4.0),
        },
    };
    assert_eq!(rect.origin.x, Px(1.0));
    assert_eq!(rect.size.height, Px(4.0));
}
