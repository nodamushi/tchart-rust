//! Hatch pattern table for DontCare fills.
//!
//! Each row's `@dontcare_color` snapshot is interned into a [`DontcareHatchPatternTable`]
//! during the waveform pass. The table assigns 1-origin sequential ids to unique colors
//! (deduping equal colors). The `<defs>` writer iterates the table and emits one
//! `<pattern id="dontcare-hatch-N">` per entry; each `<rect>` / `<polygon>` carries the
//! id and references the matching pattern via `fill="url(#dontcare-hatch-N)"`.

use std::fmt;

use crate::color::Color;

/// Identifier for one entry in [`DontcareHatchPatternTable`].
///
/// Formats as the full SVG pattern id string `dontcare-hatch-N` via [`fmt::Display`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct DontcareHatchPatternId(u32);

impl fmt::Display for DontcareHatchPatternId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "dontcare-hatch-{}", self.0)
    }
}

/// One resolved `<pattern>` definition: its id and the hatch line stroke color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DontcareHatchPattern {
    id: DontcareHatchPatternId,
    stroke_color: Color,
}

impl DontcareHatchPattern {
    pub(crate) fn id(&self) -> DontcareHatchPatternId {
        self.id
    }

    pub(crate) fn stroke_color(&self) -> Color {
        self.stroke_color
    }
}

/// Table of hatch patterns used by a single chart, in first-insertion order.
///
/// Same color values share one id (see [`Self::insert_color`]).
#[derive(Debug, Default, Clone)]
pub(crate) struct DontcareHatchPatternTable {
    patterns: Vec<DontcareHatchPattern>,
}

impl DontcareHatchPatternTable {
    /// Returns the id assigned to `color`. Re-inserting the same color returns
    /// the existing id (no duplicates).
    pub(crate) fn insert_color(&mut self, color: Color) -> DontcareHatchPatternId {
        if let Some(pattern) = self
            .patterns
            .iter()
            .find(|pattern| pattern.stroke_color == color)
        {
            return pattern.id;
        }
        let id = DontcareHatchPatternId(self.patterns.len() as u32 + 1);
        self.patterns.push(DontcareHatchPattern {
            id,
            stroke_color: color,
        });
        id
    }

    pub(crate) fn as_slice(&self) -> &[DontcareHatchPattern] {
        &self.patterns
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }
}
