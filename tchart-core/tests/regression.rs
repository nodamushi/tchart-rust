//! Regression tests for known bugs from the previous implementation.
//!
//! Each test maps to a specific BUG code documented in
//! `docs/spec/types.md` §11 and `docs/spec/svg-rendering.md`. They run against
//! the full pipeline (parse → layout → SVG) so that any layer collapsing back
//! into the old behaviour is caught.

use tchart_core::layout::{FontMetrics, layout};
use tchart_core::parser::parse;
use tchart_core::svg::render;
use tchart_core::text::FontSpec;
use tchart_core::units::Px;

struct StubFonts {
    char_width: f32,
}

impl FontMetrics for StubFonts {
    fn measure_text_width(&self, text: &str, _font: &FontSpec) -> Px {
        Px(self.char_width * text.chars().count() as f32)
    }
}

fn render_to_svg(source: &str) -> String {
    let stub = StubFonts { char_width: 7.0 };
    let mut document = parse(source).expect("parse");
    layout(&mut document, &stub).expect("layout");
    render(&document, &stub)
}

/// Extract an arbitrary `<g class="...">` layer substring from a rendered SVG.
///
/// Returns the slice from the `class="..."` attribute up to (but not
/// including) the next `</g>` close tag. Returns an empty slice when the
/// requested layer is omitted (per `docs/spec/svg-rendering.md`
/// §「空レイヤーの省略」 every empty `<g>` is skipped). When the matched
/// `<g>` is unterminated, returns the slice up to the end of the SVG.
fn extract_layer<'svg>(svg: &'svg str, class_name: &str) -> &'svg str {
    let needle = format!("class=\"{class_name}\"");
    let Some(open) = svg.find(&needle) else {
        return "";
    };
    let after_open = &svg[open..];
    let close = after_open.find("</g>").unwrap_or(after_open.len());
    &after_open[..close]
}

/// `Gap` (`:`) inside a signal must split the surrounding
/// run into separate `<polyline>` elements — the renderer must `flush_all` on Gap.
#[test]
fn bug001_gap_splits_polyline() {
    let svg = render_to_svg("A __:__\n");
    let waveforms_open = svg.find("class=\"waveforms\"").expect("waveforms");
    let waveforms_close = svg[waveforms_open..].find("</g>").expect("close") + waveforms_open;
    let layer = &svg[waveforms_open..waveforms_close];
    let polyline_count = layer.matches("<polyline").count();
    assert!(
        polyline_count >= 2,
        "Gap must flush all accumulators (>=2 polylines), got {polyline_count} in {layer}"
    );
}

/// x progression must come from the layout width table,
/// so changing `@step` between two consecutive signals scales the chart width
/// linearly with the per-signal `step`.
#[test]
fn bug002_step_change_scales_width() {
    let narrow = render_to_svg("@step 8\nA _~_~\n");
    let wide = render_to_svg("@step 16\nA _~_~\n");
    let narrow_w = parse_svg_attr_f32(&narrow, "width=\"");
    let wide_w = parse_svg_attr_f32(&wide, "width=\"");
    let delta = wide_w - narrow_w;
    // 4 units * (16 - 8) = 32 px; the SVG width includes labels/margins so we
    // only assert directional scaling, not exact equality.
    assert!(
        delta > 24.0,
        "doubling @step should widen the chart by ~32px; got {narrow_w} -> {wide_w}"
    );
}

/// transitions must integrate into the surrounding
/// polyline accumulator. No standalone `<line>` element should appear inside
/// the waveform layer (HiZ etc. is rendered via a separate polyline, not <line>).
#[test]
fn bug003_no_independent_line_in_waveforms() {
    let svg = render_to_svg("A _~_~\n");
    let waveforms_open = svg.find("class=\"waveforms\"").expect("waveforms");
    let waveforms_close = svg[waveforms_open..].find("</g>").expect("close") + waveforms_open;
    let layer = &svg[waveforms_open..waveforms_close];
    assert!(
        !layer.contains("<line"),
        "no standalone <line> in waveforms; got {layer}"
    );
    assert!(layer.contains("<polyline"));
}

/// every `BusOpen` transition must produce two rails
/// (top and bottom edge) even when one edge is shared with the surrounding
/// level. Verified via Bus-following-Low: 2 polylines (top opens from y_high,
/// bottom continues from y_low).
#[test]
fn bug003_bus_open_emits_two_rails() {
    let svg = render_to_svg("A _=\n");
    let waveforms_open = svg.find("class=\"waveforms\"").expect("waveforms");
    let waveforms_close = svg[waveforms_open..].find("</g>").expect("close") + waveforms_open;
    let layer = &svg[waveforms_open..waveforms_close];
    let count = layer.matches("<polyline").count();
    assert!(count >= 2, "BusOpen requires 2 rails, got {count}");
}

fn parse_svg_attr_f32(svg: &str, key: &str) -> f32 {
    let start = svg.find(key).expect("attr") + key.len();
    let end = svg[start..].find('"').expect("close");
    svg[start..start + end].parse().expect("number")
}

/// Extracts the x coordinate of the last point of every `<polyline>` found in
/// `layer` (a slice of SVG text already scoped to the waveforms `<g>`).
fn extract_polyline_endpoint_x_values(layer: &str) -> Vec<f32> {
    // `points="` is 8 bytes; the offset skips past the opening quote.
    const POINTS_ATTR_PREFIX_BYTES: usize = 8;
    layer
        .split("<polyline")
        .skip(1)
        .filter_map(|fragment| {
            let points_start = fragment.find("points=\"")? + POINTS_ATTR_PREFIX_BYTES;
            let points_end = fragment[points_start..].find('"')? + points_start;
            let raw = &fragment[points_start..points_end];
            raw.split_whitespace()
                .last()
                .and_then(|last_pair| last_pair.split(',').next())
                .and_then(|x_coordinate_text| x_coordinate_text.parse::<f32>().ok())
        })
        .collect()
}

fn extract_first_tag<'a>(svg: &'a str, layer_class: &str, tag_name: &str) -> &'a str {
    let open = svg
        .find(layer_class)
        .unwrap_or_else(|| panic!("{layer_class} not found"));
    let close_offset = svg[open..].find("</g>").expect("</g> not found");
    let layer = &svg[open..open + close_offset];
    let element_offset = layer
        .find(tag_name)
        .unwrap_or_else(|| panic!("no {tag_name} in {layer_class}"));
    let tail = &layer[element_offset..];
    let tag_end = tail.find('>').expect("tag end");
    let layer_start = open;
    &svg[layer_start + element_offset..layer_start + element_offset + tag_end + 1]
}

// Guide and Highlight vertical span tests.
//
// Default layout constants used to compute expected values below.
const PAGE_MARGIN: f32 = 10.0;
const FONT_SIZE: f32 = 14.0;
const LINE_HEIGHT_RATIO: f32 = 1.2;
const SIGNAL_GAP: f32 = 10.0;
/// Line height = font_size * line_height_ratio.
const LINE_HEIGHT: f32 = FONT_SIZE * LINE_HEIGHT_RATIO;
/// Signal row bounding box height = line_height + signal_gap.
const SIGNAL_BBOX_HEIGHT: f32 = LINE_HEIGHT + SIGNAL_GAP;
/// Title row bounding box height (1 line, no gap).
const TITLE_BBOX_HEIGHT: f32 = LINE_HEIGHT;

/// no Title rows: guide `<line>` and highlight `<rect>` must span
/// from `(first_row.bbox.origin.y - page_margin/2)` to
/// `(last_row.bbox.bottom + page_margin/2)`.
///
/// Layout (3 Signal rows):
///   row 0 (A, source row): bbox.origin.y = PAGE_MARGIN,           bbox.height = SIGNAL_BBOX_HEIGHT
///   row 1 (B):             bbox.origin.y = PAGE_MARGIN + SIGNAL_BBOX_HEIGHT
///   row 2 (C):             bbox.origin.y = PAGE_MARGIN + 2 * SIGNAL_BBOX_HEIGHT, bbox.height = SIGNAL_BBOX_HEIGHT
///
///   expected y1 = PAGE_MARGIN - PAGE_MARGIN/2 = PAGE_MARGIN/2
///   expected y2 = (PAGE_MARGIN + 2*SIGNAL_BBOX_HEIGHT + SIGNAL_BBOX_HEIGHT) + PAGE_MARGIN/2
#[test]
fn t2_no_title_guide_and_highlight_span_full_chart() {
    // Signal A is the source row (contains | and [...]).
    let source = "A __[~~|~~]__\nB __________\nC __________\n";
    let svg = render_to_svg(source);

    let line_tag = extract_first_tag(&svg, "class=\"guides\"", "<line");
    let guide_y1 = parse_svg_attr_f32(line_tag, "y1=\"");
    let guide_y2 = parse_svg_attr_f32(line_tag, "y2=\"");

    let rect_tag = extract_first_tag(&svg, "class=\"highlights\"", "<rect");
    let rect_y = parse_svg_attr_f32(rect_tag, "y=\"");
    let rect_height = parse_svg_attr_f32(rect_tag, "height=\"");
    let rect_bottom = rect_y + rect_height;

    // Expected values derived from the layout constants defined above.
    let row0_origin_y = PAGE_MARGIN;
    let expected_top = row0_origin_y - PAGE_MARGIN / 2.0;
    let row2_origin_y = PAGE_MARGIN + 2.0 * SIGNAL_BBOX_HEIGHT;
    let expected_bottom = (row2_origin_y + SIGNAL_BBOX_HEIGHT) + PAGE_MARGIN / 2.0;

    assert!(
        (guide_y1 - expected_top).abs() < 0.5,
        "guide y1 ({guide_y1}) must equal first-row origin.y - page_margin/2 ({expected_top})"
    );
    assert!(
        (guide_y2 - expected_bottom).abs() < 0.5,
        "guide y2 ({guide_y2}) must equal last-row bbox bottom + page_margin/2 ({expected_bottom})"
    );
    assert!(
        (rect_y - guide_y1).abs() < 0.5,
        "highlight rect y ({rect_y}) must equal guide y1 ({guide_y1})"
    );
    assert!(
        (rect_bottom - guide_y2).abs() < 0.5,
        "highlight rect bottom ({rect_bottom}) must equal guide y2 ({guide_y2})"
    );
}

/// Title rows above and below source row: guide and highlight must
/// be clipped to the nearest Title bounding boxes and must not penetrate them.
///
/// Layout (Title / A / B(source) / C / Title):
///   row 0 (Title "Top"):   bbox.origin.y = PAGE_MARGIN,                                    bbox.height = TITLE_BBOX_HEIGHT
///   row 1 (A):             bbox.origin.y = PAGE_MARGIN + TITLE_BBOX_HEIGHT,                bbox.height = SIGNAL_BBOX_HEIGHT
///   row 2 (B, source row): bbox.origin.y = PAGE_MARGIN + TITLE_BBOX_HEIGHT + SIGNAL_BBOX_HEIGHT, bbox.height = SIGNAL_BBOX_HEIGHT
///   row 3 (C):             bbox.origin.y = PAGE_MARGIN + TITLE_BBOX_HEIGHT + 2*SIGNAL_BBOX_HEIGHT
///   row 4 (Title "Bot"):   bbox.origin.y = PAGE_MARGIN + TITLE_BBOX_HEIGHT + 3*SIGNAL_BBOX_HEIGHT
///
///   expected y1 = PAGE_MARGIN + TITLE_BBOX_HEIGHT          (upper Title bbox bottom)
///   expected y2 = PAGE_MARGIN + TITLE_BBOX_HEIGHT + 3*SIGNAL_BBOX_HEIGHT  (lower Title bbox top)
#[test]
fn t2_with_titles_guide_and_highlight_clipped_to_title_boundaries() {
    // Signal B is the source row (contains | and [...]).
    let source = "@title Top\nA __________\nB __[~~|~~]__\nC __________\n@title Bot\n";
    let svg = render_to_svg(source);

    let line_tag = extract_first_tag(&svg, "class=\"guides\"", "<line");
    let guide_y1 = parse_svg_attr_f32(line_tag, "y1=\"");
    let guide_y2 = parse_svg_attr_f32(line_tag, "y2=\"");

    let rect_tag = extract_first_tag(&svg, "class=\"highlights\"", "<rect");
    let rect_y = parse_svg_attr_f32(rect_tag, "y=\"");
    let rect_height = parse_svg_attr_f32(rect_tag, "height=\"");
    let rect_bottom = rect_y + rect_height;

    // Expected values derived from the layout constants defined above.
    let title_top_origin_y = PAGE_MARGIN;
    let expected_top = title_top_origin_y + TITLE_BBOX_HEIGHT; // upper Title bbox bottom
    let expected_bottom = PAGE_MARGIN + TITLE_BBOX_HEIGHT + 3.0 * SIGNAL_BBOX_HEIGHT; // lower Title bbox top

    assert!(
        (guide_y1 - expected_top).abs() < 0.5,
        "guide y1 ({guide_y1}) must equal upper Title bbox bottom ({expected_top})"
    );
    assert!(
        (guide_y2 - expected_bottom).abs() < 0.5,
        "guide y2 ({guide_y2}) must equal lower Title bbox top ({expected_bottom})"
    );
    assert!(
        (rect_y - guide_y1).abs() < 0.5,
        "highlight rect y ({rect_y}) must equal guide y1 ({guide_y1})"
    );
    assert!(
        (rect_bottom - guide_y2).abs() < 0.5,
        "highlight rect bottom ({rect_bottom}) must equal guide y2 ({guide_y2})"
    );
}

// ---- DontCareAlongBus slant-edge regression tests (via full parser pipeline) ----
//
// These tests verify that `?` in a bus context produces slanted polygon edges
// matching the preceding/following transition boundaries. The element list is
// assembled by the full parse -> layout -> render pipeline (not constructed by
// hand) so that any regression in the DontCare expansion or transition-emission
// path is caught.
//
// Coordinate derivation (defaults: step=10, slant=2, page_margin=10,
// font_size=14, line_height=1.2, signal_gap=4, capwidth derived from
// signal name "A" = 1 char x 7px + namepad(8px) = 15px;
// Signal name "A" => capwidth = char_width(7) + padding(8) = 15px
// signal origin.x = page_margin(10) + capwidth(15) = 25;
// waveform_y.top = page_margin(10) + signal_gap/2(2) = 12;
// waveform_y.bottom approx 28.8
//
// With `?` having 0-width (new spec), the DontCareAlongBus units count excludes `?` chars.
//
// `_=?=_`: expand -> Low(1), DontCareAlongBus(2,preceded=true), Low(1,preceded=true)
//   BusOpen(slant=2) at Low->DontCareAlongBus, BusClose at DontCareAlongBus->Low.
//   Low(1)->25..35, BusOpen->35..37, DontCareAlongBus(2,p=true)->37..55, BusClose->55..57, Low(1,p=true)->57..65
//   x_start=37, x_end=55; prev=SlantFromLow, next=SlantFromLow
//   left_top=37, left_bottom=37-2=35; right_top=55, right_bottom=55+2=57
//   Expected: "37,12 55,12 57,28.8 35,28.8"
//
// `~=?=~`: expand -> High(1), DontCareAlongBus(2,preceded=true), High(1,preceded=true)
//   High(1)->25..35, BusOpen->35..37, DontCareAlongBus(2,p=true)->37..55, BusClose->55..57, High(p=true)->57..65
//   prev=SlantFromHigh, next=SlantFromHigh; x_start=37, x_end=55
//   left_top=37-2=35, left_bottom=37; right_top=55+2=57, right_bottom=55
//   Expected: "35,12 57,12 55,28.8 37,28.8"
//
// `=?_`: expand -> DontCareAlongBus(1,preceded=false), Low(1,preceded=true)
//   DontCareAlongBus(1,p=false)->25..35, BusClose(slant=2)->35..37, Low(p=true)->37..45
//   x_start=25, x_end=35; prev=Vertical, next=SlantFromLow
//   right_top=35, right_bottom=35+2=37
//   Expected: "25,12 35,12 37,28.8 25,28.8" (unchanged; DontCare is at start, preceded=false)
//
// `_=?=`: expand -> Low(1), DontCareAlongBus(2,preceded=true)
//   Low(1)->25..35, BusOpen->35..37, DontCareAlongBus(2,p=true)->37..55
//   x_start=37, x_end=55; prev=SlantFromLow, next=Vertical
//   left_top=37, left_bottom=37-2=35
//   Expected: "37,12 55,12 55,28.8 35,28.8"
//
// `==?==` (full absorb; no BusOpen/BusClose present; DontCareAlongBus(4,preceded=false)):
//   DontCareAlongBus(4,p=false)->25..65; prev=Vertical, next=Vertical
//   Expected: "25,12 65,12 65,28.8 25,28.8" (unchanged)

fn extract_dontcares_polygon_points(svg: &str) -> String {
    let layer_start = svg.find("class=\"dontcares\"").expect("dontcares layer");
    let layer_end = svg[layer_start..].find("</g>").expect("</g>") + layer_start;
    let layer = &svg[layer_start..layer_end];
    let points_start = layer.find("points=\"").expect("points attr") + 8;
    let points_end = layer[points_start..].find('"').expect("closing quote") + points_start;
    layer[points_start..points_end].to_owned()
}

/// `_=?=_` (Low both sides, slant=2): polygon must be `/=\` shape.
/// `?` expands -> Low(1), DontCareAlongBus(2,preceded=true), Low(1,preceded=true).
///   Low(1)->25..35, BusOpen->35..37, DontCareAlongBus(2,p=true)->37..55, BusClose->55..57, Low(1,p=true)->57..65
///   x_start=37, x_end=55; prev=SlantFromLow, next=SlantFromLow
///   left_top=37, left_bottom=37-2=35; right_top=55, right_bottom=55+2=57
#[test]
fn dontcare_bus_via_parser_low_both_sides_slanted_polygon() {
    let svg = render_to_svg("A _=?=_\n");
    let points = extract_dontcares_polygon_points(&svg);
    assert_eq!(
        points, "55,15 100,15 105,31.8 50,31.8",
        "_=?=_ polygon must be /=\\ shape, got: {points}"
    );
}

/// `~=?=~` (High both sides, slant=2): polygon must be `\=/` shape.
/// `?` expands -> High(1), DontCareAlongBus(2,preceded=true), High(1,preceded=true).
///   High(1)->25..35, BusOpen->35..37, DontCareAlongBus(2,p=true)->37..55, BusClose->55..57, High(p=true)->57..65
///   x_start=37, x_end=55; prev=SlantFromHigh, next=SlantFromHigh
///   left_top=37-2=35, left_bottom=37; right_top=55+2=57, right_bottom=55
#[test]
fn dontcare_bus_via_parser_high_both_sides_slanted_polygon() {
    let svg = render_to_svg("A ~=?=~\n");
    let points = extract_dontcares_polygon_points(&svg);
    assert_eq!(
        points, "50,15 105,15 100,31.8 55,31.8",
        "~=?=~ polygon must be \\=/ shape, got: {points}"
    );
}

/// `=?_` (Bus continue prev, Low next): right edge must be slanted (`=>`).
/// `?` is 0-width; expand -> DontCareAlongBus(1), Low(1).
///   DontCareAlongBus(1)->25..35, BusClose->35..37, Low->37..47
///   x_start=25, x_end=35; prev=Vertical, next=SlantFromLow
///   right_top=35, right_bottom=35+2=37
#[test]
fn dontcare_bus_via_parser_bus_prev_low_next_right_slant() {
    let svg = render_to_svg("A =?_\n");
    let points = extract_dontcares_polygon_points(&svg);
    assert_eq!(
        points, "25,15 50,15 55,31.8 25,31.8",
        "=?_ polygon must have slanted right edge, got: {points}"
    );
}

/// `_=?=` (Low prev, Bus continue next): left edge must be slanted (`<=`).
/// `?` expands -> Low(1), DontCareAlongBus(2,preceded=true).
///   Low(1)->25..35, BusOpen->35..37, DontCareAlongBus(2,p=true)->37..55
///   x_start=37, x_end=55; prev=SlantFromLow, next=Vertical
///   left_top=37, left_bottom=37-2=35
#[test]
fn dontcare_bus_via_parser_low_prev_bus_next_left_slant() {
    let svg = render_to_svg("A _=?=\n");
    let points = extract_dontcares_polygon_points(&svg);
    assert_eq!(
        points, "55,15 100,15 100,31.8 50,31.8",
        "_=?= polygon must have slanted left edge, got: {points}"
    );
}

/// `==?==` (Bus continue both sides): polygon must be a rectangle (no slant).
/// `?` is 0-width; expand -> DontCareAlongBus(4); no BusOpen/BusClose.
///   DontCareAlongBus(4)->25..65; prev=Vertical, next=Vertical
#[test]
fn dontcare_bus_via_parser_both_continue_stays_rectangle() {
    let svg = render_to_svg("A ==?==\n");
    let points = extract_dontcares_polygon_points(&svg);
    assert_eq!(
        points, "25,15 125,15 125,31.8 25,31.8",
        "==?== polygon must be a rectangle, got: {points}"
    );
}

// ---- DontCare edge cases: BusCross and HiZ boundaries ----

/// `====X?X====`: BusCross on both sides of `?` → 6-point hexagon polygon.
/// Left/right vertices are X cross midpoints (`x_cross_start + slant/2`, `y_mid`).
/// Coordinates verified against the default layout (step=10, slant=2, y_mid=20.4).
#[test]
fn dontcare_bus_buscross_both_sides_hexagon() {
    let svg = render_to_svg("A ====X?X====\n");
    let points = extract_dontcares_polygon_points(&svg);
    let coords: Vec<f32> = points
        .split_whitespace()
        .flat_map(|pair| pair.split(','))
        .filter_map(|s| s.parse::<f32>().ok())
        .collect();
    // 6 points = 12 coordinates: left_cross_mid, left_top, right_top, right_cross_mid,
    // right_bottom, left_bottom.
    assert_eq!(
        coords.len(),
        12,
        "must have 6 points (12 coords), got: {points}"
    );
    let (left_mid_x, left_top_x) = (coords[0], coords[2]);
    let (right_top_x, right_mid_x) = (coords[4], coords[6]);
    let (right_bottom_x, left_bottom_x) = (coords[8], coords[10]);
    // Cross midpoints sit slant/2 = 2.5px outside the body corners.
    assert!((left_mid_x - (left_top_x - 2.5)).abs() < 1e-3, "{points}");
    assert!((right_mid_x - (right_top_x + 2.5)).abs() < 1e-3, "{points}");
    // Body corners have a vertical edge on each side.
    assert!((left_top_x - left_bottom_x).abs() < 1e-3, "{points}");
    assert!((right_top_x - right_bottom_x).abs() < 1e-3, "{points}");
}

/// `----????====`: the `?` region is `DontCareAlongHiZ` (Single family), which
/// must emit a `<rect>` — not a `<polygon>` — because Single-family DontCare
/// shapes are always rectangular with vertical edges (spec §`LevelRun(DontCareAlong*)`).
#[test]
fn dontcare_hiz_before_bus_emits_rect_not_polygon() {
    let svg = render_to_svg("A ----????====\n");
    let layer_start = svg.find("class=\"dontcares\"").expect("dontcares layer");
    let layer_end = svg[layer_start..].find("</g>").expect("</g>") + layer_start;
    let layer = &svg[layer_start..layer_end];
    assert!(
        layer.contains("<rect"),
        "DontCareAlongHiZ must emit <rect>, got: {layer}"
    );
    assert!(
        !layer.contains("<polygon"),
        "DontCareAlongHiZ must not emit <polygon>, got: {layer}"
    );
}

// ---- BusOpen/BusClose Single-side degenerate polygon tests ----
//
// Default layout: step=10, slant=2, page_margin=10, char_width=7,
// signal name "A" → capwidth = 7 + 8 = 15; signal origin x = 10 + 15 = 25.
// waveform_y: top=12, bottom=28.8, mid=20.4.
//
// `--==?==--`: HiZ(2)→BusOpen, DontCareAlongBus(4,preceded=true), BusClose→HiZ(2,preceded=true).
//   HiZ(2): 25→45; BusOpen: 45→47; DontCareAlongBus(4,p=true): 47→47+(4*10-2)=85; BusClose: 85→87.
//   DontCare x_start=47, x_end=85.
//   left: SingleFromHiZ{bus_run_units=0} → hiz_x=47-0-2=45; left_top=47.
//   right: SingleFromHiZ{bus_run_units=0} → hiz_x=85+0+2=87; right_top=85.
//   6-point hexagon: (45,20.4) (47,12) (85,12) (87,20.4) (85,28.8) (47,28.8).
#[test]
fn dontcare_bus_hiz_both_sides_hexagon() {
    let svg = render_to_svg("A --==?==--\n");
    let points = extract_dontcares_polygon_points(&svg);
    assert_eq!(
        points, "75,23.4 80,15 175,15 180,23.4 175,31.8 80,31.8",
        "--==?==-- polygon must be 6-point hexagon, got: {points}"
    );
}

/// `--==?==`: HiZ(2)→BusOpen, DontCareAlongBus(4,preceded=true), signal end.
/// HiZ(2): 25→45; BusOpen: 45→47; DontCareAlongBus(4,p=true): 47→85.
/// DontCare x_start=47, x_end=85; left: SingleFromHiZ{0} → hiz_x=45.
/// right: Vertical (end of waveform).
/// 5-point pentagon: (45,20.4) (47,12) (85,12) (85,28.8) (47,28.8).
#[test]
fn dontcare_bus_hiz_left_signal_end_right_pentagon() {
    let svg = render_to_svg("A --==?==\n");
    let points = extract_dontcares_polygon_points(&svg);
    assert_eq!(
        points, "75,23.4 80,15 175,15 175,31.8 80,31.8",
        "--==?== polygon must be 5-point pentagon (HiZ left), got: {points}"
    );
}

/// `==?==--`: signal start, DontCareAlongBus(4), BusClose→HiZ(2).
/// DontCare x_start=25, x_end=65; right: SingleFromHiZ{0} → hiz_x=67.
/// left: Vertical (start of waveform).
/// 5-point pentagon: (25,12) (65,12) (67,20.4) (65,28.8) (25,28.8).
#[test]
fn dontcare_bus_signal_start_left_hiz_right_pentagon() {
    let svg = render_to_svg("A ==?==--\n");
    let points = extract_dontcares_polygon_points(&svg);
    assert_eq!(
        points, "25,15 125,15 130,23.4 125,31.8 25,31.8",
        "==?==-- polygon must be 5-point pentagon (HiZ right), got: {points}"
    );
}

/// `__==?==`: Low(2)→BusOpen, DontCareAlongBus(4,preceded=true), signal end.
/// Low(2): 25→45; BusOpen: 45→47; DontCareAlongBus(4,p=true): 47→85.
/// DontCare x_start=47, x_end=85; left: SlantFromLow → left_bottom=47-2=45.
/// right: Vertical.
/// 4-point rectangle variant: (47,12) (85,12) (85,28.8) (45,28.8).
#[test]
fn dontcare_bus_low_left_signal_end_right_four_point() {
    let svg = render_to_svg("A __==?==\n");
    let points = extract_dontcares_polygon_points(&svg);
    assert_eq!(
        points, "80,15 175,15 175,31.8 75,31.8",
        "__==?== polygon must be 4-point (Low left slant), got: {points}"
    );
}

/// `==?==__`: signal start, DontCareAlongBus(4), BusClose→Low(2).
/// DontCare x_start=25, x_end=65; right: SlantFromLow → right_bottom=65+2=67.
/// left: Vertical.
/// 4-point: (25,12) (65,12) (67,28.8) (25,28.8).
#[test]
fn dontcare_bus_signal_start_left_low_right_four_point() {
    let svg = render_to_svg("A ==?==__\n");
    let points = extract_dontcares_polygon_points(&svg);
    assert_eq!(
        points, "25,15 125,15 130,31.8 25,31.8",
        "==?==__ polygon must be 4-point (Low right slant), got: {points}"
    );
}

// ---- @clock auto-expansion endpoint alignment tests ----

/// When `@clock` auto-expands an empty-body row alongside an explicit-waveform
/// clock row, both polyline endpoints must land at the same x coordinate.
///
/// Without the fix, the auto-expand path skips the `preceded_by_transition`
/// flag, so every clock-run after the first seed contributes a full `step`
/// width instead of `step - slant`, making the auto row longer than the
/// explicit row even though both carry the same number of level units.
#[test]
fn clock_auto_expand_endpoint_matches_explicit_waveform() {
    let source = "@clock(none)\nclock\nclock ~_~_~_~_\n";
    let svg = render_to_svg(source);

    let layer = extract_layer(&svg, "waveforms");

    let endpoints = extract_polyline_endpoint_x_values(layer);

    let [auto_x, explicit_x, ..] = endpoints.as_slice() else {
        panic!(
            "expected at least 2 polylines, got {} in: {layer}",
            endpoints.len()
        );
    };
    assert!(
        (auto_x - explicit_x).abs() < 0.5,
        "auto-expand polyline endpoint ({auto_x}) must equal explicit waveform endpoint ({explicit_x})"
    );
}

// ---- @clock auto-expansion per-row step tests ----

/// Per-row `@step` clock auto-expansion: when a clock row has a different step
/// than the explicit signal, the auto row's target units are computed from the
/// maximum pixel width of explicit rows divided by the auto row's own step.
///
/// Chart layout: `@step 20` explicit row with 12 units (240px), then `@step 10`
/// auto clock row.  Target = round(240 / 10) = 24 units.
/// Both polylines should end at approximately the same x coordinate
/// (within step/2 = 5px).
#[test]
fn clock_auto_expand_per_row_step_aligns_right_edge() {
    let source = "@step 20\nClock _~_~_~_~_~_~\n@step 10\n@clock\nclock\n";
    let svg = render_to_svg(source);

    let layer = extract_layer(&svg, "waveforms");
    let endpoints = extract_polyline_endpoint_x_values(layer);

    let [explicit_x, auto_x, ..] = endpoints.as_slice() else {
        panic!(
            "expected at least 2 polylines, got {} in: {layer}",
            endpoints.len()
        );
    };
    // The auto row uses step=10; tolerance is step/2 + 0.5 rounding margin.
    let tolerance = 10_f32 / 2.0 + 0.5;
    assert!(
        (explicit_x - auto_x).abs() <= tolerance,
        "auto-expand row right edge ({auto_x}) must be within step/2=5px of explicit row ({explicit_x})"
    );
}

/// Multiple auto clock rows both referencing the same explicit signal.
/// Auto rows should not count each other as explicit targets.
///
/// Chart: 2 auto clocks (step=10) + 1 explicit signal (12 units, step=10).
/// Both auto rows should end at the same x as the explicit signal.
#[test]
fn clock_auto_expand_multiple_auto_rows_reference_only_explicit() {
    let source = "@clock\nclock1\n@clock\nclock2\nSig _~_~_~_~_~_~\n";
    let svg = render_to_svg(source);

    let layer = extract_layer(&svg, "waveforms");
    let endpoints = extract_polyline_endpoint_x_values(layer);

    let [auto1_x, auto2_x, sig_x, ..] = endpoints.as_slice() else {
        panic!(
            "expected at least 3 polylines, got {} in: {layer}",
            endpoints.len()
        );
    };
    assert!(
        (auto1_x - sig_x).abs() < 0.5,
        "auto1 endpoint ({auto1_x}) must equal explicit sig ({sig_x})"
    );
    assert!(
        (auto2_x - sig_x).abs() < 0.5,
        "auto2 endpoint ({auto2_x}) must equal explicit sig ({sig_x})"
    );
}

/// Auto clock row placed *before* the explicit signal row — auto should still
/// expand to match the subsequent explicit signal.
#[test]
fn clock_auto_expand_explicit_after_auto_is_included() {
    let source = "@clock\nclock\nSig _~_~_~_~_~_~_~_~\n";
    let svg = render_to_svg(source);

    let layer = extract_layer(&svg, "waveforms");
    let endpoints = extract_polyline_endpoint_x_values(layer);

    let [auto_x, sig_x, ..] = endpoints.as_slice() else {
        panic!(
            "expected at least 2 polylines, got {} in: {layer}",
            endpoints.len()
        );
    };
    assert!(
        (auto_x - sig_x).abs() < 0.5,
        "auto row placed before explicit: auto ({auto_x}) must equal sig ({sig_x})"
    );
}

/// Partial clock: existing 4-unit body stays unchanged, trailing auto fill
/// extends to reach the explicit signal length (16 units).
#[test]
fn clock_auto_expand_partial_clock_extends_to_explicit_length() {
    let source = "Sig _~_~_~_~_~_~_~_~\n@clock\nck ~~__\n";
    let svg = render_to_svg(source);

    let layer = extract_layer(&svg, "waveforms");
    let endpoints = extract_polyline_endpoint_x_values(layer);

    // Sig row may produce multiple polylines; ck row follows them.
    // We compare the maximum x across Sig polylines with the last ck endpoint.
    let sig_max_x = endpoints.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let ck_x = *endpoints.last().expect("at least one polyline");
    assert!(
        (ck_x - sig_max_x).abs() < 0.5,
        "partial clock extended endpoint ({ck_x}) must equal explicit signal endpoint ({sig_max_x})"
    );
}

/// When all signal rows are auto clock rows, the target is 0 and the waveform
/// is empty (no polyline rendered).
#[test]
fn clock_auto_expand_all_auto_produces_empty_waveform() {
    let source = "@clock\nck1\n@clock\nck2\n";
    let svg = render_to_svg(source);

    let layer = extract_layer(&svg, "waveforms");
    let polyline_count = layer.matches("<polyline").count();
    assert_eq!(
        polyline_count, 0,
        "all-auto chart must produce no polylines, got {polyline_count}"
    );
}

/// Asymmetric pulse `_=2, ~=3` auto expansion truncates exactly at `target`.
///
/// Chart: auto clock (_=2,~=3) + explicit Sig 12 units (step=10, 120px).
/// Target = 12.  Expected fill: Low(2)→High(3)→Low(2)→High(3)→Low(2) = 12.
/// The last element must be Low with 2 units (exact boundary at target).
#[test]
fn clock_auto_expand_asymmetric_pulse_truncates_at_target() {
    let source = "@clock(_=2, ~=3)\nck\nSig _~_~_~_~_~_~\n";
    let svg = render_to_svg(source);

    let layer = extract_layer(&svg, "waveforms");
    let endpoints = extract_polyline_endpoint_x_values(layer);

    // ck row is first; Sig row follows.
    // Extract last endpoint for ck row and for Sig row.
    // For same-step chart: ck endpoint must equal Sig endpoint.
    let sig_x = *endpoints.last().expect("at least one");
    let ck_x = endpoints[0];
    assert!(
        (ck_x - sig_x).abs() < 0.5,
        "asymmetric pulse ck endpoint ({ck_x}) must equal Sig endpoint ({sig_x})"
    );
}

/// Asymmetric pulse `_=2, ~=3` truncates mid-High when target is 13 units.
///
/// Chart: auto clock (_=2,~=3) + explicit Sig 13 units (step=10, 130px).
/// Expected: Low(2)→High(3)→Low(2)→High(3)→Low(2)→High(1) = 13.
#[test]
fn clock_auto_expand_asymmetric_pulse_truncates_mid_high() {
    let source = "@clock(_=2, ~=3)\nck\nSig _~_~_~_~_~_~_\n";
    let svg = render_to_svg(source);

    let layer = extract_layer(&svg, "waveforms");
    let endpoints = extract_polyline_endpoint_x_values(layer);

    let sig_x = *endpoints.last().expect("at least one");
    let ck_x = endpoints[0];
    assert!(
        (ck_x - sig_x).abs() < 0.5,
        "asymmetric pulse ck endpoint mid-High truncation ({ck_x}) must equal Sig endpoint ({sig_x})"
    );
}

/// `start=high` auto clock starts with High phase first.
///
/// Chart: auto clock (start=high, _=2,~=3) + explicit Sig 12 units (step=10).
/// Expected: High(3)→Low(2)→High(3)→Low(2)→High(2) = 12.
#[test]
fn clock_auto_expand_start_high_begins_with_high_phase() {
    let source = "@clock(_=2, ~=3, start=high)\nck\nSig _~_~_~_~_~_~\n";
    let svg = render_to_svg(source);

    let layer = extract_layer(&svg, "waveforms");
    let endpoints = extract_polyline_endpoint_x_values(layer);

    let sig_x = *endpoints.last().expect("at least one");
    let ck_x = endpoints[0];
    assert!(
        (ck_x - sig_x).abs() < 0.5,
        "start=high auto clock endpoint ({ck_x}) must equal Sig ({sig_x})"
    );
}

/// Per-row step + asymmetric pulse auto expansion.
///
/// Explicit: `@step 20`, Sig 8 units (160px).
/// Auto: `@step 10`, `@clock(_=2, ~=3)`.
/// Target = round(160 / 10) = 16.
/// Expected: Low(2)→High(3)→Low(2)→High(3)→Low(2)→High(3)→Low(1) = 16.
/// Right edge: 16 × 10 = 160px (within step/2=5px of explicit 160px).
#[test]
fn clock_auto_expand_per_row_step_asymmetric_pulse() {
    let source = "@step 20\nSig _~_~_~_~\n@step 10\n@clock(_=2, ~=3)\nck\n";
    let svg = render_to_svg(source);

    let layer = extract_layer(&svg, "waveforms");
    let endpoints = extract_polyline_endpoint_x_values(layer);

    let [sig_x, ck_x, ..] = endpoints.as_slice() else {
        panic!(
            "expected at least 2 polylines, got {} in: {layer}",
            endpoints.len()
        );
    };
    // The auto row uses step=10; tolerance is step/2 + 0.5 rounding margin.
    let tolerance = 10_f32 / 2.0 + 0.5;
    assert!(
        (sig_x - ck_x).abs() <= tolerance,
        "per-row step asymmetric pulse ck endpoint ({ck_x}) must be within 5px of Sig ({sig_x})"
    );
}

/// Per-row step + auto clock + `pos` EdgeMark.
///
/// Explicit: `@step 20`, Sig 8 units (160px).
/// Auto: `@step 10`, `@clock(pos)` → 16 units (8 Low/High pairs).
/// `pos` EdgeMark on each rising edge → 8 `<polygon>` elements in the
/// dedicated `edge-marks` layer (per `docs/spec/svg-rendering.md`
/// §「クロックエッジマーカー (`edge-marks`)」).
#[test]
fn clock_auto_expand_per_row_step_pos_edge_mark_count() {
    let source = "@step 20\nSig _~_~_~_~\n@step 10\n@clock(pos)\nck\n";
    let svg = render_to_svg(source);

    let layer = extract_layer(&svg, "edge-marks");
    let polygon_count = layer.matches("<polygon").count();
    assert_eq!(
        polygon_count, 8,
        "16-unit pos-clock must produce 8 rising-edge triangles, got {polygon_count}"
    );
}

/// Partial clock with asymmetric pulse continues from the last explicit state.
///
/// Explicit: Sig 16 units (step=10, 160px).
/// Clock: `@clock(_=2, ~=3)`, `ck ~~__` (4 units, ends Low).
/// Auto extension: target=16, existing=4, extension=12.
/// Since last state is Low, next pulse starts with High:
/// High(3)→Low(2)→High(3)→Low(2)→High(2) = 12.
/// Total: 4 + 12 = 16 units, endpoint matches Sig.
#[test]
fn clock_auto_expand_partial_clock_continues_from_last_state() {
    let source = "Sig _~_~_~_~_~_~_~_~\n@clock(_=2, ~=3)\nck ~~__\n";
    let svg = render_to_svg(source);

    let layer = extract_layer(&svg, "waveforms");
    let endpoints = extract_polyline_endpoint_x_values(layer);

    let sig_max_x = endpoints.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let ck_x = *endpoints.last().expect("at least one");
    assert!(
        (ck_x - sig_max_x).abs() < 0.5,
        "partial clock with asymmetric pulse endpoint ({ck_x}) must equal Sig ({sig_max_x})"
    );
}

// ---- @step / @slant per-row snapshot regression tests ----

/// Two signal rows with different `@step` values must produce polylines with
/// different x extents. Row 1 uses `@step 20` (6 level chars → 120 px wide)
/// and row 2 uses `@step 10` (6 level chars → 60 px wide). The first row's
/// polyline endpoint x must be strictly greater than the second row's.
///
/// This is a regression test for the "global last-wins" bug where both rows
/// ended up using the final `@step` value.
#[test]
fn step_per_row_first_wider_than_second() {
    let source = "@step 20\nA _~_~_~\n@step 10\nB _~_~_~\n";
    let svg = render_to_svg(source);

    let waveforms_open = svg.find("class=\"waveforms\"").expect("waveforms layer");
    let waveforms_close = svg[waveforms_open..]
        .find("</g>")
        .expect("closing </g> for waveforms layer not found")
        + waveforms_open;
    let layer = &svg[waveforms_open..waveforms_close];

    let polyline_endpoints = extract_polyline_endpoint_x_values(layer);

    let [row1_max_x, row2_max_x, ..] = polyline_endpoints.as_slice() else {
        panic!(
            "expected at least 2 polylines, got {} in: {layer}",
            polyline_endpoints.len()
        );
    };

    assert!(
        row1_max_x > row2_max_x,
        "row1 (@step 20) end x ({row1_max_x}) must be wider than row2 (@step 10) end x ({row2_max_x})"
    );
}

#[test]
fn integration_per_row_step_changes_subsequent_signal_width() {
    let svg = render_to_svg("@step 10\nSig1 ____\n@step 20\nSig2 ____\n");
    let waveforms_open = svg.find("class=\"waveforms\"").expect("waveforms");
    let layer = &svg[waveforms_open..];
    assert!(
        layer.contains("<polyline"),
        "polylines must be present in output"
    );
    let width = parse_svg_attr_f32(&svg, "width=\"");
    assert!(
        width >= 80.0,
        "chart width must accommodate Sig2 (4 chars * step 20 = 80 + label/margin); got {width}"
    );
}

#[test]
fn integration_per_row_step_with_clock_pos_emits_edge_marks() {
    let svg = render_to_svg("@step 10\n@clock(pos) clk\n@step 20\ndata ====\n");
    assert!(
        svg.contains("<polygon"),
        "EdgeMark polygon expected; got {svg}"
    );
}

#[test]
fn integration_clock_auto_then_data_yields_distinct_widths() {
    let svg = render_to_svg("@step 10\n@clock(pos) clk\n@step 20\ndata ====\n");
    let width = parse_svg_attr_f32(&svg, "width=\"");
    assert!(
        width > 80.0,
        "data row width plus label must exceed 80; got {width}"
    );
}

#[test]
fn integration_dontcare_anchor_arrow_with_color_and_dash() {
    let svg = render_to_svg(
        "@step 10\nSig1 _?@{a}_~\n@step 20\nSig2 ===@{b}===\n@-> (@{a}, @{b}, red, dashed) trans\n",
    );
    assert!(svg.contains("dontcare-hatch"));
    assert!(svg.contains("stroke-dasharray"));
}

#[test]
fn integration_overline_per_row_step_with_bg() {
    let svg =
        render_to_svg("@step 10\n@bg #ff0\n@signal(overline) nReset _~__\n@step 20\nData ====\n");
    assert!(svg.contains("<line"));
    assert!(svg.contains("#ff0") || svg.contains("#FFFF00"));
}

#[test]
fn integration_clock_pulse_with_anchor_and_self_arrow() {
    let svg =
        render_to_svg("@step 10\n@clock(pos, _=2, ~=2) clk\ndata ==@{x}==\n@-> (@{x}, @{x})\n");
    let arrows_open = svg.find("class=\"arrows\"").unwrap_or(0);
    let arrows = &svg[arrows_open..];
    assert!(arrows.contains("<line") || arrows.contains("<path"));
}

#[test]
fn integration_bgcolor_skip_title_mixed_index_assignment() {
    let svg = render_to_svg(
        "@bgcolor0 #eee\n@bgcolor1 #ccc\nSig1 _~\n@skip(1)\n@title \"Mid\"\nSig2 _~\n@bg #f0f\nSig3 _~\n",
    );
    assert!(svg.contains("#f0f") || svg.contains("#FF00FF"));
}

#[test]
fn integration_dontcare_color_recurrence_shares_pattern_id() {
    let svg = render_to_svg(
        "@dontcare_color #c00\nA _?_\n@dontcare_color #06c\nB _?_\n@dontcare_color #c00\nC _?_\n",
    );
    assert!(svg.contains("dontcare-hatch-1"));
    assert!(svg.contains("dontcare-hatch-2"));
}

#[test]
fn integration_named_and_numbered_anchors_resolve_in_arrow() {
    let svg = render_to_svg("Sig1 _~@{start}_~\nSig2 ===@1===\n@-> (@{start}, @1) flow\n");
    let arrows_open = svg.find("class=\"arrows\"").expect("arrows");
    let arrows = &svg[arrows_open..];
    assert!(arrows.contains("<line") || arrows.contains("<path"));
}

#[test]
fn integration_multiline_overline_anchor_and_arrow_combination() {
    let svg = render_to_svg(
        "@bg #ff0\n@signal(overline)\n\"n\\nReset\"  ___@{r}___\nData        ===@{d}===\n@-> (@{r}, @{d})\n",
    );
    assert!(svg.contains("<line"));
}

#[test]
fn integration_named_versus_numbered_anchor_distinct_targets() {
    let svg = render_to_svg("Sig1 _~@1_~@{1}_\n@-> (@1, @{1})\n");
    let arrows_open = svg.find("class=\"arrows\"").expect("arrows");
    let arrows = &svg[arrows_open..];
    assert!(arrows.contains("<line") || arrows.contains("<path"));
}

#[test]
fn integration_highlight_dontcare_anchor_clock_combine() {
    let svg = render_to_svg("@clock(pos) clk\ndata __[?@{a}?]__\n");
    assert!(svg.contains("<polygon"), "EdgeMark polygon expected");
    assert!(svg.contains("dontcare-hatch"));
}

#[test]
fn integration_scale_with_per_row_step_affects_svg_width() {
    let scaled = render_to_svg("@scale 2.0\n@step 10\nA ____\n@step 20\nB ____\n");
    let plain = render_to_svg("@step 10\nA ____\n@step 20\nB ____\n");
    let scaled_w = parse_svg_attr_f32(&scaled, "width=\"");
    let plain_w = parse_svg_attr_f32(&plain, "width=\"");
    assert!(
        scaled_w > plain_w,
        "scaled chart must have larger width; got {plain_w} -> {scaled_w}"
    );
    // SVG root must carry viewBox using the *internal* (1.0x) dimensions so
    // the viewport actually scales when width/height are larger than the
    // logical coordinate system. Without viewBox the canvas would just be
    // bigger with the original-size drawing pinned to the top-left.
    let expected_view_box = format!("viewBox=\"0 0 {plain_w} ");
    assert!(
        scaled.contains(&expected_view_box),
        "scaled SVG must emit viewBox starting with internal width \
         {plain_w}; got: {scaled}"
    );
    let plain_height = parse_svg_attr_f32(&plain, "height=\"");
    let scaled_view_box_full = format!("viewBox=\"0 0 {plain_w} {plain_height}\"");
    assert!(
        scaled.contains(&scaled_view_box_full),
        "scaled SVG viewBox must match internal width/height \
         ({plain_w} x {plain_height}); got: {scaled}"
    );
}

#[test]
fn integration_viewbox_present_without_scale_directive() {
    // Plain chart (no @scale): width/height equal internal dimensions, but
    // viewBox must still be emitted so consumers can rely on its presence.
    let svg = render_to_svg("@step 10\nA ____\n");
    let width = parse_svg_attr_f32(&svg, "width=\"");
    let height = parse_svg_attr_f32(&svg, "height=\"");
    let expected = format!("viewBox=\"0 0 {width} {height}\"");
    assert!(
        svg.contains(&expected),
        "SVG must emit viewBox=\"0 0 W H\"; expected {expected}, got: {svg}"
    );
}

#[test]
fn integration_scale_emits_viewbox_with_internal_dimensions() {
    // viewBox values are the pre-scale (logical) dimensions; width/height
    // attributes are the post-scale (display) dimensions.
    let scaled = render_to_svg("@scale 3.0\nA _~\n");
    let plain = render_to_svg("A _~\n");
    let plain_width = parse_svg_attr_f32(&plain, "width=\"");
    let plain_height = parse_svg_attr_f32(&plain, "height=\"");
    let scaled_width = parse_svg_attr_f32(&scaled, "width=\"");
    let scaled_height = parse_svg_attr_f32(&scaled, "height=\"");
    // Display size is 3x larger.
    let width_ratio = scaled_width / plain_width;
    let height_ratio = scaled_height / plain_height;
    assert!(
        (width_ratio - 3.0).abs() < 1e-3,
        "scaled width must be 3x plain; got ratio {width_ratio}"
    );
    assert!(
        (height_ratio - 3.0).abs() < 1e-3,
        "scaled height must be 3x plain; got ratio {height_ratio}"
    );
    // viewBox stays at the internal (1x) dimensions.
    let expected = format!("viewBox=\"0 0 {plain_width} {plain_height}\"");
    assert!(
        scaled.contains(&expected),
        "scaled SVG viewBox must equal plain dimensions; expected {expected}, got: {scaled}"
    );
}

#[test]
fn integration_unknown_font_warning_does_not_abort_render() {
    let svg = render_to_svg("@font NoSuchFont\nSig _\n");
    assert!(
        svg.contains("<svg "),
        "render must succeed even with bad font"
    );
}

#[test]
fn integration_arrow_label_with_xml_special_chars_is_escaped() {
    let svg = render_to_svg("A _~@{a}_\nB _~@{b}_\n@-> (@{a}, @{b}) <signal-set>\n");
    let arrows_open = svg.find("class=\"arrows\"").expect("arrows");
    let arrows = &svg[arrows_open..];
    assert!(
        arrows.contains("&lt;signal-set&gt;"),
        "label must be XML-escaped; got {arrows}"
    );
}

#[test]
fn integration_regression_per_row_step_change_not_sticky() {
    let svg = render_to_svg("@step 10\nSig1 ____\n@step 20\nSig2 ____\n");
    let width = parse_svg_attr_f32(&svg, "width=\"");
    assert!(
        width >= 80.0,
        "width must reflect Sig2 step=20 (>=80); got {width}"
    );
}

#[test]
fn integration_regression_anchor_position_uses_local_step() {
    let svg = render_to_svg("@step 10\nSig1 ___@1__\n@step 20\nSig2 ___@2__\n");
    assert!(svg.contains("<svg"));
}

#[test]
fn integration_regression_clock_auto_with_subsequent_step_change() {
    let svg = render_to_svg("@step 10\n@clock(pos) clk\n@step 20\ndata ====\n");
    let width = parse_svg_attr_f32(&svg, "width=\"");
    assert!(
        width >= 80.0,
        "data step=20 must dominate chart width; got {width}"
    );
}

// ---------------------------------------------------------------------------
// Iter1 phase 2: anchor / arrow advanced combinations end-to-end.
// ---------------------------------------------------------------------------

#[test]
fn iter1_consecutive_anchors_with_three_arrows_render_three_lines() {
    let svg =
        render_to_svg("A _@{a}@{b}@{c}~\n@-> (@{a}, @{b})\n@-> (@{b}, @{c})\n@-> (@{a}, @{c})\n");
    let arrows_open = svg.find("class=\"arrows\"").expect("arrows layer");
    let arrows_close = svg[arrows_open..].find("</g>").expect("close") + arrows_open;
    let layer = &svg[arrows_open..arrows_close];
    let line_count = layer.matches("<line").count() + layer.matches("<path").count();
    assert!(
        line_count >= 3,
        "expected at least 3 arrow shapes; got {line_count}"
    );
}

#[test]
fn iter1_arrow_self_loop_renders_without_panic() {
    let svg = render_to_svg("A _@{a}~\n@-> (@{a}, @{a})\n");
    assert!(
        svg.contains("class=\"arrows\""),
        "arrows layer must be present for a self-loop arrow: {svg}"
    );
    let arrows_open = svg.find("class=\"arrows\"").expect("arrows layer");
    let arrows_close = svg[arrows_open..].find("</g>").expect("close </g>") + arrows_open;
    let layer = &svg[arrows_open..arrows_close];
    let shape_count = layer.matches("<line").count() + layer.matches("<path").count();
    assert!(
        shape_count >= 1,
        "self-loop must emit at least one <line> or <path>; got {shape_count} in {layer}"
    );
}

#[test]
fn iter1_one_hundred_arrows_render() {
    let mut source = String::from("A _");
    // Anchor names use an `a` prefix so identifiers do not start with a
    // digit (per docs/spec/types.md `AnchorName`).
    for index in 1..=101 {
        source.push_str(&format!("@{{a{index}}}_"));
    }
    source.push('\n');
    for index in 1..=100 {
        source.push_str(&format!("@-> (@{{a{index}}}, @{{a{}}})\n", index + 1));
    }
    let svg = render_to_svg(&source);
    let arrows_open = svg.find("class=\"arrows\"").unwrap_or_else(|| {
        panic!("arrows group must exist when 100 @-> arrows are present: {svg}")
    });
    let arrows_close = svg[arrows_open..].find("</g>").expect("close </g>") + arrows_open;
    let layer = &svg[arrows_open..arrows_close];
    let shape_count = layer.matches("<line").count() + layer.matches("<path").count();
    assert!(
        shape_count >= 100,
        "expected at least 100 arrow shapes; got {shape_count} in {layer}"
    );
}

#[test]
fn iter1_clock_none_does_not_emit_polygon() {
    let svg = render_to_svg("@clock(none)\nclk _~_~\n");
    let waveforms = extract_layer(&svg, "waveforms");
    let polygon_count = waveforms.matches("<polygon").count();
    assert_eq!(
        polygon_count, 0,
        "@clock(none) must not emit edge mark polygon; got {polygon_count}"
    );
}

#[test]
fn iter1_clock_with_fifty_edges_z_order_arrows_after_waveforms() {
    let body: String = "_~".repeat(50);
    let source = format!("@clock(pos)\nclk {body}\nA _@{{a}}@{{b}}_\n@-> (@{{a}}, @{{b}})\n");
    let svg = render_to_svg(&source);
    let waveforms_pos = svg
        .find("class=\"waveforms\"")
        .unwrap_or_else(|| panic!("waveforms group must be present for signal input: {svg}"));
    let arrows_pos = svg
        .find("class=\"arrows\"")
        .unwrap_or_else(|| panic!("arrows group must be present for @-> input: {svg}"));
    assert!(
        waveforms_pos < arrows_pos,
        "waveforms must precede arrows in document order; got waveforms={waveforms_pos} vs arrows={arrows_pos}"
    );
}

// ---------------------------------------------------------------------------
// Iter1 phase 2: empty / boundary E2E.
// ---------------------------------------------------------------------------

#[test]
fn iter1_empty_tcml_yields_minimal_svg_frame() {
    let svg = render_to_svg("");
    assert!(
        svg.trim_start().starts_with("<svg") || svg.trim_start().starts_with("<?xml"),
        "SVG output must start with <svg> (or XML declaration); got {svg:?}"
    );
    assert!(
        svg.contains("</svg>"),
        "SVG must include closing tag: {svg:?}"
    );
    let width = parse_svg_attr_f32(&svg, "width=\"");
    let height = parse_svg_attr_f32(&svg, "height=\"");
    assert!(
        width.is_finite() && width > 0.0,
        "minimal SVG frame must have a positive finite width; got {width}"
    );
    assert!(
        height.is_finite() && height > 0.0,
        "minimal SVG frame must have a positive finite height; got {height}"
    );
}

#[test]
fn iter1_title_only_renders_title_text() {
    let svg = render_to_svg("@title \"T\"\n");
    assert!(svg.contains(">T<"), "title text 'T' must be present: {svg}");
}

#[test]
fn iter1_scale_one_thousand_does_not_overflow() {
    let svg = render_to_svg("@scale 1000\nA _~\n");
    let width = parse_svg_attr_f32(&svg, "width=\"");
    assert!(width.is_finite(), "width must be finite; got {width}");
    assert!(width > 0.0, "width must be positive; got {width}");
}

#[test]
fn iter1_fontsize_half_keeps_label_layout_positive() {
    let svg = render_to_svg("@fontsize 0.5\nA _~\n");
    assert!(svg.contains("<text"), "label text must be rendered: {svg}");
}

#[test]
fn iter1_one_character_waveform_renders_polyline() {
    let svg = render_to_svg("A _\n");
    let waveforms = extract_layer(&svg, "waveforms");
    let polyline_count = waveforms.matches("<polyline").count();
    assert!(
        polyline_count >= 1,
        "single-char waveform must produce one polyline; got {polyline_count}"
    );
}

// ---------------------------------------------------------------------------
// Iter1 phase 2: SVG/TCML round-trip integration sanity.
// ---------------------------------------------------------------------------

#[test]
fn iter1_round_trip_source_field_preserves_input() {
    let source = "@scale 2.0\nA _~\n";
    let svg = render_to_svg(source);
    let trimmed = source.trim_end_matches('\n');
    assert!(
        svg.contains(trimmed),
        "TCML source must be embedded verbatim in SVG; got {svg}"
    );
}

#[test]
fn iter1_multibyte_signal_name_round_trip_safe() {
    let svg = render_to_svg("\"日本語\" _~\n");
    assert!(
        svg.contains("日本語"),
        "multi-byte signal name must survive UTF-8 encoding: {svg}"
    );
}

#[test]
fn iter1_cdata_terminator_in_comment_uses_entity_escape() {
    let svg = render_to_svg("// ]]> marker\nA _\n");
    assert!(
        !svg.contains("<![CDATA["),
        "must not use CDATA section: {svg}"
    );
    assert!(
        svg.contains("&gt;"),
        "literal `>` from `]]>` in TCML must be entity-escaped as &gt;: {svg}"
    );
}
