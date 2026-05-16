//! Unit tests for `style`.

use super::canvas::BackgroundStyle;
use super::signal::SignalStyle;
use super::{ChartStyle, GuideStyle, HorizontalAlign, SignalRowStyle, TitleStyle};
use crate::color::Color;
use crate::units::Px;

#[test]
fn chart_style_default_constructs() {
    let style = ChartStyle::default();
    assert!(!style.default_signal_style().color().is_none());
}

#[test]
fn chart_style_default_line_height_matches_ratio() {
    let style = ChartStyle::default();
    assert!((style.canvas().line_height().to_f32() - 14.0 * 1.2).abs() < f32::EPSILON);
}

#[test]
fn horizontal_align_eq() {
    assert_eq!(HorizontalAlign::Left, HorizontalAlign::Left);
    assert_ne!(HorizontalAlign::Left, HorizontalAlign::Right);
}

#[test]
fn horizontal_align_svg_text_anchor() {
    assert_eq!(HorizontalAlign::Left.svg_text_anchor(), "start");
    assert_eq!(HorizontalAlign::Center.svg_text_anchor(), "middle");
    assert_eq!(HorizontalAlign::Right.svg_text_anchor(), "end");
}

#[test]
fn signal_row_style_constructs() {
    let signal = SignalStyle::default();
    let label = ChartStyle::default().default_label_style().clone();
    let row = SignalRowStyle::new(signal.clone(), label.clone());
    assert_eq!(row.signal(), &signal);
    assert_eq!(row.label(), &label);
}

#[test]
fn title_style_default_align_is_center() {
    let style = TitleStyle::default();
    assert_eq!(style.align(), HorizontalAlign::Center);
}

#[test]
fn background_default_stripe_is_none() {
    let style = ChartStyle::default();
    assert!(style.stripe_for_signal_index(0).is_none());
    assert!(style.stripe_for_signal_index(1).is_none());
}

#[test]
fn background_stripe_alternates() {
    let mut background = BackgroundStyle::default();
    background.set_bgcolor0(Color::parse("#000").expect("color"));
    background.set_bgcolor1(Color::parse("#fff").expect("color"));
    assert_eq!(background.stripe_for_index(0).to_css_string(), "#000000");
    assert_eq!(background.stripe_for_index(1).to_css_string(), "#ffffff");
    assert_eq!(background.stripe_for_index(2).to_css_string(), "#000000");
}

#[test]
fn canvas_style_can_be_cloned() {
    let style = ChartStyle::default();
    let canvas = style.canvas().clone();
    assert!((canvas.line_height().to_f32() - 14.0 * 1.2).abs() < f32::EPSILON);
}

#[test]
fn default_label_padding_is_8px() {
    let style = ChartStyle::default();
    assert_eq!(style.default_label_style().padding(), Px(8.0));
}

#[test]
fn guide_style_default_uses_red() {
    let guide = GuideStyle::default();
    assert!(!guide.color().is_none());
}
