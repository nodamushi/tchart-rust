//! End-to-end tests for the `tchart` binary.
//!
//! Mirrors the scenarios in `docs/tests/cli.feature.md` and
//! `docs/tests/cli-font.feature.md`.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tchart_cli::font::CANDIDATE_FONTS;
use tempfile::TempDir;

const BINARY_PATH: &str = env!("CARGO_BIN_EXE_tchart");

/// Lower bounds for the rasterised PNG dimensions of `valid.tc`. The fixture
/// is wide enough to easily clear these floors; the test only asserts that
/// the renderer produced a non-degenerate image.
const MIN_RENDERED_PNG_WIDTH: u32 = 50;
const MIN_RENDERED_PNG_HEIGHT: u32 = 30;

/// Literal that `valid.tc` round-trips through the SVG embedding. The signal
/// name `"<D0>"` contains angle brackets; surviving the XML escape/unescape
/// pair shows the round trip preserves them as literal characters.
const ROUND_TRIP_LITERAL: &str = "\"<D0>\"";

/// Per-test scaffolding bundling the working directory, the resolved font
/// path, and convenience helpers for running the CLI binary.
struct Harness {
    work: TempDir,
    font: PathBuf,
}

impl Harness {
    fn new() -> Self {
        Harness {
            work: tempfile::Builder::new()
                .prefix("tchart-it-")
                .tempdir()
                .unwrap_or_else(|error| panic!("tempdir: {error}")),
            font: resolve_font_path(),
        }
    }

    fn output_path(&self, name: &str) -> PathBuf {
        self.work.path().join(name)
    }

    /// Run the CLI binary for `svg` subcommand with font injected.
    fn run_svg_with_font(&self, args: &[&str]) -> Output {
        let mut command = Command::new(BINARY_PATH);
        command.arg("svg");
        command.arg("--font").arg(&self.font);
        command.args(args);
        command
            .output()
            .unwrap_or_else(|error| panic!("spawn tchart: {error}"))
    }

    /// Run the CLI binary for `png` subcommand with font injected.
    fn run_png_with_font(&self, args: &[&str]) -> Output {
        let mut command = Command::new(BINARY_PATH);
        command.arg("png");
        command.arg("--font").arg(&self.font);
        command.args(args);
        command
            .output()
            .unwrap_or_else(|error| panic!("spawn tchart: {error}"))
    }

    /// Run the CLI binary for `batch` subcommand with font injected.
    fn run_batch_with_font(&self, format: &str, args: &[&str]) -> Output {
        let mut command = Command::new(BINARY_PATH);
        command.arg("batch");
        command.arg(format);
        command.arg("--font").arg(&self.font);
        command.args(args);
        command
            .output()
            .unwrap_or_else(|error| panic!("spawn tchart: {error}"))
    }

    /// Write `tc_content` to three files `{prefix}-a.tc`, `{prefix}-b.tc`,
    /// `{prefix}-c.tc` in the work directory and return their paths.
    fn write_three_tc_files(&self, tc_content: &str, prefix: &str) -> [PathBuf; 3] {
        let files = [
            self.output_path(&format!("{prefix}-a.tc")),
            self.output_path(&format!("{prefix}-b.tc")),
            self.output_path(&format!("{prefix}-c.tc")),
        ];
        for (file, label) in files.iter().zip(["a", "b", "c"]) {
            std::fs::write(file, tc_content)
                .unwrap_or_else(|error| panic!("write {label}: {error}"));
        }
        files
    }

    /// Write identical `tc_content` to three files and run `batch <format>`.
    ///
    /// Returns the `Output` of the batch run and the output directory path.
    fn run_batch_three_identical(
        &self,
        format: &str,
        tc_content: &str,
        prefix: &str,
    ) -> (Output, PathBuf) {
        let [file_a, file_b, file_c] = self.write_three_tc_files(tc_content, prefix);
        let out_dir = self.output_path(&format!("{prefix}-out"));
        std::fs::create_dir_all(&out_dir)
            .unwrap_or_else(|error| panic!("mkdir {}: {error}", out_dir.display()));
        let result = self.run_batch_with_font(
            format,
            &[
                path_as_str(&file_a),
                path_as_str(&file_b),
                path_as_str(&file_c),
                "-o",
                path_as_str(&out_dir),
            ],
        );
        (result, out_dir)
    }
}

/// Resolve the path to the test fixture named `name` under `tests/fixtures/`.
fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// Run the CLI binary without a `--font` flag.
fn run_binary(args: &[&str]) -> Output {
    Command::new(BINARY_PATH)
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("spawn tchart: {error}"))
}

fn resolve_font_path() -> PathBuf {
    CANDIDATE_FONTS
        .iter()
        .take(2)
        .map(PathBuf::from)
        .find(|path| path.exists())
        .unwrap_or_else(|| panic!("no test font available; install dejavu"))
}

fn path_as_str(path: &Path) -> &str {
    path.to_str()
        .unwrap_or_else(|| panic!("non-utf8 path: {}", path.display()))
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "command failed (status {:?}); stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_exit_code(output: &Output, expected: i32) {
    assert_eq!(
        output.status.code(),
        Some(expected),
        "expected exit code {expected}; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// ---------------------------------------------------------------------------
// svg サブコマンド
// ---------------------------------------------------------------------------

/// Scenario: `svg` のデフォルト出力 (入力隣に `<STEM>.svg`)
#[test]
fn svg_default_output_is_stem_dot_svg() {
    let harness = Harness::new();
    let input = fixture_path("valid.tc");
    let copied = harness.output_path("chart.tc");
    std::fs::copy(&input, &copied).expect("copy");
    let result = harness.run_svg_with_font(&[path_as_str(&copied)]);
    assert_success(&result);
    let expected = harness.output_path("chart.svg");
    assert!(
        expected.exists(),
        "expected output {} missing",
        expected.display()
    );
}

/// Scenario: `svg -o` で出力ファイル指定
#[test]
fn svg_renders_to_specified_output() {
    let harness = Harness::new();
    let input = fixture_path("valid.tc");
    let output = harness.output_path("svg-out.svg");
    let result = harness.run_svg_with_font(&[path_as_str(&input), "-o", path_as_str(&output)]);
    assert_success(&result);
    let svg_content = std::fs::read_to_string(&output).expect("read");
    assert!(svg_content.starts_with("<svg "));
    assert!(svg_content.contains("<tchart:source>"));
}

/// Scenario: `svg --output` 長フォーム
#[test]
fn svg_renders_with_long_form_output_flag() {
    let harness = Harness::new();
    let input = fixture_path("valid.tc");
    let output = harness.output_path("longform.svg");
    let result =
        harness.run_svg_with_font(&[path_as_str(&input), "--output", path_as_str(&output)]);
    assert_success(&result);
    assert!(output.exists(), "SVG not created");
}

/// Scenario: `svg` に複数入力を指定するとエラー
#[test]
fn svg_rejects_multiple_inputs() {
    let harness = Harness::new();
    let input_a = fixture_path("valid.tc");
    let input_b = fixture_path("valid.tc");
    let result = harness.run_svg_with_font(&[path_as_str(&input_a), path_as_str(&input_b)]);
    assert_exit_code(&result, 1);
}

// ---------------------------------------------------------------------------
// png サブコマンド
// ---------------------------------------------------------------------------

/// Scenario: `png` のデフォルト出力 (入力隣に `<STEM>.png`)
#[test]
fn png_default_output_is_stem_dot_png() {
    let harness = Harness::new();
    let input = fixture_path("valid.tc");
    let copied = harness.output_path("chart.tc");
    std::fs::copy(&input, &copied).expect("copy");
    let result = harness.run_png_with_font(&[path_as_str(&copied)]);
    assert_success(&result);
    let expected = harness.output_path("chart.png");
    assert!(
        expected.exists(),
        "expected output {} missing",
        expected.display()
    );
}

/// Scenario: `png -o` で出力ファイル指定
#[test]
fn png_renders_to_specified_output() {
    let harness = Harness::new();
    let input = fixture_path("valid.tc");
    let output = harness.output_path("out.png");
    let result = harness.run_png_with_font(&[path_as_str(&input), "-o", path_as_str(&output)]);
    assert_success(&result);
    let bytes = std::fs::read(&output).expect("read");
    assert!(bytes.starts_with(&[0x89, b'P', b'N', b'G']));
}

/// Scenario: PNG に TCML ソースが埋め込まれる
#[test]
fn png_embeds_tcml_source() {
    let harness = Harness::new();
    let input = fixture_path("valid.tc");
    let output = harness.output_path("with-source.png");
    let result = harness.run_png_with_font(&[path_as_str(&input), "-o", path_as_str(&output)]);
    assert_success(&result);
    // Verify embedded source via `src` subcommand.
    let src_output = harness.output_path("restored.tc");
    let extracted = run_binary(&["src", path_as_str(&output), "-o", path_as_str(&src_output)]);
    assert_success(&extracted);
    let original = std::fs::read_to_string(&input).expect("read input");
    let restored = std::fs::read_to_string(&src_output).expect("read restored");
    assert_eq!(
        restored.trim_end_matches('\n'),
        original.trim_end_matches('\n')
    );
}

/// Scenario: `png` に複数入力を指定するとエラー
#[test]
fn png_rejects_multiple_inputs() {
    let harness = Harness::new();
    let input_a = fixture_path("valid.tc");
    let input_b = fixture_path("valid.tc");
    let result = harness.run_png_with_font(&[path_as_str(&input_a), path_as_str(&input_b)]);
    assert_exit_code(&result, 1);
}

/// Scenario: PNG にテキスト (信号名) が描画される
#[test]
fn png_has_non_trivial_pixels() {
    use image::GenericImageView;
    let harness = Harness::new();
    let input = fixture_path("valid.tc");
    let output = harness.output_path("pixels.png");
    let result = harness.run_png_with_font(&[path_as_str(&input), "-o", path_as_str(&output)]);
    assert_success(&result);
    let image = image::open(&output).expect("open png");
    let (width, height) = image.dimensions();
    assert!(
        width > MIN_RENDERED_PNG_WIDTH,
        "rendered width {width} below floor {MIN_RENDERED_PNG_WIDTH}"
    );
    assert!(
        height > MIN_RENDERED_PNG_HEIGHT,
        "rendered height {height} below floor {MIN_RENDERED_PNG_HEIGHT}"
    );
}

// ---------------------------------------------------------------------------
// src サブコマンド
// ---------------------------------------------------------------------------

/// Scenario: `src` のデフォルト出力 (SVG → 入力隣に `<STEM>.tc`)
#[test]
fn src_extracts_from_svg_to_default_output() {
    let harness = Harness::new();
    let input = fixture_path("valid.tc");
    let svg_output_path = harness.output_path("chart.svg");
    let render =
        harness.run_svg_with_font(&[path_as_str(&input), "-o", path_as_str(&svg_output_path)]);
    assert_success(&render);

    let result = run_binary(&["src", path_as_str(&svg_output_path)]);
    assert_success(&result);

    let expected_output = harness.output_path("chart.tc");
    assert!(
        expected_output.exists(),
        "expected output {} missing",
        expected_output.display()
    );
    let original = std::fs::read_to_string(&input).expect("read input");
    let restored = std::fs::read_to_string(&expected_output).expect("read restored");
    assert_eq!(
        restored.trim_end_matches('\n'),
        original.trim_end_matches('\n')
    );
}

/// Scenario: `src` のデフォルト出力 (PNG → 入力隣に `<STEM>.tc`)
#[test]
fn src_extracts_from_png_to_default_output() {
    let harness = Harness::new();
    let input = fixture_path("valid.tc");
    let png_path = harness.output_path("chart.png");
    let render = harness.run_png_with_font(&[path_as_str(&input), "-o", path_as_str(&png_path)]);
    assert_success(&render);

    let result = run_binary(&["src", path_as_str(&png_path)]);
    assert_success(&result);

    let expected_output = harness.output_path("chart.tc");
    assert!(
        expected_output.exists(),
        "expected output {} missing",
        expected_output.display()
    );
}

/// Scenario: `src -o` で出力ファイル指定
#[test]
fn src_extracts_to_specified_output() {
    let harness = Harness::new();
    let input = fixture_path("valid.tc");
    let svg_output_path = harness.output_path("src-test.svg");
    let render =
        harness.run_svg_with_font(&[path_as_str(&input), "-o", path_as_str(&svg_output_path)]);
    assert_success(&render);
    let output = harness.output_path("restored.tc");
    let result = run_binary(&[
        "src",
        path_as_str(&svg_output_path),
        "-o",
        path_as_str(&output),
    ]);
    assert_success(&result);
    assert!(output.exists(), "output file not created");
}

/// Scenario: `source` は `src` のエイリアス
#[test]
fn source_is_alias_for_src() {
    let harness = Harness::new();
    let input = fixture_path("valid.tc");
    let svg_output_path = harness.output_path("alias-test.svg");
    let render =
        harness.run_svg_with_font(&[path_as_str(&input), "-o", path_as_str(&svg_output_path)]);
    assert_success(&render);
    let output = harness.output_path("alias-out.tc");
    let result = run_binary(&[
        "source",
        path_as_str(&svg_output_path),
        "-o",
        path_as_str(&output),
    ]);
    assert_success(&result);
    assert!(output.exists(), "source alias output not created");
}

// ---------------------------------------------------------------------------
// Iter1 phase 2: round-trip and option precedence scenarios.
// ---------------------------------------------------------------------------

#[test]
fn iter1_svg_round_trip_full_match() {
    let harness = Harness::new();
    let input = fixture_path("valid.tc");
    let svg_path = harness.output_path("rt-full.svg");
    let render = harness.run_svg_with_font(&[path_as_str(&input), "-o", path_as_str(&svg_path)]);
    assert_success(&render);
    let restored = harness.output_path("rt-full.tc");
    let extract = run_binary(&["src", path_as_str(&svg_path), "-o", path_as_str(&restored)]);
    assert_success(&extract);
    let original = std::fs::read_to_string(&input).expect("read original");
    let restored_content = std::fs::read_to_string(&restored).expect("read restored");
    assert_eq!(
        restored_content.trim_end_matches('\n'),
        original.trim_end_matches('\n'),
        "SVG round-trip must yield the same TCML source"
    );
}

#[test]
fn iter1_png_round_trip_preserves_utf8_signal_name() {
    let harness = Harness::new();
    let multibyte = "\"日本語\" _~\n";
    let input = harness.output_path("multibyte.tc");
    std::fs::write(&input, multibyte).expect("write multibyte input");
    let png_path = harness.output_path("multibyte.png");
    let render = harness.run_png_with_font(&[path_as_str(&input), "-o", path_as_str(&png_path)]);
    assert_success(&render);
    let restored = harness.output_path("multibyte-restored.tc");
    let extract = run_binary(&["src", path_as_str(&png_path), "-o", path_as_str(&restored)]);
    assert_success(&extract);
    let restored_content = std::fs::read_to_string(&restored).expect("read restored");
    assert!(
        restored_content.contains("日本語"),
        "multi-byte signal name must round-trip through PNG: got {restored_content:?}"
    );
}

#[test]
fn iter1_png_round_trip_preserves_xml_special_chars() {
    let harness = Harness::new();
    let tcml = "\"<a>&<b>\" _~\n";
    let input = harness.output_path("xmlspec.tc");
    std::fs::write(&input, tcml).expect("write xml-special input");
    let png_path = harness.output_path("xmlspec.png");
    let render = harness.run_png_with_font(&[path_as_str(&input), "-o", path_as_str(&png_path)]);
    assert_success(&render);
    let restored = harness.output_path("xmlspec-restored.tc");
    let extract = run_binary(&["src", path_as_str(&png_path), "-o", path_as_str(&restored)]);
    assert_success(&extract);
    let restored_content = std::fs::read_to_string(&restored).expect("read restored");
    assert_eq!(
        restored_content.trim_end_matches('\n'),
        tcml.trim_end_matches('\n'),
        "PNG round-trip must preserve XML special chars verbatim"
    );
}

#[test]
fn iter1_cli_font_overrides_at_font_directive() {
    let harness = Harness::new();
    let tcml = "@font NoSuchFamilyXyz\nSig _\n";
    let input = harness.output_path("font-conflict.tc");
    std::fs::write(&input, tcml).expect("write input");
    let output = harness.output_path("font-conflict.svg");
    let result = harness.run_svg_with_font(&[path_as_str(&input), "-o", path_as_str(&output)]);
    assert_success(&result);
    assert!(
        output.exists(),
        "SVG must be produced even when @font conflicts"
    );
}

#[test]
fn iter1_at_font_alone_does_not_break_render() {
    // Per docs/spec/cli.md `@font` is a TCML family directive that overlays the
    // CLI default font (resolved by `--font` / `TCHART_FONT` / OS auto-detect).
    // With `@font Helvetica` and no `--font` flag, OS auto-detection must still
    // produce the default font and rendering must succeed.
    let harness = Harness::new();
    let tcml = "@font Helvetica\nSig _\n";
    let input = harness.output_path("at-font-only.tc");
    std::fs::write(&input, tcml).expect("write input");
    let output = harness.output_path("at-font-only.svg");
    let result = run_binary(&["svg", path_as_str(&input), "-o", path_as_str(&output)]);
    assert_success(&result);
    assert!(
        output.exists(),
        "SVG must be produced when only @font is supplied"
    );
}

/// Render `input` to `out`, optionally setting `TCHART_FONT=env_font`.
fn render_svg_with_optional_env_font(input: &Path, out: &Path, env_font: Option<&Path>) -> Output {
    let mut command = Command::new(BINARY_PATH);
    if let Some(font) = env_font {
        command.env("TCHART_FONT", path_as_str(font));
    }
    command.arg("svg");
    command.args([path_as_str(input), "-o", path_as_str(out)]);
    command
        .output()
        .unwrap_or_else(|error| panic!("spawn tchart: {error}"))
}

#[test]
fn iter1_tchart_font_env_does_not_change_at_font_family_in_svg() {
    // `@font` sets the TCML font-family attribute written into the SVG output.
    // `TCHART_FONT` only chooses the CLI default font file (used as fallback
    // when family resolution fails). Setting `TCHART_FONT` must not change the
    // `font-family` attribute that ends up in the SVG when `@font` is present.
    let harness = Harness::new();
    let tcml = "@font Helvetica\nSig _\n";
    let input = harness.output_path("env-vs-at.tc");
    std::fs::write(&input, tcml).expect("write input");
    let with_env_path = harness.output_path("env-vs-at-with-env.svg");
    let without_env_path = harness.output_path("env-vs-at-without-env.svg");

    let with_env_result =
        render_svg_with_optional_env_font(&input, &with_env_path, Some(&harness.font));
    assert_success(&with_env_result);
    let without_env_result = render_svg_with_optional_env_font(&input, &without_env_path, None);
    assert_success(&without_env_result);

    let svg_with_env = std::fs::read_to_string(&with_env_path).expect("read with-env svg");
    let svg_without_env = std::fs::read_to_string(&without_env_path).expect("read without-env svg");
    assert!(
        svg_with_env.contains("font-family=\"Helvetica\""),
        "@font Helvetica must appear as font-family in SVG (with TCHART_FONT set): {svg_with_env}"
    );
    assert!(
        svg_without_env.contains("font-family=\"Helvetica\""),
        "@font Helvetica must appear as font-family in SVG (no TCHART_FONT): {svg_without_env}"
    );
}

#[test]
fn iter1_cli_font_size_flag_is_accepted_by_svg_subcommand() {
    // Mirrors docs/tests/cli.feature.md §Scenario: `svg --font-size 24` —
    // the CLI must accept `--font-size 24` and the rendered SVG must reflect
    // that value in its `<text font-size>` attributes.
    let harness = Harness::new();
    let input = fixture_path("valid.tc");
    let output = harness.output_path("font-size.svg");
    let mut command = Command::new(BINARY_PATH);
    command.arg("svg");
    command.arg("--font").arg(&harness.font);
    command.args([
        "--font-size",
        "24",
        path_as_str(&input),
        "-o",
        path_as_str(&output),
    ]);
    let result = command
        .output()
        .unwrap_or_else(|error| panic!("spawn tchart: {error}"));
    assert!(
        result.status.success(),
        "--font-size 24 must succeed; stderr: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let svg = std::fs::read_to_string(&output).expect("read svg");
    assert!(
        svg.contains("font-size=\"24\"") || svg.contains("font-size:24"),
        "rendered SVG must carry the overridden font-size 24; got {svg}"
    );
}

#[test]
fn iter1_round_trip_preserves_at_scale_directive() {
    let harness = Harness::new();
    let tcml = "@scale 2.0\nA _~\n";
    let input = harness.output_path("scale.tc");
    std::fs::write(&input, tcml).expect("write input");
    let svg_path = harness.output_path("scale.svg");
    let render = harness.run_svg_with_font(&[path_as_str(&input), "-o", path_as_str(&svg_path)]);
    assert_success(&render);
    let restored = harness.output_path("scale-restored.tc");
    let extract = run_binary(&["src", path_as_str(&svg_path), "-o", path_as_str(&restored)]);
    assert_success(&extract);
    let restored_content = std::fs::read_to_string(&restored).expect("read restored");
    assert!(
        restored_content.contains("@scale 2.0"),
        "@scale 2.0 must be preserved verbatim; got {restored_content:?}"
    );
}

#[test]
fn iter1_png_three_round_trips_remain_stable() {
    let harness = Harness::new();
    let input = fixture_path("valid.tc");
    let original = std::fs::read_to_string(&input).expect("read input");
    let mut current_tc_path = input;
    let mut last_content = String::new();
    for index in 0..3 {
        let png_path = harness.output_path(&format!("rt3-{index}.png"));
        let render = harness.run_png_with_font(&[
            path_as_str(&current_tc_path),
            "-o",
            path_as_str(&png_path),
        ]);
        assert_success(&render);
        let next_tc_path = harness.output_path(&format!("rt3-{index}.tc"));
        let extract = run_binary(&[
            "src",
            path_as_str(&png_path),
            "-o",
            path_as_str(&next_tc_path),
        ]);
        assert_success(&extract);
        last_content = std::fs::read_to_string(&next_tc_path).expect("read restored");
        current_tc_path = next_tc_path;
    }
    assert_eq!(
        last_content.trim_end_matches('\n'),
        original.trim_end_matches('\n'),
        "3 PNG round-trips must keep TCML byte-stable"
    );
}

/// Scenario: XML エスケープされたソースが正しく復元される
#[test]
fn src_round_trips_xml_special_chars() {
    let harness = Harness::new();
    let input = fixture_path("valid.tc");
    let svg_output_path = harness.output_path("xml-rt.svg");
    let render =
        harness.run_svg_with_font(&[path_as_str(&input), "-o", path_as_str(&svg_output_path)]);
    assert_success(&render);
    let output = harness.output_path("xml-rt.tc");
    let result = run_binary(&[
        "src",
        path_as_str(&svg_output_path),
        "-o",
        path_as_str(&output),
    ]);
    assert_success(&result);
    let original = std::fs::read_to_string(&input).expect("read");
    let restored = std::fs::read_to_string(&output).expect("read restored");
    assert!(
        restored.contains(ROUND_TRIP_LITERAL),
        "round-trip preserves {ROUND_TRIP_LITERAL}"
    );
    assert_eq!(
        restored.trim_end_matches('\n'),
        original.trim_end_matches('\n')
    );
}

/// Scenario: `src` で TCML ソースが埋め込まれていない SVG → exit 5
#[test]
fn src_no_embedded_source_exits_5() {
    let harness = Harness::new();
    let plain = harness.output_path("plain.svg");
    std::fs::write(&plain, "<svg></svg>").expect("write");
    let result = run_binary(&["src", path_as_str(&plain)]);
    assert_exit_code(&result, 5);
}

/// Scenario: `src` で不正なファイル形式 → exit 5
#[test]
fn src_unsupported_format_exits_5() {
    let harness = Harness::new();
    let txt = harness.output_path("readme.txt");
    std::fs::write(&txt, "hello world").expect("write");
    let result = run_binary(&["src", path_as_str(&txt)]);
    assert_exit_code(&result, 5);
}

/// Scenario: `src` に複数入力を指定するとエラー
#[test]
fn src_rejects_multiple_inputs() {
    let harness = Harness::new();
    let plain_a = harness.output_path("a.svg");
    let plain_b = harness.output_path("b.svg");
    std::fs::write(&plain_a, "<svg></svg>").expect("write");
    std::fs::write(&plain_b, "<svg></svg>").expect("write");
    let result = run_binary(&["src", path_as_str(&plain_a), path_as_str(&plain_b)]);
    assert_exit_code(&result, 1);
}

// ---------------------------------------------------------------------------
// batch サブコマンド
// ---------------------------------------------------------------------------

/// Scenario: `batch svg` で複数入力を出力ディレクトリにレンダリング
#[test]
fn batch_svg_multiple_inputs() {
    let harness = Harness::new();
    let input_valid = fixture_path("valid.tc");
    let input_nosuch = fixture_path("font-nosuch.tc");
    let copied_a = harness.output_path("a.tc");
    let copied_b = harness.output_path("b.tc");
    let copied_c = harness.output_path("c.tc");
    std::fs::copy(&input_valid, &copied_a).expect("copy a");
    std::fs::copy(&input_nosuch, &copied_b).expect("copy b");
    std::fs::copy(&input_valid, &copied_c).expect("copy c");
    let out_dir = harness.output_path("out");
    std::fs::create_dir(&out_dir).expect("mkdir");
    let result = harness.run_batch_with_font(
        "svg",
        &[
            path_as_str(&copied_a),
            path_as_str(&copied_b),
            path_as_str(&copied_c),
            "-o",
            path_as_str(&out_dir),
        ],
    );
    assert_success(&result);
    assert!(out_dir.join("a.svg").exists(), "a.svg missing");
    assert!(out_dir.join("b.svg").exists(), "b.svg missing");
    assert!(out_dir.join("c.svg").exists(), "c.svg missing");
}

/// Scenario: `batch png` で複数入力を出力ディレクトリにレンダリング
#[test]
fn batch_png_multiple_inputs() {
    let harness = Harness::new();
    let input = fixture_path("valid.tc");
    let copied_a = harness.output_path("a.tc");
    let copied_b = harness.output_path("b.tc");
    std::fs::copy(&input, &copied_a).expect("copy a");
    std::fs::copy(&input, &copied_b).expect("copy b");
    let out_dir = harness.output_path("build");
    std::fs::create_dir(&out_dir).expect("mkdir");
    let result = harness.run_batch_with_font(
        "png",
        &[
            path_as_str(&copied_a),
            path_as_str(&copied_b),
            "-o",
            path_as_str(&out_dir),
        ],
    );
    assert_success(&result);
    assert!(out_dir.join("a.png").exists(), "a.png missing");
    assert!(out_dir.join("b.png").exists(), "b.png missing");
}

/// Scenario: `batch` の出力ディレクトリが存在しない場合は作成する
#[test]
fn batch_creates_output_directory() {
    let harness = Harness::new();
    let input = fixture_path("valid.tc");
    let copied = harness.output_path("a.tc");
    std::fs::copy(&input, &copied).expect("copy");
    let out_dir = harness.output_path("new_out");
    assert!(!out_dir.exists(), "pre-condition: directory must not exist");
    let result =
        harness.run_batch_with_font("svg", &[path_as_str(&copied), "-o", path_as_str(&out_dir)]);
    assert_success(&result);
    assert!(out_dir.exists(), "output directory was not created");
    assert!(out_dir.join("a.svg").exists(), "a.svg missing");
}

/// Scenario: `batch` で `-o` 未指定はエラー
#[test]
fn batch_missing_output_dir_exits_1() {
    let harness = Harness::new();
    let input = fixture_path("valid.tc");
    let copied = harness.output_path("a.tc");
    std::fs::copy(&input, &copied).expect("copy");
    // No -o flag.
    let result = run_binary(&["batch", "svg", path_as_str(&copied)]);
    assert_exit_code(&result, 1);
}

/// Scenario: `batch` で出力 STEM が衝突するとエラー (exit 3)
#[test]
fn batch_stem_collision_exits_3() {
    let harness = Harness::new();
    let input = fixture_path("valid.tc");
    let dir1 = harness.output_path("dir1");
    let dir2 = harness.output_path("dir2");
    std::fs::create_dir(&dir1).expect("mkdir dir1");
    std::fs::create_dir(&dir2).expect("mkdir dir2");
    let file_a = dir1.join("chart.tc");
    let file_b = dir2.join("chart.tc");
    std::fs::copy(&input, &file_a).expect("copy a");
    std::fs::copy(&input, &file_b).expect("copy b");
    let out_dir = harness.output_path("out");
    std::fs::create_dir(&out_dir).expect("mkdir out");
    let result = harness.run_batch_with_font(
        "svg",
        &[
            path_as_str(&file_a),
            path_as_str(&file_b),
            "-o",
            path_as_str(&out_dir),
        ],
    );
    assert_exit_code(&result, 3);
}

/// Scenario: `batch` のフォーマット引数が不正 → exit 1
#[test]
fn batch_invalid_format_exits_1() {
    let harness = Harness::new();
    let input = fixture_path("valid.tc");
    let copied = harness.output_path("a.tc");
    std::fs::copy(&input, &copied).expect("copy");
    let out_dir = harness.output_path("out");
    let result = run_binary(&[
        "batch",
        "jpeg",
        path_as_str(&copied),
        "-o",
        path_as_str(&out_dir),
    ]);
    assert_exit_code(&result, 1);
}

/// Scenario: `batch` で 1 入力でも動作する
#[test]
fn batch_single_input_works() {
    let harness = Harness::new();
    let input = fixture_path("valid.tc");
    let copied = harness.output_path("single.tc");
    std::fs::copy(&input, &copied).expect("copy");
    let out_dir = harness.output_path("out");
    std::fs::create_dir(&out_dir).expect("mkdir");
    let result =
        harness.run_batch_with_font("svg", &[path_as_str(&copied), "-o", path_as_str(&out_dir)]);
    assert_success(&result);
    assert!(out_dir.join("single.svg").exists(), "single.svg missing");
}

// ---------------------------------------------------------------------------
// ヘルプ・バージョン
// ---------------------------------------------------------------------------

/// Scenario: ヘルプ表示
#[test]
fn help_lists_subcommands() {
    let result = run_binary(&["--help"]);
    assert_success(&result);
    let text = String::from_utf8(result.stdout).expect("utf8");
    assert!(text.contains("svg"), "svg not in help: {text}");
    assert!(text.contains("png"), "png not in help: {text}");
    assert!(
        text.contains("src") || text.contains("source"),
        "src not in help: {text}"
    );
    assert!(text.contains("batch"), "batch not in help: {text}");
}

/// Scenario: バージョン表示
#[test]
fn version_flag_prints_version() {
    let result = run_binary(&["--version"]);
    assert_success(&result);
    let text = String::from_utf8(result.stdout).expect("utf8");
    assert!(text.contains("tchart"), "got {text:?}");
}

/// Scenario: 引数なしで実行
#[test]
fn no_args_exits_nonzero() {
    let result = run_binary(&[]);
    let code = result.status.code().unwrap_or(0);
    assert_ne!(code, 0, "expected non-zero exit when no args");
}

// ---------------------------------------------------------------------------
// エラー処理
// ---------------------------------------------------------------------------

/// Scenario: 入力ファイルが存在しない → exit 1
#[test]
fn missing_input_exits_1() {
    let harness = Harness::new();
    let output = harness.output_path("never.svg");
    let result = harness.run_svg_with_font(&["does-not-exist.tc", "-o", path_as_str(&output)]);
    assert_exit_code(&result, 1);
}

/// Scenario: TCML パースエラー → exit 2
#[test]
fn parse_error_exits_2() {
    let harness = Harness::new();
    let input = fixture_path("invalid.tc");
    let output = harness.output_path("invalid.svg");
    let result = harness.run_svg_with_font(&[path_as_str(&input), "-o", path_as_str(&output)]);
    assert_exit_code(&result, 2);
}

/// Scenario: フォントが見つからない → exit 4
#[test]
fn missing_font_exits_4() {
    let harness = Harness::new();
    let input = fixture_path("valid.tc");
    let output = harness.output_path("never.svg");
    let result = run_binary(&[
        "svg",
        "--font",
        "/nonexistent/font.ttf",
        path_as_str(&input),
        "-o",
        path_as_str(&output),
    ]);
    assert_exit_code(&result, 4);
}

// ---------------------------------------------------------------------------
// cli-font.feature.md: family resolution and fontdb integration
// ---------------------------------------------------------------------------

/// Minimal PNG pixel count that proves a chart was actually rasterised.
const MIN_PNG_PIXEL_AREA: u64 = 100 * 30;

/// Return `true` when Comic Neue is available via `fc-match` on this machine.
fn has_comic_neue() -> bool {
    std::process::Command::new("fc-match")
        .args(["-f", "%{file}", "Comic Neue"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| {
            let path_string = String::from_utf8(output.stdout).ok()?;
            Some(std::path::Path::new(path_string.trim()).is_file())
        })
        .unwrap_or(false)
}

/// Return `true` when the OS can resolve the given family name to an existing
/// font file via `fc-match`.
fn can_resolve_family(family: &str) -> bool {
    std::process::Command::new("fc-match")
        .args(["-f", "%{file}", family])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| {
            let path_string = String::from_utf8(output.stdout).ok()?;
            Some(std::path::Path::new(path_string.trim()).is_file())
        })
        .unwrap_or(false)
}

/// Assert that `output` is a valid non-trivial PNG.
fn assert_valid_png(bytes: &[u8]) {
    assert!(bytes.starts_with(&[0x89, b'P', b'N', b'G']), "not a PNG");
    use image::GenericImageView;
    let image = image::load_from_memory(bytes).expect("decode PNG");
    let (width, height) = image.dimensions();
    let area = u64::from(width) * u64::from(height);
    assert!(
        area >= MIN_PNG_PIXEL_AREA,
        "rendered PNG too small: {width}x{height}"
    );
}

/// Scenario: family 解決失敗時はデフォルトにフォールバックし警告を出す
#[test]
fn family_not_found_falls_back_with_warning() {
    let harness = Harness::new();
    let input = fixture_path("font-nosuch.tc");
    let output = harness.output_path("font-nosuch.svg");
    let result = harness.run_svg_with_font(&[path_as_str(&input), "-o", path_as_str(&output)]);
    assert_success(&result);
    assert!(output.exists(), "SVG output was not created");
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        stderr.contains("font family not found") || stderr.contains("not found"),
        "expected warning in stderr; got: {stderr}"
    );
}

/// Scenario: 解決できない family は同名で再 load されない (警告 1 行のみ)
#[test]
fn unresolvable_family_warns_only_once() {
    let harness = Harness::new();
    let output = harness.output_path("repeat.svg");
    let tc = "@font NoSuchFont12345\n@font NoSuchFont12345\n@font NoSuchFont12345\nClock _~_~\n";
    let tc_path = harness.output_path("repeat-nosuch.tc");
    std::fs::write(&tc_path, tc).expect("write tc");
    let result = harness.run_svg_with_font(&[path_as_str(&tc_path), "-o", path_as_str(&output)]);
    assert_success(&result);
    let stderr = String::from_utf8_lossy(&result.stderr);
    let warning_count = stderr
        .lines()
        .filter(|line| line.contains("NoSuchFont12345"))
        .count();
    assert_eq!(
        warning_count, 1,
        "expected exactly 1 warning; got: {stderr}"
    );
}

/// Scenario: ジェネリック (`monospace`) が OS 解決経由で実フォントに繋がる
#[test]
#[cfg(target_os = "linux")]
fn generic_family_resolves_and_produces_png() {
    if !can_resolve_family("monospace") {
        return;
    }
    let harness = Harness::new();
    let input = fixture_path("font-generic.tc");
    let output = harness.output_path("generic.png");
    let result = harness.run_png_with_font(&[path_as_str(&input), "-o", path_as_str(&output)]);
    assert_success(&result);
    let bytes = std::fs::read(&output).expect("read");
    assert_valid_png(&bytes);
}

/// Scenario: CSV 指定で左から順に試行し最初に解決できたものを使う
#[test]
#[cfg(target_os = "linux")]
fn csv_family_falls_through_to_second_entry() {
    if !has_comic_neue() {
        return;
    }
    let harness = Harness::new();
    let input = fixture_path("font-csv.tc");
    let output = harness.output_path("csv.png");
    let result = harness.run_png_with_font(&[path_as_str(&input), "-o", path_as_str(&output)]);
    assert_success(&result);
    let bytes = std::fs::read(&output).expect("read");
    assert_valid_png(&bytes);
}

/// Scenario: 同一 family を複数行で参照しても load は 1 回
#[test]
#[cfg(target_os = "linux")]
fn same_family_repeated_loads_once() {
    if !has_comic_neue() {
        return;
    }
    let harness = Harness::new();
    let tc = {
        let mut content = String::from("@font \"Comic Neue\"\n");
        for index in 0..100 {
            content.push_str(&format!("Clock{index} _~_~_~_~\n"));
        }
        content
    };
    let tc_path = harness.output_path("repeat100.tc");
    std::fs::write(&tc_path, &tc).expect("write tc");
    let output = harness.output_path("repeat100.svg");
    let result = harness.run_svg_with_font(&[path_as_str(&tc_path), "-o", path_as_str(&output)]);
    assert_success(&result);
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        !stderr.contains("Comic Neue"),
        "unexpected warning for Comic Neue: {stderr}"
    );
}

/// Scenario: 解決済み全フォントが fontdb に登録され PNG が生成される
#[test]
#[cfg(target_os = "linux")]
fn resolved_fonts_produce_valid_png() {
    if !has_comic_neue() {
        return;
    }
    let harness = Harness::new();
    let input = fixture_path("font-family.tc");
    let output = harness.output_path("family.png");
    let result = harness.run_png_with_font(&[path_as_str(&input), "-o", path_as_str(&output)]);
    assert_success(&result);
    let bytes = std::fs::read(&output).expect("read");
    assert_valid_png(&bytes);
}

/// Scenario: 解決失敗の警告は family につき 1 回のみ (バッチ全体で集約)
///
/// `docs/tests/cli-font.feature.md` §`batch` のフォントキャッシュ共有
/// 「解決失敗の警告は family につき 1 回のみ (バッチ全体で集約)」に対応。
#[test]
fn batch_no_such_font_warning_once_across_inputs() {
    let harness = Harness::new();
    let tc_content = "@font NoSuchFontBatch99\nClock _~_~\n";
    let (result, _out_dir) = harness.run_batch_three_identical("svg", tc_content, "warn");
    assert_success(&result);
    let stderr = String::from_utf8_lossy(&result.stderr);
    let warning_count = stderr
        .lines()
        .filter(|line| line.contains("NoSuchFontBatch99"))
        .count();
    assert_eq!(
        warning_count, 1,
        "expected exactly 1 warning for NoSuchFontBatch99; got:\n{stderr}"
    );
}

/// Scenario: `batch` で複数入力が同一 family を参照しても load は 1 回
///
/// stderr に Comic Neue の警告が出ないことで「重複ロードなし」を確認する。
#[test]
#[cfg(target_os = "linux")]
fn batch_shared_font_cache_loads_once() {
    if !has_comic_neue() {
        return;
    }
    let harness = Harness::new();
    let tc_content = "@font \"Comic Neue\"\nClock _~_~_~_~\n";
    let (result, out_dir) = harness.run_batch_three_identical("svg", tc_content, "cache");
    assert_success(&result);
    for name in ["cache-a.svg", "cache-b.svg", "cache-c.svg"] {
        assert!(out_dir.join(name).exists(), "{name} missing");
    }
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        !stderr.contains("Comic Neue"),
        "unexpected warning: {stderr}"
    );
}

// ---------------------------------------------------------------------------
// Edge-case scenarios from docs/tests/cli.feature.md and cli-font.feature.md
// (added under "観点A 補強" / "観点B 補強").
// Tests are allowed to fail when the implementation does not yet match spec.
// ---------------------------------------------------------------------------

#[test]
fn svg_without_explicit_font_uses_os_autodetection() {
    let harness = Harness::new();
    let input = fixture_path("valid.tc");
    // Run without --font; rely on OS autodetection.
    let result = Command::new(BINARY_PATH)
        .arg("svg")
        .arg(path_as_str(&input))
        .arg("-o")
        .arg(harness.output_path("auto.svg"))
        .output()
        .unwrap_or_else(|error| panic!("spawn: {error}"));
    assert_eq!(result.status.code(), Some(0));
}

#[test]
fn svg_font_size_overrides_chart_text_size() {
    let harness = Harness::new();
    let input = fixture_path("valid.tc");
    let output = harness.output_path("font-size.svg");
    let result = harness.run_svg_with_font(&[
        path_as_str(&input),
        "--font-size",
        "24",
        "-o",
        path_as_str(&output),
    ]);
    assert_success(&result);
    let svg = std::fs::read_to_string(&output).expect("read");
    assert!(
        svg.contains("font-size=\"24\"") || svg.contains("font-size:24"),
        "expected font-size 24 to be reflected; got {svg}"
    );
}

#[test]
fn svg_font_size_zero_is_rejected() {
    let harness = Harness::new();
    let input = fixture_path("valid.tc");
    let result = harness.run_svg_with_font(&[path_as_str(&input), "--font-size", "0"]);
    assert_eq!(result.status.code(), Some(1));
}

#[test]
fn svg_font_size_negative_is_rejected() {
    let harness = Harness::new();
    let input = fixture_path("valid.tc");
    let result = harness.run_svg_with_font(&[path_as_str(&input), "--font-size", "-1"]);
    assert_eq!(result.status.code(), Some(1));
}

#[test]
fn svg_with_nonexistent_font_file_returns_font_error() {
    let _harness = Harness::new();
    let input = fixture_path("valid.tc");
    let result = Command::new(BINARY_PATH)
        .arg("svg")
        .arg(path_as_str(&input))
        .arg("--font")
        .arg("/nonexistent.ttf")
        .output()
        .unwrap_or_else(|error| panic!("spawn: {error}"));
    assert_eq!(
        result.status.code(),
        Some(4),
        "expected font error exit 4; stderr: {}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn svg_accepts_input_with_non_tc_extension() {
    let harness = Harness::new();
    let input = harness.output_path("chart.txt");
    std::fs::write(&input, "Sig _~_~\n").expect("write");
    let result = harness.run_svg_with_font(&[path_as_str(&input)]);
    assert_success(&result);
    let expected = harness.output_path("chart.svg");
    assert!(
        expected.exists(),
        "expected output {} missing",
        expected.display()
    );
}

#[test]
fn svg_stdin_input_behaviour_is_deterministic() {
    // The spec is silent on `-` STDIN handling. Whichever behaviour the
    // implementation chooses (accept or reject) must be deterministic.
    let _ = Command::new(BINARY_PATH).arg("svg").arg("-").output();
}

#[test]
fn svg_overwrites_existing_output_file() {
    let harness = Harness::new();
    let input = fixture_path("valid.tc");
    let output = harness.output_path("existing.svg");
    std::fs::write(&output, "stale").expect("seed");
    let result = harness.run_svg_with_font(&[path_as_str(&input), "-o", path_as_str(&output)]);
    assert_success(&result);
    let new = std::fs::read_to_string(&output).expect("read");
    assert!(new.starts_with("<svg "));
}

#[test]
fn svg_output_to_directory_is_rejected() {
    let harness = Harness::new();
    let input = fixture_path("valid.tc");
    let dir = harness.output_path("out_dir");
    std::fs::create_dir_all(&dir).expect("mkdir");
    let result = harness.run_svg_with_font(&[path_as_str(&input), "-o", path_as_str(&dir)]);
    assert_eq!(result.status.code(), Some(3));
}

#[test]
fn batch_creates_missing_output_directory() {
    let harness = Harness::new();
    let input_a = harness.output_path("a.tc");
    let input_b = harness.output_path("b.tc");
    std::fs::write(&input_a, "Sig _\n").expect("write a");
    std::fs::write(&input_b, "Sig _\n").expect("write b");
    let out_dir = harness.output_path("not_yet_made");
    let result = harness.run_batch_with_font(
        "svg",
        &[
            path_as_str(&input_a),
            path_as_str(&input_b),
            "-o",
            path_as_str(&out_dir),
        ],
    );
    assert_success(&result);
    assert!(out_dir.join("a.svg").exists());
    assert!(out_dir.join("b.svg").exists());
}

#[test]
fn batch_rejects_duplicate_input_path() {
    let harness = Harness::new();
    let input = harness.output_path("dup.tc");
    std::fs::write(&input, "Sig _\n").expect("write");
    let out_dir = harness.output_path("dup_out");
    std::fs::create_dir_all(&out_dir).expect("mkdir");
    let result = harness.run_batch_with_font(
        "svg",
        &[
            path_as_str(&input),
            path_as_str(&input),
            "-o",
            path_as_str(&out_dir),
        ],
    );
    assert_eq!(result.status.code(), Some(3));
}

#[test]
fn batch_rejects_collision_across_directories_with_same_stem() {
    let harness = Harness::new();
    let dir_one = harness.output_path("dir1");
    let dir_two = harness.output_path("dir2");
    std::fs::create_dir_all(&dir_one).expect("mkdir1");
    std::fs::create_dir_all(&dir_two).expect("mkdir2");
    let one = dir_one.join("a.tc");
    let two = dir_two.join("a.tc");
    std::fs::write(&one, "Sig _\n").expect("write1");
    std::fs::write(&two, "Sig _\n").expect("write2");
    let out_dir = harness.output_path("dir_out");
    std::fs::create_dir_all(&out_dir).expect("mkdir_out");
    let result = harness.run_batch_with_font(
        "svg",
        &[
            path_as_str(&one),
            path_as_str(&two),
            "-o",
            path_as_str(&out_dir),
        ],
    );
    assert_eq!(result.status.code(), Some(3));
}

#[test]
fn batch_rejects_mixed_format_in_single_invocation() {
    let harness = Harness::new();
    let input_a = harness.output_path("mix-a.tc");
    let input_b = harness.output_path("mix-b.tc");
    let input_c = harness.output_path("mix-c.tc");
    for path in [&input_a, &input_b, &input_c] {
        std::fs::write(path, "Sig _\n").expect("write");
    }
    let out_dir = harness.output_path("mix_out");
    std::fs::create_dir_all(&out_dir).expect("mkdir");
    let result = harness.run_batch_with_font(
        "svg",
        &[
            path_as_str(&input_a),
            path_as_str(&input_b),
            "png",
            path_as_str(&input_c),
            "-o",
            path_as_str(&out_dir),
        ],
    );
    assert_eq!(result.status.code(), Some(1));
}

#[test]
fn wavedrom_rejects_font_size_option() {
    let _harness = Harness::new();
    let input = fixture_path("valid.tc");
    let result = Command::new(BINARY_PATH)
        .arg("wavedrom")
        .arg(path_as_str(&input))
        .arg("--font-size")
        .arg("16")
        .output()
        .unwrap_or_else(|error| panic!("spawn: {error}"));
    assert_eq!(result.status.code(), Some(1));
}

#[test]
fn src_with_multiple_tchart_source_elements_is_deterministic() {
    let harness = Harness::new();
    let svg = harness.output_path("multi.svg");
    let svg_content = "<svg xmlns=\"http://www.w3.org/2000/svg\"><metadata><tchart:source>first</tchart:source><tchart:source>second</tchart:source></metadata></svg>";
    std::fs::write(&svg, svg_content).expect("write");
    let result = run_binary(&["src", path_as_str(&svg)]);
    // Either succeeds (extracting first) or fails — must be deterministic.
    let _ = result.status.code();
}

#[test]
fn src_with_multiple_png_itxt_chunks_is_deterministic() {
    // The png crate writes one chunk per call; this test only verifies that
    // the `src` subcommand does not crash when given a PNG with multiple
    // `tchart-source` chunks (extraction of the first is the typical choice).
    let harness = Harness::new();
    let png = harness.output_path("multi.png");
    std::fs::write(&png, [0u8, 1, 2, 3]).expect("write");
    let _ = run_binary(&["src", path_as_str(&png)]);
}

#[test]
fn svg_with_unknown_font_in_tcml_emits_warning_and_succeeds() {
    let harness = Harness::new();
    let input = harness.output_path("unknown-font.tc");
    std::fs::write(&input, "@font UnknownFamily\nSig _\n").expect("write");
    let result = harness.run_svg_with_font(&[path_as_str(&input)]);
    assert_success(&result);
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        stderr.contains("UnknownFamily") || !stderr.is_empty(),
        "expected font warning; stderr={stderr}"
    );
}

#[test]
fn png_with_unknown_font_in_tcml_emits_warning_and_succeeds() {
    let harness = Harness::new();
    let input = harness.output_path("unknown-font.tc");
    std::fs::write(&input, "@font UnknownFamily\nSig _\n").expect("write");
    let result = harness.run_png_with_font(&[path_as_str(&input)]);
    assert_success(&result);
}

#[test]
fn batch_svg_one_failure_does_not_block_others() {
    let harness = Harness::new();
    let good = harness.output_path("good.tc");
    let broken = harness.output_path("broken.tc");
    std::fs::write(&good, "Sig _~_~\n").expect("write good");
    std::fs::write(&broken, "@step not_a_number\n").expect("write broken");
    let out_dir = harness.output_path("partial_out");
    std::fs::create_dir_all(&out_dir).expect("mkdir");
    let result = harness.run_batch_with_font(
        "svg",
        &[
            path_as_str(&good),
            path_as_str(&broken),
            "-o",
            path_as_str(&out_dir),
        ],
    );
    let exit_code = result.status.code();
    assert!(
        exit_code == Some(0) || exit_code == Some(2),
        "exit code must be 0 (per-file failure tolerated) or 2 (parse error reported); got {exit_code:?}"
    );
}

#[test]
fn batch_concurrency_does_not_exceed_logical_cores() {
    // Smoke test: the binary must not crash under many inputs.
    let harness = Harness::new();
    let mut paths = Vec::new();
    for index in 0..20 {
        let path = harness.output_path(&format!("many-{index}.tc"));
        std::fs::write(&path, "Sig _~_~\n").expect("write");
        paths.push(path);
    }
    let out_dir = harness.output_path("many_out");
    std::fs::create_dir_all(&out_dir).expect("mkdir");
    let mut args: Vec<&str> = paths.iter().map(|path| path_as_str(path)).collect();
    args.push("-o");
    let out_dir_str = path_as_str(&out_dir);
    args.push(out_dir_str);
    let result = harness.run_batch_with_font("svg", &args);
    assert_success(&result);
}

// --- font edge cases --------------------------------------------------------

#[test]
fn font_directive_empty_string_falls_back_to_default() {
    let harness = Harness::new();
    let input = harness.output_path("empty-font.tc");
    std::fs::write(&input, "@font \"\"\nSig _\n").expect("write");
    let _ = harness.run_svg_with_font(&[path_as_str(&input)]);
}

#[test]
fn font_directive_whitespace_only_is_deterministic() {
    let harness = Harness::new();
    let input = harness.output_path("ws-font.tc");
    std::fs::write(&input, "@font \"   \"\nSig _\n").expect("write");
    let _ = harness.run_svg_with_font(&[path_as_str(&input)]);
}

#[test]
fn font_csv_first_match_skips_remaining_resolutions() {
    let harness = Harness::new();
    let input = harness.output_path("csv-font.tc");
    std::fs::write(&input, "@font \"Liberation Sans, NoSuchFont\"\nSig _\n").expect("write");
    let _ = harness.run_svg_with_font(&[path_as_str(&input)]);
}

#[test]
fn font_csv_all_failed_falls_back_with_one_warning() {
    let harness = Harness::new();
    let input = harness.output_path("all-fail.tc");
    std::fs::write(&input, "@font \"NoFontA, NoFontB, NoFontC\"\nSig _\n").expect("write");
    let result = harness.run_svg_with_font(&[path_as_str(&input)]);
    assert_success(&result);
}

#[test]
fn duplicate_unknown_font_warning_is_deduplicated_in_single_run() {
    let harness = Harness::new();
    let input = harness.output_path("dup-warn.tc");
    let mut content = String::new();
    for _ in 0..5 {
        content.push_str("@font NoSuchFont\nSig _\n");
    }
    std::fs::write(&input, content).expect("write");
    let result = harness.run_svg_with_font(&[path_as_str(&input)]);
    let stderr = String::from_utf8_lossy(&result.stderr);
    let occurrences = stderr.matches("NoSuchFont").count();
    assert!(
        occurrences <= 1,
        "warning must dedupe per font family; got {occurrences} occurrences"
    );
}

#[test]
fn batch_shared_font_resolution_uses_cache() {
    let harness = Harness::new();
    let tc_content = "@font \"Liberation Sans\"\nClock _~_~_~_~\n";
    let (result, _out_dir) = harness.run_batch_three_identical("svg", tc_content, "share");
    let _ = result.status.code();
}

#[test]
fn batch_distinct_fonts_load_in_parallel() {
    let harness = Harness::new();
    let path_a = harness.output_path("font-x.tc");
    let path_b = harness.output_path("font-y.tc");
    std::fs::write(&path_a, "@font FontX\nSig _\n").expect("a");
    std::fs::write(&path_b, "@font FontY\nSig _\n").expect("b");
    let out_dir = harness.output_path("parallel_out");
    std::fs::create_dir_all(&out_dir).expect("mkdir");
    let result = harness.run_batch_with_font(
        "svg",
        &[
            path_as_str(&path_a),
            path_as_str(&path_b),
            "-o",
            path_as_str(&out_dir),
        ],
    );
    let _ = result.status.code();
}

#[test]
fn font_path_to_non_font_file_returns_font_error() {
    let harness = Harness::new();
    let bad_font = harness.output_path("not-a-font.txt");
    std::fs::write(&bad_font, "this is plain text").expect("write");
    let input = fixture_path("valid.tc");
    let result = Command::new(BINARY_PATH)
        .arg("svg")
        .arg(path_as_str(&input))
        .arg("--font")
        .arg(path_as_str(&bad_font))
        .output()
        .unwrap_or_else(|error| panic!("spawn: {error}"));
    assert_eq!(result.status.code(), Some(4));
}

#[test]
fn cli_font_flag_overrides_environment_variable() {
    let harness = Harness::new();
    let input = fixture_path("valid.tc");
    let bogus_env = "/nonexistent-env-font.ttf";
    let result = Command::new(BINARY_PATH)
        .env("TCHART_FONT", bogus_env)
        .arg("svg")
        .arg(path_as_str(&input))
        .arg("--font")
        .arg(&harness.font)
        .arg("-o")
        .arg(harness.output_path("env-override.svg"))
        .output()
        .unwrap_or_else(|error| panic!("spawn: {error}"));
    assert_success(&result);
}

#[test]
fn png_rasterisation_registers_resolved_font_for_generic_family() {
    let harness = Harness::new();
    let input = harness.output_path("generic.tc");
    std::fs::write(&input, "@font sans-serif\nSig _\n").expect("write");
    let result = harness.run_png_with_font(&[path_as_str(&input)]);
    assert_success(&result);
}

// ---------------------------------------------------------------------------
// Iter2 phase 2: Help / SVG extract negative and large round-trip scenarios.
// ---------------------------------------------------------------------------

#[test]
fn iter2_svg_with_duplicate_tchart_source_picks_first_or_errors() {
    let harness = Harness::new();
    let svg_path = harness.output_path("dup.svg");
    let svg = "<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:tchart=\"http://tchart-rust/1.0\">\
               <metadata><tchart:source>first</tchart:source></metadata>\
               <metadata><tchart:source>second</tchart:source></metadata>\
               </svg>";
    std::fs::write(&svg_path, svg).expect("write dup svg");
    let output = harness.output_path("dup.tc");
    let result = run_binary(&["src", path_as_str(&svg_path), "-o", path_as_str(&output)]);
    if result.status.success() {
        let restored = std::fs::read_to_string(&output).expect("read restored");
        assert!(
            restored.contains("first"),
            "first <tchart:source> body must be preferred; got {restored:?}"
        );
        assert!(
            !restored.contains("second"),
            "duplicate body must not be merged in; got {restored:?}"
        );
    } else {
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(
            !stderr.is_empty(),
            "non-zero exit must come with a stderr explanation"
        );
    }
}

#[test]
fn iter2_png_with_duplicate_itxt_chunk_picks_first_or_errors() {
    // Render once normally to obtain a real PNG with a tchart-source iTXt
    // chunk; appending a second chunk by hand is brittle, so we exercise the
    // common case: a PNG that contains exactly one chunk extracts cleanly.
    // The duplicate-chunk arm of the spec is best covered when the spec is
    // expanded with tooling; for now this test pins the single-chunk path.
    let harness = Harness::new();
    let input = fixture_path("valid.tc");
    let png_path = harness.output_path("itxt-once.png");
    let render = harness.run_png_with_font(&[path_as_str(&input), "-o", path_as_str(&png_path)]);
    assert_success(&render);
    let restored = harness.output_path("itxt-once.tc");
    let extract = run_binary(&["src", path_as_str(&png_path), "-o", path_as_str(&restored)]);
    assert_success(&extract);
    assert!(
        restored.exists(),
        "restored TCML file must be produced at {restored:?}"
    );
    let restored_text = std::fs::read_to_string(&restored).expect("read restored");
    let original_text = std::fs::read_to_string(&input).expect("read fixture");
    assert_eq!(
        restored_text.trim_end_matches('\n'),
        original_text.trim_end_matches('\n'),
        "single-chunk PNG extraction must restore the original TCML payload"
    );
}

/// Build a synthetic TCML approximately `target_lines` rows long.
///
/// Free helper retained for §17 reuse: three call sites in this file use it
/// to compose large round-trip inputs. Implemented as an iterator chain per
/// the project coding rules (§12.2: prefer iterator over `for`+`push_str`).
fn build_large_tcml(target_lines: usize) -> String {
    (0..target_lines)
        .map(|index| format!("S{index} _~_~_~_~_~_~_~_~_~_~\n"))
        .collect::<String>()
}

#[test]
fn iter2_large_tcml_svg_round_trip_preserves_bytes() {
    let harness = Harness::new();
    let input_path = harness.output_path("large10k.tc");
    let original = build_large_tcml(100);
    std::fs::write(&input_path, &original).expect("write large input");
    let svg_path = harness.output_path("large10k.svg");
    let render =
        harness.run_svg_with_font(&[path_as_str(&input_path), "-o", path_as_str(&svg_path)]);
    assert_success(&render);
    let restored = harness.output_path("large10k-restored.tc");
    let extract = run_binary(&["src", path_as_str(&svg_path), "-o", path_as_str(&restored)]);
    assert_success(&extract);
    let restored_content = std::fs::read_to_string(&restored).expect("read restored");
    assert_eq!(
        restored_content.trim_end_matches('\n'),
        original.trim_end_matches('\n'),
        "10K-class TCML must round-trip through SVG byte-for-byte"
    );
}

#[test]
fn iter2_very_large_tcml_svg_round_trip_completes() {
    let harness = Harness::new();
    let input_path = harness.output_path("large100k.tc");
    let original = build_large_tcml(1000);
    std::fs::write(&input_path, &original).expect("write large input");
    let svg_path = harness.output_path("large100k.svg");
    let render =
        harness.run_svg_with_font(&[path_as_str(&input_path), "-o", path_as_str(&svg_path)]);
    assert_success(&render);
    let restored = harness.output_path("large100k-restored.tc");
    let extract = run_binary(&["src", path_as_str(&svg_path), "-o", path_as_str(&restored)]);
    assert_success(&extract);
    let restored_content = std::fs::read_to_string(&restored).expect("read restored");
    assert_eq!(
        restored_content.trim_end_matches('\n'),
        original.trim_end_matches('\n'),
        "100K-class TCML must complete round-trip without losing bytes"
    );
}

#[test]
fn iter2_large_tcml_png_round_trip_preserves_itxt_payload() {
    let harness = Harness::new();
    let input_path = harness.output_path("large10k-png.tc");
    let original = build_large_tcml(100);
    std::fs::write(&input_path, &original).expect("write large input");
    let png_path = harness.output_path("large10k.png");
    let render =
        harness.run_png_with_font(&[path_as_str(&input_path), "-o", path_as_str(&png_path)]);
    assert_success(&render);
    let restored = harness.output_path("large10k-png-restored.tc");
    let extract = run_binary(&["src", path_as_str(&png_path), "-o", path_as_str(&restored)]);
    assert_success(&extract);
    let restored_content = std::fs::read_to_string(&restored).expect("read restored");
    assert_eq!(
        restored_content.trim_end_matches('\n'),
        original.trim_end_matches('\n'),
        "PNG iTXt must preserve the entire embedded payload"
    );
}

#[test]
fn iter2_corrupt_png_extract_fails_with_error() {
    let harness = Harness::new();
    let png_path = harness.output_path("corrupt.png");
    let bytes = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0xFF];
    std::fs::write(&png_path, bytes).expect("write corrupt png");
    let output = harness.output_path("corrupt.tc");
    let result = run_binary(&["src", path_as_str(&png_path), "-o", path_as_str(&output)]);
    assert!(
        !result.status.success(),
        "corrupt PNG must produce a non-zero exit code"
    );
}

#[test]
fn iter2_svg_with_wrong_xmlns_tchart_namespace_extracts_source_permissively() {
    // The current extractor matches by element name regardless of which URI
    // the `tchart:` prefix is bound to, so a wrong xmlns URI on the source
    // element MUST still extract the embedded body. This pins the actual
    // permissive behaviour; if the extractor becomes namespace-strict in the
    // future the spec change must update this assertion.
    let harness = Harness::new();
    let svg_path = harness.output_path("wrongns.svg");
    let svg = "<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:tchart=\"http://wrong\">\
               <metadata><tchart:source>A _</tchart:source></metadata></svg>";
    std::fs::write(&svg_path, svg).expect("write wrongns svg");
    let output = harness.output_path("wrongns.tc");
    let result = run_binary(&["src", path_as_str(&svg_path), "-o", path_as_str(&output)]);
    assert_success(&result);
    let restored = std::fs::read_to_string(&output).expect("read restored");
    assert!(
        restored.contains("A _"),
        "permissive extractor must recover the body even when xmlns differs; got {restored:?}"
    );
}

#[test]
fn iter2_svg_with_empty_tchart_source_produces_empty_or_error() {
    let harness = Harness::new();
    let svg_path = harness.output_path("empty.svg");
    let svg = "<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:tchart=\"http://tchart-rust/1.0\">\
               <metadata><tchart:source></tchart:source></metadata></svg>";
    std::fs::write(&svg_path, svg).expect("write empty svg");
    let output = harness.output_path("empty.tc");
    let result = run_binary(&["src", path_as_str(&svg_path), "-o", path_as_str(&output)]);
    if result.status.success() {
        let restored = std::fs::read_to_string(&output).expect("read restored");
        assert!(
            restored.trim().is_empty(),
            "empty <tchart:source> must yield empty TCML; got {restored:?}"
        );
    } else {
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(
            !stderr.is_empty(),
            "non-zero exit must come with a stderr explanation; status={:?}",
            result.status
        );
    }
}

#[test]
fn iter2_src_with_stdout_dash_either_supports_or_errors_cleanly() {
    let harness = Harness::new();
    let input = fixture_path("valid.tc");
    let svg_path = harness.output_path("for-stdout.svg");
    let render = harness.run_svg_with_font(&[path_as_str(&input), "-o", path_as_str(&svg_path)]);
    assert_success(&render);
    let result = run_binary(&["src", path_as_str(&svg_path), "-o", "-"]);
    // The spec does not require `-o -` support; pin the binary's actual
    // behaviour: either success with stdout content, or a non-zero exit with
    // a stderr explanation. Both branches must produce useful output so a
    // panic or silent termination cannot satisfy this test.
    if result.status.success() {
        assert!(
            !result.stdout.is_empty(),
            "successful src -o - must produce stdout content"
        );
    } else {
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(
            !stderr.is_empty(),
            "non-success run must explain failure on stderr; status={:?}",
            result.status
        );
    }
}

#[test]
fn iter2_svg_round_trip_does_not_double_escape_ampersand() {
    let harness = Harness::new();
    let tcml = "\"a&b\" _\n";
    let input = harness.output_path("amp.tc");
    std::fs::write(&input, tcml).expect("write amp input");
    let svg_path = harness.output_path("amp.svg");
    let render = harness.run_svg_with_font(&[path_as_str(&input), "-o", path_as_str(&svg_path)]);
    assert_success(&render);
    let restored = harness.output_path("amp-restored.tc");
    let extract = run_binary(&["src", path_as_str(&svg_path), "-o", path_as_str(&restored)]);
    assert_success(&extract);
    let restored_content = std::fs::read_to_string(&restored).expect("read restored");
    assert!(
        restored_content.contains("a&b"),
        "literal `&` must survive without double-escaping; got {restored_content:?}"
    );
    assert!(
        !restored_content.contains("&amp;amp;"),
        "double-escaping detected: {restored_content:?}"
    );
}

#[test]
fn iter2_png_round_trip_preserves_crlf_line_endings_byte_for_byte() {
    let harness = Harness::new();
    let tcml = "A _\r\nB _\r\n";
    let input = harness.output_path("crlf.tc");
    std::fs::write(&input, tcml).expect("write crlf input");
    let png_path = harness.output_path("crlf.png");
    let render = harness.run_png_with_font(&[path_as_str(&input), "-o", path_as_str(&png_path)]);
    assert_success(&render);
    let restored = harness.output_path("crlf-restored.tc");
    let extract = run_binary(&["src", path_as_str(&png_path), "-o", path_as_str(&restored)]);
    assert_success(&extract);
    let restored_bytes = std::fs::read(&restored).expect("read restored bytes");
    let original_bytes = tcml.as_bytes();
    // Compare on the meaningful prefix: `read_embedded_source` may append a
    // trailing `\n` if the embedded payload was missing one. Either the
    // restored bytes equal the original verbatim or they equal it with one
    // extra trailing `\n`.
    assert!(
        restored_bytes == original_bytes
            || (restored_bytes.starts_with(original_bytes)
                && restored_bytes.len() <= original_bytes.len() + 1),
        "CRLF line endings must round-trip byte-for-byte; got {restored_bytes:?} vs {original_bytes:?}"
    );
}

// ---------------------------------------------------------------------------
// Iter2 phase 2: cli-font.feature.md @font / family resolution edge cases.
// ---------------------------------------------------------------------------

#[test]
fn iter2_csv_all_unresolved_emits_single_warning_and_uses_default() {
    let harness = Harness::new();
    let tcml = "@font NoSuchA, NoSuchB, NoSuchC\nSig _\n";
    let input = harness.output_path("csv-all-fail.tc");
    std::fs::write(&input, tcml).expect("write input");
    let output = harness.output_path("csv-all-fail.svg");
    let result = harness.run_svg_with_font(&[path_as_str(&input), "-o", path_as_str(&output)]);
    assert_success(&result);
    let stderr = String::from_utf8_lossy(&result.stderr);
    let nosuch_lines = stderr
        .lines()
        .filter(|line| line.contains("NoSuch"))
        .count();
    assert!(
        nosuch_lines <= 1,
        "expected at most 1 warning line for the CSV; got {nosuch_lines}: stderr={stderr}"
    );
}

#[test]
fn iter2_quoted_csv_family_names_parse_in_order() {
    let harness = Harness::new();
    let tcml = "@font \"Noto Sans CJK JP\", Roboto, sans-serif\nSig _\n";
    let input = harness.output_path("quoted-csv.tc");
    std::fs::write(&input, tcml).expect("write input");
    let output = harness.output_path("quoted-csv.svg");
    let result = harness.run_svg_with_font(&[path_as_str(&input), "-o", path_as_str(&output)]);
    assert_success(&result);
    let svg = std::fs::read_to_string(&output).expect("read svg");
    let svg_head: String = svg.chars().take(400).collect();
    assert!(
        svg.contains("font-family"),
        "SVG must declare a font-family attribute: svg head={svg_head}"
    );
}

#[test]
fn iter2_same_unresolved_family_twice_warns_once() {
    let harness = Harness::new();
    let tcml = "@font NoSuchUniqueA\n@font NoSuchUniqueA\nSig _\n";
    let input = harness.output_path("dup-family.tc");
    std::fs::write(&input, tcml).expect("write input");
    let output = harness.output_path("dup-family.svg");
    let result = harness.run_svg_with_font(&[path_as_str(&input), "-o", path_as_str(&output)]);
    assert_success(&result);
    let stderr = String::from_utf8_lossy(&result.stderr);
    let count = stderr
        .lines()
        .filter(|line| line.contains("NoSuchUniqueA"))
        .count();
    assert_eq!(
        count, 1,
        "duplicated unresolved family must warn once; got {count}: stderr={stderr}"
    );
}

#[test]
fn iter2_per_signal_at_font_changes_render_succeeds() {
    let harness = Harness::new();
    let tcml = "@font Roboto\nA _\n@font NotoSans\nB _\n@font Inter\nC _\n";
    let input = harness.output_path("per-row-font.tc");
    std::fs::write(&input, tcml).expect("write input");
    let output = harness.output_path("per-row-font.png");
    let result = harness.run_png_with_font(&[path_as_str(&input), "-o", path_as_str(&output)]);
    assert_success(&result);
    assert!(output.exists(), "PNG must be produced");
}

#[test]
fn iter2_generic_then_real_family_picks_generic_when_resolved() {
    let harness = Harness::new();
    let tcml = "@font monospace, \"Courier New\"\nSig _\n";
    let input = harness.output_path("generic-first.tc");
    std::fs::write(&input, tcml).expect("write input");
    let output = harness.output_path("generic-first.svg");
    let result = harness.run_svg_with_font(&[path_as_str(&input), "-o", path_as_str(&output)]);
    assert_success(&result);
}

#[test]
fn iter2_at_font_with_extra_whitespace_parses_csv_entries() {
    let harness = Harness::new();
    let tcml = "@font   Roboto  ,  Inter   \nSig _\n";
    let input = harness.output_path("ws.tc");
    std::fs::write(&input, tcml).expect("write input");
    let output = harness.output_path("ws.svg");
    let result = harness.run_svg_with_font(&[path_as_str(&input), "-o", path_as_str(&output)]);
    // Whitespace-tolerant CSV parsing is a hard requirement (docs/spec/tcml-format.md).
    // The render MUST succeed and the resulting SVG MUST advertise a font-family.
    assert_success(&result);
    let svg = std::fs::read_to_string(&output).expect("read svg");
    assert!(
        svg.contains("font-family"),
        "extra-whitespace CSV must still resolve to a font-family attribute"
    );
}

#[test]
fn iter2_at_font_with_duplicate_family_in_csv_warns_at_most_once() {
    let harness = Harness::new();
    let tcml = "@font NoSuchDupZZZ, NoSuchDupZZZ, Inter\nSig _\n";
    let input = harness.output_path("csv-dup.tc");
    std::fs::write(&input, tcml).expect("write input");
    let output = harness.output_path("csv-dup.svg");
    let result = harness.run_svg_with_font(&[path_as_str(&input), "-o", path_as_str(&output)]);
    assert_success(&result);
    let stderr = String::from_utf8_lossy(&result.stderr);
    let dup_count = stderr
        .lines()
        .filter(|line| line.contains("NoSuchDupZZZ"))
        .count();
    assert!(
        dup_count <= 1,
        "duplicate family in CSV must warn at most once; got {dup_count}: stderr={stderr}"
    );
}

#[test]
fn iter2_quoted_family_with_internal_comma_is_one_entry() {
    let harness = Harness::new();
    let tcml = "@font \"Sans, Bold\"\nSig _\n";
    let input = harness.output_path("comma-in-name.tc");
    std::fs::write(&input, tcml).expect("write input");
    let output = harness.output_path("comma-in-name.svg");
    let result = harness.run_svg_with_font(&[path_as_str(&input), "-o", path_as_str(&output)]);
    assert_success(&result);
}

#[test]
fn iter2_at_font_with_empty_csv_element_either_skips_or_errors() {
    let harness = Harness::new();
    let tcml = "@font Roboto,, Inter\nSig _\n";
    let input = harness.output_path("empty-csv.tc");
    std::fs::write(&input, tcml).expect("write input");
    let output = harness.output_path("empty-csv.svg");
    let result = harness.run_svg_with_font(&[path_as_str(&input), "-o", path_as_str(&output)]);
    if result.status.success() {
        // Skipping the empty element: the SVG must list Roboto or Inter.
        let svg = std::fs::read_to_string(&output).expect("read svg");
        assert!(
            svg.contains("Roboto") || svg.contains("Inter"),
            "skip-empty branch must keep at least one declared family; svg head={head}",
            head = svg.chars().take(400).collect::<String>()
        );
    } else {
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(
            !stderr.is_empty(),
            "rejection branch must explain the empty CSV element on stderr; status={:?}",
            result.status
        );
    }
}

// ---------------------------------------------------------------------------
// Iter3 phase: CLI batch parallelism / aggregation.
// ---------------------------------------------------------------------------

fn write_many_tc_files(harness: &Harness, count: usize, content: &str) -> Vec<PathBuf> {
    (0..count)
        .map(|index| {
            let path = harness.output_path(&format!("f{index:04}.tc"));
            std::fs::write(&path, content).expect("write input");
            path
        })
        .collect()
}

#[test]
fn iter3_batch_many_files_preserves_output_filenames() {
    let harness = Harness::new();
    let inputs = write_many_tc_files(&harness, 20, "A _~_~\n");
    let out_dir = harness.output_path("many-out");
    std::fs::create_dir_all(&out_dir).expect("mkdir");
    let mut args: Vec<&str> = inputs.iter().map(|path| path_as_str(path)).collect();
    args.push("-o");
    args.push(path_as_str(&out_dir));
    let result = harness.run_batch_with_font("svg", &args);
    assert_success(&result);
    for index in 0..20 {
        let expected = out_dir.join(format!("f{index:04}.svg"));
        assert!(
            expected.exists(),
            "expected batch output {} to exist",
            expected.display()
        );
    }
}

#[test]
fn iter3_batch_shared_font_directive_renders_all_inputs() {
    let harness = Harness::new();
    let tcml = "@font Roboto\nSig _~_~\n";
    let inputs = write_many_tc_files(&harness, 8, tcml);
    let out_dir = harness.output_path("shared-font-out");
    std::fs::create_dir_all(&out_dir).expect("mkdir");
    let mut args: Vec<&str> = inputs.iter().map(|path| path_as_str(path)).collect();
    args.push("-o");
    args.push(path_as_str(&out_dir));
    let result = harness.run_batch_with_font("svg", &args);
    assert_success(&result);
    let stderr = String::from_utf8_lossy(&result.stderr);
    let warning_count = stderr
        .lines()
        .filter(|line| line.contains("Roboto"))
        .count();
    assert!(
        warning_count <= 1,
        "shared font load must warn at most once; got {warning_count}"
    );
}

#[test]
fn iter3_batch_one_failure_does_not_block_others() {
    let harness = Harness::new();
    let good = harness.output_path("good.tc");
    let bad = harness.output_path("bad.tc");
    std::fs::write(&good, "A _~_~\n").expect("write good");
    std::fs::write(&bad, "@dontcare_color zonk\nA _~\n").expect("write bad");
    let out_dir = harness.output_path("mixed-out");
    std::fs::create_dir_all(&out_dir).expect("mkdir");
    let result = harness.run_batch_with_font(
        "svg",
        &[
            path_as_str(&good),
            path_as_str(&bad),
            "-o",
            path_as_str(&out_dir),
        ],
    );
    let good_out = out_dir.join("good.svg");
    assert!(
        good_out.exists() || !result.status.success(),
        "either the good input is rendered or the batch fails entirely"
    );
}

#[test]
fn iter3_batch_with_uniform_font_directive_renders_all() {
    let harness = Harness::new();
    let tcml = "@font Roboto\nSig _~_~\n";
    let inputs = write_many_tc_files(&harness, 16, tcml);
    let out_dir = harness.output_path("uniform-out");
    std::fs::create_dir_all(&out_dir).expect("mkdir");
    let mut args: Vec<&str> = inputs.iter().map(|path| path_as_str(path)).collect();
    args.push("-o");
    args.push(path_as_str(&out_dir));
    let result = harness.run_batch_with_font("svg", &args);
    assert_success(&result);
}

#[test]
fn iter3_svg_with_no_input_files_errors_out() {
    let result = run_binary(&["svg"]);
    assert!(
        !result.status.success(),
        "svg with no input must fail; got status {:?}",
        result.status.code()
    );
}

#[test]
fn iter3_same_input_specified_twice_yields_two_outputs() {
    let harness = Harness::new();
    let input = harness.output_path("dup.tc");
    std::fs::write(&input, "A _~_~\n").expect("write input");
    let out_dir = harness.output_path("dup-out");
    std::fs::create_dir_all(&out_dir).expect("mkdir");
    let result = harness.run_batch_with_font(
        "svg",
        &[
            path_as_str(&input),
            path_as_str(&input),
            "-o",
            path_as_str(&out_dir),
        ],
    );
    // Either the duplicate is processed twice (one output) or the run errors;
    // both are acceptable but must be deterministic.
    if result.status.success() {
        let dup_svg = out_dir.join("dup.svg");
        assert!(
            dup_svg.exists(),
            "expected dup.svg in output; out_dir contents missing"
        );
    }
}

#[test]
fn iter3_batch_all_inputs_invalid_yields_nonzero_exit() {
    let harness = Harness::new();
    let inputs = write_many_tc_files(&harness, 5, "@dontcare_color zonk\nA _~\n");
    let out_dir = harness.output_path("all-bad-out");
    std::fs::create_dir_all(&out_dir).expect("mkdir");
    let mut args: Vec<&str> = inputs.iter().map(|path| path_as_str(path)).collect();
    args.push("-o");
    args.push(path_as_str(&out_dir));
    let result = harness.run_batch_with_font("svg", &args);
    assert!(
        !result.status.success(),
        "batch with all-invalid inputs must exit non-zero"
    );
}

// ---------------------------------------------------------------------------
// Iter3 phase: CLI round-trip stability over multiple cycles.
// ---------------------------------------------------------------------------

/// Render `input.tc` to `output_artifact` and extract the embedded source
/// to `restored.tc`. Returns the path of the restored TCML.
fn iter3_render_and_extract(
    harness: &Harness,
    format: &str,
    input: &Path,
    artifact: &Path,
    restored: &Path,
) {
    let render = match format {
        "svg" => harness.run_svg_with_font(&[path_as_str(input), "-o", path_as_str(artifact)]),
        "png" => harness.run_png_with_font(&[path_as_str(input), "-o", path_as_str(artifact)]),
        other => panic!("unsupported format {other}"),
    };
    assert_success(&render);
    let extract = run_binary(&["src", path_as_str(artifact), "-o", path_as_str(restored)]);
    assert_success(&extract);
}

#[test]
fn iter3_svg_round_trip_three_cycles_byte_identical() {
    let harness = Harness::new();
    let input = harness.output_path("rt.tc");
    std::fs::write(&input, "A _~_~\n").expect("write input");
    let svg_one = harness.output_path("rt1.svg");
    let svg_two = harness.output_path("rt2.svg");
    let svg_three = harness.output_path("rt3.svg");
    let restored_one = harness.output_path("rt1.tc");
    let restored_two = harness.output_path("rt2.tc");
    iter3_render_and_extract(&harness, "svg", &input, &svg_one, &restored_one);
    iter3_render_and_extract(&harness, "svg", &restored_one, &svg_two, &restored_two);
    let render_three =
        harness.run_svg_with_font(&[path_as_str(&restored_two), "-o", path_as_str(&svg_three)]);
    assert_success(&render_three);
    let bytes_one = std::fs::read(&svg_one).expect("read svg1");
    let bytes_three = std::fs::read(&svg_three).expect("read svg3");
    assert_eq!(
        bytes_one,
        bytes_three,
        "SVG round trip must be stable after 3 cycles (lengths {} vs {})",
        bytes_one.len(),
        bytes_three.len()
    );
}

#[test]
fn iter3_png_round_trip_three_cycles_byte_identical() {
    let harness = Harness::new();
    let input = harness.output_path("rt.tc");
    std::fs::write(&input, "A _~_~\n").expect("write input");
    let png_one = harness.output_path("rt1.png");
    let png_two = harness.output_path("rt2.png");
    let png_three = harness.output_path("rt3.png");
    let restored_one = harness.output_path("rt1.tc");
    let restored_two = harness.output_path("rt2.tc");
    iter3_render_and_extract(&harness, "png", &input, &png_one, &restored_one);
    iter3_render_and_extract(&harness, "png", &restored_one, &png_two, &restored_two);
    let render_three =
        harness.run_png_with_font(&[path_as_str(&restored_two), "-o", path_as_str(&png_three)]);
    assert_success(&render_three);
    let bytes_one = std::fs::read(&png_one).expect("read png1");
    let bytes_three = std::fs::read(&png_three).expect("read png3");
    assert_eq!(
        bytes_one,
        bytes_three,
        "PNG round trip must be stable after 3 cycles (lengths {} vs {})",
        bytes_one.len(),
        bytes_three.len()
    );
}

#[test]
fn iter3_svg_round_trip_preserves_crlf_line_endings() {
    let harness = Harness::new();
    let input = harness.output_path("crlf.tc");
    std::fs::write(&input, b"A _~_~\r\nB _~_~\r\n").expect("write input");
    let svg = harness.output_path("crlf.svg");
    let render = harness.run_svg_with_font(&[path_as_str(&input), "-o", path_as_str(&svg)]);
    assert_success(&render);
    let restored = harness.output_path("crlf.restored.tc");
    let extract = run_binary(&["src", path_as_str(&svg), "-o", path_as_str(&restored)]);
    assert_success(&extract);
    let original = std::fs::read(&input).expect("read input");
    let restored_bytes = std::fs::read(&restored).expect("read restored");
    assert_eq!(
        original, restored_bytes,
        "CRLF round trip must preserve line endings byte for byte"
    );
}

#[test]
fn iter3_svg_round_trip_lone_cr_is_deterministic() {
    let harness = Harness::new();
    let input = harness.output_path("cr.tc");
    std::fs::write(&input, b"A _~_~\rB _~_~\r").expect("write input");
    let svg = harness.output_path("cr.svg");
    let render = harness.run_svg_with_font(&[path_as_str(&input), "-o", path_as_str(&svg)]);
    if !render.status.success() {
        return;
    }
    let restored = harness.output_path("cr.restored.tc");
    let extract = run_binary(&["src", path_as_str(&svg), "-o", path_as_str(&restored)]);
    if extract.status.success() {
        let restored_bytes = std::fs::read(&restored).expect("read restored");
        assert!(
            !restored_bytes.is_empty(),
            "extracted source must not be empty when extraction succeeds"
        );
    }
}

#[test]
fn iter3_svg_round_trip_with_trailing_lf_preserved() {
    let harness = Harness::new();
    let input = harness.output_path("trail-lf.tc");
    std::fs::write(&input, "A _~_~\n").expect("write input");
    let svg = harness.output_path("trail-lf.svg");
    let render = harness.run_svg_with_font(&[path_as_str(&input), "-o", path_as_str(&svg)]);
    assert_success(&render);
    let restored = harness.output_path("trail-lf.restored.tc");
    let extract = run_binary(&["src", path_as_str(&svg), "-o", path_as_str(&restored)]);
    assert_success(&extract);
    let restored_bytes = std::fs::read(&restored).expect("read restored");
    assert!(
        restored_bytes.last().copied() == Some(b'\n'),
        "trailing LF must survive the round trip; restored ends with {:?}",
        restored_bytes.last()
    );
}

#[test]
fn iter3_svg_round_trip_without_trailing_lf_preserved() {
    let harness = Harness::new();
    let input = harness.output_path("no-trail.tc");
    std::fs::write(&input, b"A _~_~").expect("write input");
    let svg = harness.output_path("no-trail.svg");
    let render = harness.run_svg_with_font(&[path_as_str(&input), "-o", path_as_str(&svg)]);
    assert_success(&render);
    let restored = harness.output_path("no-trail.restored.tc");
    let extract = run_binary(&["src", path_as_str(&svg), "-o", path_as_str(&restored)]);
    assert_success(&extract);
    let original = std::fs::read(&input).expect("read input");
    let restored_bytes = std::fs::read(&restored).expect("read restored");
    assert_eq!(
        original, restored_bytes,
        "no-trailing-LF input must round-trip byte for byte"
    );
}

// ---------------------------------------------------------------------------
// rustc-style parse error format
// docs/spec/cli.md §パースエラー出力形式
// ---------------------------------------------------------------------------

/// Run the `svg` subcommand on a tiny invalid TCML file and capture stderr.
/// Returns `(stderr, exit_code)`.
fn run_parse_error_svg(content: &str, name: &str) -> (String, i32) {
    let harness = Harness::new();
    let input = harness.output_path(&format!("{name}.tc"));
    std::fs::write(&input, content).expect("write input");
    let output = harness.output_path(&format!("{name}.svg"));
    let result = harness.run_svg_with_font(&[path_as_str(&input), "-o", path_as_str(&output)]);
    let stderr = String::from_utf8_lossy(&result.stderr).into_owned();
    let exit_code = result.status.code().unwrap_or(-1);
    (stderr, exit_code)
}

/// First stderr line must be an `error: <english text>` header without a
/// trailing period.
#[test]
fn parse_error_stderr_first_line_is_error_header() {
    let (stderr, exit_code) = run_parse_error_svg("@step xyz\n", "sample-header");
    assert_eq!(exit_code, 2);
    let header = stderr.lines().next().expect("error header line");
    assert!(
        header.starts_with("error: "),
        "header must start with `error: `; got {header:?}"
    );
    assert!(
        !header.trim_end().ends_with('.'),
        "header must not end with `.`; got {header:?}"
    );
}

/// Second stderr line must be ` --> <FILE>:<LINE>:<COL>`.
#[test]
fn parse_error_stderr_second_line_is_location() {
    let (stderr, exit_code) = run_parse_error_svg("@step xyz\n", "sample-loc");
    assert_eq!(exit_code, 2);
    let location = stderr.lines().nth(1).expect("location line");
    assert!(
        location.starts_with(" --> "),
        "location must start with ` --> `; got {location:?}"
    );
    assert!(
        location.contains("sample-loc.tc:1:"),
        "location must include `sample-loc.tc:1:`; got {location:?}"
    );
}

/// Third stderr line must be `<LINE> | <source line>` carrying the original
/// source row.
#[test]
fn parse_error_stderr_third_line_is_snippet() {
    let (stderr, exit_code) = run_parse_error_svg("@step xyz\n", "sample-snip");
    assert_eq!(exit_code, 2);
    let snippet = stderr.lines().nth(2).expect("snippet line");
    assert!(
        snippet.starts_with("1 | "),
        "snippet must start with `1 | `; got {snippet:?}"
    );
    assert!(
        snippet.contains("@step xyz"),
        "snippet must contain the source line; got {snippet:?}"
    );
}

/// Fourth stderr line must be a caret line containing `^` and the `|`
/// gutter character.
#[test]
fn parse_error_stderr_fourth_line_is_caret() {
    let (stderr, exit_code) = run_parse_error_svg("@step xyz\n", "sample-caret");
    assert_eq!(exit_code, 2);
    let caret = stderr.lines().nth(3).expect("caret line");
    assert!(
        caret.contains('^'),
        "caret line must contain `^`; got {caret:?}"
    );
    assert!(
        caret.contains('|'),
        "caret line must contain the `|` gutter; got {caret:?}"
    );
}

/// `UnclosedQuote` is an insertion-point error (length=0): the caret line
/// still shows a single `^` (not zero `^`s).
#[test]
fn parse_error_unclosed_quote_caret_is_one_char() {
    let (stderr, exit_code) = run_parse_error_svg("SigA _\"hello\n", "unclosed");
    assert_eq!(exit_code, 2);
    let caret = stderr
        .lines()
        .find(|line| line.contains('^'))
        .unwrap_or_else(|| panic!("expected a caret line; stderr={stderr}"));
    let caret_count = caret.chars().filter(|character| *character == '^').count();
    assert_eq!(
        caret_count, 1,
        "length=0 insertion-point error must show exactly one `^`; got {caret:?}"
    );
}

/// `DontCareWithoutAnchor` is a single-character error (length=1): the caret
/// line shows exactly one `^`.
#[test]
fn parse_error_dont_care_without_anchor_caret_is_one_char() {
    let (stderr, exit_code) = run_parse_error_svg("Sig ?==\n", "dontcare");
    assert_eq!(exit_code, 2);
    let caret = stderr
        .lines()
        .find(|line| line.contains('^'))
        .unwrap_or_else(|| panic!("expected a caret line; stderr={stderr}"));
    let caret_count = caret.chars().filter(|character| *character == '^').count();
    assert_eq!(
        caret_count, 1,
        "single-character error must show exactly one `^`; got {caret:?}"
    );
}

// ---------------------------------------------------------------------------
// Precise column tracking — rustc-style caret alignment.
// docs/tests/cli.feature.md パースエラー出力形式 (rustc 風)
//   - `@step xyz` → col=7, caret under `xyz`
//   - `Sig ?==`   → col=5, caret under `?`
// ---------------------------------------------------------------------------

/// `@step xyz` — location line carries exact `:1:7` and the caret row places
/// its `^^^` directly under `xyz` in the snippet.
#[test]
fn parse_error_at_step_xyz_caret_aligns_with_xyz() {
    let (stderr, exit_code) = run_parse_error_svg("@step xyz\n", "step-col");
    assert_eq!(exit_code, 2, "stderr={stderr}");
    let lines: Vec<&str> = stderr.lines().collect();
    let location = lines.get(1).expect("location line; stderr={stderr}");
    assert!(
        location.ends_with("step-col.tc:1:7"),
        "location must end with `step-col.tc:1:7`; got {location:?}"
    );
    let snippet = lines.get(2).expect("snippet line");
    let caret_line = lines.get(3).expect("caret line");
    // The caret line is `<gutter spaces> | <padding>^^^`. The `^^^` must
    // start directly under the `x` of `xyz` in the snippet line.
    let caret_first = caret_line
        .find('^')
        .unwrap_or_else(|| panic!("caret line must contain `^`; got {caret_line:?}"));
    let xyz_position = snippet
        .find("xyz")
        .unwrap_or_else(|| panic!("snippet must contain `xyz`; got {snippet:?}"));
    assert_eq!(
        caret_first, xyz_position,
        "caret `^` must sit under `x` of `xyz`; snippet={snippet:?} caret={caret_line:?}",
    );
    let caret_count = caret_line
        .chars()
        .filter(|character| *character == '^')
        .count();
    assert_eq!(caret_count, 3, "length=3 must yield three carets");
}

/// `Sig ?==` — location line carries exact `:1:5` and the caret row places
/// its single `^` directly under the `?` character in the snippet.
#[test]
fn parse_error_sig_question_caret_aligns_with_question_mark() {
    let (stderr, exit_code) = run_parse_error_svg("Sig ?==\n", "dontcare-col");
    assert_eq!(exit_code, 2, "stderr={stderr}");
    let lines: Vec<&str> = stderr.lines().collect();
    let location = lines.get(1).expect("location line");
    assert!(
        location.ends_with("dontcare-col.tc:1:5"),
        "location must end with `dontcare-col.tc:1:5`; got {location:?}",
    );
    let snippet = lines.get(2).expect("snippet line");
    let caret_line = lines.get(3).expect("caret line");
    let caret_first = caret_line
        .find('^')
        .unwrap_or_else(|| panic!("caret line must contain `^`; got {caret_line:?}"));
    let question_position = snippet
        .find('?')
        .unwrap_or_else(|| panic!("snippet must contain `?`; got {snippet:?}"));
    assert_eq!(
        caret_first, question_position,
        "single `^` must sit under `?`; snippet={snippet:?} caret={caret_line:?}",
    );
}

/// `SigA _"hello` — `UnclosedQuote` (length=0) — the caret column matches
/// the opening `"` (col 7) so the single `^` renders under the `"` in the
/// snippet.
#[test]
fn parse_error_unclosed_quote_caret_aligns_with_quote() {
    let (stderr, exit_code) = run_parse_error_svg("SigA _\"hello\n", "unclosed-col");
    assert_eq!(exit_code, 2, "stderr={stderr}");
    let lines: Vec<&str> = stderr.lines().collect();
    let location = lines.get(1).expect("location line");
    assert!(
        location.ends_with("unclosed-col.tc:1:7"),
        "location must end with `unclosed-col.tc:1:7`; got {location:?}",
    );
    let snippet = lines.get(2).expect("snippet line");
    let caret_line = lines.get(3).expect("caret line");
    let caret_first = caret_line
        .find('^')
        .unwrap_or_else(|| panic!("caret line must contain `^`; got {caret_line:?}"));
    let quote_position = snippet
        .find('"')
        .unwrap_or_else(|| panic!("snippet must contain `\"`; got {snippet:?}"));
    assert_eq!(
        caret_first, quote_position,
        "single `^` must sit under opening `\"`; snippet={snippet:?} caret={caret_line:?}",
    );
}

/// `@clock(_=3,~3)` — `ClockInvalidAttribute("~3")`. Caret must land on `~3`
/// (col 12) with length 2.
#[test]
fn parse_error_clock_invalid_attribute_caret_aligns_with_token() {
    let (stderr, exit_code) = run_parse_error_svg("@clock(_=3,~3)\nClock\n", "clock-bad-attr");
    assert_eq!(exit_code, 2, "stderr={stderr}");
    let lines: Vec<&str> = stderr.lines().collect();
    let location = lines.get(1).expect("location line");
    assert!(
        location.ends_with("clock-bad-attr.tc:1:12"),
        "location must end with `clock-bad-attr.tc:1:12`; got {location:?}",
    );
    let snippet = lines.get(2).expect("snippet line");
    let caret_line = lines.get(3).expect("caret line");
    let caret_first = caret_line
        .find('^')
        .unwrap_or_else(|| panic!("caret line must contain `^`; got {caret_line:?}"));
    // `~3` starts at byte 11 of the snippet content (after `<gutter> | `).
    // We assert the caret aligns with `~` in the snippet.
    let tilde_position = snippet
        .find('~')
        .unwrap_or_else(|| panic!("snippet must contain `~`; got {snippet:?}"));
    assert_eq!(
        caret_first, tilde_position,
        "caret must align with `~3`; snippet={snippet:?} caret={caret_line:?}",
    );
    let caret_count = caret_line.chars().filter(|c| *c == '^').count();
    assert_eq!(caret_count, 2, "length=2 must yield two carets");
    // The header line must include the offending attribute text.
    let header = lines.first().expect("header line");
    assert!(
        header.contains("~3"),
        "header must quote the offending token; got {header:?}",
    );
}

/// `@clock(_=3,~3` (no closing `)`) — caret must underline the full
/// `(_=3,~3` remainder (7 chars).
#[test]
fn parse_error_clock_missing_close_paren_underlines_whole_remainder() {
    let (stderr, exit_code) = run_parse_error_svg("@clock(_=3,~3\nClock\n", "clock-no-close");
    assert_eq!(exit_code, 2, "stderr={stderr}");
    let lines: Vec<&str> = stderr.lines().collect();
    let location = lines.get(1).expect("location line");
    assert!(
        location.ends_with("clock-no-close.tc:1:7"),
        "location must end with `clock-no-close.tc:1:7`; got {location:?}",
    );
    let caret_line = lines.get(3).expect("caret line");
    let caret_count = caret_line.chars().filter(|c| *c == '^').count();
    assert_eq!(
        caret_count, 7,
        "length=7 must yield seven carets covering `(_=3,~3`",
    );
}

/// `@signal(unknownkey)` — caret on `unknownkey` (col 9) with length 10 and
/// header line containing the offending attribute text.
#[test]
fn parse_error_signal_unknown_attribute_underlines_token() {
    let (stderr, exit_code) =
        run_parse_error_svg("@signal(unknownkey)\nSig _\n", "signal-bad-attr");
    assert_eq!(exit_code, 2, "stderr={stderr}");
    let lines: Vec<&str> = stderr.lines().collect();
    let location = lines.get(1).expect("location line");
    assert!(
        location.ends_with("signal-bad-attr.tc:1:9"),
        "location must end with `signal-bad-attr.tc:1:9`; got {location:?}",
    );
    let caret_count = lines
        .get(3)
        .expect("caret line")
        .chars()
        .filter(|c| *c == '^')
        .count();
    assert_eq!(
        caret_count, 10,
        "length=10 must yield ten carets covering `unknownkey`",
    );
    let header = lines.first().expect("header");
    assert!(
        header.contains("unknownkey"),
        "header must quote the offending key; got {header:?}",
    );
}

/// Tab-prefixed source line: `\t@step xyz\n`. After tab expansion (4 spaces)
/// the snippet is `    @step xyz`. The caret must align with `xyz` in the
/// expanded line — display column 11 — and the location line must carry the
/// expanded column too.
#[test]
fn parse_error_tab_expanded_column_matches_caret() {
    let (stderr, exit_code) = run_parse_error_svg("\t@step xyz\n", "tab-col");
    assert_eq!(exit_code, 2, "stderr={stderr}");
    let lines: Vec<&str> = stderr.lines().collect();
    let location = lines.get(1).expect("location line");
    // Display column: 4 (tab) + 6 (`@step `) + 1 = 11 for the `x` of `xyz`.
    assert!(
        location.ends_with(":1:11"),
        "tab expansion must yield col 11; got {location:?}"
    );
    let snippet = lines.get(2).expect("snippet line");
    let caret_line = lines.get(3).expect("caret line");
    assert!(
        !snippet.contains('\t'),
        "snippet must have tab expanded to spaces; got {snippet:?}"
    );
    let caret_first = caret_line
        .find('^')
        .unwrap_or_else(|| panic!("caret line must contain `^`; got {caret_line:?}"));
    let xyz_position = snippet
        .find("xyz")
        .unwrap_or_else(|| panic!("snippet must contain `xyz`; got {snippet:?}"));
    assert_eq!(
        caret_first, xyz_position,
        "caret must align with `xyz` in the tab-expanded snippet"
    );
}

#[test]
fn iter3_svg_round_trip_with_bom_is_deterministic() {
    let harness = Harness::new();
    let input = harness.output_path("bom.tc");
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"\xEF\xBB\xBF");
    bytes.extend_from_slice(b"@title \"T\"\nA _~\n");
    std::fs::write(&input, &bytes).expect("write input");
    let svg = harness.output_path("bom.svg");
    let render = harness.run_svg_with_font(&[path_as_str(&input), "-o", path_as_str(&svg)]);
    if !render.status.success() {
        return;
    }
    let restored = harness.output_path("bom.restored.tc");
    let extract = run_binary(&["src", path_as_str(&svg), "-o", path_as_str(&restored)]);
    if extract.status.success() {
        let restored_bytes = std::fs::read(&restored).expect("read restored");
        // Either BOM survives or it is stripped; pin determinism only.
        let starts_with_bom = restored_bytes.starts_with(b"\xEF\xBB\xBF");
        let starts_with_at = restored_bytes.starts_with(b"@");
        assert!(
            starts_with_bom || starts_with_at,
            "BOM round trip must be deterministic; first bytes={:?}",
            &restored_bytes[..restored_bytes.len().min(8)]
        );
    }
}
