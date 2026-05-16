//! Chart styling types.
//!
//! See `docs/spec/types.md` §4.
//!
//! Styles are grouped into small (≤ 5-field) sub-structs so that callers can
//! pass exactly the slice of style they care about.

mod canvas;
mod label;
mod layout;
mod row;
mod signal;
mod svg_attrs;
mod title;
mod top;

pub use canvas::CanvasStyle;
pub use top::ChartStyle;

pub(crate) use label::{HorizontalAlign, LabelStyle};
pub(crate) use layout::LayoutParams;
pub(crate) use row::SignalRowStyle;
pub(crate) use signal::GuideStyle;
pub(crate) use svg_attrs::SvgAttrList;
pub(crate) use title::TitleStyle;

#[cfg(test)]
mod tests;
