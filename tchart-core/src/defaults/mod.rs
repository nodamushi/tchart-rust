//! Default values referenced by parser and style construction.
//!
//! See `docs/spec/types.md` §7. Every magic number / default literal that
//! appears in TCML defaults is centralised here so that production code never
//! contains raw constants.

use crate::style::HorizontalAlign;
use crate::units::Px;

/// Default font size (px). See `docs/spec/types.md` §7.
pub(crate) const DEFAULT_FONTSIZE_PX: Px = Px(14.0);

/// Default line-height multiplier applied to the font size.
pub const DEFAULT_LINEHEIGHT_RATIO: f32 = 1.2;

/// Default padding between signal name and waveform (px).
pub(crate) const DEFAULT_NAMEPAD_PX: Px = Px(8.0);

/// Default page margin around the chart (px).
pub(crate) const DEFAULT_PAGE_MARGIN_PX: Px = Px(10.0);

/// Default step width per waveform time unit (px).
/// Controls the x-advance per level character and per `Gap` element.
pub(crate) const DEFAULT_STEP_PX: Px = Px(25.0);

/// Default slant width for all transition kinds (px).
/// Covers `SingleEdge`, `BusOpen`, `BusClose`, and `BusCross` (cross region only).
/// Must be strictly less than `DEFAULT_STEP_PX`.
pub(crate) const DEFAULT_SLANT_PX: Px = Px(5.0);

/// Default vertical gap between adjacent signal rows (px), distributed
/// symmetrically as `gap/2` above and below each row.
/// Known as `h_space` in the spec. Old name `signal_gap` is accepted as an alias by the parser.
pub(crate) const DEFAULT_H_SPACE_PX: Px = Px(10.0);

/// Default stroke width of signal polylines (px).
pub(crate) const DEFAULT_SIGNAL_WIDTH_PX: Px = Px(1.0);

/// Default stroke width of guide lines (px).
pub(crate) const DEFAULT_GUIDE_WIDTH_PX: Px = Px(0.6);

/// Default font family identifier.
pub(crate) const DEFAULT_FONT_FAMILY: &str = "sans-serif";

/// Default signal stroke color (string form, kept for spec test cross-checks).
///
/// Production code uses [`crate::color::Color::BLACK`] directly to avoid
/// `Color::parse(...).expect(...)` at startup.
#[cfg(test)]
pub(crate) const DEFAULT_SIGNAL_COLOR: &str = "black";

/// Default guide line color (string form, kept for spec test cross-checks).
///
/// Production code uses [`crate::color::Color::RED`] directly.
#[cfg(test)]
pub(crate) const DEFAULT_GUIDE_COLOR: &str = "red";

/// Default chart background color.
///
/// Currently only referenced by spec tests; production code applies the
/// row-stripe colors (`bgcolor0`/`bgcolor1`) instead.
#[cfg(test)]
pub(crate) const DEFAULT_BG_COLOR: &str = "none";

/// Default even-row background color (string form, kept for spec test cross-checks).
///
/// Production code uses [`crate::color::Color::NONE`] directly.
#[cfg(test)]
pub(crate) const DEFAULT_BGCOLOR0: &str = "none";

/// Default odd-row background color (string form, kept for spec test cross-checks).
///
/// Production code uses [`crate::color::Color::NONE`] directly.
#[cfg(test)]
pub(crate) const DEFAULT_BGCOLOR1: &str = "none";

/// Default attribute list for `[...]` highlight rectangles.
pub(crate) const DEFAULT_HIGHLIGHT_STYLE: &[(&str, &str)] = &[("fill", "#ff8"), ("stroke", "none")];

/// Tile size (one side of the square tile) for the `dontcare-hatch` SVG pattern (px).
///
/// See `docs/spec/svg-rendering.md` §「`<defs>` (パターン定義)」.
pub(crate) const DEFAULT_DONTCARE_HATCH_TILE_PX: Px = Px(4.0);

/// Default stroke color of the hatch lines when `@dontcare_style color=` is unset.
///
/// See `docs/spec/svg-rendering.md` §「`<defs>` (パターン定義)」.
pub(crate) const DEFAULT_DONTCARE_HATCH_STROKE_COLOR: &str = "#bbb";

/// Stroke width of the hatch lines inside the `dontcare-hatch` SVG pattern (px).
/// Thin relative to the tile size so the underlying waveform remains visible.
///
/// See `docs/spec/svg-rendering.md` §「`<defs>` (パターン定義)」.
pub(crate) const DEFAULT_DONTCARE_HATCH_STROKE_WIDTH_PX: Px = Px(1.0);

/// Default extra line-gap ratio for multi-line signal names (in `lh` units).
///
/// Currently only referenced by spec tests; production code uses
/// `line_height` directly without an additional gap multiplier.
#[cfg(test)]
pub(crate) const DEFAULT_TEXT_LINE_GAP_RATIO: f32 = 0.0;

/// Default horizontal alignment for `@title` rows.
///
/// See `docs/spec/types.md` §7.
pub(crate) const DEFAULT_TITLE_ALIGN: HorizontalAlign = HorizontalAlign::Center;

// Clock triangle marker defaults.

/// Default position of the clock-edge triangle apex along the transition line.
/// `0.5` = midpoint; `0.0` = line root; `1.0` = line tip.
///
/// See `docs/spec/types.md` §7.
pub(crate) const DEFAULT_CLOCKMARK_POSITION: f32 = 0.5;

/// Default height of the clock-edge triangle (along the line direction), in px.
///
/// See `docs/spec/types.md` §7.
pub(crate) const DEFAULT_CLOCKMARK_HEIGHT_PX: Px = Px(7.5);

/// Default base width of the clock-edge triangle (perpendicular to the line), in px.
///
/// When this default value is used (i.e. neither the per-call `mark_width`
/// nor the global `@clockmark_width` is set), `resolve_clock_mark_style`
/// applies a step-linked shrink: `min(DEFAULT_CLOCKMARK_WIDTH_PX, step * 2/3)`.
/// See `docs/spec/tcml-format.md` §「`clockmark_width` の step 連動縮小」.
pub(crate) const DEFAULT_CLOCKMARK_WIDTH_PX: Px = Px(6.0);

// Note: `clockmark_color` defaults to the current `signal_color` at parse time.
// No separate constant is defined for it (see `docs/spec/types.md` §7).

// Signal name overline defaults.

/// Gap between the signal-name cap-top and the overline, in px.
///
/// See `docs/spec/types.md` §7.
pub(crate) const DEFAULT_OVERLINE_GAP_PX: Px = Px(2.0);

/// Stroke width of the signal-name overline, in px.
///
/// See `docs/spec/types.md` §7.
pub(crate) const DEFAULT_OVERLINE_THICKNESS_PX: Px = Px(1.0);

/// Ratio used to estimate cap-height from font size when font metrics are
/// unavailable: `cap_height = font.size * DEFAULT_CAP_HEIGHT_RATIO`.
///
/// See `docs/spec/types.md` §7.
pub(crate) const DEFAULT_CAP_HEIGHT_RATIO: f32 = 0.7;

/// Outline color painted behind arrow label text to ensure readability.
///
/// See `docs/spec/svg-rendering.md` §「矢印 (`arrows`)」§「ラベル」.
pub(crate) const DEFAULT_ARROW_LABEL_OUTLINE_COLOR: &str = "#ffffff";

/// Outline stroke width (px) painted behind arrow label text.
///
/// See `docs/spec/svg-rendering.md` §「矢印 (`arrows`)」§「ラベル」.
pub(crate) const DEFAULT_ARROW_LABEL_OUTLINE_WIDTH_PX: f32 = 2.0;

/// Default stroke color for `@ruler` background guide lines (`#a0a0a0`,
/// string form, kept for spec test cross-checks).
///
/// Production code uses [`crate::color::Color::RULER_DEFAULT`] directly.
/// See `docs/spec/tcml-format.md` §「`@ruler` の詳細」.
#[cfg(test)]
pub(crate) const DEFAULT_RULER_COLOR: &str = "#a0a0a0";

#[cfg(test)]
mod tests;
