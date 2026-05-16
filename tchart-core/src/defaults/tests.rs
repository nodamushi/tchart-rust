use super::{
    DEFAULT_BG_COLOR, DEFAULT_BGCOLOR0, DEFAULT_BGCOLOR1, DEFAULT_CLOCKMARK_HEIGHT_PX,
    DEFAULT_CLOCKMARK_WIDTH_PX, DEFAULT_FONT_FAMILY, DEFAULT_FONTSIZE_PX, DEFAULT_GUIDE_COLOR,
    DEFAULT_GUIDE_WIDTH_PX, DEFAULT_H_SPACE_PX, DEFAULT_HIGHLIGHT_STYLE, DEFAULT_LINEHEIGHT_RATIO,
    DEFAULT_NAMEPAD_PX, DEFAULT_PAGE_MARGIN_PX, DEFAULT_RULER_COLOR, DEFAULT_SIGNAL_COLOR,
    DEFAULT_SIGNAL_WIDTH_PX, DEFAULT_SLANT_PX, DEFAULT_STEP_PX, DEFAULT_TEXT_LINE_GAP_RATIO,
    DEFAULT_TITLE_ALIGN,
};
use crate::color::Color;
use crate::style::HorizontalAlign;
use crate::text::FontFamily;
use crate::units::Px;

#[test]
fn pixel_defaults_have_expected_values() {
    assert_eq!(DEFAULT_FONTSIZE_PX, Px(14.0));
    assert_eq!(DEFAULT_NAMEPAD_PX, Px(8.0));
    assert_eq!(DEFAULT_PAGE_MARGIN_PX, Px(10.0));
    assert_eq!(DEFAULT_STEP_PX, Px(25.0));
    assert_eq!(DEFAULT_SLANT_PX, Px(5.0));
    assert_eq!(DEFAULT_H_SPACE_PX, Px(10.0));
    assert_eq!(DEFAULT_SIGNAL_WIDTH_PX, Px(1.0));
    assert_eq!(DEFAULT_GUIDE_WIDTH_PX, Px(0.6));
}

#[test]
fn ratio_defaults_have_expected_values() {
    assert!((DEFAULT_LINEHEIGHT_RATIO - 1.2).abs() < f32::EPSILON);
    assert!((DEFAULT_TEXT_LINE_GAP_RATIO - 0.0).abs() < f32::EPSILON);
}

#[test]
fn font_family_default_parses_as_font_family() {
    let family = FontFamily::parse(DEFAULT_FONT_FAMILY).expect("default family parses");
    assert_eq!(family.as_str(), "sans-serif");
}

#[test]
fn color_defaults_parse_as_colors() {
    Color::parse(DEFAULT_SIGNAL_COLOR).expect("signal color parses");
    Color::parse(DEFAULT_GUIDE_COLOR).expect("guide color parses");
    Color::parse(DEFAULT_BG_COLOR).expect("bg color parses");
    Color::parse(DEFAULT_BGCOLOR0).expect("bgcolor0 parses");
    Color::parse(DEFAULT_BGCOLOR1).expect("bgcolor1 parses");
    Color::parse(DEFAULT_RULER_COLOR).expect("ruler color parses");
}

#[test]
fn default_ruler_color_matches_color_constant() {
    let parsed = Color::parse(DEFAULT_RULER_COLOR).expect("ruler color parses");
    assert_eq!(parsed, Color::RULER_DEFAULT);
}

#[test]
fn highlight_style_table_is_non_empty() {
    assert!(!DEFAULT_HIGHLIGHT_STYLE.is_empty());
}

// DEFAULT_TITLE_ALIGN must be Center.
#[test]
fn default_title_align_is_center() {
    assert_eq!(DEFAULT_TITLE_ALIGN, HorizontalAlign::Center);
}

#[test]
fn clockmark_default_width_is_6_and_height_is_7_5() {
    // Per docs/spec/tcml-format.md global-settings table.
    assert_eq!(DEFAULT_CLOCKMARK_WIDTH_PX, Px(6.0));
    assert_eq!(DEFAULT_CLOCKMARK_HEIGHT_PX, Px(7.5));
}
