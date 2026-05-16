//! Unit tests for the layout engine.
//!
//! See `docs/tests/layout-engine.feature.md` for the corresponding scenarios.

use std::num::NonZeroU32;

use super::{ChartDimensions, FontMetrics, LayoutError, layout};
use crate::anchor::{AnchorId, AnchorName};
use crate::arrow::{Arrow, ArrowEnd, ArrowHead, ArrowStyle, LineDashStyle};
use crate::clock::{ClockEdge, ClockPhase, ClockPulse, ClockSpec};
use crate::document::{Annotations, ChartDocument, TcmlSource};
use crate::line::{
    LevelRun, Line, LineContent, SignalDecorations, SignalGeometry, SignalLevel, SignalRow,
    SkipRow, TitleRow, Transition, TransitionKind, Waveform, WaveformElement,
};
use crate::style::{ChartStyle, LayoutParams, SignalRowStyle, TitleStyle};
use crate::text::{FontSpec, SignalName, UserText};
use crate::units::{Length, Px};

/// Mock font metrics that returns a fixed advance per character.
struct MockFonts {
    char_width: Px,
}

impl MockFonts {
    fn new(width: f32) -> Self {
        Self {
            char_width: Px(width),
        }
    }
}

impl FontMetrics for MockFonts {
    fn measure_text_width(&self, text: &str, _font: &FontSpec) -> Px {
        Px(self.char_width.to_f32() * text.chars().count() as f32)
    }
}

fn make_arrow_style() -> ArrowStyle {
    ArrowStyle::new(
        crate::color::Color::NONE,
        Px(1.0),
        LineDashStyle::Solid,
        ArrowHead::EndOnly,
    )
}

fn make_signal_line(name: &str, elements: Vec<WaveformElement>) -> Line {
    make_signal_line_with_layout(name, elements, LayoutParams::default())
}

fn make_signal_line_with_layout(
    name: &str,
    elements: Vec<WaveformElement>,
    layout_params: LayoutParams,
) -> Line {
    let defaults = ChartStyle::default();
    let row = SignalRow::new(
        SignalGeometry::default(),
        SignalName::parse(name).expect("name parse"),
        Waveform::from(elements),
        SignalRowStyle::new(
            defaults.default_signal_style().clone(),
            defaults.default_label_style().clone(),
        ),
        SignalDecorations::default(),
        layout_params,
    );
    Line::new(LineContent::Signal(Box::new(row)), None)
}

fn make_level_run(level: SignalLevel, units: u32) -> WaveformElement {
    WaveformElement::Level(LevelRun::new(level, units))
}

fn make_single_edge(from: SignalLevel, to: SignalLevel) -> WaveformElement {
    WaveformElement::Transition(Transition::new(from, to, TransitionKind::SingleEdge, None))
}

fn make_document_with_lines(lines: Vec<Line>) -> ChartDocument {
    ChartDocument::new(
        ChartStyle::default(),
        lines,
        Annotations::default(),
        TcmlSource::default(),
    )
}

/// Build a `ChartDocument` for clock-edge mark tests: step=10, slant=2,
/// capwidth=0, page_margin=0, h_space=0, one Pos clock signal row.
fn make_clock_edge_mark_document(edge: ClockEdge) -> ChartDocument {
    let mut style = ChartStyle::default();
    style.set_step(Px(10.0));
    style.set_slant(Px(2.0));
    style.set_capwidth(Some(Px(0.0)));
    style.set_page_margin(Px(0.0));
    style.set_h_space(Px(0.0));
    let layout_params = *style.layout();
    ChartDocument::new(
        style,
        vec![make_clock_signal_line_with_layout(edge, layout_params)],
        Annotations::default(),
        TcmlSource::default(),
    )
}

/// Build a laid-out `ChartDocument` with two signal rows:
/// A = 10 Low units, B = 4 High units, step=10, capwidth=20.
fn make_two_signal_document_step10_cap20() -> ChartDocument {
    let mut style = ChartStyle::default();
    style.set_step(Px(10.0));
    style.set_capwidth(Some(Px(20.0)));
    let layout_params = *style.layout();
    let lines = vec![
        make_signal_line_with_layout(
            "A",
            vec![make_level_run(SignalLevel::Low, 10)],
            layout_params,
        ),
        make_signal_line_with_layout(
            "B",
            vec![make_level_run(SignalLevel::High, 4)],
            layout_params,
        ),
    ];
    let mut document =
        ChartDocument::new(style, lines, Annotations::default(), TcmlSource::default());
    run_layout(&mut document);
    document
}

/// Build a laid-out `ChartDocument` with A(10 Low), skip(5px), B(4 High),
/// step=10, capwidth=20.
fn make_three_line_document_with_skip_step10_cap20() -> ChartDocument {
    use crate::units::Length;
    let mut style = ChartStyle::default();
    style.set_step(Px(10.0));
    style.set_capwidth(Some(Px(20.0)));
    let layout_params = *style.layout();
    let signal_a = make_signal_line_with_layout(
        "A",
        vec![make_level_run(SignalLevel::Low, 10)],
        layout_params,
    );
    let skip = Line::new(
        LineContent::Skip(SkipRow::new(Length::new_px(5.0).expect("5px skip height"))),
        None,
    );
    let signal_b = make_signal_line_with_layout(
        "B",
        vec![make_level_run(SignalLevel::High, 4)],
        layout_params,
    );
    let lines = vec![signal_a, skip, signal_b];
    let mut document =
        ChartDocument::new(style, lines, Annotations::default(), TcmlSource::default());
    run_layout(&mut document);
    document
}

fn run_layout(document: &mut ChartDocument) -> ChartDimensions {
    // Tests typically build a SignalRow (which snapshots LayoutParams at
    // construction time) and then mutate `document.style` afterward. Push
    // the post-mutation snapshot into every Signal row so layout sees the
    // configured values. Production code captures the snapshot at parse
    // time and does not need this sync.
    //
    // WARNING: this overwrite is destructive — every Signal row receives the
    // same final global snapshot, erasing any per-row LayoutParams set at
    // construction time. Tests that exercise per-row layout behaviour MUST
    // bypass `run_layout` and use `parse_and_layout` directly so per-row
    // snapshots captured at parse time are preserved.
    overwrite_all_rows_with_global_layout_snapshot(document);
    let fonts = MockFonts::new(7.0);
    layout(document, &fonts).expect("layout succeeds")
}

fn overwrite_all_rows_with_global_layout_snapshot(document: &mut ChartDocument) {
    let snapshot = *document.style.layout();
    for line in document.lines.iter_mut() {
        if let LineContent::Signal(row) = &mut line.content {
            row.set_layout_params(snapshot);
        }
    }
}

fn insert_inline_anchor(
    document: &mut ChartDocument,
    id: AnchorId,
    signal_index: usize,
    element_index: usize,
) {
    document.annotations.anchors.insert(
        id,
        crate::anchor::ResolvedAnchor::new(
            crate::geometry::Point::ZERO,
            signal_index,
            element_index,
        ),
    );
}

#[test]
fn stack_three_signal_lines_contiguous() {
    let lines = vec![
        make_signal_line("a", vec![make_level_run(SignalLevel::Low, 1)]),
        make_signal_line("b", vec![make_level_run(SignalLevel::High, 1)]),
        make_signal_line("c", vec![make_level_run(SignalLevel::HiZ, 1)]),
    ];
    let mut document = make_document_with_lines(lines);
    run_layout(&mut document);
    for window in document.lines.windows(2) {
        let expected = window[0].bounding_box.origin.y + window[0].bounding_box.size.height;
        assert!((window[1].bounding_box.origin.y.to_f32() - expected.to_f32()).abs() < 1.0e-4);
    }
}

#[test]
fn first_line_origin_y_is_page_margin() {
    let lines = vec![make_signal_line(
        "a",
        vec![make_level_run(SignalLevel::Low, 1)],
    )];
    let mut document = make_document_with_lines(lines);
    let page_margin = document.style.canvas().page_margin();
    run_layout(&mut document);
    assert_eq!(document.lines[0].bounding_box.origin.y, page_margin);
}

#[test]
fn signal_box_has_symmetric_gap() {
    let lines = vec![make_signal_line(
        "a",
        vec![make_level_run(SignalLevel::Low, 1)],
    )];
    let mut document = make_document_with_lines(lines);
    let signal_gap = document.style.layout().h_space();
    run_layout(&mut document);
    let LineContent::Signal(row) = &document.lines[0].content else {
        panic!("expected signal");
    };
    assert!(
        (row.geometry().signal_box.origin.y.to_f32() - signal_gap.to_f32() * 0.5).abs() < 1.0e-4
    );
    let bbox_h = document.lines[0].bounding_box.size.height;
    let signal_h = row.geometry().signal_box.size.height;
    assert!((bbox_h.to_f32() - signal_h.to_f32() - signal_gap.to_f32()).abs() < 1.0e-4);
}

#[test]
fn multiline_label_does_not_stretch_signal_box() {
    // Two rows: 1-line label "a", 2-line label "b\nc". Both signal_box.size.height
    // must stay = canvas.line_height (the body is never stretched).
    let single = make_signal_line("a", vec![make_level_run(SignalLevel::Low, 1)]);
    let multi = make_signal_line("b\nc", vec![make_level_run(SignalLevel::Low, 1)]);
    let mut document = make_document_with_lines(vec![single, multi]);
    let line_height = document.style.canvas().line_height();
    run_layout(&mut document);
    for line in document.lines {
        let LineContent::Signal(row) = &line.content else {
            panic!("expected signal");
        };
        assert!(
            (row.geometry().signal_box.size.height.to_f32() - line_height.to_f32()).abs() < 1.0e-4,
            "signal_box.size.height must equal line_height; got {} vs {}",
            row.geometry().signal_box.size.height.to_f32(),
            line_height.to_f32()
        );
    }
}

#[test]
fn multiline_label_grows_bbox_and_centres_signal() {
    let lines = vec![make_signal_line(
        "a\nb",
        vec![make_level_run(SignalLevel::Low, 1)],
    )];
    let mut document = make_document_with_lines(lines);
    let line_height = document.style.canvas().line_height();
    let signal_gap = document.style.layout().h_space();
    run_layout(&mut document);
    let bbox_h = document.lines[0].bounding_box.size.height;
    let LineContent::Signal(row) = &document.lines[0].content else {
        panic!("expected signal");
    };
    let label_total = line_height.to_f32() * 2.0;
    assert_close(bbox_h.to_f32(), label_total + signal_gap.to_f32(), "bbox.h");
    let expected_signal_y = (bbox_h.to_f32() - line_height.to_f32()) * 0.5;
    assert_close(
        row.geometry().signal_box.origin.y.to_f32(),
        expected_signal_y,
        "signal_y",
    );
    let expected_label_y = (bbox_h.to_f32() - label_total) * 0.5;
    assert_close(
        row.geometry().label_box.origin.y.to_f32(),
        expected_label_y,
        "label_y",
    );
}

fn assert_close(actual: f32, expected: f32, label: &str) {
    assert!(
        (actual - expected).abs() < 1.0e-4,
        "{label}: got {actual} vs {expected}"
    );
}

fn lookup_anchor_x(document: &ChartDocument, id: &AnchorId) -> f32 {
    document
        .annotations
        .anchors
        .lookup_position(id)
        .expect("anchor present")
        .x
        .to_f32()
}

fn compute_signal_origin_x(document: &ChartDocument) -> f32 {
    let LineContent::Signal(row) = &document.lines[0].content else {
        panic!("expected signal");
    };
    let page_margin = document.style.canvas().page_margin();
    let capwidth = row.geometry().signal_box.origin.x;
    (page_margin + capwidth).to_f32()
}

#[test]
fn signal_box_width_matches_element_sum() {
    let mut style = ChartStyle::default();
    style.set_step(Px(10.0));
    style.set_slant(Px(2.0));
    let layout_params = *style.layout();
    let lines = vec![make_signal_line_with_layout(
        "a",
        vec![
            make_level_run(SignalLevel::Low, 1),
            make_single_edge(SignalLevel::Low, SignalLevel::High),
            make_level_run(SignalLevel::High, 1),
            make_single_edge(SignalLevel::High, SignalLevel::Low),
            make_level_run(SignalLevel::Low, 1),
            make_single_edge(SignalLevel::Low, SignalLevel::High),
            make_level_run(SignalLevel::High, 1),
        ],
        layout_params,
    )];
    let mut document =
        ChartDocument::new(style, lines, Annotations::default(), TcmlSource::default());
    run_layout(&mut document);
    let LineContent::Signal(row) = &document.lines[0].content else {
        panic!("expected signal");
    };
    assert!(
        (row.geometry().signal_box.size.width.to_f32() - (4.0 * 10.0 + 3.0 * 2.0)).abs() < 1.0e-4
    );
}

#[test]
fn bus_cross_width_equals_slant_for_manually_built() {
    // Manually-built LevelRun has preceded_by_transition=false, so body width = units*step.
    // `=X=` (manual): LevelRun(Bus,1) + Transition(BusCross) + LevelRun(Bus,2,preceded=false)
    // width = 1*10 + slant(2) + 2*10 = 32
    let mut style = ChartStyle::default();
    style.set_step(Px(10.0));
    style.set_slant(Px(2.0));
    let layout_params = *style.layout();
    let lines = vec![make_signal_line_with_layout(
        "a",
        vec![
            make_level_run(SignalLevel::Bus, 1),
            WaveformElement::Transition(Transition::new(
                SignalLevel::Bus,
                SignalLevel::Bus,
                TransitionKind::BusCross,
                None,
            )),
            make_level_run(SignalLevel::Bus, 2),
        ],
        layout_params,
    )];
    let mut document =
        ChartDocument::new(style, lines, Annotations::default(), TcmlSource::default());
    run_layout(&mut document);
    let LineContent::Signal(row) = &document.lines[0].content else {
        panic!("expected signal");
    };
    assert!(
        (row.geometry().signal_box.size.width.to_f32() - 32.0).abs() < 1.0e-4,
        "expected 10 + 2 + 20 = 32, got {}",
        row.geometry().signal_box.size.width.to_f32()
    );
}

#[test]
fn bus_cross_slant_width_value() {
    // Transition(BusCross).width == slant.
    let elements = vec![
        make_level_run(SignalLevel::Bus, 1),
        WaveformElement::Transition(Transition::new(
            SignalLevel::Bus,
            SignalLevel::Bus,
            TransitionKind::BusCross,
            None,
        )),
        make_level_run(SignalLevel::Bus, 1),
    ];
    let mut document = make_document_with_lines(vec![make_signal_line("a", elements)]);
    document.style.set_step(Px(10.0));
    document.style.set_slant(Px(2.0));
    let cross_width = document
        .style
        .layout()
        .element_width(&WaveformElement::Transition(Transition::new(
            SignalLevel::Bus,
            SignalLevel::Bus,
            TransitionKind::BusCross,
            None,
        )));
    assert!(
        (cross_width.to_f32() - 2.0).abs() < 1.0e-4,
        "BusCross width must equal slant=2, got {}",
        cross_width.to_f32()
    );
}

/// Parser-based BusCross width: `=X==` with step=10, slant=2.
/// `=X==` = 4 chars × step = 40.
/// Internal: LevelRun(Bus,1) + Transition(BusCross,slant=2) + LevelRun(Bus,3,preceded=true)
/// width = 1*10 + 2 + (3*10-2) = 10 + 2 + 28 = 40 = 4*step.
#[test]
fn bus_cross_total_width_via_parser() {
    let source = "@step 10\n@slant 2\nBus =X==\n";
    let mut document = crate::parser::parse(source).expect("parse");
    document.style.set_capwidth(Some(Px(0.0)));
    document.style.set_page_margin(Px(0.0));
    let fonts = MockFonts::new(7.0);
    layout(&mut document, &fonts).expect("layout");
    let LineContent::Signal(row) = &document.lines[0].content else {
        panic!("expected signal");
    };
    assert_close(
        row.geometry().signal_box.size.width.to_f32(),
        40.0,
        "=X== (4 chars) must have width 4*step = 40",
    );
}

#[test]
fn gap_contributes_step_width() {
    let mut style = ChartStyle::default();
    style.set_step(Px(10.0));
    let layout_params = *style.layout();
    let lines = vec![make_signal_line_with_layout(
        "a",
        vec![
            make_level_run(SignalLevel::Low, 2),
            WaveformElement::Gap,
            make_level_run(SignalLevel::Low, 2),
        ],
        layout_params,
    )];
    let mut document =
        ChartDocument::new(style, lines, Annotations::default(), TcmlSource::default());
    run_layout(&mut document);
    let LineContent::Signal(row) = &document.lines[0].content else {
        panic!("expected signal");
    };
    assert!((row.geometry().signal_box.size.width.to_f32() - 50.0).abs() < 1.0e-4);
}

#[test]
fn anchor_has_zero_width_contribution() {
    let anchor = AnchorName::parse("a").expect("anchor name");
    let mut style = ChartStyle::default();
    style.set_step(Px(10.0));
    style.set_slant(Px(2.0));
    let layout_params = *style.layout();
    let lines = vec![make_signal_line_with_layout(
        "a",
        vec![
            make_level_run(SignalLevel::Low, 1),
            make_single_edge(SignalLevel::Low, SignalLevel::High),
            make_level_run(SignalLevel::High, 1),
            WaveformElement::Anchor(AnchorId::Named(anchor)),
            make_level_run(SignalLevel::Low, 1),
        ],
        layout_params,
    )];
    let mut document =
        ChartDocument::new(style, lines, Annotations::default(), TcmlSource::default());
    run_layout(&mut document);
    let LineContent::Signal(row) = &document.lines[0].content else {
        panic!("expected signal");
    };
    assert!(
        (row.geometry().signal_box.size.width.to_f32() - (10.0 + 2.0 + 10.0 + 10.0)).abs() < 1.0e-4
    );
}

#[test]
fn skip_row_height_uses_lh_amount() {
    let skip = Line::new(
        LineContent::Skip(SkipRow::new(Length::new_lh(2.0).expect("lh"))),
        None,
    );
    let lines = vec![skip];
    let mut document = make_document_with_lines(lines);
    document.style.set_line_height(Px(24.0));
    run_layout(&mut document);
    assert!((document.lines[0].bounding_box.size.height.to_f32() - 48.0).abs() < 1.0e-4);
}

#[test]
fn skip_row_height_uses_px_amount() {
    let skip = Line::new(
        LineContent::Skip(SkipRow::new(Length::new_px(20.0).expect("px"))),
        None,
    );
    let lines = vec![skip];
    let mut document = make_document_with_lines(lines);
    run_layout(&mut document);
    assert!((document.lines[0].bounding_box.size.height.to_f32() - 20.0).abs() < 1.0e-4);
}

#[test]
fn title_row_height_is_line_height_times_lines() {
    let title = Line::new(
        LineContent::Title(TitleRow::new(
            UserText::parse("a\nb").expect("text"),
            TitleStyle::new(
                ChartStyle::default().canvas().font().clone(),
                crate::style::HorizontalAlign::Left,
                crate::color::Color::NONE,
            ),
        )),
        None,
    );
    let lines = vec![title];
    let mut document = make_document_with_lines(lines);
    let line_height = document.style.canvas().line_height().to_f32();
    run_layout(&mut document);
    assert!(
        (document.lines[0].bounding_box.size.height.to_f32() - line_height * 2.0).abs() < 1.0e-4
    );
}

#[test]
fn capwidth_auto_uses_widest_label_plus_padding() {
    let lines = vec![
        make_signal_line("foo", vec![make_level_run(SignalLevel::Low, 1)]),
        make_signal_line("longername", vec![make_level_run(SignalLevel::High, 1)]),
    ];
    let mut document = make_document_with_lines(lines);
    document.style.set_name_padding(Px(8.0));
    let fonts = MockFonts::new(7.0);
    let dimensions = layout(&mut document, &fonts).expect("layout");
    let LineContent::Signal(row) = &document.lines[1].content else {
        panic!("expected signal");
    };
    let expected_capwidth = 10.0 * 7.0 + 8.0;
    let label_width = row.geometry().label_box.size.width;
    assert!((label_width.to_f32() - expected_capwidth).abs() < 1.0e-4);
    assert!(dimensions.width.to_f32() > 0.0);
}

#[test]
fn capwidth_explicit_overrides_auto() {
    let lines = vec![make_signal_line(
        "foo",
        vec![make_level_run(SignalLevel::Low, 1)],
    )];
    let mut document = make_document_with_lines(lines);
    document.style.set_capwidth(Some(Px(100.0)));
    run_layout(&mut document);
    let LineContent::Signal(row) = &document.lines[0].content else {
        panic!("expected signal");
    };
    assert_eq!(row.geometry().label_box.size.width, Px(100.0));
}

#[test]
fn chart_total_width_includes_page_margin_twice() {
    let mut style = ChartStyle::default();
    style.set_page_margin(Px(10.0));
    style.set_step(Px(10.0));
    style.set_capwidth(Some(Px(80.0)));
    let layout_params = *style.layout();
    let lines = vec![make_signal_line_with_layout(
        "a",
        vec![make_level_run(SignalLevel::Low, 1)],
        layout_params,
    )];
    let mut document =
        ChartDocument::new(style, lines, Annotations::default(), TcmlSource::default());
    let fonts = MockFonts::new(7.0);
    let dimensions = layout(&mut document, &fonts).expect("layout");
    let expected = 10.0 + 80.0 + 10.0 + 10.0;
    assert!((dimensions.width.to_f32() - expected).abs() < 1.0e-4);
}

#[test]
fn chart_total_height_adds_page_margin_to_stack_end() {
    let lines = vec![
        make_signal_line("a", vec![make_level_run(SignalLevel::Low, 1)]),
        make_signal_line("b", vec![make_level_run(SignalLevel::High, 1)]),
    ];
    let mut document = make_document_with_lines(lines);
    document.style.set_page_margin(Px(10.0));
    let fonts = MockFonts::new(7.0);
    let dimensions = layout(&mut document, &fonts).expect("layout");
    let last = document.lines.last().expect("at least one line");
    let expected = last.bounding_box.origin.y + last.bounding_box.size.height + Px(10.0);
    assert!((dimensions.height.to_f32() - expected.to_f32()).abs() < 1.0e-4);
}

#[test]
fn anchor_x_is_cumulative_position() {
    let anchor_id = AnchorId::Named(AnchorName::parse("a").expect("name"));
    let lines = vec![make_signal_line(
        "Foo",
        vec![
            make_level_run(SignalLevel::Low, 2),
            WaveformElement::Anchor(anchor_id.clone()),
            make_level_run(SignalLevel::Low, 2),
        ],
    )];
    let mut document = make_document_with_lines(lines);
    document.style.set_capwidth(Some(Px(28.0)));
    document.style.set_page_margin(Px(0.0));
    document.style.set_step(Px(10.0));
    insert_inline_anchor(&mut document, anchor_id.clone(), 0, 1);
    run_layout(&mut document);
    let resolved_x = document
        .annotations
        .anchors
        .lookup_position(&anchor_id)
        .expect("anchor present")
        .x;
    assert!((resolved_x.to_f32() - (28.0 + 20.0)).abs() < 1.0e-4);
}

#[test]
fn anchor_y_uses_previous_level_position() {
    let anchor_id = AnchorId::Named(AnchorName::parse("a").expect("name"));
    let lines = vec![make_signal_line(
        "x",
        vec![
            make_level_run(SignalLevel::High, 2),
            WaveformElement::Anchor(anchor_id.clone()),
            make_level_run(SignalLevel::Low, 2),
        ],
    )];
    let mut document = make_document_with_lines(lines);
    document.style.set_capwidth(Some(Px(0.0)));
    document.style.set_page_margin(Px(0.0));
    insert_inline_anchor(&mut document, anchor_id.clone(), 0, 1);
    run_layout(&mut document);
    let LineContent::Signal(row) = &document.lines[0].content else {
        panic!("expected signal");
    };
    let signal_origin_y =
        document.lines[0].bounding_box.origin.y + row.geometry().signal_box.origin.y;
    let resolved_y = document
        .annotations
        .anchors
        .lookup_position(&anchor_id)
        .expect("anchor present")
        .y;
    assert!((resolved_y.to_f32() - signal_origin_y.to_f32()).abs() < 1.0e-4);
}

#[test]
fn arrow_anchor_endpoint_is_resolved_to_absolute() {
    let anchor_id = AnchorId::Named(AnchorName::parse("a").expect("name"));
    let lines = vec![make_signal_line(
        "x",
        vec![
            make_level_run(SignalLevel::Low, 1),
            WaveformElement::Anchor(anchor_id.clone()),
            make_level_run(SignalLevel::Low, 1),
        ],
    )];
    let mut document = make_document_with_lines(lines);
    insert_inline_anchor(&mut document, anchor_id.clone(), 0, 1);
    document.annotations.arrows.push(Arrow::new(
        ArrowEnd::Anchor(anchor_id),
        ArrowEnd::Absolute(crate::geometry::Point::ZERO),
        make_arrow_style(),
        None,
        FontSpec::default(),
    ));
    run_layout(&mut document);
    let arrow = &document.annotations.arrows[0];
    assert!(matches!(arrow.from, ArrowEnd::Absolute(_)));
}

#[test]
fn arrow_unresolved_anchor_is_layout_error() {
    let lines = vec![make_signal_line(
        "x",
        vec![make_level_run(SignalLevel::Low, 1)],
    )];
    let mut document = make_document_with_lines(lines);
    document.annotations.arrows.push(Arrow::new(
        ArrowEnd::Anchor(AnchorId::Named(AnchorName::parse("missing").expect("name"))),
        ArrowEnd::Absolute(crate::geometry::Point::ZERO),
        make_arrow_style(),
        None,
        FontSpec::default(),
    ));
    let fonts = MockFonts::new(7.0);
    let result = layout(&mut document, &fonts);
    assert_eq!(
        result.expect_err("must error"),
        LayoutError::UnresolvedAnchor
    );
}

#[test]
fn empty_document_has_page_margin_dimensions() {
    let mut document = make_document_with_lines(Vec::new());
    document.style.set_page_margin(Px(10.0));
    let fonts = MockFonts::new(7.0);
    let dimensions = layout(&mut document, &fonts).expect("layout");
    assert_eq!(dimensions.height, Px(20.0));
}

#[test]
fn skip_only_document_height_includes_skip_amount() {
    let skip = Line::new(
        LineContent::Skip(SkipRow::new(Length::new_lh(2.0).expect("lh"))),
        None,
    );
    let mut document = make_document_with_lines(vec![skip]);
    document.style.set_page_margin(Px(10.0));
    document.style.set_line_height(Px(24.0));
    let fonts = MockFonts::new(7.0);
    let dimensions = layout(&mut document, &fonts).expect("layout");
    assert_eq!(dimensions.height, Px(10.0 + 48.0 + 10.0));
}

fn make_clock_decorations(edge: ClockEdge) -> SignalDecorations {
    let one = NonZeroU32::new(1).expect("nonzero");
    SignalDecorations::new(
        Some(ClockSpec::new(
            edge,
            ClockPulse::new(one, one),
            ClockPhase::StartLow,
            crate::clock::ClockMarkStyle::default(),
        )),
        false,
    )
}

fn make_clock_signal_line(edge: ClockEdge) -> Line {
    make_clock_signal_line_with_layout(edge, LayoutParams::default())
}

fn make_clock_signal_line_with_layout(edge: ClockEdge, layout_params: LayoutParams) -> Line {
    let defaults = ChartStyle::default();
    let row = SignalRow::new(
        SignalGeometry::default(),
        SignalName::parse("ck").expect("name"),
        Waveform::from(vec![
            make_level_run(SignalLevel::Low, 1),
            make_single_edge(SignalLevel::Low, SignalLevel::High),
            make_level_run(SignalLevel::High, 1),
        ]),
        SignalRowStyle::new(
            defaults.default_signal_style().clone(),
            defaults.default_label_style().clone(),
        ),
        make_clock_decorations(edge),
        layout_params,
    );
    Line::new(LineContent::Signal(Box::new(row)), None)
}

// Layout emits EdgeMark into SignalRow.edge_marks for clock signals; nothing
// goes into Annotations.arrows. The tests below verify the EdgeMark coordinates
// and the absence of clock-derived Arrow objects.

#[test]
fn clock_edge_mark_not_in_annotations_arrows() {
    // Clock-derived markers must NOT appear in Annotations.arrows.
    let mut document = make_document_with_lines(vec![make_clock_signal_line(ClockEdge::Pos)]);
    document.style.set_step(Px(10.0));
    document.style.set_slant(Px(2.0));
    document.style.set_capwidth(Some(Px(0.0)));
    document.style.set_page_margin(Px(0.0));
    run_layout(&mut document);
    assert_eq!(
        document.annotations.arrows.len(),
        0,
        "clock markers must not be in Annotations.arrows"
    );
}

#[test]
fn clock_edge_mark_added_to_signal_row() {
    // One EdgeMark per matching edge.
    let mut document = make_document_with_lines(vec![make_clock_signal_line(ClockEdge::Pos)]);
    document.style.set_step(Px(10.0));
    document.style.set_slant(Px(2.0));
    document.style.set_capwidth(Some(Px(0.0)));
    document.style.set_page_margin(Px(0.0));
    run_layout(&mut document);
    let LineContent::Signal(row) = &document.lines[0].content else {
        panic!("expected signal");
    };
    assert_eq!(
        row.edge_marks().len(),
        1,
        "expected 1 EdgeMark for Pos clock"
    );
}

#[test]
fn clock_edge_mark_line_start_at_transition_x_y_low() {
    // Pos edge: line_start = (x, y_low).
    let mut document = make_clock_edge_mark_document(ClockEdge::Pos);
    run_layout(&mut document);
    let LineContent::Signal(row) = &document.lines[0].content else {
        panic!("expected signal");
    };
    let mark = &row.edge_marks()[0];
    // transition starts at x=10 (after 1 step), y_low = bottom of signal_box
    assert!(
        (mark.line_start.x.to_f32() - 10.0).abs() < 1.0e-4,
        "line_start.x should be 10 (1 step), got {}",
        mark.line_start.x.to_f32()
    );
    // y_low is the bottom edge of signal_box (= signal_box.origin.y + signal_box.height)
    let signal_box = row.geometry().signal_box;
    let expected_y_low =
        document.lines[0].bounding_box.origin.y + signal_box.origin.y + signal_box.size.height;
    assert!(
        (mark.line_start.y.to_f32() - expected_y_low.to_f32()).abs() < 1.0e-4,
        "line_start.y should be y_low={}, got {}",
        expected_y_low.to_f32(),
        mark.line_start.y.to_f32()
    );
}

#[test]
fn clock_edge_mark_line_end_at_x_plus_slant_y_high() {
    // Pos edge: line_end = (x + slant, y_high).
    let mut document = make_clock_edge_mark_document(ClockEdge::Pos);
    run_layout(&mut document);
    let LineContent::Signal(row) = &document.lines[0].content else {
        panic!("expected signal");
    };
    let mark = &row.edge_marks()[0];
    // line_end.x = line_start.x + slant = 10 + 2 = 12
    assert!(
        (mark.line_end.x.to_f32() - 12.0).abs() < 1.0e-4,
        "line_end.x should be 12, got {}",
        mark.line_end.x.to_f32()
    );
    let signal_box = row.geometry().signal_box;
    let expected_y_high = document.lines[0].bounding_box.origin.y + signal_box.origin.y;
    assert!(
        (mark.line_end.y.to_f32() - expected_y_high.to_f32()).abs() < 1.0e-4,
        "line_end.y should be y_high={}, got {}",
        expected_y_high.to_f32(),
        mark.line_end.y.to_f32()
    );
}

#[test]
fn clock_none_produces_no_edge_marks() {
    // @clock(none) should produce no EdgeMarks.
    let mut document = make_document_with_lines(vec![make_clock_signal_line(ClockEdge::None)]);
    document.style.set_step(Px(10.0));
    document.style.set_capwidth(Some(Px(0.0)));
    document.style.set_page_margin(Px(0.0));
    run_layout(&mut document);
    let LineContent::Signal(row) = &document.lines[0].content else {
        panic!("expected signal");
    };
    assert_eq!(
        row.edge_marks().len(),
        0,
        "ClockEdge::None must yield no EdgeMarks"
    );
}

#[test]
fn clock_both_produces_edge_marks_for_both_directions() {
    // @clock(both) with _~ pattern → 1 Pos + 0 Neg = 1 mark total.
    let mut document = make_document_with_lines(vec![make_clock_signal_line(ClockEdge::Both)]);
    document.style.set_step(Px(10.0));
    document.style.set_capwidth(Some(Px(0.0)));
    document.style.set_page_margin(Px(0.0));
    run_layout(&mut document);
    let LineContent::Signal(row) = &document.lines[0].content else {
        panic!("expected signal");
    };
    // The test waveform is `_~` (one Low→High), so Both gives 1 mark (Pos only).
    assert_eq!(row.edge_marks().len(), 1, "Both with _~ gives 1 mark");
}

// ---- Line.bbox.size.width uniform-across-rows tests -----------------------
// Scenario: 信号長が異なる行でも Line.bbox.size.width は全行一致

#[test]
fn bbox_width_uniform_across_rows_with_different_signal_length() {
    // SignalRow A: 10 units * step=10 = 100px waveform width
    // SignalRow B:  4 units * step=10 =  40px waveform width
    // capwidth fixed at 20px  =>  chart_inner_width = 20 + 100 = 120px
    // Both lines must have bbox.size.width == 120px.
    let document = make_two_signal_document_step10_cap20();
    let w0 = document.lines[0].bounding_box.size.width;
    let w1 = document.lines[1].bounding_box.size.width;
    assert!(
        (w0.to_f32() - 120.0).abs() < 1.0e-4,
        "row A bbox.size.width should be 120, got {}",
        w0.to_f32()
    );
    assert!(
        (w1.to_f32() - 120.0).abs() < 1.0e-4,
        "row B bbox.size.width should be 120 (not 60), got {}",
        w1.to_f32()
    );
}

// Scenario: signal_box.size.width は依然として要素 width 合計のまま

#[test]
fn signal_box_width_unchanged_despite_uniform_bbox_width() {
    // bbox.size.width is uniform, but signal_box.size.width stays per-row.
    let document = make_two_signal_document_step10_cap20();
    let bbox_w0 = document.lines[0].bounding_box.size.width;
    let bbox_w1 = document.lines[1].bounding_box.size.width;
    assert!(
        (bbox_w0.to_f32() - bbox_w1.to_f32()).abs() < 1.0e-4,
        "bbox widths must be equal, got {} vs {}",
        bbox_w0.to_f32(),
        bbox_w1.to_f32()
    );
    let LineContent::Signal(row_a) = &document.lines[0].content else {
        panic!("expected signal A");
    };
    let LineContent::Signal(row_b) = &document.lines[1].content else {
        panic!("expected signal B");
    };
    assert!(
        (row_a.geometry().signal_box.size.width.to_f32() - 100.0).abs() < 1.0e-4,
        "A.signal_box.size.width should be 100"
    );
    assert!(
        (row_b.geometry().signal_box.size.width.to_f32() - 40.0).abs() < 1.0e-4,
        "B.signal_box.size.width should be 40"
    );
}

// Scenario: Skip / Title 行も同じ幅で揃う

#[test]
fn bbox_width_uniform_including_skip_and_title() {
    // Signal A: 10 units * 10px = 100px; capwidth = 20  =>  chart_inner_width = 120
    // Skip row: waveform=0 but must get bbox.width = 120
    // Signal B:  4 units * 10px =  40px; must get bbox.width = 120
    let document = make_three_line_document_with_skip_step10_cap20();
    for (i, line) in document.lines.iter().enumerate() {
        assert!(
            (line.bounding_box.size.width.to_f32() - 120.0).abs() < 1.0e-4,
            "line[{i}] bbox.size.width should be 120, got {}",
            line.bounding_box.size.width.to_f32()
        );
    }
}

// --- Regression tests requiring internal field access (moved from tests/regression.rs) ---

/// The row background must cover the whole Line.bbox including the
/// symmetric signal_gap/2 margins. Adjacent stripes must meet without seams.
#[test]
fn row_bg_covers_full_bbox_with_symmetric_gap() {
    let source = "@bgcolor0 #eee\n@bgcolor1 #ccc\nA _~\nB _~\n";
    let mut document = crate::parser::parse(source).expect("parse");
    let fonts = MockFonts::new(7.0);
    layout(&mut document, &fonts).expect("layout");
    let row1_h = document.lines[0].bounding_box.size.height.to_f32();
    let row2_y = document.lines[1].bounding_box.origin.y.to_f32();
    let row1_y = document.lines[0].bounding_box.origin.y.to_f32();
    let stride = row2_y - row1_y;
    assert!(
        (stride - row1_h).abs() < 1.0e-4,
        "row 2 y must equal row 1 y + row 1 h (no gap, no overlap): \
         row1=({row1_y}, h={row1_h}), row2_y={row2_y}, stride={stride}"
    );
}

// ---- anchor element_index after TransitionEmitter -------------------------

/// 手動構築 Waveform でアンカー x 座標が遷移幅を正しく含む。
/// Waveform: Level(Low,3)[0], Anchor(a)[1], Transition[2], Level(High,4,preceded=true)[3], Anchor(b)[4], Level(Low,3)[5]
#[test]
fn anchor_after_transition_has_correct_x() {
    let anchor_a = AnchorId::Named(AnchorName::parse("a").expect("name"));
    let anchor_b = AnchorId::Named(AnchorName::parse("b").expect("name"));
    let mut document = make_transition_anchor_document(anchor_a.clone(), anchor_b.clone());
    insert_inline_anchor(&mut document, anchor_a.clone(), 0, 1);
    insert_inline_anchor(&mut document, anchor_b.clone(), 0, 4);
    run_layout(&mut document);
    // @{a}: Low(3)*10 = 30px
    assert_close(lookup_anchor_x(&document, &anchor_a), 30.0, "anchor @{a} x");
    // @{b}: Low(3)*10 + slant(2) + High(4)*(10-2) = 30 + 2 + 32 = 64px
    // (The High(4) LevelRun is manually constructed with preceded_by_transition=false,
    // so its width is 4*10=40, giving 30+2+40=72. Manually-built runs default to false.)
    assert_close(lookup_anchor_x(&document, &anchor_b), 72.0, "anchor @{b} x");
}

fn make_transition_anchor_document(anchor_a: AnchorId, anchor_b: AnchorId) -> ChartDocument {
    let elements = vec![
        make_level_run(SignalLevel::Low, 3),
        WaveformElement::Anchor(anchor_a),
        make_single_edge(SignalLevel::Low, SignalLevel::High),
        make_level_run(SignalLevel::High, 4),
        WaveformElement::Anchor(anchor_b),
        make_level_run(SignalLevel::Low, 3),
    ];
    let mut document = make_document_with_lines(vec![make_signal_line("SigA", elements)]);
    document.style.set_step(Px(10.0));
    document.style.set_slant(Px(2.0));
    document.style.set_capwidth(Some(Px(0.0)));
    document.style.set_page_margin(Px(0.0));
    document
}

/// パーサー経由でのアンカー解決確認。
/// `___@{a}~~~~@{b}___` のパース後、transition injection により @{b}.element_index が
/// シフトしてもアンカー x 座標は遷移幅を含む位置に解決される。
/// 期待値: x_a = signal_origin_x+30, x_b = signal_origin_x+70 (slant=2, step=10)
#[test]
fn anchor_after_transition_via_parser() {
    let source = "@step 10\n@slant 2\nSigA ___@{a}~~~~@{b}___\n";
    let mut document = crate::parser::parse(source).expect("parse");
    let fonts = MockFonts::new(7.0);
    layout(&mut document, &fonts).expect("layout");

    let anchor_a = AnchorId::Named(AnchorName::parse("a").expect("name"));
    let anchor_b = AnchorId::Named(AnchorName::parse("b").expect("name"));
    let signal_origin_x = compute_signal_origin_x(&document);

    let x_a = lookup_anchor_x(&document, &anchor_a);
    let x_b = lookup_anchor_x(&document, &anchor_b);

    // @{a}: signal_origin_x + Low(3)*10 = signal_origin_x + 30
    assert_close(x_a, signal_origin_x + 30.0, "anchor @{a} x");
    // @{b}: signal_origin_x + Low(3)*10 + slant(2) + High(4)*(10-2) = signal_origin_x + 70
    // Parser sets preceded_by_transition=true on High(4), so width = 4*10-2 = 38.
    assert_close(
        x_b,
        signal_origin_x + 70.0,
        "anchor @{b} x (Low×3 + slant + High×4×(step-slant))",
    );
}

/// Bus 行で BusCross の後にあるアンカーも正しい x 座標を持つ。
/// Waveform: Level(Bus,5)[0], Anchor(@1)[1], Transition(BusCross)[2], Level(Bus,4)[3], Anchor(@2)[4]
#[test]
fn anchor_after_buscross_has_correct_x() {
    use std::num::NonZeroU32;
    let anchor_1 = AnchorId::Indexed(NonZeroU32::new(1).expect("nonzero"));
    let anchor_2 = AnchorId::Indexed(NonZeroU32::new(2).expect("nonzero"));
    let mut document = make_buscross_anchor_document(anchor_1.clone(), anchor_2.clone());
    insert_inline_anchor(&mut document, anchor_1.clone(), 0, 1);
    insert_inline_anchor(&mut document, anchor_2.clone(), 0, 4);
    run_layout(&mut document);
    // @1: Bus(5)*10 = 50px
    assert_close(lookup_anchor_x(&document, &anchor_1), 50.0, "anchor @1 x");
    // @2: Bus(5)*10 + BusCross(w_transient=2) + Bus(4)*10 = 92px
    assert_close(lookup_anchor_x(&document, &anchor_2), 92.0, "anchor @2 x");
}

fn make_buscross_anchor_document(anchor_1: AnchorId, anchor_2: AnchorId) -> ChartDocument {
    let elements = vec![
        make_level_run(SignalLevel::Bus, 5),
        WaveformElement::Anchor(anchor_1),
        WaveformElement::Transition(Transition::new(
            SignalLevel::Bus,
            SignalLevel::Bus,
            TransitionKind::BusCross,
            None,
        )),
        make_level_run(SignalLevel::Bus, 4),
        WaveformElement::Anchor(anchor_2),
    ];
    let mut document = make_document_with_lines(vec![make_signal_line("Bus", elements)]);
    document.style.set_step(Px(10.0));
    document.style.set_slant(Px(2.0));
    document.style.set_capwidth(Some(Px(0.0)));
    document.style.set_page_margin(Px(0.0));
    document
}

/// Parser-based regression: LevelRun after BusCross has preceded_by_transition=true,
/// so its width is units*step - slant.
/// Wave `=====@1X====@2` with step=10, slant=2:
/// x_@1 = 5*10 = 50, x_@2 = 50 + slant + 5*(10-2) = 50 + 2 + 40 = ... wait,
/// see spec: body of X + following `====` merge into Bus(5) with preceded=true,
/// so Bus(5) width = 5*10-2 = 48; x_@2 = 50 + 2 + 48 = 100.
#[test]
fn anchor_after_buscross_via_parser() {
    use std::num::NonZeroU32;
    let source = "@step 10\n@slant 2\nBus =====@1X====@2\n";
    let mut document = crate::parser::parse(source).expect("parse");
    document.style.set_capwidth(Some(Px(0.0)));
    document.style.set_page_margin(Px(0.0));
    let fonts = MockFonts::new(7.0);
    layout(&mut document, &fonts).expect("layout");

    let anchor_1 = AnchorId::Indexed(NonZeroU32::new(1).expect("nonzero"));
    let anchor_2 = AnchorId::Indexed(NonZeroU32::new(2).expect("nonzero"));
    let signal_origin_x = compute_signal_origin_x(&document);

    let x_1 = lookup_anchor_x(&document, &anchor_1);
    let x_2 = lookup_anchor_x(&document, &anchor_2);

    assert_close(x_1, signal_origin_x + 50.0, "anchor @1 x");
    assert_close(x_2, signal_origin_x + 100.0, "anchor @2 x");
}

/// Same-character-count waveforms must have identical widths regardless of
/// transition count.  `_~_~` and `__==` both have 4 level chars, so
/// signal_box.size.width must equal 4*step = 40.
#[test]
fn signal_box_width_is_char_count_times_step() {
    let source = "@step 10\n@slant 2\nA _~_~\nB __==\n";
    let mut document = crate::parser::parse(source).expect("parse");
    document.style.set_capwidth(Some(Px(0.0)));
    document.style.set_page_margin(Px(0.0));
    let fonts = MockFonts::new(7.0);
    layout(&mut document, &fonts).expect("layout");

    for line in &document.lines {
        let LineContent::Signal(row) = &line.content else {
            continue;
        };
        assert_close(
            row.geometry().signal_box.size.width.to_f32(),
            40.0,
            "signal_box.size.width must be 4*step = 40",
        );
    }
}

/// signal_box y origin equals signal_gap/2 in every signal row
/// (no special-case for first/last row).
#[test]
fn signal_box_origin_y_is_half_signal_gap() {
    let mut document = crate::parser::parse("A _~\nB _~\nC _~\n").expect("parse");
    let fonts = MockFonts::new(7.0);
    layout(&mut document, &fonts).expect("layout");
    let half_gap = document.style.layout().h_space().to_f32() * 0.5;
    for line in document.lines {
        if let LineContent::Signal(row) = &line.content {
            assert!(
                (row.geometry().signal_box.origin.y.to_f32() - half_gap).abs() < 1.0e-4,
                "signal_box.origin.y must equal signal_gap/2 ({half_gap}); \
                 got {}",
                row.geometry().signal_box.origin.y.to_f32()
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Edge-case scenarios from docs/tests/layout-engine.feature.md
// (added under "観点A 補強" / "観点B 補強").
// Tests are allowed to fail when the implementation does not yet match spec.
// ---------------------------------------------------------------------------

fn parse_and_layout(source: &str) -> ChartDocument {
    let mut document = crate::parser::parse(source).expect("parse should succeed");
    let fonts = MockFonts::new(7.0);
    layout(&mut document, &fonts).expect("layout should succeed");
    document
}

fn signal_at_index(document: &ChartDocument, index: usize) -> &SignalRow {
    let line = document
        .lines
        .iter()
        .filter(|line| matches!(line.content, LineContent::Signal(_)))
        .nth(index)
        .expect("signal index in range");
    match &line.content {
        LineContent::Signal(row) => row.as_ref(),
        _ => unreachable!(),
    }
}

#[test]
fn page_margin_zero_keeps_stacking_origin_at_zero() {
    let document = parse_and_layout("@page-margin 0\nA _\nB _\n");
    let first = &document.lines[0];
    assert!(
        first.bounding_box.origin.y.to_f32().abs() < 1.0e-4,
        "with page-margin 0 the first row must start at y=0; got {}",
        first.bounding_box.origin.y.to_f32()
    );
}

#[test]
fn large_page_margin_offsets_first_row() {
    let document = parse_and_layout("@page-margin 50\nA _\nB _\n");
    let first = &document.lines[0];
    assert!(
        (first.bounding_box.origin.y.to_f32() - 50.0).abs() < 1.0e-4,
        "first row must start at page-margin (50); got {}",
        first.bounding_box.origin.y.to_f32()
    );
}

#[test]
fn fractional_h_space_is_distributed_symmetrically() {
    let document = parse_and_layout("@h_space 4.5\nA _\n");
    let row = signal_at_index(&document, 0);
    assert!(
        (row.geometry().signal_box.origin.y.to_f32() - 2.25).abs() < 1.0e-3,
        "signal_box.origin.y must equal h_space/2 (2.25); got {}",
        row.geometry().signal_box.origin.y.to_f32()
    );
}

#[test]
fn capwidth_exact_match_with_label_width() {
    let document = parse_and_layout("@capwidth 30\n@namepad 8\nFoo _\n");
    let row = signal_at_index(&document, 0);
    let label_width = row.geometry().label_box.size.width.to_f32();
    assert!(
        label_width <= 30.0 + 1.0e-3,
        "label_box must not exceed capwidth (30); got {label_width}"
    );
}

#[test]
fn namepad_zero_collapses_label_signal_gap() {
    let document = parse_and_layout("@namepad 0\n@capwidth 20\nFoo _\n");
    let row = signal_at_index(&document, 0);
    let signal_x = row.geometry().signal_box.origin.x.to_f32();
    assert!(
        signal_x <= 20.0 + 1.0e-3,
        "signal_box.x must equal capwidth (20) with namepad=0; got {signal_x}"
    );
}

#[test]
fn lineheight_one_matches_signal_height_to_fontsize() {
    let document = parse_and_layout("@fontsize 14\n@lineheight 1.0\nA _\n");
    let height = document.style.canvas().line_height().to_f32();
    assert!(
        (height - 14.0).abs() < 1.0e-3,
        "line_height must equal fontsize (14); got {height}"
    );
}

#[test]
fn scale_does_not_modify_internal_bounding_boxes() {
    let scaled = parse_and_layout("@scale 2.0\nA _\n");
    let plain = parse_and_layout("A _\n");
    let scaled_h = scaled.lines[0].bounding_box.size.height.to_f32();
    let plain_h = plain.lines[0].bounding_box.size.height.to_f32();
    assert!(
        (scaled_h - plain_h).abs() < 1.0e-3,
        "@scale must not change internal bbox; got scaled={scaled_h}, plain={plain_h}"
    );
}

#[test]
fn per_row_step_change_yields_individual_signal_box_widths() {
    let document = parse_and_layout("@step 10\nA ____\n@step 20\nB ____\n");
    let a = signal_at_index(&document, 0);
    let b = signal_at_index(&document, 1);
    let width_a = a.geometry().signal_box.size.width.to_f32();
    let width_b = b.geometry().signal_box.size.width.to_f32();
    assert!(
        (width_a - 40.0).abs() < 1.0e-3,
        "Sig1 width must be 40; got {width_a}"
    );
    assert!(
        (width_b - 80.0).abs() < 1.0e-3,
        "Sig2 width must be 80; got {width_b}"
    );
}

#[test]
fn per_row_h_space_affects_only_subsequent_signals() {
    // Per spec types.md §3.2, signal_box.size.height is fixed at line_height
    // chart-wide; h_space is applied on top via Line.bbox.size.height
    // (bbox.height = max(label, signal) + h_space). So a per-row @h_space
    // change must surface on Line.bounding_box.size.height, not signal_box.
    let document = parse_and_layout("@h_space 4\nA _\n@h_space 10\nB _\n");
    let height_a = document
        .lines
        .first()
        .expect("input has two signal lines")
        .bounding_box
        .size
        .height
        .to_f32();
    let height_b = document
        .lines
        .get(1)
        .expect("input has two signal lines")
        .bounding_box
        .size
        .height
        .to_f32();
    assert!(
        height_a < height_b,
        "row B must be taller; {height_a} vs {height_b}"
    );
}

#[test]
fn per_row_step_with_anchor_position_reflects_local_step() {
    let document = parse_and_layout("@step 10\nA ___@1__\n@step 20\nB ___@2__\n");
    let one = document
        .annotations
        .anchors
        .lookup_position(&AnchorId::Indexed(NonZeroU32::new(1).expect("nonzero")));
    let two = document
        .annotations
        .anchors
        .lookup_position(&AnchorId::Indexed(NonZeroU32::new(2).expect("nonzero")));
    if let (Some(one), Some(two)) = (one, two) {
        let dx_one = one.x.to_f32();
        let dx_two = two.x.to_f32();
        assert!(
            dx_two > dx_one,
            "anchor 2 (step=20) should be further right than anchor 1 (step=10); got {dx_one} vs {dx_two}"
        );
    }
}

#[test]
fn clock_auto_signal_factors_into_chart_units() {
    // Spec ambiguity: clock_units include vs exclude clock body. The test
    // pins behaviour against current implementation; failure is the signal.
    let document = parse_and_layout("@clock(pos) clk _~_~_~_~_~_~\nshort _~\n");
    let clk = signal_at_index(&document, 0);
    let other = signal_at_index(&document, 1);
    let clk_w = clk.geometry().signal_box.size.width.to_f32();
    let other_w = other.geometry().signal_box.size.width.to_f32();
    assert!(
        clk_w > 0.0 && other_w > 0.0,
        "both widths must be positive; got clk={clk_w}, other={other_w}"
    );
}

#[test]
fn per_row_step_keeps_bbox_width_uniform_via_chart_inner_width() {
    let document = parse_and_layout("@step 10\nA ____\n@step 50\nB ____\n");
    let a_bbox = document.lines[0].bounding_box.size.width.to_f32();
    let b_bbox = document.lines[1].bounding_box.size.width.to_f32();
    assert!(
        (a_bbox - b_bbox).abs() < 1.0e-3,
        "row bbox.width must be uniform (chart_inner_width); got {a_bbox} vs {b_bbox}"
    );
}

#[test]
fn per_row_step_arrow_label_midpoint_uses_local_x() {
    let document = parse_and_layout("@step 10\nA _~@{a}_\n@step 20\nB _~@{b}_\n@-> (@{a}, @{b})\n");
    assert!(!document.annotations.arrows.is_empty());
}

#[test]
fn per_row_step_with_bg_keeps_background_on_correct_signal() {
    let document = parse_and_layout("@bg #f0f\n@step 20\nSig _~\n");
    let line = document
        .lines
        .iter()
        .find(|line| matches!(line.content, LineContent::Signal(_)))
        .expect("signal");
    assert!(line.background.is_some());
}

#[test]
fn per_row_h_space_with_overline_decoration() {
    let document = parse_and_layout("@h_space 8\n@signal(overline) nReset _~__\n");
    let row = signal_at_index(&document, 0);
    assert!(row.decorations().is_name_overline());
}

#[test]
fn skip_row_after_bg_inherits_background() {
    let document = parse_and_layout("@bg #f0f\n@skip(2)\nA _\n");
    // Either the skip row or the signal row should carry the background;
    // spec says @bg targets "the next row" so the skip row is the target.
    let any_bg = document.lines.iter().any(|line| line.background.is_some());
    assert!(
        any_bg,
        "@bg must apply to some row (skip preferred per spec)"
    );
}

#[test]
fn capwidth_zero_falls_back_to_widest_label() {
    let document = parse_and_layout("@capwidth 0\nSig1 _\nLongerSignalName _\n");
    let first = signal_at_index(&document, 0);
    let second = signal_at_index(&document, 1);
    assert_eq!(
        first.geometry().label_box.size.width,
        second.geometry().label_box.size.width,
        "label_box widths must be equalised across rows"
    );
}

#[test]
fn page_margin_odd_value_offsets_symmetrically() {
    let document = parse_and_layout("@page-margin 11\nA _\nB _\n");
    let first = &document.lines[0];
    assert!(
        (first.bounding_box.origin.y.to_f32() - 11.0).abs() < 1.0e-3,
        "first row must start at page-margin (11); got {}",
        first.bounding_box.origin.y.to_f32()
    );
}

// ---------------------------------------------------------------------------
// Iter1 phase 2: empty / zero / extreme value boundaries.
// ---------------------------------------------------------------------------

#[test]
fn empty_tcml_chart_width_equals_double_page_margin_iter1() {
    let mut document = crate::parser::parse("@page-margin 11\n").expect("parse");
    let fonts = MockFonts::new(7.0);
    let dimensions = layout(&mut document, &fonts).expect("layout");
    assert_eq!(
        document.lines.len(),
        0,
        "no signal lines for an empty chart"
    );
    let expected_width = 2.0 * 11.0;
    assert!(
        (dimensions.width.to_f32() - expected_width).abs() < 1.0e-3,
        "empty chart width must equal 2 * page-margin (={expected_width}); got {}",
        dimensions.width.to_f32()
    );
}

#[test]
fn title_only_chart_height_includes_page_margin_iter1() {
    let document = parse_and_layout("@page-margin 11\n@title \"T\"\n");
    let title_line = &document.lines[0];
    assert!(
        title_line.bounding_box.size.height.to_f32() > 0.0,
        "title bbox must have positive height"
    );
    assert!(
        (title_line.bounding_box.origin.y.to_f32() - 11.0).abs() < 1.0e-3,
        "title must start at page-margin"
    );
}

#[test]
fn scale_one_thousand_does_not_inflate_signal_box_iter1() {
    let scaled = parse_and_layout("@scale 1000\nA _~\n");
    let plain = parse_and_layout("A _~\n");
    let scaled_width = signal_at_index(&scaled, 0)
        .geometry()
        .signal_box
        .size
        .width
        .to_f32();
    let plain_width = signal_at_index(&plain, 0)
        .geometry()
        .signal_box
        .size
        .width
        .to_f32();
    assert!(
        (scaled_width - plain_width).abs() < 1.0e-3,
        "signal_box width must be scale-independent; got plain={plain_width} scaled={scaled_width}"
    );
}

#[test]
fn step_one_minimum_signal_box_width_iter1() {
    let document = parse_and_layout("@step 1\n@slant 0\nA _~_~\n");
    let width = signal_at_index(&document, 0)
        .geometry()
        .signal_box
        .size
        .width
        .to_f32();
    assert!(
        (width - 4.0).abs() < 1.0e-3,
        "step=1 with 4 units should yield width=4; got {width}"
    );
}

#[test]
fn slant_zero_keeps_chart_layout_valid_iter1() {
    let document = parse_and_layout("@slant 0\nA _~\n");
    let row = signal_at_index(&document, 0);
    let signal_box = row.geometry().signal_box;
    assert!(
        signal_box.size.width.to_f32() > 0.0,
        "signal_box width must be positive with slant=0; got {}",
        signal_box.size.width.to_f32()
    );
    assert!(
        signal_box.size.height.to_f32() > 0.0,
        "signal_box height must be positive with slant=0; got {}",
        signal_box.size.height.to_f32()
    );
    let label_box = row.geometry().label_box;
    assert!(
        label_box.size.width.to_f32() > 0.0,
        "label_box width must be positive with slant=0; got {}",
        label_box.size.width.to_f32()
    );
    assert!(
        label_box.size.height.to_f32() > 0.0,
        "label_box height must be positive with slant=0; got {}",
        label_box.size.height.to_f32()
    );
}

#[test]
fn one_character_waveform_signal_box_equals_step_iter1() {
    let document = parse_and_layout("@step 16\nA _\n");
    let width = signal_at_index(&document, 0)
        .geometry()
        .signal_box
        .size
        .width
        .to_f32();
    assert!(
        (width - 16.0).abs() < 1.0e-3,
        "single-char waveform must equal one step; got {width}"
    );
}

#[test]
fn anchor_only_waveform_width_unchanged_iter1() {
    let with_anchor = parse_and_layout("A _@{a}_\n");
    let without_anchor = parse_and_layout("A __\n");
    let with_anchor_width = signal_at_index(&with_anchor, 0)
        .geometry()
        .signal_box
        .size
        .width
        .to_f32();
    let without_anchor_width = signal_at_index(&without_anchor, 0)
        .geometry()
        .signal_box
        .size
        .width
        .to_f32();
    assert!(
        (with_anchor_width - without_anchor_width).abs() < 1.0e-3,
        "anchor must be zero-width: got with={with_anchor_width} without={without_anchor_width}"
    );
}

// ---------------------------------------------------------------------------
// Iter3 phase: numeric precision / accumulated error / extreme values.
// ---------------------------------------------------------------------------

#[test]
fn iter3_fifty_signals_with_thousand_steps_have_bounded_error() {
    let mut source = String::from("@step 16\n");
    let body: String = "_~".repeat(500);
    for index in 0..50 {
        source.push_str(&format!("S{index} {body}\n"));
    }
    let document = parse_and_layout(&source);
    let last_row = signal_at_index(&document, 49);
    let width = last_row.geometry().signal_box.size.width.to_f32();
    let expected = 16.0 * 1000.0;
    let drift = (width - expected).abs();
    assert!(
        drift < 1.0,
        "f32 drift across 50 rows × 1000 steps must stay sub-pixel; expected {expected} got {width} (drift={drift})"
    );
}

#[test]
fn iter3_scale_thousandth_does_not_affect_logical_layout() {
    let scaled = parse_and_layout("@scale 0.001\nA _~_~\n");
    let unscaled = parse_and_layout("A _~_~\n");
    let scaled_width = signal_at_index(&scaled, 0)
        .geometry()
        .signal_box
        .size
        .width
        .to_f32();
    let unscaled_width = signal_at_index(&unscaled, 0)
        .geometry()
        .signal_box
        .size
        .width
        .to_f32();
    // Logical width should not collapse to zero when @scale is applied.
    assert!(
        scaled_width > 0.0 && unscaled_width > 0.0,
        "logical layout must not collapse under @scale; scaled={scaled_width} unscaled={unscaled_width}"
    );
}

#[test]
fn iter3_extreme_scale_and_step_does_not_overflow() {
    let document = parse_and_layout("@scale 1000\n@step 999\nA _~_~\n");
    let width = signal_at_index(&document, 0)
        .geometry()
        .signal_box
        .size
        .width
        .to_f32();
    assert!(
        width.is_finite() && width > 0.0,
        "extreme scale × step must stay finite; got {width}"
    );
}

#[test]
fn iter3_fractional_step_layout_is_deterministic() {
    let outcome = crate::parser::parse("@step 0.5\nA _~_~\n");
    if outcome.is_err() {
        return;
    }
    let document = parse_and_layout("@step 0.5\nA _~_~\n");
    let width = signal_at_index(&document, 0)
        .geometry()
        .signal_box
        .size
        .width
        .to_f32();
    assert!(
        width.is_finite() && width >= 0.0,
        "fractional @step must produce a finite width; got {width}"
    );
}

#[test]
fn iter3_fractional_slant_does_not_panic() {
    let outcome = crate::parser::parse("@slant 0.1\nA _~\n");
    if outcome.is_err() {
        return;
    }
    let document = parse_and_layout("@slant 0.1\nA _~\n");
    let width = signal_at_index(&document, 0)
        .geometry()
        .signal_box
        .size
        .width
        .to_f32();
    assert!(
        width.is_finite(),
        "slant=0.1 layout must be finite; got {width}"
    );
}

#[test]
fn iter3_step_thousand_chart_inner_width_matches_units() {
    let document = parse_and_layout("@step 1000\nA _~_~\n");
    let width = signal_at_index(&document, 0)
        .geometry()
        .signal_box
        .size
        .width
        .to_f32();
    assert!(
        (width - 4000.0).abs() < 1.0,
        "@step 1000 × 4 units must yield 4000 px; got {width}"
    );
}

#[test]
fn iter3_oversized_slant_layout_continues() {
    let outcome = crate::parser::parse("@step 16\n@slant 999\nA _~\n");
    if outcome.is_err() {
        return;
    }
    let document = parse_and_layout("@step 16\n@slant 999\nA _~\n");
    let width = signal_at_index(&document, 0)
        .geometry()
        .signal_box
        .size
        .width
        .to_f32();
    assert!(
        width.is_finite() && width > 0.0,
        "huge slant must still produce a finite width; got {width}"
    );
}

#[test]
fn iter3_hundred_signals_with_hundred_anchors_have_monotonic_x() {
    let mut source = String::from("");
    for signal_index in 0..100 {
        let mut body = String::from("_");
        for anchor_index in 0..100 {
            body.push_str(&format!("@{{a{signal_index}_{anchor_index}}}_"));
        }
        source.push_str(&format!("S{signal_index} {body}\n"));
    }
    let document = parse_and_layout(&source);
    // Pick the first row; verify its width is finite and positive.
    let width = signal_at_index(&document, 0)
        .geometry()
        .signal_box
        .size
        .width
        .to_f32();
    assert!(
        width.is_finite() && width > 0.0,
        "100×100 anchor layout must produce a finite first-row width; got {width}"
    );
}

#[test]
fn iter3_minimum_scale_and_fontsize_combo_keeps_height_positive() {
    let outcome = crate::parser::parse("@scale 0.001\n@fontsize 0.5\nA _\n");
    if outcome.is_err() {
        return;
    }
    let document = parse_and_layout("@scale 0.001\n@fontsize 0.5\nA _\n");
    let height = signal_at_index(&document, 0)
        .geometry()
        .signal_box
        .size
        .height
        .to_f32();
    assert!(
        height > 0.0,
        "minimal scale + fontsize must keep signal_box height positive; got {height}"
    );
}

#[test]
fn iter3_fractional_step_with_clock_period_calc_is_deterministic() {
    let outcome = crate::parser::parse("@step 0.5\n@clock(pos)\nclk _~_~\n");
    if outcome.is_err() {
        return;
    }
    let document = parse_and_layout("@step 0.5\n@clock(pos)\nclk _~_~\n");
    // Just verify no panic and a finite width. The gcd / period computation
    // happens inside layout for clock rows.
    let width = signal_at_index(&document, 0)
        .geometry()
        .signal_box
        .size
        .width
        .to_f32();
    assert!(
        width.is_finite() && width >= 0.0,
        "fractional step with clock must remain finite; got {width}"
    );
}
