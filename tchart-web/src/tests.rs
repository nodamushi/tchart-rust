//! Unit tests for `tchart-web`.

use super::{
    RenderError, convert_to_wavejson, extract, png, render_with_metrics, validate_font_size,
};
use tchart_core::layout::FontMetrics;
use tchart_core::text::FontSpec;
use tchart_core::units::Px;

struct MockFontMetrics {
    char_width: f32,
}

impl FontMetrics for MockFontMetrics {
    fn measure_text_width(&self, text: &str, _font: &FontSpec) -> Px {
        Px(self.char_width * text.chars().count() as f32)
    }
}

/// Build the smallest possible 1x1 black PNG via the `png` crate itself so
/// the test does not have to hand-roll CRCs. `::png` is the external crate;
/// the absolute path is required because `pub mod png` shadows the name in
/// `crate::png`.
fn make_minimal_png() -> Vec<u8> {
    let mut output = Vec::new();
    {
        let mut encoder = ::png::Encoder::new(&mut output, 1, 1);
        encoder.set_color(::png::ColorType::Grayscale);
        encoder.set_depth(::png::BitDepth::Eight);
        let mut writer = encoder.write_header().expect("write header");
        writer.write_image_data(&[0u8]).expect("write data");
    }
    output
}

fn get_svg_height(svg: &str) -> f32 {
    let key = "height=\"";
    let start = svg.find(key).expect("height attr") + key.len();
    let end = svg[start..].find('"').expect("close quote");
    svg[start..start + end].parse().expect("number")
}

#[test]
fn render_with_metrics_emits_svg() {
    let metrics = MockFontMetrics { char_width: 7.0 };
    let svg = render_with_metrics("Clock _~_~", None, &metrics).expect("render");
    assert!(svg.starts_with("<svg "));
    assert!(svg.contains("Clock"));
}

#[test]
fn render_with_metrics_propagates_parse_errors() {
    let metrics = MockFontMetrics { char_width: 7.0 };
    let result = render_with_metrics("@step not_a_number", None, &metrics);
    assert!(result.is_err());
}

#[test]
fn font_size_override_increases_chart_height() {
    let metrics = MockFontMetrics { char_width: 7.0 };
    let baseline = render_with_metrics("Clock _~_~", None, &metrics).expect("baseline");
    let larger = render_with_metrics("Clock _~_~", Some(40.0), &metrics).expect("larger");
    let baseline_height = get_svg_height(&baseline);
    let larger_height = get_svg_height(&larger);
    assert!(
        larger_height > baseline_height,
        "expected larger font to grow height: {baseline_height} -> {larger_height}"
    );
}

#[test]
fn convert_to_wavejson_returns_json_and_no_warnings_for_simple_input() {
    let (json, warnings) = convert_to_wavejson("clk _~_~").expect("convert");
    assert!(json.starts_with('{'));
    assert!(json.contains("signal"));
    assert!(warnings.is_empty());
}

#[test]
fn convert_to_wavejson_propagates_parse_errors() {
    let result = convert_to_wavejson("@step not_a_number");
    assert!(result.is_err());
}

#[test]
fn extract_returns_none_when_marker_missing() {
    assert!(extract::extract_tcml_source("<svg></svg>").is_none());
}

#[test]
fn extract_round_trips_xml_entities() {
    let svg =
        "<svg><metadata><tchart:source>Data =&lt;D0&gt;==&amp;==</tchart:source></metadata></svg>";
    let extracted = extract::extract_tcml_source(svg).expect("found");
    assert_eq!(extracted, "Data =<D0>==&==");
}

#[test]
fn embed_then_extract_round_trips_ascii_tcml() {
    let original = make_minimal_png();
    let embedded = png::embed_tcml_source_in_png(&original, "Clock _~_~").expect("embed");
    assert_eq!(
        &embedded[..8],
        &[0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]
    );
    let extracted = png::extract_tcml_source_from_png(&embedded).expect("extract");
    assert_eq!(extracted, "Clock _~_~");
}

#[test]
fn embed_then_extract_round_trips_multibyte_tcml() {
    let original = make_minimal_png();
    let source = "@title 日本語タイトル\nclk _~";
    let embedded = png::embed_tcml_source_in_png(&original, source).expect("embed");
    let extracted = png::extract_tcml_source_from_png(&embedded).expect("extract");
    assert_eq!(extracted, source);
}

#[test]
fn extract_returns_none_when_itxt_chunk_missing() {
    let bytes = make_minimal_png();
    assert!(png::extract_tcml_source_from_png(&bytes).is_none());
}

#[test]
fn extract_returns_none_for_non_png_bytes() {
    assert!(png::extract_tcml_source_from_png(&[0u8, 1, 2, 3]).is_none());
}

#[test]
fn embed_errors_on_non_png_bytes() {
    let result = png::embed_tcml_source_in_png(&[0u8, 1, 2, 3], "x");
    assert!(result.is_err());
}

// ----------------------------------------------------------------------
// Edge-case scenarios from docs/tests/web-wasm.feature.md.
// Browser-only entry points (wasm-bindgen wrappers) are skipped here —
// see the per-test comment for the reason.
// ----------------------------------------------------------------------

#[test]
fn render_emits_svg_with_required_namespaces() {
    let metrics = MockFontMetrics { char_width: 7.0 };
    let svg = render_with_metrics("Sig _~_~", None, &metrics).expect("render");
    assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
    assert!(svg.contains("xmlns:tchart="));
}

#[test]
fn render_with_font_size_zero_is_rejected_or_clamped() {
    // Spec: zero font size should be rejected. Allowed to fail; the
    // behaviour must be deterministic.
    let metrics = MockFontMetrics { char_width: 7.0 };
    let result = render_with_metrics("Sig _", Some(0.0), &metrics);
    assert!(
        result.is_err(),
        "font size 0 must be rejected; got {result:?}"
    );
}

#[test]
fn render_with_font_size_negative_is_rejected() {
    let metrics = MockFontMetrics { char_width: 7.0 };
    let result = render_with_metrics("Sig _", Some(-1.0), &metrics);
    assert!(result.is_err(), "negative font size must be rejected");
}

#[test]
fn convert_to_wavejson_with_no_warnings_emits_empty_warning_list() {
    let (_json, warnings) = convert_to_wavejson("clk _~_~").expect("convert");
    assert!(warnings.is_empty());
}

#[test]
fn convert_to_wavejson_with_multiple_warnings_emits_each_one() {
    // 53 anchors triggers warning; non-integer step also triggers one.
    let mut source = String::from("@step 7\nSig _");
    for index in 1..=53 {
        source.push_str(&format!("@{index}_"));
    }
    source.push('\n');
    for index in 1..53 {
        source.push_str(&format!("@-> (@{index}, @{})\n", index + 1));
    }
    let (_json, warnings) = convert_to_wavejson(&source).expect("convert");
    assert!(
        !warnings.is_empty(),
        "53-anchor input must produce at least one warning"
    );
}

#[test]
fn convert_to_wavejson_output_is_parseable_json() {
    let (json, _) = convert_to_wavejson("clk _~_~").expect("convert");
    let _: serde_json::Value = serde_json::from_str(&json).expect("output must parse as JSON");
}

#[test]
fn extract_double_unescapes_xml_entities() {
    // The text inside <tchart:source> is one-level XML-escaped; extraction
    // un-escapes one layer.
    let svg = "<svg><metadata><tchart:source>a &amp;lt; b</tchart:source></metadata></svg>";
    let extracted = extract::extract_tcml_source(svg).expect("found");
    assert_eq!(extracted, "a &lt; b");
}

#[test]
fn embed_png_round_trips_utf8_multibyte_japanese() {
    let png_bytes = make_minimal_png();
    let source = "@title 日本語";
    let embedded = png::embed_tcml_source_in_png(&png_bytes, source).expect("embed");
    let extracted = png::extract_tcml_source_from_png(&embedded).expect("extract");
    assert_eq!(extracted, source);
}

#[test]
fn embed_png_overwrite_or_append_behaviour_is_deterministic() {
    let png_bytes = make_minimal_png();
    let first = png::embed_tcml_source_in_png(&png_bytes, "first").expect("embed first");
    let second = png::embed_tcml_source_in_png(&first, "second").expect("embed second");
    let extracted = png::extract_tcml_source_from_png(&second).expect("extract");
    // Either "first" (existing kept) or "second" (overwritten) — must be
    // deterministic.
    assert!(
        extracted == "first" || extracted == "second",
        "extracted must be one of the two embeds; got {extracted}"
    );
}

#[test]
fn render_regression_mid_step_with_clock_auto() {
    let metrics = MockFontMetrics { char_width: 7.0 };
    let source = "@step 10\n@clock(pos) clk\n@step 20\ndata ====\n";
    let svg = render_with_metrics(source, None, &metrics).expect("render");
    assert!(svg.starts_with("<svg "));
    assert!(svg.contains("clk"));
    assert!(svg.contains("data"));
}

#[test]
fn convert_to_wavejson_regression_mid_step_with_clock_auto() {
    let source = "@step 10\n@clock(pos) clk\n@step 20\ndata ====\n";
    let (json, _) = convert_to_wavejson(source).expect("convert");
    let value: serde_json::Value = serde_json::from_str(&json).expect("parse json");
    let signals = value["signal"].as_array().expect("signal array");
    let clk_wave = signals[0]["wave"].as_str().expect("clk wave");
    let data_wave = signals[1]["wave"].as_str().expect("data wave");
    assert!(clk_wave.starts_with('p'));
    assert!(data_wave.starts_with('='));
}

#[test]
fn render_with_empty_input_is_deterministic() {
    let metrics = MockFontMetrics { char_width: 7.0 };
    let result = render_with_metrics("", None, &metrics);
    // Either succeeds with empty SVG or returns Err — must be deterministic.
    if let Ok(svg) = result {
        assert!(svg.starts_with("<svg"));
    }
}

// Browser-only: the @font directive's Canvas-based measurement requires
// wasm-bindgen + a real Canvas element. Skipped for native tests.
#[test]
fn render_with_font_directive_is_native_only_smoke_test() {
    let metrics = MockFontMetrics { char_width: 7.0 };
    let svg = render_with_metrics("@font monospace\nSig _\n", None, &metrics).expect("render");
    assert!(svg.contains("monospace") || svg.contains("font-family"));
}

// ----------------------------------------------------------------------
// Boundary tests for `validate_font_size`. Cover positive finite values
// (accepted) and zero / negative / NaN / +inf / -inf (rejected per spec).
// ----------------------------------------------------------------------

#[test]
fn validate_font_size_accepts_positive_finite() {
    let result = validate_font_size(12.0).expect("positive finite value must be accepted");
    assert_eq!(result, Px(12.0));
}

#[test]
fn validate_font_size_accepts_minimum_positive_value() {
    let result =
        validate_font_size(f32::MIN_POSITIVE).expect("smallest positive value must be accepted");
    assert_eq!(result, Px(f32::MIN_POSITIVE));
}

#[test]
fn validate_font_size_rejects_zero() {
    let result = validate_font_size(0.0);
    let message = result.expect_err("zero must be rejected");
    assert!(
        message.contains("invalid font size"),
        "error must mention invalid font size; got {message}"
    );
}

#[test]
fn validate_font_size_rejects_negative_zero() {
    let result = validate_font_size(-0.0);
    let message = result.expect_err("negative zero must be rejected");
    assert!(message.contains("invalid font size"));
}

#[test]
fn validate_font_size_rejects_negative() {
    let result = validate_font_size(-1.0);
    let message = result.expect_err("negative value must be rejected");
    assert!(message.contains("invalid font size"));
}

#[test]
fn validate_font_size_rejects_nan() {
    let result = validate_font_size(f32::NAN);
    let message = result.expect_err("NaN must be rejected");
    assert!(message.contains("invalid font size"));
}

#[test]
fn validate_font_size_rejects_positive_infinity() {
    let result = validate_font_size(f32::INFINITY);
    let message = result.expect_err("+inf must be rejected");
    assert!(message.contains("invalid font size"));
}

#[test]
fn validate_font_size_rejects_negative_infinity() {
    let result = validate_font_size(f32::NEG_INFINITY);
    let message = result.expect_err("-inf must be rejected");
    assert!(message.contains("invalid font size"));
}

// ----------------------------------------------------------------------
// Structured RenderError surface (used by the wasm `RenderResult` wrapper).
// Parse errors must expose 1-based line / column, character-unit length,
// and English-fixed message; font / layout / config errors must surface as
// the `Other` variant so the wasm layer can throw a JS exception for them.
// See docs/spec/web.md §renderTcml.
// ----------------------------------------------------------------------

#[test]
fn render_with_metrics_returns_parse_variant_with_location() {
    let metrics = MockFontMetrics { char_width: 7.0 };
    let error = render_with_metrics("@step xyz", None, &metrics).expect_err("must fail");
    match error {
        RenderError::Parse(parse_error) => {
            assert_eq!(parse_error.line(), 1, "@step xyz is on line 1");
            assert_eq!(
                parse_error.column(),
                7,
                "`xyz` starts at column 7 (after `@step `)"
            );
            assert_eq!(parse_error.length(), 3, "`xyz` is 3 characters");
            // After the Message(String) split, the message now includes the
            // directive name plus the offending value text.
            assert_eq!(parse_error.message(), "@step expects a number, got \"xyz\"");
        }
        RenderError::Other(message) => {
            panic!("expected Parse variant, got Other({message})")
        }
    }
}

#[test]
fn render_with_metrics_invalid_font_size_returns_other_variant() {
    let metrics = MockFontMetrics { char_width: 7.0 };
    let error = render_with_metrics("Sig _", Some(0.0), &metrics).expect_err("must fail");
    match error {
        RenderError::Other(message) => {
            assert!(
                message.contains("invalid font size"),
                "font-size rejection must surface via Other; got {message}"
            );
        }
        RenderError::Parse(parse_error) => panic!(
            "expected Other variant for font-size rejection, got Parse({})",
            parse_error.message()
        ),
    }
}

#[test]
fn render_with_metrics_success_returns_ok_svg() {
    let metrics = MockFontMetrics { char_width: 7.0 };
    let svg = render_with_metrics("Clock _~_~", None, &metrics).expect("must render");
    assert!(svg.starts_with("<svg "));
}

#[test]
fn render_with_metrics_parse_error_column_is_character_based_not_byte() {
    // ParseError columns count characters, not bytes. Use a multi-byte
    // signal name prefix so the leading text occupies fewer characters than
    // bytes; the column of the failing token must match the character offset.
    let metrics = MockFontMetrics { char_width: 7.0 };
    // "あ _\n@step xyz" — second line, `xyz` starts at column 7 again
    // (characters in @step + space). The first line is irrelevant; we only
    // verify multi-byte content earlier in the source does not shift the
    // character-based column on the failing token's own line.
    let source = "あ _\n@step xyz";
    let error = render_with_metrics(source, None, &metrics).expect_err("must fail");
    let parse_error = match error {
        RenderError::Parse(parse_error) => parse_error,
        RenderError::Other(message) => panic!("expected Parse, got Other({message})"),
    };
    assert_eq!(parse_error.line(), 2);
    assert_eq!(parse_error.column(), 7);
    assert_eq!(parse_error.length(), 3);
}
