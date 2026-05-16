//! tchart-core — TCML parser, layout engine, and SVG renderer core library.
//!
//! Platform-independent pure computation library. Font metrics are injected
//! externally via the `FontMetrics` trait when the layout module is available.

pub(crate) mod anchor;
pub(crate) mod arrow;
pub(crate) mod clock;
pub(crate) mod color;
pub mod defaults;
pub mod document;
pub mod errors;
pub(crate) mod geometry;
pub mod layout;
pub(crate) mod line;
pub mod parser;
pub(crate) mod style;
pub mod svg;
pub mod text;
pub mod units;
pub mod wavedrom;
