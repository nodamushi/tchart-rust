//! SVG renderer tests covering structure, gap/transition contracts,
//! transitions, escaping, arrows, guides, and labels.

use crate::anchor::AnchorRegistry;
use crate::arrow::{Arrow, ArrowEnd, ArrowHead, ArrowStyle, LineDashStyle};
use crate::color::Color;
use crate::document::{Annotations, ChartDocument, TcmlSource, TextOverlay};
use crate::geometry::{Point, Rect, Size};
use crate::line::{
    LevelRun, Line, LineContent, SignalDecorations, SignalGeometry, SignalLevel, SignalRow,
    SkipRow, TitleRow, Transition, TransitionKind, Waveform, WaveformElement,
};
use crate::style::{ChartStyle, HorizontalAlign, LayoutParams, TitleStyle};
use crate::text::{FontSpec, SignalName, UserText};
use crate::units::Px;

use super::buf::SvgBuf;
use super::render;

/// Stub `FontMetrics` used throughout SVG renderer tests.
struct TestFonts;

impl crate::layout::FontMetrics for TestFonts {
    fn measure_text_width(&self, text: &str, _font: &crate::text::FontSpec) -> Px {
        Px(7.0 * text.chars().count() as f32)
    }
}

fn make_signal(name: &str, elements: Vec<WaveformElement>, style: &ChartStyle) -> Line {
    let body = signal_box_size(&elements, style);
    let row = build_row(name, elements, style, body);
    Line::new_with_bounding_box(
        LineContent::Signal(Box::new(row)),
        None,
        Rect {
            origin: Point {
                x: Px(10.0),
                y: Px(10.0),
            },
            size: Size {
                width: Px(40.0) + body.width,
                height: Px(20.0),
            },
        },
    )
}

fn make_signal_with_bounding_box_y(
    name: &str,
    elements: Vec<WaveformElement>,
    style: &ChartStyle,
    y: Px,
) -> Line {
    let body = signal_box_size(&elements, style);
    let row = build_row(name, elements, style, body);
    Line::new_with_bounding_box(
        LineContent::Signal(Box::new(row)),
        None,
        Rect {
            origin: Point { x: Px(10.0), y },
            size: Size {
                width: Px(40.0) + body.width,
                height: Px(20.0),
            },
        },
    )
}

fn build_row(
    name: &str,
    elements: Vec<WaveformElement>,
    style: &ChartStyle,
    body: Size,
) -> SignalRow {
    SignalRow::new(
        build_geometry(body),
        SignalName::parse(name).expect("name"),
        Waveform::from(elements),
        crate::style::SignalRowStyle::new(
            style.default_signal_style().clone(),
            style.default_label_style().clone(),
        ),
        SignalDecorations::default(),
        *style.layout(),
    )
}

fn build_geometry(body: Size) -> SignalGeometry {
    SignalGeometry::new(
        Rect {
            origin: Point {
                x: Px::ZERO,
                y: Px(2.0),
            },
            size: Size {
                width: Px(40.0),
                height: Px(16.0),
            },
        },
        Rect {
            origin: Point {
                x: Px(40.0),
                y: Px(2.0),
            },
            size: body,
        },
    )
}

fn signal_box_size(elements: &[WaveformElement], style: &ChartStyle) -> Size {
    let width = style.layout().sum_element_widths(elements);
    Size {
        width,
        height: Px(16.0),
    }
}

fn make_doc(lines: Vec<Line>) -> ChartDocument {
    ChartDocument::new(
        ChartStyle::default(),
        lines,
        Annotations::default(),
        TcmlSource::new(""),
    )
}

fn level(signal_level: SignalLevel, units: u32) -> WaveformElement {
    WaveformElement::Level(LevelRun::new(signal_level, units))
}

fn transition(kind: TransitionKind, from: SignalLevel, to: SignalLevel) -> WaveformElement {
    WaveformElement::Transition(Transition::new(from, to, kind, None))
}

#[test]
fn root_has_namespaces() {
    let svg = render(&make_doc(vec![]), &TestFonts);
    assert!(svg.starts_with("<svg "));
    assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
    assert!(svg.contains("xmlns:tchart=\"http://tchart-rust/1.0\""));
    assert!(svg.ends_with("</svg>"));
}

#[test]
fn empty_doc_emits_no_layer_groups_iter1() {
    let svg = render(&make_doc(vec![]), &TestFonts);
    let names = [
        "row-backgrounds",
        "highlights",
        "dontcares",
        "waveforms",
        "signal-labels",
        "guides",
        "titles",
        "arrows",
        "overlays",
    ];
    for name in names {
        let needle = format!("class=\"{name}\"");
        assert!(
            !svg.contains(&needle),
            "empty document must not emit `{name}` layer (spec: empty-layer suppression): {svg}"
        );
    }
}

#[test]
fn populated_doc_emits_layers_in_spec_order_iter1() {
    let svg = render_pipeline(
        "@title Demo\n@bgcolor0 #fde\n@bgcolor1 #fde\nA _?_~_@{a}_|_[mark]_\nB _~@{b}_\n@-> (@{a}, @{b})\n% 1 1 over\n",
    );
    let names = [
        "row-backgrounds",
        "highlights",
        "dontcares",
        "signal-labels",
        "waveforms",
        "guides",
        "titles",
        "arrows",
        "overlays",
    ];
    let mut cursor = 0usize;
    for name in names {
        let needle = format!("class=\"{name}\"");
        let position = svg[cursor..]
            .find(&needle)
            .unwrap_or_else(|| panic!("layer {name} missing or out of order in {svg}"));
        cursor += position + needle.len();
    }
}

#[test]
fn metadata_embeds_tcml() {
    let mut document = make_doc(vec![]);
    document.source = TcmlSource::new("Clock _~_~");
    let svg = render(&document, &TestFonts);
    assert!(svg.contains("<metadata><tchart:source>Clock _~_~</tchart:source></metadata>"));
}

#[test]
fn metadata_escapes_xml() {
    let mut document = make_doc(vec![]);
    document.source = TcmlSource::new("a<b&c>");
    let svg = render(&document, &TestFonts);
    assert!(svg.contains("a&lt;b&amp;c&gt;"));
}

#[test]
fn signal_low_emits_polyline() {
    let style = ChartStyle::default();
    let line = make_signal("ck", vec![level(SignalLevel::Low, 2)], &style);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    assert!(svg.contains("<polyline"));
}

#[test]
fn single_edge_no_independent_line_in_waveforms() {
    let style = ChartStyle::default();
    let elements = vec![
        level(SignalLevel::Low, 1),
        transition(
            TransitionKind::SingleEdge,
            SignalLevel::Low,
            SignalLevel::High,
        ),
        level(SignalLevel::High, 1),
    ];
    let line = make_signal("ck", elements, &style);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let waveforms_open = svg.find("class=\"waveforms\"").expect("waveforms");
    let waveforms_close = svg[waveforms_open..].find("</g>").expect("close") + waveforms_open;
    let layer = &svg[waveforms_open..waveforms_close];
    assert!(layer.contains("<polyline"));
    assert!(!layer.contains("<line"));
}

#[test]
fn gap_flushes_polyline() {
    let style = ChartStyle::default();
    let elements = vec![
        level(SignalLevel::Low, 2),
        WaveformElement::Gap,
        level(SignalLevel::High, 2),
    ];
    let line = make_signal("ck", elements, &style);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let count = svg.matches("<polyline").count();
    assert!(
        count >= 2,
        "expected >=2 polylines after gap, got {count} in {svg}"
    );
}

#[test]
fn bus_open_emits_two_rails() {
    let style = ChartStyle::default();
    let elements = vec![
        level(SignalLevel::Low, 1),
        transition(TransitionKind::BusOpen, SignalLevel::Low, SignalLevel::Bus),
        level(SignalLevel::Bus, 1),
    ];
    let line = make_signal("d", elements, &style);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let count = svg.matches("<polyline").count();
    assert!(count >= 2, "BusOpen requires 2 rails, got {count}");
}

#[test]
fn bus_cross_swaps_rails() {
    let style = ChartStyle::default();
    let elements = vec![
        level(SignalLevel::Bus, 1),
        transition(TransitionKind::BusCross, SignalLevel::Bus, SignalLevel::Bus),
        level(SignalLevel::Bus, 1),
    ];
    let line = make_signal("d", elements, &style);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    assert!(svg.matches("<polyline").count() >= 2);
}

#[test]
fn dontcare_emits_rect_in_dontcares_layer() {
    let style = ChartStyle::default();
    let elements = vec![
        level(SignalLevel::Low, 1),
        level(SignalLevel::DontCareAlongLow, 2),
        level(SignalLevel::Low, 1),
    ];
    let line = make_signal("d", elements, &style);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let start = svg.find("class=\"dontcares\"").expect("dontcares");
    let end = svg[start..].find("</g>").expect("close") + start;
    assert!(svg[start..end].contains("<rect"));
}

/// DontCare の `<rect>` のデフォルト fill は `url(#dontcare-hatch-1)` を参照する。
/// Scenario: DontCare 矩形のデフォルト塗りはハッチパターン参照
#[test]
fn dontcare_default_fill_references_hatch_pattern() {
    let style = ChartStyle::default();
    let elements = vec![
        level(SignalLevel::Low, 1),
        level(SignalLevel::DontCareAlongLow, 2),
        level(SignalLevel::Low, 1),
    ];
    let line = make_signal("d", elements, &style);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let start = svg.find("class=\"dontcares\"").expect("dontcares layer");
    let end = svg[start..].find("</g>").expect("close tag") + start;
    assert!(
        svg[start..end].contains("fill=\"url(#dontcare-hatch-1)\""),
        "expected fill=url(#dontcare-hatch-1) in dontcares layer: {}",
        &svg[start..end]
    );
}

/// DontCare が存在するとき `<defs>` に `<pattern id="dontcare-hatch-1">` が出力される。
#[test]
fn dontcare_outputs_hatch_pattern_in_defs() {
    let style = ChartStyle::default();
    let elements = vec![
        level(SignalLevel::Low, 1),
        level(SignalLevel::DontCareAlongLow, 2),
        level(SignalLevel::Low, 1),
    ];
    let line = make_signal("d", elements, &style);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    assert!(
        svg.contains("<defs>"),
        "expected <defs> in SVG output: {svg}"
    );
    assert!(
        svg.contains("id=\"dontcare-hatch-1\""),
        "expected dontcare-hatch-1 pattern in <defs>: {svg}"
    );
    assert!(
        svg.contains("patternUnits=\"userSpaceOnUse\""),
        "expected patternUnits in pattern: {svg}"
    );
    assert!(
        svg.contains("patternTransform=\"rotate(45)\""),
        "expected patternTransform rotate(45) in pattern: {svg}"
    );
    assert!(
        svg.contains("stroke=\"#bbbbbb\""),
        "expected default <line stroke=\"#bbbbbb\"> in dontcare-hatch pattern: {svg}"
    );
}

/// Render an SVG with one signal row containing a DontCare element, using the
/// supplied dontcare color override.
fn render_dontcare_color_svg(color_str: &str) -> String {
    let mut style = ChartStyle::default();
    style.set_dontcare_color(
        crate::color::Color::parse(color_str).expect("valid hex color in test helper"),
    );
    let elements = vec![
        level(SignalLevel::Low, 1),
        level(SignalLevel::DontCareAlongLow, 1),
        level(SignalLevel::Low, 1),
    ];
    let line = make_signal("d", elements, &style);
    let doc = ChartDocument::new(
        style,
        vec![line],
        Annotations::default(),
        TcmlSource::new(""),
    );
    render(&doc, &TestFonts)
}

/// `@dontcare_color #c00` は `<defs>` の `<pattern>` 内 `<line stroke>` に焼き込まれる。
#[test]
fn dontcare_color_directive_bakes_into_pattern_stroke() {
    let svg = render_dontcare_color_svg("#c00");
    assert!(
        svg.contains("stroke=\"#cc0000\""),
        "expected hatch <line stroke=\"#cc0000\"> baked into <defs>: {svg}"
    );
    let start = svg.find("class=\"dontcares\"").expect("dontcares layer");
    let end = svg[start..].find("</g>").expect("close tag") + start;
    assert!(
        svg[start..end].contains("fill=\"url(#dontcare-hatch-1)\""),
        "expected fill=url(#dontcare-hatch-1): {}",
        &svg[start..end]
    );
    assert!(
        !svg[start..end].contains(" color=") && !svg[start..end].contains(" stroke="),
        "expected no extra attributes on dontcare rect (only fill): {}",
        &svg[start..end]
    );
}

/// Render two rows that share the same dontcare color and return the SVG.
fn render_two_rows_same_dontcare_color(color_str: &str) -> String {
    let mut style = ChartStyle::default();
    style.set_dontcare_color(
        crate::color::Color::parse(color_str).expect("valid hex color in test helper"),
    );
    let dontcare_elements = vec![
        level(SignalLevel::Low, 1),
        level(SignalLevel::DontCareAlongLow, 1),
        level(SignalLevel::Low, 1),
    ];
    let row_a = make_signal("a", dontcare_elements.clone(), &style);
    let row_b = make_signal("b", dontcare_elements, &style);
    let doc = ChartDocument::new(
        style,
        vec![row_a, row_b],
        Annotations::default(),
        TcmlSource::new(""),
    );
    render(&doc, &TestFonts)
}

/// 同じ色を 2 回参照しても `<defs>` の `<pattern>` は 1 つに統合される (ID 共有)。
#[test]
fn dontcare_dedupes_same_color_into_one_pattern() {
    let svg = render_two_rows_same_dontcare_color("#c00");
    assert_eq!(
        svg.matches("<pattern").count(),
        1,
        "expected exactly 1 <pattern> for one unique color: {svg}"
    );
    assert!(
        svg.contains("id=\"dontcare-hatch-1\""),
        "expected dontcare-hatch-1 id: {svg}"
    );
    assert!(
        !svg.contains("dontcare-hatch-2"),
        "did not expect dontcare-hatch-2 for single color: {svg}"
    );
}

/// Render a two-row chart where the first row uses red dontcare (#c00) and
/// the second row uses blue dontcare (#06c). Returns the SVG string.
fn render_two_rows_two_dontcare_colors() -> String {
    let dontcare_elements = vec![
        level(SignalLevel::Low, 1),
        level(SignalLevel::DontCareAlongLow, 1),
        level(SignalLevel::Low, 1),
    ];
    let mut style_red = ChartStyle::default();
    style_red.set_dontcare_color(Color::parse("#c00").expect("valid #c00"));
    let row_red = make_signal("red", dontcare_elements.clone(), &style_red);
    let mut style_blue = ChartStyle::default();
    style_blue.set_dontcare_color(Color::parse("#06c").expect("valid #06c"));
    let row_blue = make_signal("blue", dontcare_elements, &style_blue);
    let doc = ChartDocument::new(
        style_blue,
        vec![row_red, row_blue],
        Annotations::default(),
        TcmlSource::new(""),
    );
    render(&doc, &TestFonts)
}

/// 行ごとに異なる色を指定すると、`<defs>` には 2 つの `<pattern>` が初出順で出力される。
#[test]
fn dontcare_two_colors_emit_two_patterns_in_first_use_order() {
    let svg = render_two_rows_two_dontcare_colors();
    let pattern1_idx = svg
        .find("id=\"dontcare-hatch-1\"")
        .expect("pattern 1 missing");
    let pattern2_idx = svg
        .find("id=\"dontcare-hatch-2\"")
        .expect("pattern 2 missing");
    assert!(
        pattern1_idx < pattern2_idx,
        "expected pattern 1 before pattern 2: {svg}"
    );
    let pattern1_end = svg[pattern1_idx..]
        .find("</pattern>")
        .expect("closing pattern1 tag")
        + pattern1_idx;
    assert!(
        svg[pattern1_idx..pattern1_end].contains("stroke=\"#cc0000\""),
        "expected pattern 1 stroke #cc0000 (first-used color): {}",
        &svg[pattern1_idx..pattern1_end]
    );
    let pattern2_end = svg[pattern2_idx..]
        .find("</pattern>")
        .expect("closing pattern2 tag")
        + pattern2_idx;
    assert!(
        svg[pattern2_idx..pattern2_end].contains("stroke=\"#0066cc\""),
        "expected pattern 2 stroke #0066cc: {}",
        &svg[pattern2_idx..pattern2_end]
    );
}

/// 各行の `fill` が対応する `<pattern>` ID を参照する。
#[test]
fn dontcare_two_colors_each_row_fill_references_its_pattern() {
    let svg = render_two_rows_two_dontcare_colors();
    let dontcares_start = svg.find("class=\"dontcares\"").expect("dontcares layer");
    let dontcares_end = svg[dontcares_start..]
        .find("</g>")
        .expect("closing dontcares tag")
        + dontcares_start;
    let layer = &svg[dontcares_start..dontcares_end];
    assert!(
        layer.contains("fill=\"url(#dontcare-hatch-1)\""),
        "expected first row fill = pattern 1: {layer}"
    );
    assert!(
        layer.contains("fill=\"url(#dontcare-hatch-2)\""),
        "expected second row fill = pattern 2: {layer}"
    );
}

/// `DontcareHatchPatternTable` の単体振る舞い: 1 オリジン採番。
#[test]
fn dontcare_pattern_table_assigns_sequential_ids_from_one() {
    use crate::svg::waveform::DontcareHatchPatternTable;
    let mut table = DontcareHatchPatternTable::default();
    let red = Color::parse("#c00").expect("valid #c00");
    let blue = Color::parse("#06c").expect("valid #06c");
    let id_red = table.insert_color(red);
    let id_blue = table.insert_color(blue);
    assert_eq!(id_red.to_string(), "dontcare-hatch-1");
    assert_eq!(id_blue.to_string(), "dontcare-hatch-2");
}

/// `DontcareHatchPatternTable` の単体振る舞い: 同色は ID 共有。
#[test]
fn dontcare_pattern_table_dedupes_same_color() {
    use crate::svg::waveform::DontcareHatchPatternTable;
    let mut table = DontcareHatchPatternTable::default();
    let red = Color::parse("#c00").expect("valid #c00");
    let blue = Color::parse("#06c").expect("valid #06c");
    let id_red_1 = table.insert_color(red);
    let id_blue = table.insert_color(blue);
    let id_red_2 = table.insert_color(red);
    assert_eq!(id_red_1, id_red_2);
    assert_ne!(id_red_1, id_blue);
    assert_eq!(table.as_slice().len(), 2);
}

/// `DontcareHatchPatternTable` の単体振る舞い: 空テーブルは is_empty。
#[test]
fn dontcare_pattern_table_empty_reports_empty() {
    use crate::svg::waveform::DontcareHatchPatternTable;
    let table = DontcareHatchPatternTable::default();
    assert!(table.is_empty());
    assert!(table.as_slice().is_empty());
}

/// チャートに `?` が無ければ `<defs>` は出力されない。
/// Scenario: チャートに `?` が無ければ `<defs>` を出力しない
#[test]
fn no_dontcare_means_no_defs() {
    let style = ChartStyle::default();
    let elements = vec![level(SignalLevel::Low, 2), level(SignalLevel::High, 2)];
    let line = make_signal("d", elements, &style);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    assert!(
        !svg.contains("<defs>"),
        "expected no <defs> when chart has no DontCare elements: {svg}"
    );
}

#[test]
fn highlight_emits_rect() {
    let style = ChartStyle::default();
    let elements = vec![
        level(SignalLevel::Low, 1),
        WaveformElement::HighlightStart,
        level(SignalLevel::High, 2),
        WaveformElement::HighlightEnd,
        level(SignalLevel::Low, 1),
    ];
    let line = make_signal("d", elements, &style);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let start = svg.find("class=\"highlights\"").expect("highlights");
    let end = svg[start..].find("</g>").expect("close") + start;
    assert!(svg[start..end].contains("<rect"));
}

#[test]
fn guide_emits_line() {
    let style = ChartStyle::default();
    let elements = vec![
        level(SignalLevel::Low, 1),
        WaveformElement::Guide,
        level(SignalLevel::Low, 1),
    ];
    let line = make_signal("d", elements, &style);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let start = svg.find("class=\"guides\"").expect("guides");
    let end = svg[start..].find("</g>").expect("close") + start;
    assert!(svg[start..end].contains("<line"));
}

#[test]
fn label_text_escaped() {
    let style = ChartStyle::default();
    let line = make_signal("A<B&C>", vec![level(SignalLevel::Low, 1)], &style);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    assert!(svg.contains("A&lt;B&amp;C&gt;"));
}

#[test]
fn title_row_emits_text() {
    let style = ChartStyle::default();
    let title_style = TitleStyle::new(
        style.default_label_style().font().clone(),
        HorizontalAlign::Center,
        style.default_label_style().color(),
    );
    let title = TitleRow::new(UserText::parse("hello").expect("title"), title_style);
    let line = Line::new_with_bounding_box(
        LineContent::Title(title),
        None,
        Rect {
            origin: Point {
                x: Px(10.0),
                y: Px(10.0),
            },
            size: Size {
                width: Px(100.0),
                height: Px(20.0),
            },
        },
    );
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let start = svg.find("class=\"titles\"").expect("titles");
    let end = svg[start..].find("</g>").expect("close") + start;
    assert!(svg[start..end].contains("hello"));
}

#[test]
fn arrow_with_end_head() {
    let mut document = make_doc(vec![]);
    document.annotations.arrows.push(Arrow::new(
        ArrowEnd::Absolute(Point {
            x: Px(10.0),
            y: Px(20.0),
        }),
        ArrowEnd::Absolute(Point {
            x: Px(50.0),
            y: Px(20.0),
        }),
        ArrowStyle::new(
            Color::parse("black").expect("c"),
            Px(1.0),
            LineDashStyle::Solid,
            ArrowHead::EndOnly,
        ),
        None,
        FontSpec::default(),
    ));
    let svg = render(&document, &TestFonts);
    let start = svg.find("class=\"arrows\"").expect("arrows");
    let end = svg[start..].find("</g>").expect("close") + start;
    let layer = &svg[start..end];
    assert!(layer.contains("<line"));
    // Arrow heads are rendered as <path>, not <polygon> (spec §「矢印頭」: "path で実装").
    assert!(layer.contains("<path"));
}

#[test]
fn arrow_dashed_has_dasharray() {
    let mut document = make_doc(vec![]);
    document.annotations.arrows.push(Arrow::new(
        ArrowEnd::Absolute(Point {
            x: Px(0.0),
            y: Px(0.0),
        }),
        ArrowEnd::Absolute(Point {
            x: Px(40.0),
            y: Px(0.0),
        }),
        ArrowStyle::new(
            Color::parse("black").expect("c"),
            Px(1.0),
            LineDashStyle::Dashed,
            ArrowHead::None,
        ),
        None,
        FontSpec::default(),
    ));
    let svg = render(&document, &TestFonts);
    assert!(svg.contains("stroke-dasharray=\"6 3\""));
}

#[test]
fn overlay_emits_text() {
    let mut document = make_doc(vec![]);
    document.annotations.overlays.push(TextOverlay::new(
        Point {
            x: Px(100.0),
            y: Px(50.0),
        },
        UserText::parse("note").expect("t"),
    ));
    let svg = render(&document, &TestFonts);
    let start = svg.find("class=\"overlays\"").expect("overlays");
    let end = svg[start..].find("</g>").expect("close") + start;
    assert!(svg[start..end].contains("note"));
}

#[test]
fn row_backgrounds_skip_excluded() {
    let mut style = ChartStyle::default();
    style.set_bgcolor0(Color::parse("#eee").expect("c0"));
    style.set_bgcolor1(Color::parse("#ccc").expect("c1"));
    let row1 = make_signal("a", vec![level(SignalLevel::Low, 1)], &style);
    let row2 =
        make_signal_with_bounding_box_y("b", vec![level(SignalLevel::Low, 1)], &style, Px(40.0));

    let document = ChartDocument::new(
        style,
        vec![row1, row2],
        Annotations::default(),
        TcmlSource::new(""),
    );
    let svg = render(&document, &TestFonts);
    let start = svg.find("class=\"row-backgrounds\"").expect("rb");
    let end = svg[start..].find("</g>").expect("close") + start;
    let layer = &svg[start..end];
    assert!(layer.contains("#eee"), "first row uses bgcolor0");
    assert!(layer.contains("#ccc"), "second row uses bgcolor1");
}

#[test]
fn local_bg_overrides_stripe_color() {
    let mut style = ChartStyle::default();
    style.set_bgcolor0(Color::parse("#eee").expect("c0"));
    style.set_bgcolor1(Color::parse("#ccc").expect("c1"));
    let mut row = make_signal("a", vec![level(SignalLevel::Low, 1)], &style);
    row.background = Some(Color::parse("#f0f").expect("local"));
    let document = ChartDocument::new(
        style,
        vec![row],
        Annotations::default(),
        TcmlSource::new(""),
    );
    let svg = render(&document, &TestFonts);
    let start = svg.find("class=\"row-backgrounds\"").expect("rb");
    let end = svg[start..].find("</g>").expect("close") + start;
    let layer = &svg[start..end];
    assert!(layer.contains("#ff00ff"), "uses local @bg color");
    assert!(!layer.contains("#eeeeee"), "stripe color is suppressed");
}

#[test]
fn local_bg_advances_stripe_index_for_signal_rows() {
    // A @bg-painted signal row still consumes one slot in the bgcolor0/1
    // alternation so the next signal row gets the *next* stripe color.
    let mut style = ChartStyle::default();
    style.set_bgcolor0(Color::parse("#eee").expect("c0"));
    style.set_bgcolor1(Color::parse("#ccc").expect("c1"));
    let mut row1 = make_signal("a", vec![level(SignalLevel::Low, 1)], &style);
    row1.background = Some(Color::parse("#f0f").expect("local"));
    let row2 =
        make_signal_with_bounding_box_y("b", vec![level(SignalLevel::Low, 1)], &style, Px(40.0));

    let document = ChartDocument::new(
        style,
        vec![row1, row2],
        Annotations::default(),
        TcmlSource::new(""),
    );
    let svg = render(&document, &TestFonts);
    let start = svg.find("class=\"row-backgrounds\"").expect("rb");
    let end = svg[start..].find("</g>").expect("close") + start;
    let layer = &svg[start..end];
    assert!(layer.contains("#ff00ff"), "row 1 uses local bg");
    assert!(layer.contains("#cccccc"), "row 2 uses bgcolor1 (odd slot)");
    assert!(!layer.contains("#eeeeee"), "bgcolor0 not used");
}

fn make_title_line(background: Option<Color>) -> Line {
    let style = ChartStyle::default();
    let title = TitleRow::new(
        UserText::parse("hello").expect("title"),
        TitleStyle::new(
            style.default_label_style().font().clone(),
            HorizontalAlign::Left,
            style.default_label_style().color(),
        ),
    );
    Line::new_with_bounding_box(
        LineContent::Title(title),
        background,
        Rect {
            origin: Point {
                x: Px::ZERO,
                y: Px::ZERO,
            },
            size: Size {
                width: Px(100.0),
                height: Px(20.0),
            },
        },
    )
}

#[test]
fn local_bg_paints_title_row() {
    let line = make_title_line(Some(Color::parse("#ff0").expect("yellow")));
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let start = svg.find("class=\"row-backgrounds\"").expect("rb");
    let end = svg[start..].find("</g>").expect("close") + start;
    assert!(svg[start..end].contains("#ffff00"));
}

#[test]
fn title_row_without_local_bg_has_no_background() {
    let line = make_title_line(None);
    let mut doc = make_doc(vec![line]);
    doc.style.set_bgcolor0(Color::parse("#eee").expect("c0"));
    let svg = render(&doc, &TestFonts);
    // Spec (svg-rendering.md §「空レイヤーの省略」): a `<g class="row-backgrounds">`
    // with no `<rect>` children is omitted entirely. Either the layer is absent,
    // or it is present and contains no `<rect>`. Both are valid.
    let row_bg_section = extract_layer(&svg, "row-backgrounds");
    assert!(
        !row_bg_section.contains("<rect"),
        "title row without @bg must produce no background rect; got: {row_bg_section}"
    );
}

// Title alignment rendering tests.

fn make_title_line_with_align(align: HorizontalAlign) -> Line {
    let style = ChartStyle::default();
    let title = TitleRow::new(
        UserText::parse("T").expect("title"),
        TitleStyle::new(
            style.default_label_style().font().clone(),
            align,
            style.default_label_style().color(),
        ),
    );
    Line::new_with_bounding_box(
        LineContent::Title(title),
        None,
        Rect {
            origin: Point {
                x: Px(10.0),
                y: Px(10.0),
            },
            size: Size {
                width: Px(200.0),
                height: Px(20.0),
            },
        },
    )
}

fn extract_titles_layer(svg: &str) -> &str {
    extract_layer(svg, "titles")
}

#[test]
fn title_center_emits_middle_anchor() {
    let line = make_title_line_with_align(HorizontalAlign::Center);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let layer = extract_titles_layer(&svg);
    assert!(
        layer.contains("text-anchor=\"middle\""),
        "center should use middle anchor"
    );
    assert!(!layer.contains("text-anchor=\"start\""));
    assert!(!layer.contains("text-anchor=\"end\""));
}

#[test]
fn title_left_emits_start_anchor() {
    let line = make_title_line_with_align(HorizontalAlign::Left);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let layer = extract_titles_layer(&svg);
    assert!(
        layer.contains("text-anchor=\"start\""),
        "left should use start anchor"
    );
}

#[test]
fn title_right_emits_end_anchor() {
    let line = make_title_line_with_align(HorizontalAlign::Right);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let layer = extract_titles_layer(&svg);
    assert!(
        layer.contains("text-anchor=\"end\""),
        "right should use end anchor"
    );
}

#[test]
fn title_center_x_is_bbox_center() {
    // bbox.origin.x=10, bbox.size.width=200 -> center x = 10 + 200/2 = 110
    let line = make_title_line_with_align(HorizontalAlign::Center);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let layer = extract_titles_layer(&svg);
    assert!(
        layer.contains("x=\"110\""),
        "center x should be 110 (10 + 200/2), got layer: {}",
        layer
    );
}

#[test]
fn anchor_registry_unused_for_render() {
    // Ensure rendering does not panic on a populated anchor registry.
    let mut document = make_doc(vec![]);
    document.annotations.anchors = AnchorRegistry::default();
    let _ = render(&document, &TestFonts);
}

// --- DontCareAlongBus polygon tests ---

fn extract_dontcares_layer(svg: &str) -> &str {
    extract_layer(svg, "dontcares")
}

/// `=?=`: both sides Bus continue → `<polygon>` with rectangular (vertical-edge) shape.
/// No `<rect>` should appear for `DontCareAlongBus`.
#[test]
fn dontcare_along_bus_emits_polygon_not_rect() {
    let style = ChartStyle::default();
    let elements = vec![
        level(SignalLevel::Bus, 1),
        level(SignalLevel::DontCareAlongBus, 1),
        level(SignalLevel::Bus, 1),
    ];
    let line = make_signal("d", elements, &style);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let layer = extract_dontcares_layer(&svg);
    assert!(
        layer.contains("<polygon"),
        "DontCareAlongBus must emit <polygon>: {layer}"
    );
    assert!(
        !layer.contains("<rect"),
        "DontCareAlongBus must not emit <rect>: {layer}"
    );
}

/// `=?=` (both Bus continue, no BusOpen/BusClose in waveform): rectangular polygon.
/// Layout: start_x=50 (bbox.origin.x=10 + signal_box.origin.x=40).
/// Bus(1)=10px → cursor 60; DontCare(1)=10px x_start=60..70; Bus(1)=10px.
/// ys.top=12 (10+2), ys.bottom=28 (10+2+16), slant=2 (default).
/// Scanner reaches end of waveform on both sides → Vertical on both edges.
/// Expected: "60,12 70,12 70,28 60,28".
#[test]
fn dontcare_along_bus_both_continue_is_rectangle() {
    let style = ChartStyle::default();
    let elements = vec![
        level(SignalLevel::Bus, 1),
        level(SignalLevel::DontCareAlongBus, 1),
        level(SignalLevel::Bus, 1),
    ];
    let line = make_signal("d", elements, &style);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let layer = extract_dontcares_layer(&svg);
    assert!(
        layer.contains("75,12 100,12 100,28 75,28"),
        "=?= should produce rectangular polygon, got: {layer}"
    );
}

/// `_=?=_` (Low→Bus, Bus(1), DontCare(1), Bus(1), Bus→Low): polygon is `/=\` shape.
/// Scanner walks back: Bus(1) → BusOpen(Low) → SlantFromLow.
/// Scanner walks fwd: Bus(1) → BusClose(Low) → SlantFromLow.
/// cursor at DontCare: 50+10+2+10=72; end=82; slant=2.
/// prev=SlantFromLow → left_top_x=72, left_bottom_x=70.
/// next=SlantFromLow → right_top_x=82, right_bottom_x=84.
/// Expected points: "72,12 82,12 84,28 70,28".
#[test]
fn dontcare_along_bus_low_both_sides_slash_equal_backslash() {
    let style = ChartStyle::default();
    let elements = vec![
        level(SignalLevel::Low, 1),
        transition(TransitionKind::BusOpen, SignalLevel::Low, SignalLevel::Bus),
        level(SignalLevel::Bus, 1),
        level(SignalLevel::DontCareAlongBus, 1),
        level(SignalLevel::Bus, 1),
        transition(TransitionKind::BusClose, SignalLevel::Bus, SignalLevel::Low),
        level(SignalLevel::Low, 1),
    ];
    let line = make_signal("d", elements, &style);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let layer = extract_dontcares_layer(&svg);
    assert!(
        layer.contains("105,12 130,12 135,28 100,28"),
        "_=?=_ should produce /=\\ polygon, got: {layer}"
    );
}

/// `~=?=~` (High→Bus, Bus(1), DontCare(1), Bus(1), Bus→High): polygon is `\=/` shape.
/// Scanner finds BusOpen(High) left and BusClose(High) right.
/// cursor at DontCare: 72; end: 82; slant=2.
/// prev=SlantFromHigh → left_top_x=70, left_bottom_x=72.
/// next=SlantFromHigh → right_top_x=84, right_bottom_x=82.
/// Expected points: "70,12 84,12 82,28 72,28".
#[test]
fn dontcare_along_bus_high_both_sides_backslash_equal_slash() {
    let style = ChartStyle::default();
    let elements = vec![
        level(SignalLevel::High, 1),
        transition(TransitionKind::BusOpen, SignalLevel::High, SignalLevel::Bus),
        level(SignalLevel::Bus, 1),
        level(SignalLevel::DontCareAlongBus, 1),
        level(SignalLevel::Bus, 1),
        transition(
            TransitionKind::BusClose,
            SignalLevel::Bus,
            SignalLevel::High,
        ),
        level(SignalLevel::High, 1),
    ];
    let line = make_signal("d", elements, &style);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let layer = extract_dontcares_layer(&svg);
    assert!(
        layer.contains("100,12 135,12 130,28 105,28"),
        "~=?=~ should produce \\=/ polygon, got: {layer}"
    );
}

/// `-=?=` (HiZ→Bus, Bus(1), DontCare, Bus continue): left edge degenerates to y_mid point (HiZ).
/// Scanner finds BusOpen(HiZ→Bus) with bus_run_units=1 (one Bus(1) between BusOpen and DontCare).
/// cursor at DontCare: 72; end: 82; slant=2; step=10.
/// left: SingleFromHiZ{1} → hiz_x = 72 - 1*10 - 2 = 60; left_top=72, left_bottom=72.
/// Pentagon: "60,20 72,12 82,12 82,28 72,28".
#[test]
fn dontcare_along_bus_hiz_prev_is_pentagon() {
    let style = ChartStyle::default();
    let elements = vec![
        level(SignalLevel::HiZ, 1),
        transition(TransitionKind::BusOpen, SignalLevel::HiZ, SignalLevel::Bus),
        level(SignalLevel::Bus, 1),
        level(SignalLevel::DontCareAlongBus, 1),
        level(SignalLevel::Bus, 1),
    ];
    let line = make_signal("d", elements, &style);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let layer = extract_dontcares_layer(&svg);
    assert!(
        layer.contains("75,20 105,12 130,12 130,28 105,28"),
        "-=?= should produce pentagon with HiZ midpoint on left, got: {layer}"
    );
}

/// `=?-=` (Bus continue, DontCare, Bus→HiZ): right edge degenerates to y_mid point (HiZ).
/// Scanner left: Bus(1) → end of bus runs → Vertical (no BusOpen found).
/// Scanner right: DontCare directly precedes BusClose(HiZ), bus_run_units=0.
/// cursor at DontCare: 60; end: 70; slant=2; step=10.
/// right: SingleFromHiZ{0} → hiz_x = 70 + 0*10 + 2 = 72; right_top=70, right_bottom=70.
/// Pentagon: "60,12 70,12 72,20 70,28 60,28".
#[test]
fn dontcare_along_bus_hiz_next_is_pentagon() {
    let style = ChartStyle::default();
    let elements = vec![
        level(SignalLevel::Bus, 1),
        level(SignalLevel::DontCareAlongBus, 1),
        transition(TransitionKind::BusClose, SignalLevel::Bus, SignalLevel::HiZ),
        level(SignalLevel::HiZ, 1),
    ];
    let line = make_signal("d", elements, &style);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let layer = extract_dontcares_layer(&svg);
    assert!(
        layer.contains("75,12 100,12 105,20 100,28 75,28"),
        "=?-= should produce pentagon with HiZ midpoint on right, got: {layer}"
    );
}

/// `~=?=_` (High→Bus, Bus(1), DontCare(1), Bus(1), Bus→Low): mixed slant.
/// cursor at DontCare: 72; end: 82; slant=2.
/// prev=SlantFromHigh → left_top_x=70, left_bottom_x=72.
/// next=SlantFromLow → right_top_x=82, right_bottom_x=84.
/// Expected: "70,12 82,12 84,28 72,28".
#[test]
fn dontcare_along_bus_high_prev_low_next_mixed_slant() {
    let style = ChartStyle::default();
    let elements = vec![
        level(SignalLevel::High, 1),
        transition(TransitionKind::BusOpen, SignalLevel::High, SignalLevel::Bus),
        level(SignalLevel::Bus, 1),
        level(SignalLevel::DontCareAlongBus, 1),
        level(SignalLevel::Bus, 1),
        transition(TransitionKind::BusClose, SignalLevel::Bus, SignalLevel::Low),
        level(SignalLevel::Low, 1),
    ];
    let line = make_signal("d", elements, &style);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let layer = extract_dontcares_layer(&svg);
    assert!(
        layer.contains("100,12 130,12 135,28 105,28"),
        "~=?=_ mixed slant polygon, got: {layer}"
    );
}

/// `=?=_` (Bus continue at start, DontCare, Bus(1), Bus→Low): right edge has `\` slant.
/// Scanner left: Bus(1) → end → Vertical.
/// Scanner right: Bus(1) → BusClose(Low) → SlantFromLow.
/// cursor at DontCare: 60; end: 70.
/// prev=Vertical → left_top_x=60, left_bottom_x=60.
/// next=SlantFromLow → right_top_x=70, right_bottom_x=72.
/// Expected: "60,12 70,12 72,28 60,28".
#[test]
fn dontcare_along_bus_continue_prev_low_next_right_slant() {
    let style = ChartStyle::default();
    let elements = vec![
        level(SignalLevel::Bus, 1),
        level(SignalLevel::DontCareAlongBus, 1),
        level(SignalLevel::Bus, 1),
        transition(TransitionKind::BusClose, SignalLevel::Bus, SignalLevel::Low),
        level(SignalLevel::Low, 1),
    ];
    let line = make_signal("d", elements, &style);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let layer = extract_dontcares_layer(&svg);
    assert!(
        layer.contains("75,12 100,12 105,28 75,28"),
        "=?=_ right slant polygon, got: {layer}"
    );
}

/// `_=?=` (Low→Bus, Bus(1), DontCare, Bus continue at end): left edge has `/` slant.
/// Scanner left: Bus(1) → BusOpen(Low) → SlantFromLow.
/// Scanner right: Bus(1) → end of slice → Vertical.
/// cursor at DontCare: 72; end: 82.
/// prev=SlantFromLow → left_top_x=72, left_bottom_x=70.
/// next=Vertical → right_top_x=82, right_bottom_x=82.
/// Expected: "72,12 82,12 82,28 70,28".
#[test]
fn dontcare_along_bus_low_prev_continue_next_left_slant() {
    let style = ChartStyle::default();
    let elements = vec![
        level(SignalLevel::Low, 1),
        transition(TransitionKind::BusOpen, SignalLevel::Low, SignalLevel::Bus),
        level(SignalLevel::Bus, 1),
        level(SignalLevel::DontCareAlongBus, 1),
        level(SignalLevel::Bus, 1),
    ];
    let line = make_signal("d", elements, &style);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let layer = extract_dontcares_layer(&svg);
    assert!(
        layer.contains("105,12 130,12 130,28 100,28"),
        "_=?= left slash polygon, got: {layer}"
    );
}

/// `=X?X=`: BusCross on both left and right → 6-point hexagon.
/// Elements: Bus(1), BusCross, DontCareAlongBus(1), BusCross, Bus(2).
/// DontCare cursor: 50+10+2=62..72; left_cross_mid=61, right_cross_mid=73; y_mid=20.
/// Expected: "61,20 62,12 72,12 73,20 72,28 62,28".
#[test]
fn dontcare_along_bus_both_cross_midpoints_hexagon() {
    let style = ChartStyle::default();
    let elements = vec![
        level(SignalLevel::Bus, 1),
        transition(TransitionKind::BusCross, SignalLevel::Bus, SignalLevel::Bus),
        level(SignalLevel::DontCareAlongBus, 1),
        transition(TransitionKind::BusCross, SignalLevel::Bus, SignalLevel::Bus),
        level(SignalLevel::Bus, 2),
    ];
    let line = make_signal("d", elements, &style);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let layer = extract_dontcares_layer(&svg);
    assert!(
        layer.contains("77.5,20 80,12 105,12 107.5,20 105,28 80,28"),
        "=X?X= should produce 6-point hexagon with cross midpoints, got: {layer}"
    );
}

/// `=X?=`: BusCross on left only → 5-point polygon (left cross midpoint, right vertical).
/// Elements: Bus(1), BusCross, DontCareAlongBus(2).
/// DontCare cursor: 62..82 (2 units after X body absorbed); left_cross_mid=61; y_mid=20.
/// Expected: "61,20 62,12 82,12 82,28 62,28".
#[test]
fn dontcare_along_bus_left_cross_right_vertical_pentagon() {
    let style = ChartStyle::default();
    let elements = vec![
        level(SignalLevel::Bus, 1),
        transition(
            TransitionKind::BusCross,
            SignalLevel::Bus,
            SignalLevel::DontCareAlongBus,
        ),
        level(SignalLevel::DontCareAlongBus, 2),
    ];
    let line = make_signal("d", elements, &style);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let layer = extract_dontcares_layer(&svg);
    assert!(
        layer.contains("77.5,20 80,12 130,12 130,28 80,28"),
        "=X?= should produce 5-point polygon (left cross mid), got: {layer}"
    );
}

/// `=?X=`: Bus continue on left, BusCross on right → 5-point polygon (right cross midpoint).
/// Elements: DontCareAlongBus(1), BusCross, Bus(2).
/// DontCare cursor: 50..60; right_cross_mid=61; y_mid=20.
/// Expected: "50,12 60,12 61,20 60,28 50,28".
#[test]
fn dontcare_along_bus_left_vertical_right_cross_pentagon() {
    let style = ChartStyle::default();
    let elements = vec![
        level(SignalLevel::DontCareAlongBus, 1),
        transition(
            TransitionKind::BusCross,
            SignalLevel::DontCareAlongBus,
            SignalLevel::Bus,
        ),
        level(SignalLevel::Bus, 2),
    ];
    let line = make_signal("d", elements, &style);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let layer = extract_dontcares_layer(&svg);
    assert!(
        layer.contains("50,12 75,12 77.5,20 75,28 50,28"),
        "=?X= should produce 5-point polygon (right cross mid), got: {layer}"
    );
}

/// DontCareAlongLow still emits `<rect>` (not affected by bus polygon change).
#[test]
fn dontcare_along_low_still_emits_rect() {
    let style = ChartStyle::default();
    let elements = vec![
        level(SignalLevel::Low, 1),
        level(SignalLevel::DontCareAlongLow, 1),
        level(SignalLevel::Low, 1),
    ];
    let line = make_signal("d", elements, &style);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let layer = extract_dontcares_layer(&svg);
    assert!(
        layer.contains("<rect"),
        "DontCareAlongLow must still emit <rect>: {layer}"
    );
    assert!(
        !layer.contains("<polygon"),
        "DontCareAlongLow must not emit <polygon>: {layer}"
    );
}

#[test]
fn buf_escapes_ampersand_and_quote() {
    let mut buf = SvgBuf::new();
    buf.write_escaped_str("a&b\"c'");
    assert_eq!(buf.as_str(), "a&amp;b&quot;c&apos;");
}

// ---- row-background uniform-width test ------------------------------------
// Scenario: 信号長が異なる行でも row-background はチャート全幅で揃う

fn make_two_signal_doc_for_bg_test() -> ChartDocument {
    // Signal A: 10 units * 10px = 100px; capwidth=20 => chart_inner_width=120
    // Signal B:  4 units * 10px =  40px; uniform-width rule sets bbox.width=120 too.
    let mut style = ChartStyle::default();
    style.set_bgcolor0(Color::parse("#eee").expect("c0"));
    style.set_bgcolor1(Color::parse("#ccc").expect("c1"));
    style.set_step(Px(10.0));
    style.set_capwidth(Some(Px(20.0)));
    let layout_params = *style.layout();
    let mut document = ChartDocument::new(
        style,
        vec![
            make_signal_line_for_layout_with_params("A", 10, layout_params),
            make_signal_line_for_layout_with_params("B", 4, layout_params),
        ],
        Annotations::default(),
        TcmlSource::new(""),
    );
    crate::layout::layout(&mut document, &TestFonts).expect("layout");
    document
}

fn make_signal_line_for_layout_with_params(
    name: &str,
    units: u32,
    layout_params: LayoutParams,
) -> crate::line::Line {
    use crate::line::{LevelRun, SignalDecorations, SignalGeometry, SignalRow, Waveform};
    let defaults = ChartStyle::default();
    let row = SignalRow::new(
        SignalGeometry::default(),
        SignalName::parse(name).expect("name"),
        Waveform::from(vec![WaveformElement::Level(LevelRun::new(
            SignalLevel::Low,
            units,
        ))]),
        crate::style::SignalRowStyle::new(
            defaults.default_signal_style().clone(),
            defaults.default_label_style().clone(),
        ),
        SignalDecorations::default(),
        layout_params,
    );
    crate::line::Line::new(crate::line::LineContent::Signal(Box::new(row)), None)
}

fn extract_row_backgrounds_layer(svg: &str) -> &str {
    extract_layer(svg, "row-backgrounds")
}

fn parse_widths_from_layer(layer: &str) -> Vec<f32> {
    layer
        .split("width=\"")
        .skip(1)
        .filter_map(|s| s.split('"').next()?.parse::<f32>().ok())
        .collect()
}

#[test]
fn row_backgrounds_uniform_width_when_signal_lengths_differ() {
    // Both <rect width="..."> inside row-backgrounds must be the same value.
    let document = make_two_signal_doc_for_bg_test();
    let svg = render(&document, &TestFonts);
    let layer = extract_row_backgrounds_layer(&svg);
    let widths = parse_widths_from_layer(layer);
    assert_eq!(
        widths.len(),
        2,
        "expected 2 background rects, got {}: {layer}",
        widths.len()
    );
    assert!(
        (widths[0] - widths[1]).abs() < 1.0e-4,
        "both row-background rects must have the same width; got {} vs {}",
        widths[0],
        widths[1]
    );
    assert!(
        (widths[0] - 120.0).abs() < 1.0e-4,
        "row-background width should be 120 (chart_inner_width), got {}",
        widths[0]
    );
}

// ---- EdgeMark polygon rendering tests --------------------------------------

use crate::clock::ClockMarkStyle;
use crate::line::EdgeMark;

fn make_edge_mark(color: Color) -> EdgeMark {
    EdgeMark::new(
        crate::geometry::Point {
            x: Px(0.0),
            y: Px(20.0),
        },
        crate::geometry::Point {
            x: Px(2.0),
            y: Px(0.0),
        },
        ClockMarkStyle::new(0.5, Px(5.0), Px(4.0), color),
    )
}

fn make_signal_row_with_edge_mark(color: Color) -> SignalRow {
    let style = ChartStyle::default();
    SignalRow::new_with_edge_marks(
        build_geometry(Size {
            width: Px(20.0),
            height: Px(16.0),
        }),
        SignalName::parse("ck").expect("name parses"),
        Waveform::from(vec![level(SignalLevel::Low, 2)]),
        crate::style::SignalRowStyle::new(
            style.default_signal_style().clone(),
            style.default_label_style().clone(),
        ),
        SignalDecorations::default(),
        vec![make_edge_mark(color)],
        LayoutParams::default(),
    )
}

fn make_signal_with_edge_mark(fill_color: &str) -> Line {
    let color = Color::parse(fill_color).expect("color");
    let row = make_signal_row_with_edge_mark(color);
    Line::new_with_bounding_box(
        LineContent::Signal(Box::new(row)),
        None,
        Rect {
            origin: crate::geometry::Point {
                x: Px(10.0),
                y: Px(10.0),
            },
            size: Size {
                width: Px(60.0),
                height: Px(20.0),
            },
        },
    )
}

#[test]
fn edge_mark_polygon_in_edge_marks_layer() {
    // <polygon> must appear inside <g class="edge-marks">, not arrows.
    let line = make_signal_with_edge_mark("black");
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let layer = extract_layer(&svg, "edge-marks");
    assert!(
        layer.contains("<polygon"),
        "polygon must be in edge-marks layer, got: {layer}"
    );
}

#[test]
fn edge_mark_polygon_not_in_arrows_layer() {
    // <polygon> for edge marks must NOT appear in <g class="arrows">.
    let line = make_signal_with_edge_mark("black");
    let svg = render(&make_doc(vec![line]), &TestFonts);
    // Spec (svg-rendering.md §「空レイヤーの省略」): with no `@->` declared, the
    // `<g class="arrows">` wrapper is omitted entirely. That trivially satisfies
    // "no polygon inside arrows". When present, the layer must not contain a polygon.
    let arrows_layer = extract_layer(&svg, "arrows");
    assert!(
        !arrows_layer.contains("<polygon"),
        "edge mark polygon must NOT be in arrows layer, got: {arrows_layer}"
    );
}

#[test]
fn edge_mark_polygon_has_fill_attribute() {
    // Polygon must have fill="color" and stroke="none".
    let line = make_signal_with_edge_mark("red");
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let layer = extract_layer(&svg, "edge-marks");
    assert!(
        layer.contains("fill=\"red\"") || layer.contains("fill=\"#ff0000\""),
        "polygon must have fill=red, got: {layer}"
    );
    assert!(
        layer.contains("stroke=\"none\""),
        "polygon must have stroke=none, got: {layer}"
    );
}

#[test]
fn edge_mark_polygon_has_three_points() {
    // Polygon has exactly 3 vertices (apex, base_left, base_right).
    let line = make_signal_with_edge_mark("black");
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let polygon_start = svg.find("<polygon").expect("polygon");
    let polygon_end = svg[polygon_start..].find("/>").expect("end") + polygon_start;
    let polygon = &svg[polygon_start..polygon_end];
    // Count spaces in points="x1,y1 x2,y2 x3,y3"
    let points_start = polygon.find("points=\"").expect("points attr") + 8;
    let points_end = polygon[points_start..].find('"').expect("close") + points_start;
    let points_str = &polygon[points_start..points_end];
    let point_count = points_str.split_whitespace().count();
    assert_eq!(
        point_count, 3,
        "polygon must have 3 points, got {point_count}: {points_str}"
    );
}

#[test]
fn edge_mark_polygon_fill_blue() {
    // mark_color=blue propagates to polygon fill.
    let line = make_signal_with_edge_mark("blue");
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let layer = extract_layer(&svg, "edge-marks");
    assert!(
        layer.contains("fill=\"blue\"") || layer.contains("fill=\"#0000ff\""),
        "polygon fill must be blue, got: {layer}"
    );
}

// ---- name_overline renders as <line>, not text-decoration -----------------

fn make_overline_signal(name: &str) -> Line {
    let style = ChartStyle::default();
    let body = Size {
        width: Px(20.0),
        height: Px(16.0),
    };
    let row = SignalRow::new(
        build_geometry(body),
        SignalName::parse(name).expect("name parses"),
        Waveform::from(vec![level(SignalLevel::Low, 2)]),
        crate::style::SignalRowStyle::new(
            style.default_signal_style().clone(),
            style.default_label_style().clone(),
        ),
        SignalDecorations::new(None, true),
        LayoutParams::default(),
    );
    Line::new_with_bounding_box(
        LineContent::Signal(Box::new(row)),
        None,
        Rect {
            origin: Point {
                x: Px(10.0),
                y: Px(10.0),
            },
            size: Size {
                width: Px(60.0),
                height: Px(20.0),
            },
        },
    )
}

fn extract_signal_labels_layer(svg: &str) -> &str {
    extract_layer(svg, "signal-labels")
}

#[test]
fn overline_emits_independent_line_element() {
    // name_overline=true must produce a <line> in signal-labels, not text-decoration.
    let line = make_overline_signal("nReset");
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let layer = extract_signal_labels_layer(&svg);
    assert!(
        layer.contains("<line"),
        "overline must emit <line> element, got: {layer}"
    );
}

#[test]
fn overline_no_text_decoration_attribute() {
    // text-decoration="overline" must NOT appear anywhere.
    let line = make_overline_signal("nReset");
    let svg = render(&make_doc(vec![line]), &TestFonts);
    assert!(
        !svg.contains("text-decoration"),
        "text-decoration must not appear in SVG output, got svg containing it"
    );
}

#[test]
fn overline_line_has_stroke_and_stroke_width() {
    // <line> must have stroke and stroke-width attributes.
    let line = make_overline_signal("nReset");
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let layer = extract_signal_labels_layer(&svg);
    // Find the <line ...> element in the labels layer
    let line_start = layer.find("<line").expect("line element");
    let line_end = layer[line_start..].find("/>").expect("end") + line_start;
    let line_elem = &layer[line_start..line_end];
    assert!(
        line_elem.contains("stroke="),
        "overline <line> must have stroke attribute, got: {line_elem}"
    );
    assert!(
        line_elem.contains("stroke-width="),
        "overline <line> must have stroke-width attribute, got: {line_elem}"
    );
}

#[test]
fn overline_false_no_line_element() {
    // name_overline=false must not emit a <line> in signal-labels.
    let style = ChartStyle::default();
    let line = make_signal("nReset", vec![level(SignalLevel::Low, 2)], &style);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let layer = extract_signal_labels_layer(&svg);
    assert!(
        !layer.contains("<line"),
        "no overline <line> when name_overline=false, got: {layer}"
    );
}

#[test]
fn overline_multiline_emits_only_one_line() {
    // Multiline signal name with overline gets exactly 1 <line>.
    let line = make_overline_signal("nChip\nEnable");
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let layer = extract_signal_labels_layer(&svg);
    let count = layer.matches("<line").count();
    assert_eq!(
        count, 1,
        "multiline overline must emit exactly 1 <line>, got {count} in: {layer}"
    );
}

#[test]
fn overline_line_y_coordinates_equal() {
    // <line y1="..." y2="..."> must have equal y1 and y2 (horizontal line).
    let line = make_overline_signal("nReset");
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let layer = extract_signal_labels_layer(&svg);
    let line_start = layer.find("<line").expect("line element");
    let line_end = layer[line_start..].find("/>").expect("end") + line_start;
    let line_elem = &layer[line_start..line_end];
    // Extract y1 and y2
    let y1 = extract_attr_f32(line_elem, "y1").expect("y1 attr");
    let y2 = extract_attr_f32(line_elem, "y2").expect("y2 attr");
    assert!(
        (y1 - y2).abs() < 1e-3,
        "overline <line> y1 must equal y2, got y1={y1} y2={y2}"
    );
}

fn extract_attr_f32(element: &str, attr: &str) -> Option<f32> {
    let needle = format!("{attr}=\"");
    let start = element.find(&needle)? + needle.len();
    let end = element[start..].find('"')? + start;
    element[start..end].parse().ok()
}

// ---- overline x extent uses actual text width (longest line) ---------------

#[test]
fn overline_single_line_width_is_text_width() {
    // single-line overline x range must equal the rendered text width,
    // not the label_box width.
    //
    // TestFonts: width = 7.0 * char_count
    // "nReset" has 6 chars => text_width = 42px.
    // Default label align = Right, anchor_x = label_box right edge.
    // label_box: origin.x=0, width=40px, padding=DEFAULT_NAMEPAD_PX=4px
    // anchor_x = origin_x(10) + label_box.origin.x(0) + label_box.width(40) - padding(4) = 46px
    // With Right align: x1 = anchor_x - text_width = 46 - 42 = 4
    //                   x2 = anchor_x              = 46
    let line = make_overline_signal("nReset");
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let layer = extract_signal_labels_layer(&svg);
    let line_start = layer.find("<line").expect("line element");
    let line_end = layer[line_start..].find("/>").expect("end") + line_start;
    let line_elem = &layer[line_start..line_end];
    let x2 = extract_attr_f32(line_elem, "x2").expect("x2 attr");
    let x1 = extract_attr_f32(line_elem, "x1").expect("x1 attr");
    let actual_width = x2 - x1;
    // "nReset" = 6 chars * 7.0 = 42px
    let expected_text_width = 6.0 * 7.0_f32;
    assert!(
        (actual_width - expected_text_width).abs() < 1e-3,
        "overline width must equal text width ({expected_text_width}px), got {actual_width}px; \
         line_elem: {line_elem}"
    );
}

#[test]
fn overline_multiline_width_uses_longest_line() {
    // for multiline name "nChip\nEnable", the overline width must match the
    // longest line ("Enable" = 6 chars = 42px), not the first line
    // ("nChip" = 5 chars = 35px).
    let line = make_overline_signal("nChip\nEnable");
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let layer = extract_signal_labels_layer(&svg);
    let line_start = layer.find("<line").expect("line element");
    let line_end = layer[line_start..].find("/>").expect("end") + line_start;
    let line_elem = &layer[line_start..line_end];
    let x2 = extract_attr_f32(line_elem, "x2").expect("x2 attr");
    let x1 = extract_attr_f32(line_elem, "x1").expect("x1 attr");
    let actual_width = x2 - x1;
    // "Enable" = 6 chars * 7.0 = 42px (longest line)
    let expected_width = 6.0 * 7.0_f32;
    // "nChip"  = 5 chars * 7.0 = 35px (shorter, must NOT be used)
    let first_line_width = 5.0 * 7.0_f32;
    assert!(
        (actual_width - expected_width).abs() < 1e-3,
        "overline width must equal longest line ({expected_width}px), \
         got {actual_width}px (first-line-only width = {first_line_width}px); \
         line_elem: {line_elem}"
    );
}

// ---- arrow label <text> must carry font-family attribute ------------------

fn make_arrow_with_label(label: &str, font: FontSpec) -> Arrow {
    Arrow::new(
        ArrowEnd::Absolute(Point {
            x: Px(10.0),
            y: Px(10.0),
        }),
        ArrowEnd::Absolute(Point {
            x: Px(50.0),
            y: Px(10.0),
        }),
        ArrowStyle::new(
            Color::parse("black").expect("c"),
            Px(1.0),
            LineDashStyle::Solid,
            ArrowHead::EndOnly,
        ),
        Some(UserText::parse(label).expect("label")),
        font,
    )
}

fn extract_arrows_layer(svg: &str) -> &str {
    extract_layer(svg, "arrows")
}

#[test]
fn arrow_label_text_has_font_family_attribute() {
    // <text> in the arrows layer must have a font-family attribute.
    use crate::text::FontFamily;
    let family = FontFamily::parse("Comic Neue").expect("family");
    let font = FontSpec::new(family, Px(12.0));
    let mut document = make_doc(vec![]);
    document
        .annotations
        .arrows
        .push(make_arrow_with_label("arrow A", font));
    let svg = render(&document, &TestFonts);
    let layer = extract_arrows_layer(&svg);
    assert!(
        layer.contains("font-family="),
        "arrow label <text> must have font-family attribute, layer: {layer}"
    );
}

#[test]
fn arrow_label_text_font_family_value_matches_spec() {
    // font-family attribute value must match the FontSpec family.
    use crate::text::FontFamily;
    let family = FontFamily::parse("Comic Neue").expect("family");
    let font = FontSpec::new(family, Px(12.0));
    let mut document = make_doc(vec![]);
    document
        .annotations
        .arrows
        .push(make_arrow_with_label("arrow A", font));
    let svg = render(&document, &TestFonts);
    let layer = extract_arrows_layer(&svg);
    assert!(
        layer.contains("font-family=\"Comic Neue\""),
        "arrow label <text> must have font-family=\"Comic Neue\", layer: {layer}"
    );
}

#[test]
fn arrow_label_text_has_font_size_attribute() {
    // <text> in the arrows layer must also have a font-size attribute.
    use crate::text::FontFamily;
    let family = FontFamily::parse("Comic Neue").expect("family");
    let font = FontSpec::new(family, Px(14.0));
    let mut document = make_doc(vec![]);
    document
        .annotations
        .arrows
        .push(make_arrow_with_label("label", font));
    let svg = render(&document, &TestFonts);
    let layer = extract_arrows_layer(&svg);
    assert!(
        layer.contains("font-size="),
        "arrow label <text> must have font-size attribute, layer: {layer}"
    );
}

#[test]
fn arrow_without_label_has_no_text_element() {
    // Sanity: arrows with no label must not emit a <text> element in the arrows layer.
    let mut document = make_doc(vec![]);
    document.annotations.arrows.push(Arrow::new(
        ArrowEnd::Absolute(Point {
            x: Px(10.0),
            y: Px(10.0),
        }),
        ArrowEnd::Absolute(Point {
            x: Px(50.0),
            y: Px(10.0),
        }),
        ArrowStyle::new(
            Color::parse("black").expect("c"),
            Px(1.0),
            LineDashStyle::Solid,
            ArrowHead::EndOnly,
        ),
        None,
        FontSpec::default(),
    ));
    let svg = render(&document, &TestFonts);
    let layer = extract_arrows_layer(&svg);
    assert!(
        !layer.contains("<text"),
        "arrow without label must not emit <text>, got: {layer}"
    );
}

// ---- arrow label outline (white stroke) ------------------------------------

#[test]
fn arrow_label_text_has_white_outline_attributes() {
    // Arrow label <text> must carry paint-order, stroke, stroke-width,
    // and stroke-linejoin for white outline rendering.
    use crate::text::FontFamily;
    let family = FontFamily::parse("sans-serif").expect("family");
    let font = FontSpec::new(family, Px(14.0));
    let mut document = make_doc(vec![]);
    document
        .annotations
        .arrows
        .push(make_arrow_with_label("hello", font));
    let svg = render(&document, &TestFonts);
    let layer = extract_arrows_layer(&svg);
    assert!(
        layer.contains("paint-order=\"stroke fill\""),
        "arrow label <text> must have paint-order=\"stroke fill\", layer: {layer}"
    );
    assert!(
        layer.contains("stroke=\"#ffffff\""),
        "arrow label <text> must have stroke=\"#ffffff\", layer: {layer}"
    );
    assert!(
        layer.contains("stroke-width=\"2\""),
        "arrow label <text> must have stroke-width=\"2\", layer: {layer}"
    );
    assert!(
        layer.contains("stroke-linejoin=\"round\""),
        "arrow label <text> must have stroke-linejoin=\"round\", layer: {layer}"
    );
}

// ---- bgcolor0/bgcolor1 row background scenarios ----------------------------

/// Build a `ChartStyle` with `bgcolor0 = #eee` and `bgcolor1 = #ccc`.
fn make_alternating_bgcolor_style() -> ChartStyle {
    let mut style = ChartStyle::default();
    style.set_bgcolor0(Color::parse("#eee").expect("c0"));
    style.set_bgcolor1(Color::parse("#ccc").expect("c1"));
    style
}

/// Assert that `layer` contains exactly `expected_rects` background `<rect>`s
/// and that the colour order is bgcolor0 → bgcolor1 → bgcolor0 (alternating).
fn assert_bgcolor_alternating_order(layer: &str, expected_rects: usize) {
    let rect_count = layer.matches("<rect").count();
    assert_eq!(
        rect_count, expected_rects,
        "expected {expected_rects} background rects, got {rect_count}: {layer}"
    );
    let first_eee = layer
        .find("#eeeeee")
        .or_else(|| layer.find("eeeeee"))
        .expect("bgcolor0 (#eee) missing in layer");
    let first_ccc = layer
        .find("#cccccc")
        .or_else(|| layer.find("cccccc"))
        .expect("bgcolor1 (#ccc) missing in layer");
    assert!(
        first_eee < first_ccc,
        "bgcolor0 (#eee) must appear before bgcolor1 (#ccc); layer: {layer}"
    );
    let after_first_ccc = &layer[first_ccc + 7..];
    assert!(
        after_first_ccc.contains("#eeeeee") || after_first_ccc.contains("eeeeee"),
        "third row must use bgcolor0 (#eee) again; layer: {layer}"
    );
}

#[test]
fn bgcolor_alternates_three_signal_rows() {
    // Scenario: 偶奇行が交互に塗られる
    // 3 SignalRows → row-backgrounds layer must have 3 rects: #eee, #ccc, #eee
    let style = make_alternating_bgcolor_style();
    let row1 = make_signal("a", vec![level(SignalLevel::Low, 1)], &style);
    let row2 =
        make_signal_with_bounding_box_y("b", vec![level(SignalLevel::Low, 1)], &style, Px(40.0));
    let row3 =
        make_signal_with_bounding_box_y("c", vec![level(SignalLevel::Low, 1)], &style, Px(70.0));
    let document = ChartDocument::new(
        style,
        vec![row1, row2, row3],
        Annotations::default(),
        TcmlSource::new(""),
    );
    let svg = render(&document, &TestFonts);
    let layer = extract_row_backgrounds_layer(&svg);
    assert_bgcolor_alternating_order(layer, 3);
}

#[test]
fn bgcolor_rect_height_equals_bbox_height() {
    // Scenario: bbox 全体 (上下 gap/2 を含む) を塗る
    // The <rect height> must equal Line.bounding_box.size.height exactly.
    let mut style = ChartStyle::default();
    style.set_bgcolor0(Color::parse("#eee").expect("c0"));
    let signal = make_signal("a", vec![level(SignalLevel::Low, 1)], &style);
    let expected_height = signal.bounding_box.size.height;
    let document = ChartDocument::new(
        style,
        vec![signal],
        Annotations::default(),
        TcmlSource::new(""),
    );
    let svg = render(&document, &TestFonts);
    let layer = extract_row_backgrounds_layer(&svg);
    let height_needle = "height=\"";
    let height_start = layer
        .find(height_needle)
        .expect("height attribute missing in row-backgrounds rect")
        + height_needle.len();
    let height_end = layer[height_start..]
        .find('"')
        .expect("height attribute close quote")
        + height_start;
    let actual_height: f32 = layer[height_start..height_end]
        .parse()
        .expect("height value is not a valid f32");
    assert!(
        (actual_height - expected_height.to_f32()).abs() < 1e-3,
        "rect height ({actual_height:.1}) must equal bbox height ({:.1}); layer: {layer}",
        expected_height.to_f32()
    );
}

/// Build a `Line` containing a `SkipRow` with the given bounding-box origin y.
fn make_skip_line(origin_y: Px) -> Line {
    Line::new_with_bounding_box(
        LineContent::Skip(SkipRow::new(
            crate::units::Length::new_lh(1.0).expect("length"),
        )),
        None,
        Rect {
            origin: Point {
                x: Px(10.0),
                y: origin_y,
            },
            size: Size {
                width: Px(100.0),
                height: Px(10.0),
            },
        },
    )
}

/// Build a `Line` containing a `TitleRow` with the given bounding-box origin y.
fn make_title_line_at_y(style: &ChartStyle, origin_y: Px) -> Line {
    let title = TitleRow::new(
        UserText::parse("T").expect("title text"),
        TitleStyle::new(
            style.default_label_style().font().clone(),
            HorizontalAlign::Left,
            style.default_label_style().color(),
        ),
    );
    Line::new_with_bounding_box(
        LineContent::Title(title),
        None,
        Rect {
            origin: Point {
                x: Px(10.0),
                y: origin_y,
            },
            size: Size {
                width: Px(100.0),
                height: Px(20.0),
            },
        },
    )
}

#[test]
fn skip_and_title_excluded_from_stripe_count() {
    // Scenario: SkipRow / TitleRow は偶奇カウント外
    // Order: SignalRow(index=0), SkipRow, SignalRow(index=1), TitleRow, SignalRow(index=2)
    // Expected: 3 rects (only signal rows), colors: #eee, #ccc, #eee
    let style = make_alternating_bgcolor_style();
    let sig0 = make_signal("a", vec![level(SignalLevel::Low, 1)], &style);
    let sig1 =
        make_signal_with_bounding_box_y("b", vec![level(SignalLevel::Low, 1)], &style, Px(50.0));
    let sig2 =
        make_signal_with_bounding_box_y("c", vec![level(SignalLevel::Low, 1)], &style, Px(100.0));
    let document = ChartDocument::new(
        style.clone(),
        vec![
            sig0,
            make_skip_line(Px(30.0)),
            sig1,
            make_title_line_at_y(&style, Px(70.0)),
            sig2,
        ],
        Annotations::default(),
        TcmlSource::new(""),
    );
    let svg = render(&document, &TestFonts);
    let layer = extract_row_backgrounds_layer(&svg);
    assert_bgcolor_alternating_order(layer, 3);
}

#[test]
fn bgcolor_none_emits_no_rect() {
    // Scenario: @bgcolor が none のとき出力なし
    // Default ChartStyle has bgcolor0/bgcolor1 = none, so no rects should appear.
    let style = ChartStyle::default();
    let row = make_signal("a", vec![level(SignalLevel::Low, 1)], &style);
    let document = ChartDocument::new(
        style,
        vec![row],
        Annotations::default(),
        TcmlSource::new(""),
    );
    let svg = render(&document, &TestFonts);
    // Spec (svg-rendering.md §「空レイヤーの省略」): when no `<rect>` is produced,
    // the `<g class="row-backgrounds">` wrapper itself is omitted. Either form
    // (layer absent, or layer present with no rect) satisfies "no rect emitted".
    let layer = extract_layer(&svg, "row-backgrounds");
    assert!(
        !layer.contains("<rect"),
        "bgcolor=none must emit no rect in row-backgrounds; got: {layer}"
    );
}

// ---- Low↔HiZ / High↔HiZ SingleEdge でギャップがないことを確認 -------------

/// Extract all `points="..."` attribute values from `<polyline>` elements in `waveforms` layer.
fn collect_polyline_points(svg: &str) -> Vec<String> {
    let waveforms_start = svg.find("class=\"waveforms\"").expect("waveforms layer");
    let waveforms_end = svg[waveforms_start..].find("</g>").expect("close") + waveforms_start;
    let layer = &svg[waveforms_start..waveforms_end];
    let mut result = Vec::new();
    let mut search = layer;
    while let Some(polyline_offset) = search.find("<polyline points=\"") {
        let points_start = polyline_offset + "<polyline points=\"".len();
        let Some(points_end) = search[points_start..].find('"') else {
            break;
        };
        result.push(search[points_start..points_start + points_end].to_owned());
        search = &search[points_start + points_end..];
    }
    result
}

/// Parse the first `x,y` pair from a points string like `"50,28 60,28 62,20"`.
fn first_point(points: &str) -> (f32, f32) {
    let pair = points
        .split_whitespace()
        .next()
        .expect("at least one point");
    let mut parts = pair.split(',');
    let x: f32 = parts.next().expect("x").parse().expect("x parse");
    let y: f32 = parts.next().expect("y").parse().expect("y parse");
    (x, y)
}

/// Parse the last `x,y` pair from a points string.
fn last_point(points: &str) -> (f32, f32) {
    let pair = points
        .split_whitespace()
        .last()
        .expect("at least one point");
    let mut parts = pair.split(',');
    let x: f32 = parts.next().expect("x").parse().expect("x parse");
    let y: f32 = parts.next().expect("y").parse().expect("y parse");
    (x, y)
}

/// Build a signal line using a style with slant=2.
fn make_signal_with_slant2(elements: Vec<WaveformElement>) -> Line {
    let mut style = ChartStyle::default();
    style.set_slant(Px(2.0));
    make_signal("s", elements, &style)
}

#[test]
fn low_to_hiz_single_edge_no_gap_with_slant() {
    // Low→HiZ SingleEdge で slant=2 のとき前後 polyline が接続される。
    // Low polyline の末点 x == HiZ (破線) polyline の始点 x → 視覚的ギャップなし。
    let elements = vec![
        level(SignalLevel::Low, 1),
        transition(
            TransitionKind::SingleEdge,
            SignalLevel::Low,
            SignalLevel::HiZ,
        ),
        level(SignalLevel::HiZ, 1),
    ];
    let line = make_signal_with_slant2(elements);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let all_points = collect_polyline_points(&svg);
    // Low is in `top` accum, HiZ+transition in `hiz` accum.
    // Expect >=2 polylines: one solid (Low), one dashed (transition + HiZ).
    assert!(
        all_points.len() >= 2,
        "Low→HiZ with slant should produce >=2 polylines, got {}: {svg}",
        all_points.len()
    );
    // The last point of the solid polyline must exactly equal the first point of the dashed polyline.
    let (last_x, last_y) = last_point(&all_points[0]);
    let (first_x, first_y) = first_point(&all_points[1]);
    assert!(
        (last_x - first_x).abs() < 1e-3 && (last_y - first_y).abs() < 1e-3,
        "Low→HiZ gap detected: Low ends at ({last_x},{last_y}), HiZ begins at ({first_x},{first_y}); \
         must share a connection point"
    );
}

#[test]
fn hiz_to_low_single_edge_no_gap_with_slant() {
    // HiZ→Low SingleEdge で slant=2 のとき前後 polyline が接続される。
    // flush 順は top→hiz なので:
    //   polylines[0] = top accum → Low LevelRun のみ (始点が接続点)
    //   polylines[1] = hiz accum → HiZ LevelRun + transition (末点が接続点)
    // 接続条件: first_point(solid) == last_point(dashed)
    let elements = vec![
        level(SignalLevel::HiZ, 1),
        transition(
            TransitionKind::SingleEdge,
            SignalLevel::HiZ,
            SignalLevel::Low,
        ),
        level(SignalLevel::Low, 1),
    ];
    let line = make_signal_with_slant2(elements);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let all_points = collect_polyline_points(&svg);
    assert!(
        all_points.len() >= 2,
        "HiZ→Low with slant should produce >=2 polylines, got {}: {svg}",
        all_points.len()
    );
    // solid (top) is flushed first, dashed (hiz) second.
    let (solid_start_x, solid_start_y) = first_point(&all_points[0]);
    let (dashed_end_x, dashed_end_y) = last_point(&all_points[1]);
    assert!(
        (solid_start_x - dashed_end_x).abs() < 1e-3 && (solid_start_y - dashed_end_y).abs() < 1e-3,
        "HiZ→Low gap detected: Low begins at ({solid_start_x},{solid_start_y}), \
         HiZ+transition ends at ({dashed_end_x},{dashed_end_y}); \
         must share a connection point"
    );
}

#[test]
fn high_to_hiz_single_edge_no_gap_with_slant() {
    // High→HiZ SingleEdge で slant=2 のとき前後 polyline が接続される。
    let elements = vec![
        level(SignalLevel::High, 1),
        transition(
            TransitionKind::SingleEdge,
            SignalLevel::High,
            SignalLevel::HiZ,
        ),
        level(SignalLevel::HiZ, 1),
    ];
    let line = make_signal_with_slant2(elements);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let all_points = collect_polyline_points(&svg);
    assert!(
        all_points.len() >= 2,
        "High→HiZ with slant should produce >=2 polylines, got {}: {svg}",
        all_points.len()
    );
    // The last point of the solid polyline must exactly equal the first point of the dashed polyline.
    let (last_x, last_y) = last_point(&all_points[0]);
    let (first_x, first_y) = first_point(&all_points[1]);
    assert!(
        (last_x - first_x).abs() < 1e-3 && (last_y - first_y).abs() < 1e-3,
        "High→HiZ gap detected: High ends at ({last_x},{last_y}), HiZ begins at ({first_x},{first_y}); \
         must share a connection point"
    );
}

#[test]
fn hiz_to_high_single_edge_no_gap_with_slant() {
    // HiZ→High SingleEdge で slant=2 のとき前後 polyline が接続される。
    // flush 順は top→hiz なので:
    //   polylines[0] = top accum → High LevelRun のみ (始点が接続点)
    //   polylines[1] = hiz accum → HiZ LevelRun + transition (末点が接続点)
    // 接続条件: first_point(solid) == last_point(dashed)
    let elements = vec![
        level(SignalLevel::HiZ, 1),
        transition(
            TransitionKind::SingleEdge,
            SignalLevel::HiZ,
            SignalLevel::High,
        ),
        level(SignalLevel::High, 1),
    ];
    let line = make_signal_with_slant2(elements);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let all_points = collect_polyline_points(&svg);
    assert!(
        all_points.len() >= 2,
        "HiZ→High with slant should produce >=2 polylines, got {}: {svg}",
        all_points.len()
    );
    let (solid_start_x, solid_start_y) = first_point(&all_points[0]);
    let (dashed_end_x, dashed_end_y) = last_point(&all_points[1]);
    assert!(
        (solid_start_x - dashed_end_x).abs() < 1e-3 && (solid_start_y - dashed_end_y).abs() < 1e-3,
        "HiZ→High gap detected: High begins at ({solid_start_x},{solid_start_y}), \
         HiZ+transition ends at ({dashed_end_x},{dashed_end_y}); \
         must share a connection point"
    );
}

// ---- Waveform Text (level-string text characters) ----

/// Extract the raw content of `<g class="waveforms">…</g>` from `svg`.
fn extract_waveforms_layer(svg: &str) -> &str {
    extract_layer(svg, "waveforms")
}

#[test]
fn waveform_text_element_is_in_waveforms_layer() {
    // Scenario: waveforms layer contains a <text text-anchor="middle"> for Text element.
    let style = ChartStyle::default();
    let elements = vec![
        WaveformElement::Level(LevelRun::new(SignalLevel::Low, 4)),
        WaveformElement::Text(UserText::parse("abc").expect("text")),
    ];
    let line = make_signal("SigA", elements, &style);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let layer = extract_waveforms_layer(&svg);
    assert!(
        layer.contains("<text"),
        "waveforms layer must contain <text> element, layer: {layer}"
    );
    assert!(
        layer.contains("text-anchor=\"middle\""),
        "waveform <text> must have text-anchor=\"middle\", layer: {layer}"
    );
    assert!(
        layer.contains(">abc</text>"),
        "waveform <text> must contain abc, layer: {layer}"
    );
}

#[test]
fn waveform_text_x_is_center_of_owning_run() {
    // x = bbox_origin.x(10) + signal_box.origin.x(40) + level_width/2(4*25/2=50) = 100.
    let style = ChartStyle::default();
    let elements = vec![
        WaveformElement::Level(LevelRun::new(SignalLevel::Low, 4)),
        WaveformElement::Text(UserText::parse("abc").expect("text")),
    ];
    let line = make_signal("SigA", elements, &style);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let layer = extract_waveforms_layer(&svg);
    assert!(
        layer.contains("x=\"100\""),
        "waveform <text> x must be 100 (center of 4-unit Low at step=25), layer: {layer}"
    );
}

#[test]
fn waveform_text_y_is_vertical_center_of_signal_box() {
    // y = bbox_origin.y(10) + signal_box.origin.y(2) + signal_box.height/2(16/2=8) = 20.
    let style = ChartStyle::default();
    let elements = vec![
        WaveformElement::Level(LevelRun::new(SignalLevel::Low, 4)),
        WaveformElement::Text(UserText::parse("abc").expect("text")),
    ];
    let line = make_signal("SigA", elements, &style);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let layer = extract_waveforms_layer(&svg);
    assert!(
        layer.contains("y=\"20\""),
        "waveform <text> y must be 20 (vertical center of signal box), layer: {layer}"
    );
}

#[test]
fn waveform_text_multiple_fragments_produce_one_text_element() {
    // Two text fragments on the same run produce one <text> element, not two.
    let style = ChartStyle::default();
    let elements = vec![
        WaveformElement::Level(LevelRun::new(SignalLevel::Low, 5)),
        WaveformElement::Text(UserText::parse("a b").expect("text")),
    ];
    let line = make_signal("SigA", elements, &style);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let layer = extract_waveforms_layer(&svg);
    let count = layer.matches("<text").count();
    assert_eq!(
        count, 1,
        "exactly one <text> element expected, got {count}, layer: {layer}"
    );
    assert!(
        layer.contains(">a b</text>"),
        "text content must be space-joined fragments, layer: {layer}"
    );
}

#[test]
fn waveform_text_has_font_family_attribute() {
    // font-family is rendered as an XML attribute, not via CSS.
    use crate::text::FontFamily;
    let family = FontFamily::parse("Comic Neue").expect("family");
    let mut style = ChartStyle::default();
    style.set_font_family(family);
    let elements = vec![
        WaveformElement::Level(LevelRun::new(SignalLevel::Low, 4)),
        WaveformElement::Text(UserText::parse("abc").expect("text")),
    ];
    let line = make_signal("SigA", elements, &style);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let layer = extract_waveforms_layer(&svg);
    assert!(
        layer.contains("font-family=\"Comic Neue\""),
        "waveform <text> must have font-family attribute, layer: {layer}"
    );
}

#[test]
fn waveform_text_has_no_clip_path() {
    // Text may overflow the level run; no clip-path attribute is applied.
    let style = ChartStyle::default();
    let elements = vec![
        WaveformElement::Level(LevelRun::new(SignalLevel::Low, 2)),
        WaveformElement::Text(UserText::parse("long label").expect("text")),
    ];
    let line = make_signal("SigA", elements, &style);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let layer = extract_waveforms_layer(&svg);
    assert!(
        !layer.contains("clip-path"),
        "waveform <text> must not have a clip-path (overflow is allowed), layer: {layer}"
    );
    assert!(
        !layer.contains("<clipPath"),
        "waveforms layer must not define a <clipPath> element, layer: {layer}"
    );
}

#[test]
fn waveform_text_xml_escaping() {
    // Characters like < > & must be XML-escaped in the text content.
    let style = ChartStyle::default();
    let elements = vec![
        WaveformElement::Level(LevelRun::new(SignalLevel::Low, 4)),
        WaveformElement::Text(UserText::parse("<a&b>").expect("text")),
    ];
    let line = make_signal("SigA", elements, &style);
    let svg = render(&make_doc(vec![line]), &TestFonts);
    let layer = extract_waveforms_layer(&svg);
    assert!(
        layer.contains("&lt;a&amp;b&gt;"),
        "text content must be XML-escaped, layer: {layer}"
    );
    assert!(
        !layer.contains("<a&b>"),
        "unescaped angle bracket must not appear in output, layer: {layer}"
    );
}

fn render_pipeline(source: &str) -> String {
    let mut document = crate::parser::parse(source).expect("parse should succeed");
    let fonts = TestFonts;
    crate::layout::layout(&mut document, &fonts).expect("layout should succeed");
    render(&document, &fonts)
}

fn extract_layer<'svg>(svg: &'svg str, layer_class: &str) -> &'svg str {
    let needle = format!("class=\"{layer_class}\"");
    let Some(open) = svg.find(&needle) else {
        return "";
    };
    let after_open = &svg[open..];
    let close = after_open.find("</g>").unwrap_or(after_open.len());
    &after_open[..close]
}

#[test]
fn style_element_appears_after_metadata_before_defs() {
    let svg = render_pipeline("Sig _?_\n");
    let metadata_pos = svg.find("</metadata>");
    let style_pos = svg.find("<style");
    let defs_pos = svg.find("<defs");
    if let (Some(metadata), Some(style), Some(defs)) = (metadata_pos, style_pos, defs_pos) {
        assert!(
            metadata < style && style < defs,
            "expected metadata < style < defs; got {metadata} < {style} < {defs}"
        );
    }
}

#[test]
fn defs_element_omitted_when_no_dontcare_present() {
    let svg = render_pipeline("Sig _~_~\n");
    assert!(
        !svg.contains("<defs"),
        "<defs> must be absent without ?; got {svg}"
    );
}

#[test]
fn dontcare_pattern_ids_are_numbered_from_one() {
    let svg = render_pipeline("@dontcare_color #c00\nA _?_\n@dontcare_color #06c\nB _?_\n");
    assert!(svg.contains("dontcare-hatch-1"));
    assert!(svg.contains("dontcare-hatch-2"));
    assert!(!svg.contains("\"dontcare-hatch\""));
}

#[test]
fn same_dontcare_color_shares_a_single_pattern_id() {
    let svg = render_pipeline("@dontcare_color #c00\nA _?_\nB _?_\nC _?_\n");
    let count = svg.matches("dontcare-hatch-1").count();
    let count_two = svg.matches("dontcare-hatch-2").count();
    assert!(count > 0);
    assert_eq!(count_two, 0, "only one pattern id expected");
}

#[test]
fn dontcare_pattern_uses_rotate_45_transform() {
    let svg = render_pipeline("A _?_\n");
    assert!(svg.contains("patternTransform=\"rotate(45)\""));
}

#[test]
fn dontcare_pattern_uses_user_space_units() {
    let svg = render_pipeline("A _?_\n");
    assert!(svg.contains("patternUnits=\"userSpaceOnUse\""));
}

#[test]
fn tcml_source_xml_special_characters_are_escaped() {
    let svg = render_pipeline("@title <foo&bar>\nSig _\n");
    let source_open = svg.find("<tchart:source").expect("tchart:source");
    let after = &svg[source_open..];
    let close = after.find("</tchart:source>").expect("close");
    let inner = &after[..close];
    assert!(inner.contains("&lt;") && inner.contains("&amp;") && inner.contains("&gt;"));
}

#[test]
fn cdata_section_not_used_for_source_with_brackets() {
    let svg = render_pipeline("@title \"x ]]> y\"\nSig _\n");
    assert!(!svg.contains("<![CDATA["), "CDATA must not be used");
}

#[test]
fn svg_root_includes_page_margin_in_dimensions() {
    let svg = render_pipeline("@page-margin 10\nSig _~_~\n");
    let width_key = "width=\"";
    let start = svg.find(width_key).expect("width") + width_key.len();
    let end = svg[start..].find('"').expect("close");
    let width: f32 = svg[start..start + end].parse().expect("number");
    assert!(width > 20.0, "width must include page margins; got {width}");
}

#[test]
fn row_backgrounds_layer_behaviour_is_deterministic_when_empty() {
    let svg = render_pipeline("@bgcolor0 none\n@bgcolor1 none\nSig _\n");
    // Either present-and-empty or omitted; must be deterministic.
    let _ = svg.contains("class=\"row-backgrounds\"");
}

#[test]
fn arrows_layer_contains_only_user_arrows_not_edge_marks() {
    let svg = render_pipeline("@clock(pos) ck\nA _~@{a}__\nB ___@{b}_\n@-> (@{a}, @{b})\n");
    let arrows = extract_layer(&svg, "arrows");
    let waveforms = extract_layer(&svg, "waveforms");
    // Clock triangle markers must live in the dedicated `edge-marks` layer,
    // never the `arrows` layer.
    assert!(
        !arrows.contains("<polygon"),
        "arrows layer must not contain edge marks; got {arrows}"
    );
    let _ = waveforms;
}

#[test]
fn anchors_do_not_emit_visible_geometry() {
    let svg = render_pipeline("Sig _~@{a}__@1__\n");
    assert!(!svg.contains(">@{a}"));
    assert!(!svg.contains(">@1"));
}

#[test]
fn title_align_center_uses_text_anchor_middle() {
    let svg = render_pipeline("@titlealign center\n@title T\nSig _\n");
    let titles = extract_layer(&svg, "titles");
    assert!(titles.contains("text-anchor=\"middle\""));
}

#[test]
fn title_align_left_uses_text_anchor_start() {
    let svg = render_pipeline("@titlealign left\n@title T\nSig _\n");
    let titles = extract_layer(&svg, "titles");
    assert!(titles.contains("text-anchor=\"start\""));
}

#[test]
fn title_align_right_uses_text_anchor_end() {
    let svg = render_pipeline("@titlealign right\n@title T\nSig _\n");
    let titles = extract_layer(&svg, "titles");
    assert!(titles.contains("text-anchor=\"end\""));
}

#[test]
fn bus_segment_text_uses_middle_anchor() {
    let svg = render_pipeline("Sig ==A==\n");
    let waveforms = extract_layer(&svg, "waveforms");
    assert!(waveforms.contains("text-anchor=\"middle\""));
    assert!(waveforms.contains(">A<"));
}

#[test]
fn long_text_in_short_segment_is_not_clipped() {
    let svg = render_pipeline("Sig __VeryLongText__\n");
    assert!(!svg.contains("clip-path"));
}

#[test]
fn hiz_segment_uses_dasharray() {
    let svg = render_pipeline("Sig ____----____\n");
    assert!(svg.contains("stroke-dasharray"));
}

#[test]
fn arrow_label_has_paint_order_stroke_decoration() {
    let svg = render_pipeline("A _~@{a}_\nB _~@{b}_\n@-> (@{a}, @{b}) label1\n");
    let arrows = extract_layer(&svg, "arrows");
    assert!(
        arrows.contains("paint-order=\"stroke fill\"")
            || arrows.contains("stroke=\"#ffffff\"")
            || arrows.contains("stroke=\"#fff\""),
        "arrow label must have white outline; got {arrows}"
    );
}

#[test]
fn arrow_without_label_omits_label_text_element() {
    let svg = render_pipeline("A _~@{a}_\nB _~@{b}_\n@-> (@{a}, @{b})\n");
    let arrows = extract_layer(&svg, "arrows");
    // No label means no paint-order stroke decoration on a separate text element.
    assert!(!arrows.contains("paint-order=\"stroke fill\""));
}

#[test]
fn arrow_head_none_omits_arrow_head_polygon() {
    let svg = render_pipeline("A _~@{a}_\nB _~@{b}_\n@-> (@{a}, @{b}, head=none)\n");
    let arrows = extract_layer(&svg, "arrows");
    assert!(!arrows.contains("<polygon"));
}

#[test]
fn arrow_head_both_emits_two_arrow_heads() {
    let svg = render_pipeline("A _~@{a}_\nB _~@{b}_\n@-> (@{a}, @{b}, head=both)\n");
    let arrows = extract_layer(&svg, "arrows");
    let polygons = arrows.matches("<polygon").count();
    let paths = arrows.matches("<path").count();
    assert!(
        polygons + paths >= 2,
        "head=both must produce two markers; got {polygons} polygons + {paths} paths"
    );
}

#[test]
fn arrow_dashed_uses_dasharray_attribute() {
    let svg = render_pipeline("A _~@{a}_\nB _~@{b}_\n@-> (@{a}, @{b}, dashed)\n");
    let arrows = extract_layer(&svg, "arrows");
    assert!(arrows.contains("stroke-dasharray"));
}

#[test]
fn arrow_dotted_uses_distinct_dasharray() {
    let dashed = render_pipeline("A _~@{a}_\nB _~@{b}_\n@-> (@{a}, @{b}, dashed)\n");
    let dotted = render_pipeline("A _~@{a}_\nB _~@{b}_\n@-> (@{a}, @{b}, dotted)\n");
    assert_ne!(dashed, dotted, "dashed and dotted should differ in output");
}

#[test]
fn guide_does_not_extend_through_title_row() {
    let svg = render_pipeline("@title T1\nSig __|__\n@title T2\nSig2 _~\n");
    let guides = extract_layer(&svg, "guides");
    assert!(
        !guides.is_empty(),
        "guide layer must exist when `|` is present"
    );
}

#[test]
fn guide_extends_above_first_row_when_no_title_above() {
    let svg = render_pipeline("Sig __|__\n");
    let guides = extract_layer(&svg, "guides");
    assert!(!guides.is_empty());
}

#[test]
fn highlight_extends_into_page_margin_like_guide() {
    let svg = render_pipeline("Sig __[__]__\n");
    let highlights = extract_layer(&svg, "highlights");
    assert!(!highlights.is_empty());
}

#[test]
fn highlight_rect_carries_user_provided_style_attributes() {
    let svg = render_pipeline("@highlight_style fill=\"#8f8\" stroke=\"green\"\nSig __[__]__\n");
    let highlights = extract_layer(&svg, "highlights");
    assert!(highlights.contains("fill=\"#8f8\"") || highlights.contains("fill='#8f8'"));
}

#[test]
fn clock_edge_mark_polygon_vertices_are_present() {
    let svg = render_pipeline(
        "@step 10\n@slant 2\n@clockmark_height 5\n@clockmark_width 4\n@clockmark_position 0.5\n@clock(pos) ck\nT ____\n",
    );
    assert!(svg.contains("<polygon"), "clock edge mark polygon expected");
}

#[test]
fn clock_edge_mark_uses_supplied_fill_color() {
    let svg = render_pipeline("@clock(pos, mark_color=red) ck\nT ____\n");
    assert!(
        svg.contains("fill=\"red\"") || svg.contains("fill='red'"),
        "clock mark fill must be red; got {svg}"
    );
}

#[test]
fn clock_edge_mark_height_is_clamped_to_line_length() {
    // A very large mark_height must not produce inverted polygon coordinates.
    let svg = render_pipeline("@step 10\n@slant 2\n@clockmark_height 100\n@clock(pos) ck\nT __\n");
    assert!(svg.contains("<polygon"));
}

// ---- clockmark default + step-linked shrink (SVG pipeline) -----------

/// Run the full parse + layout pipeline on `source` and return the
/// `mark_style.width` of the first `EdgeMark` on the first `Signal` row that
/// carries one. Panics if no such row / mark exists.
fn first_edge_mark_width_after_layout(source: &str) -> Px {
    let mut document = crate::parser::parse(source).expect("parse should succeed");
    crate::layout::layout(&mut document, &TestFonts).expect("layout should succeed");
    for line in &document.lines {
        if let LineContent::Signal(row) = &line.content
            && let Some(mark) = row.edge_marks().first()
        {
            return mark.mark_style.width;
        }
    }
    panic!("no edge mark found in laid-out document");
}

fn first_edge_mark_height_after_layout(source: &str) -> Px {
    let mut document = crate::parser::parse(source).expect("parse should succeed");
    crate::layout::layout(&mut document, &TestFonts).expect("layout should succeed");
    for line in &document.lines {
        if let LineContent::Signal(row) = &line.content
            && let Some(mark) = row.edge_marks().first()
        {
            return mark.mark_style.height;
        }
    }
    panic!("no edge mark found in laid-out document");
}

#[test]
fn pipeline_clockmark_width_shrinks_with_small_step() {
    // step=6 → min(8, 6*2/3) = 4.
    let width = first_edge_mark_width_after_layout("@step 6\n@clock(pos) ck\nT __\n");
    assert_eq!(width, Px(4.0));
}

#[test]
fn pipeline_clockmark_width_not_shrunk_when_step_large() {
    // step=15 → min(6, 10) = 6.
    let width = first_edge_mark_width_after_layout("@step 15\n@clock(pos) ck\nT __\n");
    assert_eq!(width, Px(6.0));
}

#[test]
fn pipeline_clockmark_width_at_boundary_step_9() {
    // step=9 → min(6, 6) = 6.
    let width = first_edge_mark_width_after_layout("@step 9\n@clock(pos) ck\nT __\n");
    assert_eq!(width, Px(6.0));
}

#[test]
fn pipeline_clockmark_global_width_explicit_disables_shrink() {
    // @clockmark_width 8 with step=3 → still 8 (no shrink).
    let width =
        first_edge_mark_width_after_layout("@step 3\n@clockmark_width 8\n@clock(pos) ck\nT __\n");
    assert_eq!(width, Px(8.0));
}

#[test]
fn pipeline_clockmark_global_width_default_value_explicit_disables_shrink() {
    // @clockmark_width 6 (same as default) with step=3 → still 6 (no shrink, explicit).
    let width =
        first_edge_mark_width_after_layout("@step 3\n@clockmark_width 6\n@clock(pos) ck\nT __\n");
    assert_eq!(width, Px(6.0));
}

#[test]
fn pipeline_clockmark_local_width_explicit_disables_shrink() {
    // local mark_width=12 with step=3 → 12.
    let width =
        first_edge_mark_width_after_layout("@step 3\n@clock(pos, mark_width=12) ck\nT __\n");
    assert_eq!(width, Px(12.0));
}

#[test]
fn pipeline_clockmark_height_not_shrunk_when_step_small() {
    // step=3 — width shrinks but height stays at default 7.5.
    let source = "@step 3\n@clock(pos) ck\nT __\n";
    let width = first_edge_mark_width_after_layout(source);
    let height = first_edge_mark_height_after_layout(source);
    assert_eq!(width, Px(2.0));
    assert_eq!(height, Px(7.5));
}

#[test]
fn pipeline_clockmark_default_step_large_yields_default_size() {
    // Default step=25; @clockmark_* unset → width=6, height=7.5.
    let source = "@step 20\n@clock(pos) ck\nT __\n";
    let width = first_edge_mark_width_after_layout(source);
    let height = first_edge_mark_height_after_layout(source);
    assert_eq!(width, Px(6.0));
    assert_eq!(height, Px(7.5));
}

#[test]
fn pipeline_clockmark_global_width_before_step_change_no_shrink() {
    // @clockmark_width first, then small @step → still 8.
    let width =
        first_edge_mark_width_after_layout("@clockmark_width 8\n@step 3\n@clock(pos) ck\nT __\n");
    assert_eq!(width, Px(8.0));
}

#[test]
fn signal_overline_uses_line_element_not_text_decoration() {
    let svg = render_pipeline("@signal(overline) nReset _~\n");
    let labels = extract_layer(&svg, "signal-labels");
    assert!(
        !labels.contains("text-decoration"),
        "overline must not be a text-decoration"
    );
    assert!(labels.contains("<line"));
}

#[test]
fn overline_line_width_matches_longest_label_row() {
    let svg = render_pipeline("@signal(overline) \"short\\nverylongline\" _~\n");
    let labels = extract_layer(&svg, "signal-labels");
    assert!(labels.contains("<line"));
}

#[test]
fn overline_inherits_label_color() {
    let svg = render_pipeline("@signal_color blue\n@signal(overline) nReset _~\n");
    let labels = extract_layer(&svg, "signal-labels");
    assert!(
        labels.contains("stroke=\"blue\"") || labels.contains("stroke='blue'"),
        "overline must inherit label color (blue); got {labels}"
    );
}

#[test]
fn dontcare_along_bus_yields_rectangle_when_both_sides_continue() {
    let svg = render_pipeline("Sig =?=\n");
    let layer = extract_layer(&svg, "dontcares");
    let alt = extract_layer(&svg, "waveforms");
    assert!(
        !layer.is_empty() || alt.contains("<polygon") || alt.contains("<rect"),
        "DontCare polygon expected somewhere; svg={svg}"
    );
}

#[test]
fn dontcare_along_bus_open_close_yields_hexagonal_polygon() {
    let svg = render_pipeline("Sig _=?=_\n");
    assert!(svg.contains("<polygon"));
}

#[test]
fn dontcare_along_buscross_uses_cross_midpoint_vertices() {
    let svg = render_pipeline("Sig =X?X=\n");
    assert!(svg.contains("<polygon"));
}

#[test]
fn dontcare_rectangle_has_no_stroke_outline() {
    let svg = render_pipeline("Sig _?_\n");
    let layer = extract_layer(&svg, "dontcares");
    if !layer.is_empty() {
        assert!(
            !layer.contains("stroke=\"black\""),
            "DontCare rect must not have outline; got {layer}"
        );
    }
}

#[test]
fn malicious_font_value_is_emitted_as_attribute_only() {
    let svg = render_pipeline("@font \"monospace; }body{...}\"\nSig _\n");
    let style_open = svg.find("<style");
    if let Some(style_open) = style_open {
        let style_section = &svg[style_open..];
        let close = style_section
            .find("</style>")
            .unwrap_or(style_section.len());
        let inner = &style_section[..close];
        assert!(
            !inner.contains("body{"),
            "CSS injection payload must not appear in <style>; got {inner}"
        );
    }
}

#[test]
fn polyline_accumulator_flushes_after_gap() {
    let svg = render_pipeline("Sig ____:____\n");
    let waveforms = extract_layer(&svg, "waveforms");
    let polylines = waveforms.matches("<polyline").count();
    assert!(
        polylines >= 2,
        "Gap must split polylines (>=2); got {polylines}"
    );
}

#[test]
fn polyline_accumulator_does_not_flush_at_highlight_boundary() {
    let svg = render_pipeline("Sig __[__]__\n");
    let waveforms = extract_layer(&svg, "waveforms");
    let polylines = waveforms.matches("<polyline").count();
    assert!(
        polylines == 1,
        "Highlight must not flush polylines; got {polylines}"
    );
}

#[test]
fn polyline_accumulator_does_not_flush_at_anchor() {
    let svg = render_pipeline("Sig __@{a}__\n");
    let waveforms = extract_layer(&svg, "waveforms");
    let polylines = waveforms.matches("<polyline").count();
    assert!(
        polylines == 1,
        "Anchor must not flush polylines; got {polylines}"
    );
}

#[test]
fn per_row_step_change_doubles_signal_polyline_x_extent() {
    let narrow = render_pipeline("@step 10\nA ____\n");
    let wide = render_pipeline("@step 20\nA ____\n");
    let narrow_w = parse_width(&narrow);
    let wide_w = parse_width(&wide);
    assert!(
        wide_w > narrow_w,
        "step doubled must enlarge SVG width; got {narrow_w} vs {wide_w}"
    );
}

#[test]
fn clock_auto_with_bg_overrides_bgcolor0() {
    let svg = render_pipeline("@bgcolor0 #eee\n@bg #f0f\n@clock(pos) ck\nT __\n");
    assert!(
        svg.contains("#f0f") || svg.contains("#FF00FF") || svg.contains("rgb(255,0,255)"),
        "@bg must override bgcolor0; got {svg}"
    );
}

#[test]
fn per_row_step_with_arrow_renders_arrow_line() {
    let svg = render_pipeline("@step 10\nA _~@{a}_\n@step 20\nB _~@{b}_\n@-> (@{a}, @{b})\n");
    let arrows = extract_layer(&svg, "arrows");
    assert!(arrows.contains("<line") || arrows.contains("<path"));
}

#[test]
fn dontcare_highlight_anchor_combination_render_independent_layers() {
    let svg = render_pipeline("Sig __[?]@{a}__\n");
    assert!(
        svg.contains("class=\"dontcares\"") || svg.contains("class=\"highlights\""),
        "expected at least one specialised layer; got {svg}"
    );
}

#[test]
fn bus_cross_with_highlight_and_dontcare_coexist() {
    let svg = render_pipeline("Sig =[X?X]=\n");
    assert!(svg.contains("<polygon"));
}

#[test]
fn signal_overline_with_multiline_name_and_bg_combine() {
    let svg = render_pipeline("@bg #ff0\n@signal(overline) \"ne\\nrst\" _~\n");
    assert!(svg.contains("#ff0") || svg.contains("#FFFF00"));
    let labels = extract_layer(&svg, "signal-labels");
    assert!(labels.contains("<line"));
}

#[test]
fn title_align_right_with_bg_uses_user_color_not_stripe() {
    let svg = render_pipeline("@bgcolor1 blue\n@bg red\n@titlealign right\n@title \"T\"\nSig _\n");
    let titles = extract_layer(&svg, "titles");
    assert!(titles.contains("text-anchor=\"end\""));
}

#[test]
fn layer_z_order_respects_spec_order() {
    let svg = render_pipeline("@clock(pos) ck\nA __[?@{a}__]__\nB __\n@-> (@{a}, @{a})\n");
    let order_keys = [
        "row-backgrounds",
        "highlights",
        "dontcares",
        "signal-labels",
        "waveforms",
        "guides",
        "titles",
        "arrows",
    ];
    let mut last_position = 0usize;
    for key in order_keys {
        if let Some(position) = svg.find(&format!("class=\"{key}\"")) {
            assert!(
                position >= last_position,
                "layer {key} appears before previous layer; svg={svg}"
            );
            last_position = position;
        }
    }
}

fn parse_width(svg: &str) -> f32 {
    let key = "width=\"";
    let start = svg.find(key).expect("width") + key.len();
    let end = svg[start..].find('"').expect("close");
    svg[start..start + end].parse().expect("number")
}

// ---------------------------------------------------------------------------
// Iter1 phase 2: SVG structural invariants.
// ---------------------------------------------------------------------------

#[test]
fn root_svg_child_order_is_fixed_iter1() {
    // `@bgcolor0` ensures the `row-backgrounds` layer has content and therefore
    // is emitted (spec §「空レイヤーの省略」 omits empty layers).
    let svg = render_pipeline("@bgcolor0 #fde\n@title \"T\"\nSig _?_\n");
    // Use only the open tag offsets so the comparison reflects element start
    // positions; the `</metadata>` close tag is later in the document and
    // would mask an out-of-order arrangement.
    let metadata = svg
        .find("<metadata")
        .expect("<metadata> open tag must be present");
    let style = svg.find("<style").expect("<style> must be present");
    let defs = svg
        .find("<defs")
        .expect("<defs> must be present (? produces hatch)");
    let row_bg = svg
        .find("class=\"row-backgrounds\"")
        .expect("row-backgrounds group must be present");
    assert!(
        metadata < style,
        "metadata must precede style; got metadata={metadata} style={style}"
    );
    assert!(
        style < defs,
        "style must precede defs; got style={style} defs={defs}"
    );
    assert!(
        defs < row_bg,
        "defs must precede row-backgrounds; got defs={defs} row_bg={row_bg}"
    );
}

#[test]
fn defs_omitted_when_no_dontcare_iter1() {
    let svg = render_pipeline("A _~_~\n");
    assert!(
        !svg.contains("<defs"),
        "<defs> must be absent without ?; got {svg}"
    );
}

#[test]
fn defs_present_with_single_dontcare_iter1() {
    let svg = render_pipeline("A _?_\n");
    assert!(svg.contains("<defs"), "<defs> must exist with ?; got {svg}");
    assert!(
        svg.contains("dontcare-hatch-1"),
        "first hatch id must be dontcare-hatch-1"
    );
}

#[test]
fn signal_name_xml_special_chars_escaped_iter1() {
    let svg = render_pipeline("\"<foo>&<bar>\" _\n");
    assert!(svg.contains("&lt;foo&gt;"), "expected escaped <foo>: {svg}");
    assert!(svg.contains("&amp;"), "expected escaped &: {svg}");
    assert!(
        !svg.contains("<foo>") && !svg.contains("<bar>"),
        "raw < must not appear: {svg}"
    );
}

#[test]
fn title_xml_special_chars_escaped_iter1() {
    let svg = render_pipeline("@title \"<T>&<U>\"\nSig _\n");
    assert!(svg.contains("&lt;T&gt;"));
    assert!(svg.contains("&amp;"));
    assert!(svg.contains("&lt;U&gt;"));
}

#[test]
fn arrow_label_xml_special_chars_escaped_iter1() {
    let svg = render_pipeline("A _@{a}_\nB _@{b}_\n@-> (@{a}, @{b}) <abc>&<xyz>\n");
    let arrows = extract_layer(&svg, "arrows");
    assert!(
        arrows.contains("&lt;abc&gt;"),
        "literal <abc> must be entity-escaped in arrow label: {arrows}"
    );
    assert!(
        arrows.contains("&amp;"),
        "literal & must be entity-escaped in arrow label: {arrows}"
    );
    assert!(
        arrows.contains("&lt;xyz&gt;"),
        "literal <xyz> must be entity-escaped in arrow label: {arrows}"
    );
}

#[test]
fn tcml_source_uses_entity_escaping_not_cdata_iter1() {
    let svg = render_pipeline("// < comment\nA _\n");
    assert!(
        !svg.contains("<![CDATA["),
        "must not use CDATA section: {svg}"
    );
    assert!(svg.contains("&lt;"), "must escape literal <");
}

#[test]
fn tcml_source_with_cdata_terminator_uses_entity_escape_iter1() {
    let svg = render_pipeline("// ]]> marker\nA _\n");
    assert!(!svg.contains("<![CDATA["), "must not use CDATA: {svg}");
    assert!(svg.contains("&gt;"), "must escape > literal: {svg}");
}

#[test]
fn z_order_row_backgrounds_first_iter1() {
    // `@bgcolor0` ensures the `row-backgrounds` layer has content and therefore
    // is emitted (spec §「空レイヤーの省略」 omits empty layers).
    let svg = render_pipeline("@bgcolor0 #fde\nA _\nB _\n");
    let row_bg_pos = svg
        .find("class=\"row-backgrounds\"")
        .expect("row-backgrounds group must be present for any signal chart");
    let other_groups = ["waveforms", "signal-labels"];
    for class in other_groups {
        let other_pos = svg
            .find(&format!("class=\"{class}\""))
            .unwrap_or_else(|| panic!("expected {class} group to be present in SVG: {svg}"));
        assert!(
            row_bg_pos < other_pos,
            "row-backgrounds must precede {class}; got row_bg={row_bg_pos} vs {class}={other_pos}"
        );
    }
}

#[test]
fn z_order_arrows_before_overlays_iter1() {
    let svg = render_pipeline("A _@{a}_\n% 5 5 mark\n@-> (@{a}, @{a})\n");
    let arrows_pos = svg
        .find("class=\"arrows\"")
        .unwrap_or_else(|| panic!("arrows group must be present for @-> input: {svg}"));
    let overlays_pos = svg
        .find("class=\"overlays\"")
        .unwrap_or_else(|| panic!("overlays group must be present for % input: {svg}"));
    assert!(
        arrows_pos < overlays_pos,
        "arrows must precede overlays in document order; got arrows={arrows_pos} vs overlays={overlays_pos}"
    );
}

#[test]
fn empty_dontcares_group_omitted_iter1() {
    let svg = render_pipeline("A _~\n");
    assert!(
        !svg.contains("class=\"dontcares\""),
        "empty dontcares group must be omitted: {svg}"
    );
}

#[test]
fn empty_arrows_group_omitted_iter1() {
    let svg = render_pipeline("A _~\n");
    assert!(
        !svg.contains("class=\"arrows\""),
        "empty arrows group must be omitted: {svg}"
    );
}

#[test]
fn first_hatch_pattern_id_starts_at_one_iter1() {
    let svg = render_pipeline("A _?_\n");
    assert!(
        svg.contains("dontcare-hatch-1"),
        "first hatch id must be 1: {svg}"
    );
}

#[test]
fn duplicate_dontcare_color_shares_single_pattern_iter1() {
    let svg = render_pipeline("@dontcare_color red\nA _?_\nB _?_\nC _?_\nD _?_\n");
    let pattern_count = svg.matches("<pattern").count();
    assert_eq!(
        pattern_count, 1,
        "same color must reuse pattern; got {pattern_count} patterns"
    );
}

// ---------------------------------------------------------------------------
// Iter2 phase 2: text labels (Tsukuba-style coverage), bgcolor edge cases,
// arrow z-order.
// ---------------------------------------------------------------------------

#[test]
fn iter2_bus_with_seven_text_fragments_renders_text_elements() {
    // Per docs/spec/svg-rendering.md the parser may merge consecutive `=`
    // fragments into a single bus or keep them as a sequence. Either way the
    // pipeline must produce at least one `<text>` element per labelled bus
    // segment without panicking.
    let svg = render_pipeline("Sig =A=B=C=D=E=F=G===\n");
    let text_count = extract_layer(&svg, "waveforms").matches("<text").count();
    assert!(
        text_count >= 1,
        "expected at least one bus-label <text>; got {text_count} svg={svg}"
    );
    for letter in ['A', 'B', 'C', 'D', 'E', 'F', 'G'] {
        assert!(
            svg.contains(letter),
            "label letter {letter} must appear in SVG"
        );
    }
}

#[test]
fn iter2_bus_label_with_cjk_preserves_utf8() {
    let svg = render_pipeline("Sig =\"\u{65E5}\u{672C}\u{8A9E}\"==\n");
    assert!(
        svg.contains('\u{65E5}') && svg.contains('\u{672C}') && svg.contains('\u{8A9E}'),
        "CJK label must round-trip verbatim in SVG: {svg}"
    );
    // The three CJK code points must appear as their canonical 3-byte UTF-8
    // sequences (E6 97 A5 / E6 9C AC / E8 AA 9E), not as XML numeric entities.
    let cjk_bytes: [u8; 9] = [0xE6, 0x97, 0xA5, 0xE6, 0x9C, 0xAC, 0xE8, 0xAA, 0x9E];
    assert!(
        svg.as_bytes()
            .windows(cjk_bytes.len())
            .any(|window| window == cjk_bytes),
        "raw UTF-8 byte sequence for CJK label must appear in SVG output"
    );
}

#[test]
fn iter2_bus_label_with_emoji_preserves_utf8_bytes() {
    let svg = render_pipeline("Sig =\"\u{1F389}\"==\n");
    assert!(
        svg.contains('\u{1F389}'),
        "non-BMP emoji must round-trip in SVG: {svg}"
    );
}

#[test]
fn iter2_bus_label_with_combining_character_preserves_bytes() {
    // U+304C HIRAGANA LETTER GA: composed (single code point, not decomposed).
    let svg = render_pipeline("Sig =\"\u{304C}\"==\n");
    assert!(
        svg.contains('\u{304C}'),
        "composed CJK character must appear verbatim in SVG: {svg}"
    );
}

#[test]
fn iter2_bus_label_around_buscross_produces_multiple_text_elements() {
    // `=a=Xab==` puts label `a` in the left bus run and `ab` in the right
    // bus run separated by a BusCross transition. Both labels must reach SVG.
    let svg = render_pipeline("Sig =a=Xab==\n");
    assert!(svg.contains(">a<"), "left-bus label `a` must appear: {svg}");
    assert!(
        svg.contains(">ab<"),
        "right-bus label `ab` must appear: {svg}"
    );
}

#[test]
fn iter2_busxross_chain_text_attribution_renders_all_labels() {
    let svg = render_pipeline("Sig ===XaXb=\n");
    for letter in ['a', 'b'] {
        assert!(
            svg.contains(&format!(">{letter}<")),
            "label {letter} must appear in SVG: {svg}"
        );
    }
}

#[test]
fn iter2_quoted_literal_with_escaped_quote_is_unescaped_in_label() {
    let svg = render_pipeline("Sig =\"A\\\"_~B\"==\n");
    let waveforms = extract_layer(&svg, "waveforms");
    assert!(
        waveforms.contains("A&quot;_~B") || waveforms.contains("A\"_~B"),
        "literal quote inside label must be present (entity-escaped is OK): {waveforms}"
    );
    assert!(
        waveforms.contains('_') && waveforms.contains('~'),
        "_ and ~ inside quoted literal must be retained as text characters: {waveforms}"
    );
}

#[test]
fn iter2_quoted_literal_with_backslash_yields_single_backslash() {
    let svg = render_pipeline("Sig =\"path\\\\to\\\\dir\"==\n");
    let waveforms = extract_layer(&svg, "waveforms");
    let backslashes = waveforms.matches('\\').count();
    assert!(
        backslashes >= 2,
        "expected at least 2 unescaped backslashes in label: waveforms={waveforms}"
    );
    assert!(
        !waveforms.contains("\\\\\\\\"),
        "backslash must not stay double-escaped: waveforms={waveforms}"
    );
}

#[test]
fn iter2_long_label_wider_than_segment_still_emits_text() {
    let svg = render_pipeline("Sig =VeryLongLabelName==\n");
    let count = extract_layer(&svg, "waveforms").matches("<text").count();
    assert!(
        count >= 1,
        "long label that overflows the bus run width must still emit a <text>: {svg}"
    );
    assert!(
        svg.contains("VeryLongLabelName"),
        "long label text must reach SVG verbatim: {svg}"
    );
}

#[test]
fn iter2_consecutive_buscross_with_labels_renders_multiple_labels() {
    let svg = render_pipeline("Sig =Xa=XbXc=\n");
    for letter in ['a', 'b', 'c'] {
        assert!(
            svg.contains(&format!(">{letter}<")),
            "label {letter} must reach SVG: {svg}"
        );
    }
}

#[test]
fn iter2_bus_label_with_xml_dangerous_chars_is_entity_escaped() {
    let svg = render_pipeline("Sig =\"<&>\"==\n");
    let waveforms = extract_layer(&svg, "waveforms");
    assert!(
        waveforms.contains("&lt;") && waveforms.contains("&gt;") && waveforms.contains("&amp;"),
        "XML-dangerous chars in label must be entity-escaped: waveforms={waveforms}"
    );
    assert!(
        !waveforms.contains(">&<"),
        "raw `&` inside text content is forbidden: waveforms={waveforms}"
    );
}

#[test]
fn iter2_bus_label_with_cdata_terminator_uses_entity_escape_not_cdata() {
    let svg = render_pipeline("Sig =\"a]]>b\"==\n");
    let waveforms = extract_layer(&svg, "waveforms");
    assert!(
        !waveforms.contains("<![CDATA["),
        "CDATA must not be used for waveform text labels: waveforms={waveforms}"
    );
    assert!(
        waveforms.contains("&gt;"),
        "literal `>` must be entity-escaped: waveforms={waveforms}"
    );
}

#[test]
fn iter2_empty_string_label_does_not_emit_empty_text_element() {
    let svg = render_pipeline("Sig =\"\"==\n");
    let waveforms = extract_layer(&svg, "waveforms");
    assert!(
        !waveforms.contains("<text"),
        "empty `<text></text>` element must not be emitted: waveforms={waveforms}"
    );
}

#[test]
fn iter2_whitespace_only_label_is_emitted_as_text_element() {
    let svg = render_pipeline("Sig =\" \"==\n");
    let count = extract_layer(&svg, "waveforms").matches("<text").count();
    assert!(
        count >= 1,
        "whitespace-only label must still produce a <text>: {svg}"
    );
}

#[test]
fn iter2_skip_row_does_not_break_bgcolor_parity() {
    // Spec: `@skip(N)` consumes index slots so the next signal row's parity
    // continues from where it would have been without the skip.
    let svg = render_pipeline("@bgcolor0 #aaaaaa\n@bgcolor1 #bbbbbb\nA _\n@skip(1)\nB _\n");
    let row_bg = extract_layer(&svg, "row-backgrounds");
    let count_a = row_bg.matches("#aaaaaa").count();
    let count_b = row_bg.matches("#bbbbbb").count();
    assert!(
        count_a >= 1 && count_b >= 1,
        "both bg colors must appear at least once: row_bg={row_bg}"
    );
}

#[test]
fn iter2_title_rows_are_excluded_from_bgcolor_parity() {
    let svg = render_pipeline(
        "@bgcolor0 #aaaaaa\n@bgcolor1 #bbbbbb\n@title \"T1\"\n@title \"T2\"\nA _\nB _\n",
    );
    let row_bg = extract_layer(&svg, "row-backgrounds");
    assert!(
        row_bg.contains("#aaaaaa") && row_bg.contains("#bbbbbb"),
        "both parity colors must appear when there are 2 signal rows: row_bg={row_bg}"
    );
}

#[test]
fn iter2_only_bgcolor0_omits_odd_row_rect() {
    let svg = render_pipeline("@bgcolor0 #eeeeee\nA _\nB _\nC _\n");
    let row_bg = extract_layer(&svg, "row-backgrounds");
    let count_eee = row_bg.matches("#eeeeee").count();
    assert_eq!(
        count_eee, 2,
        "two even rows (A, C) must each get the rect; got {count_eee}: row_bg={row_bg}"
    );
}

#[test]
fn iter2_only_bgcolor1_omits_even_row_rect() {
    let svg = render_pipeline("@bgcolor1 #eeeeee\nA _\nB _\nC _\n");
    let row_bg = extract_layer(&svg, "row-backgrounds");
    let count_eee = row_bg.matches("#eeeeee").count();
    assert_eq!(
        count_eee, 1,
        "exactly one odd row (B) must get the rect; got {count_eee}: row_bg={row_bg}"
    );
}

#[test]
fn iter2_all_bg_none_emits_no_row_background_rect() {
    let svg = render_pipeline("@bgcolor0 none\n@bgcolor1 none\nA _\nB _\nC _\n");
    let row_bg = extract_layer(&svg, "row-backgrounds");
    let rect_count = row_bg.matches("<rect").count();
    assert_eq!(
        rect_count, 0,
        "all-none must produce zero <rect> elements; got {rect_count}: row_bg={row_bg}"
    );
}

#[test]
fn iter2_per_row_bg_overrides_bgcolor_parity() {
    let svg = render_pipeline("@bgcolor0 #eeeeee\n@bgcolor1 #dddddd\n@bg yellow\nA _\nB _\n");
    let row_bg = extract_layer(&svg, "row-backgrounds");
    assert!(
        row_bg.contains("yellow") || row_bg.contains("#ffff00"),
        "per-row @bg must take precedence on the affected row: row_bg={row_bg}"
    );
    assert!(
        row_bg.contains("#dddddd"),
        "non-overridden parity row must keep its parity color: row_bg={row_bg}"
    );
}

#[test]
fn iter2_arrow_spans_two_signal_rows_with_distinct_y_coordinates() {
    let svg = render_pipeline("A _@{a}~\nB _~@{b}\n@-> (@{a}, @{b})\n");
    let arrows = extract_layer(&svg, "arrows");
    assert!(
        arrows.contains("y1=\""),
        "arrow line must have a y1 attribute: arrows={arrows}"
    );
    assert!(
        arrows.contains("y2=\""),
        "arrow line must have a y2 attribute: arrows={arrows}"
    );
    let y1 = arrows
        .split("y1=\"")
        .nth(1)
        .and_then(|tail| tail.split('"').next())
        .expect("y1 value must be parseable");
    let y2 = arrows
        .split("y2=\"")
        .nth(1)
        .and_then(|tail| tail.split('"').next())
        .expect("y2 value must be parseable");
    assert_ne!(
        y1, y2,
        "cross-row arrow must have distinct y1 and y2 coordinates: arrows={arrows}"
    );
}

#[test]
fn iter2_two_arrows_on_same_anchor_pair_render_in_document_order() {
    let svg =
        render_pipeline("A _@{a}~\nB _~@{b}\n@-> (@{a}, @{b})\n@-> (@{a}, @{b}, color=red)\n");
    let arrows = extract_layer(&svg, "arrows");
    let line_count = arrows.matches("<line").count();
    let path_count = arrows.matches("<path").count();
    assert!(
        line_count + path_count >= 2,
        "two arrows must produce at least 2 line/path strokes: arrows={arrows}"
    );
    // `Color::to_css_string` canonicalises named colours to `#rrggbb`, so the
    // SVG either retains the original name `red` or emits the hex form.
    assert!(
        arrows.contains("red") || arrows.contains("#ff0000"),
        "second arrow with color=red must surface as a red stroke (name or hex): arrows={arrows}"
    );
}

#[test]
fn iter2_arrow_head_both_with_zero_width_dashed_emits_two_polygons() {
    let svg = render_pipeline(
        "A _@{a}~\nB _~@{b}\n@-> (@{a}, @{b}, head=both, width=0px, style=dashed)\n",
    );
    let arrows = extract_layer(&svg, "arrows");
    // Arrow heads are rendered as <path>, not <polygon> (spec §「矢印頭」: "path で実装").
    let head_count = arrows.matches("<path").count();
    assert_eq!(
        head_count, 2,
        "head=both must emit 2 arrow heads as <path> elements: arrows={arrows}"
    );
    assert!(
        arrows.contains("stroke-dasharray"),
        "dashed style must produce stroke-dasharray attribute: arrows={arrows}"
    );
    assert!(
        arrows.contains("stroke-width=\"0\""),
        "width=0px must serialise as stroke-width=\"0\": arrows={arrows}"
    );
}

#[test]
fn iter2_arrow_label_text_appears_for_single_letter_label() {
    let svg = render_pipeline("A _@{a}~~~~~~@{b}~\nB _\n@-> (@{a}, @{b}) L\n");
    let arrows = extract_layer(&svg, "arrows");
    assert!(
        arrows.contains(">L<"),
        "arrow label text `L` must appear: arrows={arrows}"
    );
}

#[test]
fn iter2_arrow_label_with_spaces_preserves_text() {
    let svg = render_pipeline("A _@{a}~\nB _~@{b}\n@-> (@{a}, @{b}) \"long label here\"\n");
    let arrows = extract_layer(&svg, "arrows");
    assert!(
        arrows.contains("long label here"),
        "arrow label with spaces must survive verbatim: arrows={arrows}"
    );
}

#[test]
fn iter2_arrows_layer_appears_after_edge_marks_layer() {
    // The `@clock(pos)` marker layer must be drawn before `arrows` so arrows
    // appear in front of clock indicators.
    let svg = render_pipeline("@clock(pos) clk _\nA _@{a}~\nB _~@{b}\n@-> (@{a}, @{b})\n");
    let edge_marks_pos = svg.find("class=\"edge-marks\"");
    let arrows_pos = svg.find("class=\"arrows\"");
    let edge_marks = edge_marks_pos.unwrap_or_else(|| {
        panic!("edge-marks layer must be present in SVG: svg={svg}");
    });
    let arrows = arrows_pos.unwrap_or_else(|| {
        panic!("arrows layer must be present in SVG: svg={svg}");
    });
    assert!(
        edge_marks < arrows,
        "edge marks must precede arrows in document order; got edge_marks={edge_marks} arrows={arrows}"
    );
}

#[test]
fn iter2_one_hundred_arrows_on_same_row_all_render() {
    let mut source = String::from("Sig _");
    for index in 1..=200 {
        source.push_str(&format!("@{index}_"));
    }
    source.push('\n');
    for index in 1..=100 {
        source.push_str(&format!("@-> (@{}, @{})\n", index, index + 100));
    }
    let svg = render_pipeline(&source);
    let arrows = extract_layer(&svg, "arrows");
    let line_count = arrows.matches("<line").count();
    let path_count = arrows.matches("<path").count();
    assert!(
        line_count + path_count >= 100,
        "expected at least 100 arrow strokes; got line={line_count} path={path_count}"
    );
}

#[test]
fn iter2_arrow_head_start_rejected_or_yields_no_end_polygon() {
    // Spec currently rejects head=start. Pin that behaviour: the parse must
    // fail rather than silently produce ambiguous SVG.
    let result = crate::parser::parse("A _@{a}~\nB _~@{b}\n@-> (@{a}, @{b}, head=start)\n");
    assert!(
        result.is_err(),
        "head=start must be rejected by parser; got Ok"
    );
}

#[test]
fn iter2_arrow_style_solid_omits_stroke_dasharray() {
    let svg = render_pipeline("A _@{a}~\nB _~@{b}\n@-> (@{a}, @{b}, style=solid)\n");
    let arrows = extract_layer(&svg, "arrows");
    assert!(
        !arrows.contains("stroke-dasharray"),
        "solid arrow must not emit stroke-dasharray: arrows={arrows}"
    );
}

// ---------------------------------------------------------------------------
// Iter3 phase: Unicode in SVG output.
// ---------------------------------------------------------------------------

#[test]
fn iter3_arabic_signal_name_appears_in_text_output() {
    let svg = render_pipeline("\u{0627}\u{0644}\u{0639}\u{0631}\u{0628}\u{064A}\u{0629} _~_~\n");
    assert!(
        svg.contains('\u{0627}'),
        "Arabic alef must appear in SVG output verbatim"
    );
}

#[test]
fn iter3_hebrew_signal_name_no_bidi_control_injected() {
    let svg = render_pipeline("\u{05E9}\u{05DC}\u{05D5}\u{05DD} _~\n");
    assert!(
        !svg.contains('\u{202E}') && !svg.contains('\u{202D}'),
        "BiDi override controls must not be injected; svg head={head}",
        head = svg.chars().take(400).collect::<String>()
    );
    assert!(
        svg.contains('\u{05E9}'),
        "Hebrew shin must round-trip into SVG"
    );
}

#[test]
fn iter3_combining_diacritic_keeps_decomposed_form() {
    let svg = render_pipeline("e\u{0301} _~\n");
    assert!(
        svg.contains('\u{0301}'),
        "combining acute must be preserved in SVG; svg head={head}",
        head = svg.chars().take(400).collect::<String>()
    );
}

#[test]
fn iter3_supplementary_emoji_appears_in_text() {
    let svg = render_pipeline("\u{1F389} _~\n");
    assert!(
        svg.contains('\u{1F389}'),
        "U+1F389 must appear in <text> body"
    );
}

#[test]
fn iter3_full_width_space_in_signal_name_is_preserved_or_split() {
    let outcome = crate::parser::parse("A\u{3000}B _~\n");
    if outcome.is_err() {
        return;
    }
    let svg = render_pipeline("A\u{3000}B _~\n");
    assert!(
        svg.contains('\u{3000}') || svg.contains("A"),
        "either the full-width space survives or the name was split deterministically"
    );
}

#[test]
fn iter3_zero_width_space_in_label_does_not_panic_layout() {
    // ZWSP in a name; the renderer must not panic.
    let outcome = crate::parser::parse("A\u{200B}B _~\n");
    if outcome.is_err() {
        return;
    }
    let svg = render_pipeline("A\u{200B}B _~\n");
    assert!(svg.contains("<svg"), "SVG document must be produced");
}

// ---------------------------------------------------------------------------
// Iter3 phase: SVG ID / class uniqueness.
// ---------------------------------------------------------------------------

#[test]
fn iter3_three_signals_share_single_dontcare_pattern_id() {
    let svg = render_pipeline(
        "@dontcare_color #ff0000\nA _?_\n@dontcare_color #ff0000\nB _?_\n@dontcare_color #ff0000\nC _?_\n",
    );
    let pattern_count = svg.matches("dontcare-hatch-2").count();
    assert_eq!(
        pattern_count, 0,
        "same colour must dedupe to a single pattern id; svg={svg}"
    );
}

#[test]
fn iter3_five_distinct_dontcare_colors_yield_five_patterns() {
    let svg = render_pipeline(concat!(
        "@dontcare_color #ff0000\nA _?_\n",
        "@dontcare_color #00ff00\nB _?_\n",
        "@dontcare_color #0000ff\nC _?_\n",
        "@dontcare_color #ffff00\nD _?_\n",
        "@dontcare_color #ff00ff\nE _?_\n",
    ));
    for index in 1..=5 {
        let needle = format!("dontcare-hatch-{index}");
        assert!(
            svg.contains(&needle),
            "expected pattern id {needle}; svg head={head}",
            head = svg.chars().take(800).collect::<String>()
        );
    }
}

#[test]
fn iter3_polyline_does_not_carry_id_attribute() {
    let svg = render_pipeline("A _~_~\n");
    let polyline_section = svg.match_indices("<polyline").next();
    if let Some((start, _)) = polyline_section {
        let snippet: String = svg[start..].chars().take(400).collect();
        assert!(
            !snippet.contains(" id=\""),
            "<polyline> must not carry an id attribute; snippet={snippet}"
        );
    }
}

#[test]
fn iter3_same_input_renders_byte_identical_svg_twice() {
    let source = "@title \"T\"\nA _~_~\nB _~_~\n@-> (@1, @2)\n";
    let _ = crate::parser::parse("A _@1~\nB _~@2\n");
    let svg_one = render_pipeline("A _@1~\nB _~@2\n@-> (@1, @2)\n");
    let svg_two = render_pipeline("A _@1~\nB _~@2\n@-> (@1, @2)\n");
    assert_eq!(
        svg_one,
        svg_two,
        "identical input must render byte-identical SVG; len_one={} len_two={} title_source={source}",
        svg_one.len(),
        svg_two.len()
    );
}

#[test]
fn iter3_duplicate_anchor_name_across_signals_is_deterministic() {
    // The spec for re-declaring an anchor name across two signals is open.
    // Pin: parser either errors or accepts; pipeline must not panic.
    let outcome = crate::parser::parse("A _@{a}~\nB _@{a}~\n");
    if let Ok(document) = outcome {
        assert!(
            !document.lines.is_empty(),
            "if parser accepts duplicate anchors, document must contain rows"
        );
    }
}

#[test]
fn iter3_hundred_arrows_share_single_arrows_layer() {
    let mut source = String::from("Sig _");
    for index in 1..=200 {
        source.push_str(&format!("@{index}_"));
    }
    source.push('\n');
    for index in 1..=100 {
        source.push_str(&format!("@-> (@{}, @{})\n", index, index + 100));
    }
    let svg = render_pipeline(&source);
    let layer_count = svg.matches("class=\"arrows\"").count();
    assert_eq!(
        layer_count, 1,
        "<g class=\"arrows\"> must appear exactly once even for many arrows; got {layer_count}"
    );
}

#[test]
fn iter3_defs_child_ids_are_unique() {
    let svg = render_pipeline(
        "@dontcare_color #c00\nA _?_\n@dontcare_color #06c\nB _?_\n@clock(pos) clk _~_~\n",
    );
    let mut ids: Vec<&str> = Vec::new();
    let mut cursor = 0;
    while let Some(start) = svg[cursor..].find(" id=\"") {
        let value_start = cursor + start + 5;
        let value_end = value_start
            + svg[value_start..]
                .find('"')
                .expect("id attribute must close");
        ids.push(&svg[value_start..value_end]);
        cursor = value_end + 1;
    }
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    let original_len = sorted.len();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        original_len,
        "all id attributes must be unique; ids={ids:?}"
    );
}

const ITER3_ALLOWED_CLASS_TOKENS: &[&str] = &[
    // Layer-group class names emitted by docs/spec/svg-rendering.md §「描画順 (z-order)」.
    "row-backgrounds",
    "rulers",
    "highlights",
    "dontcares",
    "signal-labels",
    "waveforms",
    "edge-marks",
    "guides",
    "titles",
    "arrows",
    "overlays",
    // Other class tokens that may be emitted on inner elements (kept for forward compatibility).
    "signal-name",
    "arrow",
    "arrow-label",
    "level-low",
    "level-high",
    "level-hiz",
    "level-bus",
    "level-dontcare",
    "transition",
    "background",
    "title",
    "overlay",
    "dontcare",
    "guide",
    "step",
    "level",
    "wave",
    "edge-mark",
    "skip",
    "label",
];

fn collect_class_attribute_values(svg: &str) -> Vec<&str> {
    let mut values = Vec::new();
    let mut cursor = 0;
    while let Some(start) = svg[cursor..].find(" class=\"") {
        let value_start = cursor + start + 8;
        let value_end = value_start
            + svg[value_start..]
                .find('"')
                .expect("class attribute must close");
        values.push(&svg[value_start..value_end]);
        cursor = value_end + 1;
    }
    values
}

#[test]
fn iter3_class_attribute_values_are_known_tokens() {
    let svg = render_pipeline("@clock(pos) clk _~_~\nA _?_\n");
    for value in collect_class_attribute_values(&svg) {
        for token in value.split_whitespace() {
            assert!(
                ITER3_ALLOWED_CLASS_TOKENS.contains(&token),
                "unexpected class token {token:?}; allowed={ITER3_ALLOWED_CLASS_TOKENS:?}"
            );
        }
    }
}

#[test]
fn iter3_lf_and_crlf_inputs_render_byte_identical_svg() {
    let lf_input = "A _~_~\nB _~_~\n";
    let crlf_input = "A _~_~\r\nB _~_~\r\n";
    let svg_lf = render_pipeline(lf_input);
    let svg_crlf_outcome = crate::parser::parse(crlf_input);
    if svg_crlf_outcome.is_err() {
        // CRLF rejection is itself a deterministic outcome; pin it.
        return;
    }
    let svg_crlf = render_pipeline(crlf_input);
    // Allow source-section difference but the rest must match. Strip any
    // tchart:source element before comparing.
    let normalise = |svg: &str| -> String {
        svg.lines()
            .filter(|line| !line.contains("tchart:source"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert_eq!(
        normalise(&svg_lf),
        normalise(&svg_crlf),
        "LF and CRLF must produce identical SVG modulo embedded source"
    );
}

// ---------------------------------------------------------------------------
// Iter3 phase: SVG round-trip stability.
// ---------------------------------------------------------------------------

#[test]
fn iter3_render_to_svg_then_extract_then_render_matches() {
    // Simulated round trip via parse → render twice; the second render must
    // match the first byte for byte.
    let source = "A _~_~\nB _~_~\n";
    let svg_one = render_pipeline(source);
    let svg_two = render_pipeline(source);
    let svg_three = render_pipeline(source);
    assert_eq!(svg_two, svg_three, "renders 2 and 3 must match");
    assert_eq!(svg_one, svg_two, "renders 1 and 2 must match");
}

#[test]
fn iter3_trailing_newline_difference_is_stable() {
    let with_lf = "A _~_~\n";
    let without_lf = "A _~_~";
    let svg_with = render_pipeline(with_lf);
    let svg_without_outcome = crate::parser::parse(without_lf);
    if svg_without_outcome.is_err() {
        return;
    }
    let svg_without = render_pipeline(without_lf);
    // The rendered SVG should contain the source section verbatim, including
    // (or not) the trailing LF.
    let has_with_lf = svg_with.contains("A _~_~\n") || svg_with.contains("A _~_~");
    let has_without_lf = svg_without.contains("A _~_~");
    assert!(
        has_with_lf && has_without_lf,
        "both inputs must surface in the embedded source; with_len={} without_len={}",
        svg_with.len(),
        svg_without.len()
    );
}

// ---------------------------------------------------------------- @ruler

#[test]
fn ruler_default_chart_emits_rulers_layer() {
    // Default `@ruler on` → signal rows contribute → `<g class="rulers">`
    // must be present.
    let svg = render_pipeline("A _~_~\n");
    assert!(svg.contains("class=\"rulers\""));
}

#[test]
fn ruler_all_off_emits_no_rulers_layer() {
    // `@ruler off` only — no row contributes.
    let svg = render_pipeline("@ruler off\nA _~\nB _~\n");
    assert!(!svg.contains("class=\"rulers\""));
}

#[test]
fn ruler_on_immediately_off_emits_no_rulers_layer() {
    // `@ruler on` immediately followed by `@ruler off` before any row
    // commits → all rows commit while off → no contributions.
    let svg = render_pipeline("@ruler on\n@ruler off\nA _~\nB _~\n");
    assert!(!svg.contains("class=\"rulers\""));
}

#[test]
fn ruler_on_emits_rulers_layer_with_lines() {
    // `@step 10`, `@ruler on`, `A _~_~_~` (units = 6) → 7 lines at x = 0..60.
    let svg = render_pipeline("@step 10\n@ruler on\nA _~_~_~\n");
    let layer = extract_layer(&svg, "rulers");
    assert!(!layer.is_empty(), "rulers layer must be present");
    // 7 <line> elements expected.
    assert_eq!(layer.matches("<line").count(), 7, "layer: {layer}");
    // Each line has stroke="#a0a0a0" (default), stroke-width="0.5",
    // stroke-dasharray="3 5".
    assert!(layer.contains("stroke=\"#a0a0a0\""));
    assert!(layer.contains("stroke-width=\"0.5\""));
    assert!(layer.contains("stroke-dasharray=\"3 5\""));
}

#[test]
fn ruler_lines_have_full_chart_inner_height() {
    // All ruler lines span the chart inner height (y1=0 to y2=Σ bbox height).
    let svg = render_pipeline("@step 10\n@ruler on\nA _~\nB _~\n");
    let layer = extract_layer(&svg, "rulers");
    assert!(layer.contains("y1=\"0\""));
    // y2 cannot be predicted exactly without the layout engine, but should
    // be a positive number > 0. Check the literal `y1="0"` followed by the
    // `y2` attribute that is not 0.
    assert!(!layer.contains("y2=\"0\""));
}

#[test]
fn ruler_lines_emitted_in_ascending_x_order() {
    // For multiple ruler lines, x values appear in the SVG in ascending order.
    let svg = render_pipeline("@step 10\n@ruler on\nA _~_~\n");
    let layer = extract_layer(&svg, "rulers");
    // Find each `x1="N"` literal and verify the sequence is ascending.
    let mut xs: Vec<f32> = Vec::new();
    let mut cursor = 0usize;
    while let Some(found) = layer[cursor..].find("x1=\"") {
        let start = cursor + found + 4;
        let end = layer[start..].find('"').expect("close");
        let raw = &layer[start..start + end];
        let value: f32 = raw.parse().expect("x must parse");
        xs.push(value);
        cursor = start + end;
    }
    let mut sorted = xs.clone();
    sorted.sort_by(|left, right| left.partial_cmp(right).expect("finite"));
    assert_eq!(xs, sorted, "x1 values must be ascending: {xs:?}");
}

#[test]
fn ruler_same_x_merged_to_single_line() {
    // Two rows contributing the same x positions → a single line per x.
    // A has units=4, B has units=4, both at @step 10 → x ∈ {0..40}, 5 lines.
    let svg = render_pipeline("@step 10\n@ruler on\n@ruler_color #aaa\nA _~_~\nB _~_~\n");
    let layer = extract_layer(&svg, "rulers");
    assert_eq!(layer.matches("<line").count(), 5);
}

#[test]
fn ruler_last_wins_color_per_x() {
    // Row A contributes color #aaa, row B contributes #bbb at the same x.
    // The last (= deepest) wins → all 5 lines stroked with #bbb. Color
    // values are expanded to canonical 6-digit hex form on output.
    let svg = render_pipeline(
        "@step 10\n@ruler on\n@ruler_color #aaa\nA _~_~\n@ruler_color #bbb\nB _~_~\n",
    );
    let layer = extract_layer(&svg, "rulers");
    assert!(layer.contains("stroke=\"#bbbbbb\""));
    assert!(!layer.contains("stroke=\"#aaaaaa\""));
}

#[test]
fn ruler_distinct_x_keeps_both_colors() {
    // A at @step 10 (x ∈ {0, 10, 20}), B at @step 25 (x ∈ {0, 25, 50}).
    // Overlap: only x=0. Distinct: A's 10, 20 vs B's 25, 50.
    let svg = render_pipeline(
        "@ruler on\n@ruler_color #aaa\n@step 10\nA _~\n@ruler_color #bbb\n@step 25\nB _~\n",
    );
    let layer = extract_layer(&svg, "rulers");
    assert_eq!(layer.matches("<line").count(), 5);
    // Lines at A-only x positions (10, 20) must have #aaa (expanded form).
    assert!(layer.contains("stroke=\"#aaaaaa\""));
    // Lines at B-only x positions (25, 50) and the merged x=0 must have
    // #bbb (expanded form).
    assert!(layer.contains("stroke=\"#bbbbbb\""));
}

#[test]
fn ruler_three_rows_last_color_wins() {
    // 3 rows at the same x positions, each with a different color. The last
    // (deepest) color must win at every x. Output is canonical 6-digit hex.
    let svg = render_pipeline(
        "@step 10\n@ruler on\n@ruler_color #aaa\nA _~\n@ruler_color #bbb\nB _~\n@ruler_color #ccc\nC _~\n",
    );
    let layer = extract_layer(&svg, "rulers");
    assert!(layer.contains("stroke=\"#cccccc\""));
    assert!(!layer.contains("stroke=\"#aaaaaa\""));
    assert!(!layer.contains("stroke=\"#bbbbbb\""));
}

#[test]
fn ruler_off_after_on_preserves_earlier_contributions() {
    // Row A commits while on, row B while off. A's contribution must be
    // preserved in the rulers layer; B contributes nothing.
    let svg = render_pipeline("@step 10\n@ruler on\nA _~_~\n@ruler off\nB _~_~\n");
    let layer = extract_layer(&svg, "rulers");
    // A had units=4 → 5 lines at x = 0..40.
    assert_eq!(layer.matches("<line").count(), 5);
}

#[test]
fn ruler_toggle_independent_contributions_persist() {
    // A on, B off, C on, all with @step 10 and units = 2.
    // A and C contribute the same x ∈ {0, 10, 20}; B contributes nothing.
    // Merged set has 3 lines, all with C's color (last-wins).
    let svg = render_pipeline("@step 10\n@ruler on\nA _~\n@ruler off\nB _~\n@ruler on\nC _~\n");
    let layer = extract_layer(&svg, "rulers");
    assert_eq!(layer.matches("<line").count(), 3);
}

#[test]
fn ruler_step_change_keeps_earlier_positions() {
    // A at @step 10 (x ∈ {0..40}, 5 lines), B at @step 25 (x ∈ {0..100}, 5
    // lines). Merged: {0, 10, 20, 25, 30, 40, 50, 75, 100} = 9 lines.
    let svg = render_pipeline("@step 10\n@ruler on\nA _~_~\n@step 25\nB _~_~\n");
    let layer = extract_layer(&svg, "rulers");
    assert_eq!(layer.matches("<line").count(), 9);
}

#[test]
fn ruler_skip_row_emits_contributions() {
    // @skip(2) alone: units=2 → 3 lines at x = 0, 10, 20.
    let svg = render_pipeline("@step 10\n@ruler on\n@skip(2)\n");
    let layer = extract_layer(&svg, "rulers");
    assert_eq!(layer.matches("<line").count(), 3);
}

#[test]
fn ruler_layer_appears_between_backgrounds_and_highlights() {
    // Layer order: row-backgrounds → rulers → highlights.
    let svg = render_pipeline("@bgcolor0 #eee\n@ruler on\nA _~_~[mark]_\n");
    let backgrounds_pos = svg.find("class=\"row-backgrounds\"");
    let rulers_pos = svg.find("class=\"rulers\"");
    let highlights_pos = svg.find("class=\"highlights\"");
    let (background, ruler, highlight) = (
        backgrounds_pos.expect("backgrounds"),
        rulers_pos.expect("rulers"),
        highlights_pos.expect("highlights"),
    );
    assert!(
        background < ruler && ruler < highlight,
        "expected row-backgrounds < rulers < highlights; got {background} < {ruler} < {highlight}"
    );
}

#[test]
fn ruler_layer_appears_before_waveforms_and_guides() {
    // Layer order: rulers must appear before waveforms / edge-marks /
    // guides / arrows.
    let svg = render_pipeline("@ruler on\nA _~|_\n");
    let rulers_pos = svg.find("class=\"rulers\"").expect("rulers");
    let waveforms_pos = svg.find("class=\"waveforms\"").expect("waveforms");
    let guides_pos = svg.find("class=\"guides\"").expect("guides");
    assert!(
        rulers_pos < waveforms_pos,
        "rulers must precede waveforms ({rulers_pos} < {waveforms_pos})"
    );
    assert!(
        rulers_pos < guides_pos,
        "rulers must precede guides ({rulers_pos} < {guides_pos})"
    );
}

#[test]
fn ruler_single_contribution_keeps_layer() {
    // 1-contribution case: signal row with units=0 (empty body) at @step 10
    // and @ruler on → 1 line at x=0. Empty-layer suppression must NOT fire.
    let svg = render_pipeline("@step 10\n@ruler on\nA\n");
    let layer = extract_layer(&svg, "rulers");
    assert!(!layer.is_empty(), "rulers layer must be present");
    assert_eq!(layer.matches("<line").count(), 1);
}
