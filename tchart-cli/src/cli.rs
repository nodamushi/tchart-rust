//! CLI argument structures (clap).
//!
//! See `docs/spec/cli.md`.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};

use tchart_core::parser::parse;
use tchart_core::wavedrom::to_wavejson;

use crate::batch::run_batch;
use crate::error::CliError;
use crate::extract;
use crate::font::resolve_font_path;
use crate::parse_error_format::format_parse_failure;
use crate::render::render_single;
use crate::validate::validate_font_size;

/// Root CLI parse result. Fields are public per the CLI parse result exception.
#[derive(Parser, Debug)]
#[command(name = "tchart", version, about = "TCML chart renderer")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

/// Dispatch the parsed CLI to the appropriate subcommand, print any error to
/// stderr, and return a process exit code.
///
/// `CliError::Parse` is written as the rustc-style 4-component message
/// (`error:` / ` --> ` location / snippet / caret) defined in
/// `docs/spec/cli.md` §パースエラー出力形式. Every other variant keeps the
/// pre-existing one-line `tchart: <message>` form so input / output / font
/// errors remain unchanged.
///
/// Exit code semantics are defined in `docs/spec/cli.md` §終了コード.
pub fn dispatch(cli: Cli) -> ExitCode {
    let result = match cli.command {
        Command::Svg(args) => run_svg(args),
        Command::Png(args) => run_png(args),
        Command::Src(args) | Command::Source(args) => run_src(args),
        Command::Wavedrom(args) => run_wavedrom(args),
        Command::Batch(args) => run_batch(args),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            match &error {
                CliError::Parse(failure) => eprint!("{}", format_parse_failure(failure)),
                other => eprintln!("tchart: {other}"),
            }
            ExitCode::from(&error)
        }
    }
}

/// Subcommand selector.
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Render TCML to SVG.
    Svg(SvgArgs),
    /// Render TCML to PNG (rasterised via resvg).
    Png(PngArgs),
    /// Extract embedded TCML source from a tchart-generated SVG or PNG file.
    Src(SrcArgs),
    /// Alias for `src`.
    Source(SrcArgs),
    /// Convert TCML to WaveDrom (WaveJSON) format.
    Wavedrom(WavedromArgs),
    /// Batch-render multiple TCML inputs in parallel.
    Batch(BatchArgs),
}

/// Arguments for `tchart svg`.
///
/// Fields are public as this is a CLI parse result (clap derive).
#[derive(Parser, Debug)]
pub struct SvgArgs {
    /// Input TCML file(s). Only one is accepted; providing two or more is an error.
    #[arg(value_name = "INPUT", num_args = 1..)]
    pub inputs: Vec<PathBuf>,

    /// Output file path. Defaults to `<STEM>.svg` next to the input file.
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Font file (.ttf / .otf). Falls back to `TCHART_FONT` env var or system search.
    #[arg(long, value_name = "FILE")]
    pub font: Option<PathBuf>,

    /// Override the default font size (px).
    #[arg(long, value_name = "SIZE")]
    pub font_size: Option<f32>,
}

/// Arguments for `tchart png`.
///
/// Fields are public as this is a CLI parse result (clap derive).
#[derive(Parser, Debug)]
pub struct PngArgs {
    /// Input TCML file(s). Only one is accepted; providing two or more is an error.
    #[arg(value_name = "INPUT", num_args = 1..)]
    pub inputs: Vec<PathBuf>,

    /// Output file path. Defaults to `<STEM>.png` next to the input file.
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Font file (.ttf / .otf). Falls back to `TCHART_FONT` env var or system search.
    #[arg(long, value_name = "FILE")]
    pub font: Option<PathBuf>,

    /// Override the default font size (px).
    #[arg(long, value_name = "SIZE")]
    pub font_size: Option<f32>,
}

/// Arguments for `tchart src` / `tchart source`.
///
/// Fields are public as this is a CLI parse result (clap derive).
#[derive(Parser, Debug)]
pub struct SrcArgs {
    /// Input SVG or PNG file(s). Only one is accepted; providing two or more is an error.
    #[arg(value_name = "FILE", num_args = 1..)]
    pub inputs: Vec<PathBuf>,

    /// Output file path. Defaults to `<STEM>.tc` next to the input file.
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,
}

/// Arguments for `tchart wavedrom`.
///
/// Fields are public as this is a CLI parse result (clap derive).
/// `--font` and `--font-size` are intentionally absent: WaveDrom output
/// contains no font information.
#[derive(Parser, Debug)]
pub struct WavedromArgs {
    /// Input TCML file(s). Only one is accepted; providing two or more is an error.
    #[arg(value_name = "INPUT", num_args = 1..)]
    pub inputs: Vec<PathBuf>,

    /// Output file path. Defaults to `<STEM>.json` next to the input file.
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,
}

/// Output format for `batch`.
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum BatchFormat {
    /// SVG markup.
    Svg,
    /// Rasterised PNG.
    Png,
}

/// Arguments for `tchart batch`.
///
/// Fields are public as this is a CLI parse result (clap derive).
#[derive(Parser, Debug)]
pub struct BatchArgs {
    /// Output format: `svg` or `png`.
    #[arg(value_name = "svg|png")]
    pub format: BatchFormat,

    /// Input TCML files (.tc). One or more required.
    #[arg(value_name = "INPUT", num_args = 1..)]
    pub inputs: Vec<PathBuf>,

    /// Output directory. Each input is written as `<DIR>/<STEM>.<ext>`.
    #[arg(short, long, value_name = "DIR", required = true)]
    pub output: PathBuf,

    /// Font file (.ttf / .otf). Falls back to `TCHART_FONT` env var or system search.
    #[arg(long, value_name = "FILE")]
    pub font: Option<PathBuf>,

    /// Override the default font size (px).
    #[arg(long, value_name = "SIZE")]
    pub font_size: Option<f32>,
}

fn run_svg(args: SvgArgs) -> Result<(), CliError> {
    let font_size = validate_font_size(args.font_size)?;
    let input = require_single_input(&args.inputs)?;
    let source = read_input_file(input)?;
    let font_path = resolve_font_path(args.font.as_deref())?;
    let output_path = args
        .output
        .unwrap_or_else(|| compute_default_output_path(input, "svg"));
    let rendered = render_single(input, source, &font_path, font_size)?;
    write_output(&output_path, &rendered.into_svg_bytes())
}

fn run_png(args: PngArgs) -> Result<(), CliError> {
    let font_size = validate_font_size(args.font_size)?;
    let input = require_single_input(&args.inputs)?;
    let source = read_input_file(input)?;
    let font_path = resolve_font_path(args.font.as_deref())?;
    let output_path = args
        .output
        .unwrap_or_else(|| compute_default_output_path(input, "png"));
    let rendered = render_single(input, source, &font_path, font_size)?;
    let bytes = rendered.into_png_bytes()?;
    write_output(&output_path, &bytes)
}

fn run_src(args: SrcArgs) -> Result<(), CliError> {
    let input = require_single_input(&args.inputs)?;
    let source = extract::read_embedded_source(input)?;
    let bytes = source.as_bytes();
    // `-o -` (single hyphen) is the canonical stdout marker for the `src`
    // subcommand. See `docs/spec/cli.md` for the stdout-marker rule.
    if args.output.as_deref().is_some_and(is_stdout_marker) {
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        return handle
            .write_all(bytes)
            .and_then(|()| handle.flush())
            .map_err(CliError::output_write_stdout);
    }
    let output_path = args
        .output
        .unwrap_or_else(|| compute_default_output_path(input, "tc"));
    write_output(&output_path, bytes)
}

/// Whether `path` is exactly the single-hyphen stdout marker (`-`).
///
/// We compare against the exact `OsStr` `"-"` so that a regular file named
/// `-`, when passed as e.g. `./-`, is still treated as a file path. This
/// matches the convention used by most POSIX tools.
fn is_stdout_marker(path: &Path) -> bool {
    path.as_os_str() == "-"
}

fn run_wavedrom(args: WavedromArgs) -> Result<(), CliError> {
    let input = require_single_input(&args.inputs)?;
    let source = read_input_file(input)?;
    let output_path = args
        .output
        .unwrap_or_else(|| compute_default_output_path(input, "json"));
    let document =
        parse(&source).map_err(|error| CliError::parse_with_file(input, source.clone(), error))?;
    let (json, warnings) = to_wavejson(&document);
    for warning in &warnings {
        eprintln!("{warning}");
    }
    write_output(&output_path, json.as_bytes())
}

/// Enforce the single-input constraint for `svg`, `png`, and `src`.
///
/// Returns a reference to the sole path, or an error when zero or more than
/// one path was supplied.
fn require_single_input(inputs: &[PathBuf]) -> Result<&Path, CliError> {
    match inputs {
        [single] => Ok(single.as_path()),
        [] => Err(CliError::MissingInput),
        _ => Err(CliError::TooManyInputs),
    }
}

fn read_input_file(path: &Path) -> Result<String, CliError> {
    std::fs::read_to_string(path).map_err(|source| CliError::input_read(path, source))
}

fn write_output(path: &Path, bytes: &[u8]) -> Result<(), CliError> {
    std::fs::write(path, bytes).map_err(|source| CliError::output_write_file(path, source))
}

/// Compute the default output path: replace the input extension with `new_ext`,
/// or append it when the input has no extension.
///
/// The spec defines: "入力隣に `<STEM>.<拡張子>`".
fn compute_default_output_path(input: &Path, new_ext: &str) -> PathBuf {
    input.with_extension(new_ext)
}
