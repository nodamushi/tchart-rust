//! End-to-end tests for the `tchart wavedrom` subcommand.
//!
//! Mirrors the scenarios in `docs/tests/wavedrom.feature.md`
//! §"CLI: `tchart wavedrom` サブコマンド".

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

const BINARY_PATH: &str = env!("CARGO_BIN_EXE_tchart");

/// Per-test scaffolding bundling the working directory and helpers.
struct Harness {
    work: TempDir,
}

impl Harness {
    fn new() -> Self {
        Harness {
            work: tempfile::Builder::new()
                .prefix("tchart-wavedrom-it-")
                .tempdir()
                .unwrap_or_else(|error| panic!("tempdir: {error}")),
        }
    }

    fn output_path(&self, name: &str) -> PathBuf {
        self.work.path().join(name)
    }

    fn run_wavedrom(&self, args: &[&str]) -> Output {
        Command::new(BINARY_PATH)
            .arg("wavedrom")
            .args(args)
            .output()
            .unwrap_or_else(|error| panic!("spawn tchart: {error}"))
    }
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn path_as_str(path: &Path) -> &str {
    path.to_str()
        .unwrap_or_else(|| panic!("non-utf8 path: {}", path.display()))
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
// wavedrom サブコマンド
// ---------------------------------------------------------------------------

/// Scenario: デフォルト出力 (入力隣に `<STEM>.json`)
#[test]
fn wavedrom_default_output_is_stem_dot_json() {
    let harness = Harness::new();
    let input = fixture_path("valid.tc");
    let copied = harness.output_path("chart.tc");
    std::fs::copy(&input, &copied).expect("copy");
    let result = harness.run_wavedrom(&[path_as_str(&copied)]);
    assert_exit_code(&result, 0);
    let expected = harness.output_path("chart.json");
    assert!(
        expected.exists(),
        "expected output {} missing; stderr: {}",
        expected.display(),
        String::from_utf8_lossy(&result.stderr)
    );
    let content = std::fs::read_to_string(&expected).expect("read json");
    serde_json::from_str::<serde_json::Value>(&content)
        .unwrap_or_else(|error| panic!("output is not valid JSON: {error}\ncontent: {content}"));
}

/// Scenario: `-o` で出力ファイル指定
#[test]
fn wavedrom_renders_to_specified_output() {
    let harness = Harness::new();
    let input = fixture_path("valid.tc");
    let output = harness.output_path("out.json");
    let result = harness.run_wavedrom(&[path_as_str(&input), "-o", path_as_str(&output)]);
    assert_exit_code(&result, 0);
    assert!(
        output.exists(),
        "expected output {} missing; stderr: {}",
        output.display(),
        String::from_utf8_lossy(&result.stderr)
    );
    let content = std::fs::read_to_string(&output).expect("read json");
    serde_json::from_str::<serde_json::Value>(&content)
        .unwrap_or_else(|error| panic!("output is not valid JSON: {error}"));
}

/// Scenario: 複数入力はエラー (exit 1)
#[test]
fn wavedrom_rejects_multiple_inputs() {
    let harness = Harness::new();
    let input_a = fixture_path("valid.tc");
    let input_b = fixture_path("valid.tc");
    let result = harness.run_wavedrom(&[path_as_str(&input_a), path_as_str(&input_b)]);
    assert_exit_code(&result, 1);
}

/// Scenario: 入力ファイル不在で終了コード 1
#[test]
fn wavedrom_missing_input_exits_1() {
    let harness = Harness::new();
    let output = harness.output_path("never.json");
    let result = harness.run_wavedrom(&["does-not-exist.tc", "-o", path_as_str(&output)]);
    assert_exit_code(&result, 1);
}

/// Scenario: TCML パースエラーで終了コード 2
#[test]
fn wavedrom_parse_error_exits_2() {
    let harness = Harness::new();
    let input = fixture_path("invalid.tc");
    let output = harness.output_path("invalid.json");
    let result = harness.run_wavedrom(&[path_as_str(&input), "-o", path_as_str(&output)]);
    assert_exit_code(&result, 2);
}

/// Scenario: 出力先ディレクトリ書き込み不能で終了コード 3
#[test]
#[cfg(unix)]
fn wavedrom_unwritable_output_exits_3() {
    use std::os::unix::fs::PermissionsExt;
    let harness = Harness::new();
    let input = fixture_path("valid.tc");
    let readonly_dir = harness.output_path("readonly");
    std::fs::create_dir_all(&readonly_dir).expect("mkdir readonly");
    std::fs::set_permissions(&readonly_dir, std::fs::Permissions::from_mode(0o555))
        .expect("set readonly");
    let output = readonly_dir.join("out.json");
    let result = harness.run_wavedrom(&[path_as_str(&input), "-o", path_as_str(&output)]);
    assert_exit_code(&result, 3);
}

/// Scenario: フォント関連オプションは受け付けない (exit 1)
#[test]
fn wavedrom_rejects_font_option() {
    let harness = Harness::new();
    let input = fixture_path("valid.tc");
    let result = harness.run_wavedrom(&[path_as_str(&input), "--font", "/path/to/font.ttf"]);
    assert_exit_code(&result, 1);
}

/// Scenario: `docs/images/sample.tc` を変換しても valid JSON が生成される
#[test]
fn wavedrom_sample_tc_produces_valid_json() {
    let sample = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/images/sample.tc");
    if !sample.exists() {
        return;
    }
    let harness = Harness::new();
    let output = harness.output_path("sample.json");
    let result = harness.run_wavedrom(&[path_as_str(&sample), "-o", path_as_str(&output)]);
    assert_exit_code(&result, 0);
    let content = std::fs::read_to_string(&output).expect("read json");
    serde_json::from_str::<serde_json::Value>(&content)
        .unwrap_or_else(|error| panic!("output is not valid JSON: {error}"));
}

// ---------------------------------------------------------------------------
// Edge-case scenario from docs/tests/cli.feature.md (wavedrom regression).
// ---------------------------------------------------------------------------

#[test]
fn wavedrom_regression_mid_step_with_clock_auto() {
    let harness = Harness::new();
    let input = harness.output_path("regression.tc");
    std::fs::write(&input, "@step 10\n@clock(pos) clk\n@step 20\ndata ====\n").expect("write");
    let output = harness.output_path("regression.json");
    let result = harness.run_wavedrom(&[path_as_str(&input), "-o", path_as_str(&output)]);
    assert_exit_code(&result, 0);
    let content = std::fs::read_to_string(&output).expect("read json");
    let value: serde_json::Value =
        serde_json::from_str(&content).expect("output must be valid JSON");
    let signals = value["signal"].as_array().expect("signal must be array");
    let clk_wave = signals[0]["wave"].as_str().expect("clk wave");
    let data_wave = signals[1]["wave"].as_str().expect("data wave");
    assert!(
        clk_wave.starts_with('p'),
        "clk wave must start with p; got {clk_wave}"
    );
    assert!(
        data_wave.starts_with('='),
        "data wave must start with =; got {data_wave}"
    );
}
