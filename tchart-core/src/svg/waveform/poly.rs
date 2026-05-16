//! Polyline accumulator implementing the gap-flush and exhaustive-transition contracts.
//!
//! See `docs/spec/svg-rendering.md` "Polyline 蓄積器 (`PolyAccum`)" and
//! `docs/spec/types.md` §11.1 / §11.3.

use crate::svg::buf::{SvgBuf, WriteSvgOn};
use crate::units::Px;

/// One polyline accumulator. Push points, then `flush` to emit a `<polyline>`.
#[derive(Debug, Default)]
pub(super) struct PolyAccum {
    points: Vec<(Px, Px)>,
}

impl PolyAccum {
    /// Empty accumulator.
    pub(super) fn new() -> Self {
        Self::default()
    }

    /// Append a point. If the last point matches `(x, y)`, dedup.
    pub(super) fn push(&mut self, x: Px, y: Px) {
        if self.points.last() == Some(&(x, y)) {
            return;
        }
        self.points.push((x, y));
    }

    /// Emit a `<polyline>` element for accumulated points and clear them.
    pub(super) fn flush(&mut self, buf: &mut SvgBuf, dash: Option<&'static str>) {
        if self.points.len() >= 2 {
            buf.write(&Polyline {
                points: &self.points,
                dash,
            });
        }
        self.points.clear();
    }
}

/// `<polyline>` element with the listed points and optional dasharray.
struct Polyline<'points> {
    points: &'points [(Px, Px)],
    dash: Option<&'static str>,
}

impl WriteSvgOn for Polyline<'_> {
    fn write_svg_on(&self, target: &mut SvgBuf) {
        target.write_literal("<polyline points=\"");
        let mut first = true;
        for (x, y) in self.points {
            if !first {
                target.write_char(' ');
            }
            first = false;
            target.write_px(*x);
            target.write_char(',');
            target.write_px(*y);
        }
        target.write_literal("\"");
        if let Some(value) = self.dash {
            target.write_static_attribute("stroke-dasharray", value);
        }
        target.write_literal("/>");
    }
}
