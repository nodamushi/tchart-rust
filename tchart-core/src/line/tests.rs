//! Unit tests for `line`.

use super::{
    LevelRun, LevelShape, Line, LineContent, SignalDecorations, SignalGeometry, SignalLevel,
    SignalRow, SkipRow, TitleRow, Transition, TransitionKind, Waveform, WaveformElement,
};
use crate::style::{ChartStyle, HorizontalAlign, LayoutParams, SignalRowStyle, TitleStyle};
use crate::text::{SignalName, UserText};
use crate::units::Length;

#[test]
fn signal_level_shape_covers_low() {
    assert_eq!(SignalLevel::Low.into_shape(), LevelShape::Single);
}

#[test]
fn signal_level_shape_covers_high() {
    assert_eq!(SignalLevel::High.into_shape(), LevelShape::Single);
}

#[test]
fn signal_level_shape_covers_hiz() {
    assert_eq!(SignalLevel::HiZ.into_shape(), LevelShape::Single);
}

#[test]
fn signal_level_shape_covers_bus() {
    assert_eq!(SignalLevel::Bus.into_shape(), LevelShape::Double);
}

#[test]
fn signal_level_shape_covers_dontcare_low() {
    assert_eq!(
        SignalLevel::DontCareAlongLow.into_shape(),
        LevelShape::FillSingle
    );
}

#[test]
fn signal_level_shape_covers_dontcare_high() {
    assert_eq!(
        SignalLevel::DontCareAlongHigh.into_shape(),
        LevelShape::FillSingle
    );
}

#[test]
fn signal_level_shape_covers_dontcare_hiz() {
    assert_eq!(
        SignalLevel::DontCareAlongHiZ.into_shape(),
        LevelShape::FillSingle
    );
}

#[test]
fn signal_level_shape_covers_dontcare_bus() {
    assert_eq!(
        SignalLevel::DontCareAlongBus.into_shape(),
        LevelShape::FillDouble
    );
}

#[test]
fn waveform_pushes_elements() {
    let mut waveform = Waveform::default();
    waveform.push(WaveformElement::Level(LevelRun::new(SignalLevel::Low, 2)));
    waveform.push(WaveformElement::Gap);
    assert_eq!(waveform.len(), 2);
}

#[test]
fn level_run_constructs() {
    let run = LevelRun::new(SignalLevel::Bus, 4);
    assert_eq!(run.level(), SignalLevel::Bus);
    assert_eq!(run.units(), 4);
}

#[test]
fn transition_kind_eq() {
    assert_eq!(TransitionKind::SingleEdge, TransitionKind::SingleEdge);
    assert_ne!(TransitionKind::BusOpen, TransitionKind::BusClose);
}

#[test]
fn transition_constructs() {
    let trans = Transition::new(
        SignalLevel::Low,
        SignalLevel::High,
        TransitionKind::SingleEdge,
        None,
    );
    assert_eq!(trans.source, SignalLevel::Low);
    assert_eq!(trans.kind, TransitionKind::SingleEdge);
}

#[test]
fn signal_row_constructs() {
    let style = ChartStyle::default();
    let row = SignalRow::new(
        SignalGeometry::default(),
        SignalName::parse("clk").expect("name"),
        Waveform::default(),
        SignalRowStyle::new(
            style.default_signal_style().clone(),
            style.default_label_style().clone(),
        ),
        SignalDecorations::default(),
        LayoutParams::default(),
    );
    assert_eq!(row.name().as_str(), "clk");
    assert!(!row.decorations().is_name_overline());
}

#[test]
fn skip_row_constructs() {
    let skip = SkipRow::new(Length::new_lh(2.5).expect("length"));
    assert_eq!(skip.amount, Length::Lh(2.5));
}

#[test]
fn title_row_constructs() {
    let style = ChartStyle::default();
    let title = TitleRow::new(
        UserText::parse("Hello").expect("text"),
        TitleStyle::new(
            style.canvas().font().clone(),
            HorizontalAlign::Center,
            style.default_label_style().color(),
        ),
    );
    assert_eq!(title.style.align(), HorizontalAlign::Center);
}

#[test]
fn line_with_skip_content() {
    let skip = SkipRow::new(Length::new_px(20.0).expect("length"));
    let line = Line::new(LineContent::Skip(skip), None);
    match line.content {
        LineContent::Skip(value) => assert_eq!(value.amount, Length::Px(20.0)),
        _ => panic!("expected skip"),
    }
}
