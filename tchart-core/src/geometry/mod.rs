//! 2D geometry primitives in resolved pixel coordinates.
//!
//! See `docs/spec/types.md` §1.3.
//!
//! All fields use [`Px`](crate::units::Px) so layout calculations operate in a
//! single, resolved unit. Whether a [`Rect`] is global or local depends on
//! context (e.g. `Line.bbox` is global, `SignalRow.geometry.label_box` is
//! local to `Line.bbox.origin`).

use crate::units::Px;
use std::ops::{Add, Div, Mul, Sub};

/// A point in pixel coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(crate) struct Point {
    /// Horizontal coordinate.
    pub(crate) x: Px,
    /// Vertical coordinate.
    pub(crate) y: Px,
}

/// A size in pixel coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(crate) struct Size {
    /// Width.
    pub(crate) width: Px,
    /// Height.
    pub(crate) height: Px,
}

/// A rectangle described by its origin and size.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(crate) struct Rect {
    /// Top-left corner.
    pub(crate) origin: Point,
    /// Extent.
    pub(crate) size: Size,
}

impl Point {
    /// The origin point `(0, 0)`.
    pub(crate) const ZERO: Point = Point {
        x: Px::ZERO,
        y: Px::ZERO,
    };

    pub(crate) const fn new(x: Px, y: Px) -> Self {
        Self { x, y }
    }

    pub(crate) const fn new_f32(x: f32, y: f32) -> Self {
        Self { x: Px(x), y: Px(y) }
    }

    /// Return (original length, normalized point)
    pub(crate) fn normal(self) -> (f32, Self) {
        let length = f32::hypot(self.x.to_f32(), self.y.to_f32());
        if length > 1e-6 {
            (length, self / length)
        } else {
            (length, Point::new_f32(1.0, 0.0))
        }
    }

    /// Clockwise 90° rotation of `self` about the origin (in SVG screen
    /// coordinates, where +y points downward).
    ///
    /// For a unit vector `(x, y)` returns `(-y, x)` — the perpendicular
    /// pointing to the visual right of the original direction. Used for
    /// computing clock-edge triangle bases per `docs/spec/svg-rendering.md`
    /// §「クロックエッジマーカー」 (`perpendicular_unit.x = -line_direction.y`,
    /// `perpendicular_unit.y = line_direction.x`).
    pub(crate) fn perpendicular_clockwise(self) -> Self {
        Point::new(self.y * -1.0, self.x)
    }
}

impl Add for Point {
    type Output = Point;
    fn add(self, rhs: Point) -> Point {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Point {
    type Output = Point;
    fn sub(self, rhs: Point) -> Point {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul<f32> for Point {
    type Output = Point;
    fn mul(self, rhs: f32) -> Point {
        Point::new(self.x * rhs, self.y * rhs)
    }
}

impl Div<f32> for Point {
    type Output = Point;
    fn div(self, rhs: f32) -> Point {
        Point::new(self.x / rhs, self.y / rhs)
    }
}

impl Size {
    /// The zero-extent size `(0, 0)`.
    #[cfg(test)]
    pub(crate) const ZERO: Size = Size {
        width: Px::ZERO,
        height: Px::ZERO,
    };

    pub(crate) const fn new(width: Px, height: Px) -> Self {
        Self { width, height }
    }
}

impl Mul<f32> for Size {
    type Output = Size;
    fn mul(self, rhs: f32) -> Size {
        Size::new(self.width * rhs, self.height * rhs)
    }
}

impl Rect {
    /// The zero rectangle (origin and size both zero).
    #[cfg(test)]
    pub(crate) const ZERO: Rect = Rect {
        origin: Point::ZERO,
        size: Size {
            width: Px::ZERO,
            height: Px::ZERO,
        },
    };

    pub(crate) const fn new(x: Px, y: Px, width: Px, height: Px) -> Self {
        Self {
            origin: Point { x, y },
            size: Size { width, height },
        }
    }
}

#[cfg(test)]
mod tests;
