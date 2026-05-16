//! CLI-level error type with mapped process exit codes.
//!
//! Spec: `docs/spec/cli.md` §終了コード.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use tchart_core::errors::ParseError;
use tchart_core::layout::LayoutError;

/// TCML parse failure together with the input context needed by CLI / wasm
/// front ends to render the rustc-style 4-component error message
/// (`docs/spec/cli.md` §パースエラー出力形式).
///
/// Carries:
///
/// - the input file path; rendered as `<stdin>` when reading from standard
///   input (the CLI does not currently support stdin, so the no-path branch
///   is an internal fallback).
/// - the full TCML source text. The renderer slices this by line to produce
///   the snippet line.
/// - the raw [`ParseError`] from the parser.
///
/// Fields are private because `source` and `error` are semantically coupled
/// (the error's line / column index into `source`). The only construction
/// path is [`ParseFailure::from_file`].
#[derive(Debug)]
pub(crate) struct ParseFailure {
    /// Path of the input file. `None` denotes the standard-input pseudo-path.
    path: Option<PathBuf>,
    /// Full TCML source the parser was reading from.
    source: String,
    /// Parse error from `tchart_core::parser::parse`.
    error: ParseError,
}

impl ParseFailure {
    /// Attach `path` and the full TCML `source` to a [`ParseError`].
    pub(crate) fn from_file(path: &Path, source: String, error: ParseError) -> Self {
        Self {
            path: Some(path.to_path_buf()),
            source,
            error,
        }
    }

    /// Input file path that the failed parse was reading from. `None` denotes
    /// the standard-input pseudo-path.
    pub(crate) fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Full TCML source text the parser was reading from. The renderer slices
    /// this by line to produce the snippet line.
    pub(crate) fn source(&self) -> &str {
        &self.source
    }

    /// The underlying parser error.
    pub(crate) fn error(&self) -> &ParseError {
        &self.error
    }
}

impl std::fmt::Display for ParseFailure {
    /// Fallback formatting used only when the CLI top-level dispatcher chose
    /// not to render the rustc-style 4-component output. The CLI normally
    /// dispatches `CliError::Parse` through `format_parse_failure`
    /// (`parse_error_format` module) before falling through to this `Display`
    /// impl.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.error)
    }
}

impl std::error::Error for ParseFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

/// Where a write operation was targeting when it failed.
///
/// `OutputWrite` failures used to embed sentinel strings such as `<png-encode>`
/// in a `PathBuf`. Splitting them into a dedicated enum keeps the path-only
/// case as a real path and gives the encoder/intermediate stages distinct,
/// type-safe identities.
#[derive(Debug)]
pub(crate) enum OutputDestination {
    /// Write to a regular file at this path.
    File(PathBuf),
    /// Failure inside one of the PNG encoder stages (see [`PngEncodeStage`]).
    PngEncodeStage(PngEncodeStage),
    /// Write to standard output (`src -o -`).
    Stdout,
}

/// Identifies which stage of the PNG encoder produced an error.
///
/// These are not file paths; they exist solely to make `OutputWrite` errors
/// distinguishable when the failure happened during in-memory encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PngEncodeStage {
    /// `usvg` parse step (SVG → tree).
    UsvgParse,
    /// SVG size validation (width/height out of range).
    SvgSize,
    /// `tiny_skia::Pixmap` allocation.
    PixmapAlloc,
    /// `tiny_skia` pixmap-to-raw-PNG encoding step (before iTXt injection).
    PngEncode,
    /// `png` encoder failed to produce iTXt chunk metadata.
    Itxt,
    /// `png` decode while reading the rasterised PNG back for re-encoding.
    PngDecode,
    /// `png` decode while pulling the next frame.
    PngDecodeFrame,
    /// `png` encode while writing the PNG header.
    PngHeader,
    /// `png` encode while writing the pixel data.
    PngData,
}

/// Reasons font resolution / loading can fail.
#[derive(Debug, thiserror::Error)]
pub(crate) enum FontError {
    /// `--font` argument or `TCHART_FONT` env var pointed at a missing path.
    #[error("font file not found: {}", .0.display())]
    NotFound(PathBuf),
    /// Underlying I/O error while reading the font file.
    #[error("could not read font file {}: {source}", .path.display())]
    ReadFailed {
        path: PathBuf,
        source: std::io::Error,
    },
    /// `fontdue` rejected the font bytes.
    #[error("could not parse font: {0}")]
    ParseFailed(String),
    /// Loaded font has no family name (font is structurally invalid).
    #[error("loaded font has no family name")]
    NoFamilyName,
    /// No system font could be located via the candidate list.
    #[error("no system font found; specify --font <FILE> or set TCHART_FONT")]
    AutodetectFailed,
}

/// Reasons the `extract` subcommand can fail.
#[derive(Debug, thiserror::Error)]
pub(crate) enum ExtractError {
    /// SVG body was not valid UTF-8.
    #[error("SVG is not valid UTF-8")]
    SvgNotUtf8,
    /// `<tchart:source>` element is absent.
    #[error("no <tchart:source> element")]
    SvgNoSourceElement,
    /// `<tchart:source>` opens but never closes.
    #[error("unclosed <tchart:source>")]
    SvgUnclosedSource,
    /// PNG file failed to decode.
    #[error("invalid PNG: {0}")]
    PngInvalid(String),
    /// PNG decoded but `tchart-source` iTXt chunk was missing.
    #[error("no tchart-source text chunk")]
    PngNoSourceChunk,
    /// `tchart-source` iTXt chunk failed to decode as UTF-8.
    #[error("iTXt decode: {0}")]
    PngItxtDecode(String),
}

/// Errors surfaced by the CLI binary. Each variant maps to a fixed exit code
/// (see [`CliError::exit_code`] and `docs/spec/cli.md` §終了コード).
///
/// `OutputWrite`'s `destination` is split out into [`OutputDestination`]
/// rather than a `PathBuf` so that the variant carries the type of target it
/// was writing to (file / encoder stage) without sentinel strings.
///
/// `#[non_exhaustive]` is set even though this is a binary crate, because the
/// crate also exposes a `lib` target for integration tests. The attribute
/// prevents test code from exhaustively matching variants directly, pushing
/// tests toward the public constructor helpers and keeping the test surface
/// stable when new variants are added.
///
/// `ParseError` / `LayoutError` / `FontError` / `ExtractError` carry their
/// own context, so they convert into [`CliError`] automatically via
/// `#[from]`. `std::io::Error` deliberately does not — the CLI needs to know
/// which path was being read/written, so callers attach the path manually
/// via `map_err` (or [`CliError::input_read`] / [`CliError::output_write_file`])
/// rather than via a blanket `From<io::Error>` impl.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub(crate) enum CliError {
    /// `<INPUT>` was omitted on the command line (usage error).
    #[error("no input file given")]
    MissingInput,
    /// More than one `<INPUT>` was given to a single-input subcommand (usage error).
    #[error("too many input files: this subcommand accepts exactly one input")]
    TooManyInputs,
    /// `--font-size <SIZE>` was 0, negative, or non-finite (usage error).
    ///
    /// Spec: `docs/spec/cli.md` §`svg` / §`png` / §`batch` — `--font-size` must
    /// be a strictly positive finite value.
    #[error("invalid font size: {0} (must be a positive finite number)")]
    InvalidFontSize(f32),
    /// Input file could not be read.
    #[error("failed to read {}: {source}", .path.display())]
    InputRead {
        path: PathBuf,
        source: std::io::Error,
    },
    /// TCML parse error together with input context (path + full source).
    /// Context is required so that the CLI top-level dispatcher can render
    /// the rustc-style 4-component error format (`error:` header + ` --> `
    /// location line + snippet + caret) defined in `docs/spec/cli.md`
    /// §パースエラー出力形式.
    #[error("parse error: {0}")]
    Parse(ParseFailure),
    /// Layout produced an error (e.g. unresolved anchor). Layout failures
    /// share the "output error" exit code because the spec table does not
    /// have a dedicated layout slot — once layout fails the CLI cannot
    /// produce output, so it is surfaced as such.
    #[error("layout error: {0}")]
    Layout(#[from] LayoutError),
    /// Output write failure. `destination` records what we were writing to.
    #[error("failed to write {destination}: {source}")]
    OutputWrite {
        destination: OutputDestination,
        source: std::io::Error,
    },
    /// Font resolution / loading failure.
    #[error("font error: {0}")]
    Font(#[from] FontError),
    /// `src` / `source`: target file missing the embedded TCML payload, or
    /// unsupported format.
    #[error("extract error: {0}")]
    Extract(#[from] ExtractError),
    /// `batch`: two or more input files share the same stem, producing a
    /// colliding output name.
    #[error("output file name collision: multiple inputs share stem `{0}`")]
    StemCollision(String),
    /// `batch`: one or more files failed; reports success/failure counts.
    #[error(
        "batch: {success_count} succeeded, {failure_count} failed; \
         first failure: {}: {first_failure}",
        .first_failure_path.display()
    )]
    BatchPartialFailure {
        /// Number of files that were written successfully.
        success_count: usize,
        /// Number of files that failed.
        failure_count: usize,
        /// Path of the first failed input.
        first_failure_path: PathBuf,
        /// Error from the first failed input.
        first_failure: Box<CliError>,
    },
}

/// Process exit code class for [`CliError`]. The numeric value follows
/// `docs/spec/cli.md` §終了コード.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum CliExitCode {
    /// Input-side failure (missing file, missing argument).
    Input = 1,
    /// TCML parse failure.
    Parse = 2,
    /// Output-side failure (write, layout-stage failure).
    Output = 3,
    /// Font resolution / loading failure.
    Font = 4,
    /// `extract`: missing payload or unsupported format.
    Extract = 5,
}

impl CliError {
    /// Construct an [`InputRead`](CliError::InputRead) for `path`.
    pub(crate) fn input_read(path: &Path, source: std::io::Error) -> Self {
        CliError::InputRead {
            path: path.to_path_buf(),
            source,
        }
    }

    /// Wrap a [`ParseError`] with the file context needed for the rustc-style
    /// formatter. Used by every subcommand wrapper that calls
    /// `tchart_core::parser::parse`; the resulting `CliError::Parse` carries
    /// the source path, the full TCML text, and the underlying parse error.
    pub(crate) fn parse_with_file(path: &Path, source: String, error: ParseError) -> Self {
        CliError::Parse(ParseFailure::from_file(path, source, error))
    }

    /// Construct an [`OutputWrite`](CliError::OutputWrite) for a regular file path.
    pub(crate) fn output_write_file(path: &Path, source: std::io::Error) -> Self {
        CliError::OutputWrite {
            destination: OutputDestination::File(path.to_path_buf()),
            source,
        }
    }

    /// Construct an [`OutputWrite`](CliError::OutputWrite) for stdout (`src -o -`).
    pub(crate) fn output_write_stdout(source: std::io::Error) -> Self {
        CliError::OutputWrite {
            destination: OutputDestination::Stdout,
            source,
        }
    }

    /// Construct an [`OutputWrite`](CliError::OutputWrite) for a PNG encoder stage.
    pub(crate) fn output_write_png_stage(
        stage: PngEncodeStage,
        message: impl Into<String>,
    ) -> Self {
        CliError::OutputWrite {
            destination: OutputDestination::PngEncodeStage(stage),
            source: std::io::Error::other(message.into()),
        }
    }

    /// Construct a [`BatchPartialFailure`](CliError::BatchPartialFailure).
    pub(crate) fn batch_partial_failure(
        success_count: usize,
        failure_count: usize,
        first_failure_path: &Path,
        first_failure: CliError,
    ) -> Self {
        CliError::BatchPartialFailure {
            success_count,
            failure_count,
            first_failure_path: first_failure_path.to_path_buf(),
            first_failure: Box::new(first_failure),
        }
    }

    /// Process exit code class for this error.
    ///
    /// `BatchPartialFailure` inherits the exit code class of its first failed
    /// input so that the per-file failure category (parse error, input error,
    /// etc.) is preserved end-to-end. The spec's exit-code table classifies
    /// failures by their origin (input vs parse vs output), not by whether
    /// they were observed inside a batch run.
    ///
    /// Recursion safety: the recursive call on `first_failure` terminates in
    /// one step because `BatchPartialFailure` is only constructed by the
    /// `batch` subcommand from per-file errors, and per-file errors never
    /// themselves carry a `BatchPartialFailure` (batches do not nest). Callers
    /// that wrap a `BatchPartialFailure` inside another `BatchPartialFailure`
    /// would still terminate (the inner one resolves recursively), but no such
    /// caller exists in the current code base.
    pub(crate) fn exit_code(&self) -> CliExitCode {
        match self {
            CliError::MissingInput
            | CliError::TooManyInputs
            | CliError::InputRead { .. }
            | CliError::InvalidFontSize(_) => CliExitCode::Input,
            CliError::Parse(_) => CliExitCode::Parse,
            CliError::Layout(_) | CliError::OutputWrite { .. } | CliError::StemCollision(_) => {
                CliExitCode::Output
            }
            CliError::BatchPartialFailure { first_failure, .. } => first_failure.exit_code(),
            CliError::Font(_) => CliExitCode::Font,
            CliError::Extract(_) => CliExitCode::Extract,
        }
    }
}

impl std::fmt::Display for OutputDestination {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputDestination::File(path) => write!(formatter, "{}", path.display()),
            OutputDestination::PngEncodeStage(stage) => {
                let label = match stage {
                    PngEncodeStage::UsvgParse => "usvg-parse",
                    PngEncodeStage::SvgSize => "svg-size",
                    PngEncodeStage::PixmapAlloc => "pixmap",
                    PngEncodeStage::PngEncode => "png-encode",
                    PngEncodeStage::Itxt => "itxt",
                    PngEncodeStage::PngDecode => "png-decode",
                    PngEncodeStage::PngDecodeFrame => "png-decode-frame",
                    PngEncodeStage::PngHeader => "png-header",
                    PngEncodeStage::PngData => "png-data",
                };
                write!(formatter, "<{label}>")
            }
            OutputDestination::Stdout => write!(formatter, "<stdout>"),
        }
    }
}

impl From<CliExitCode> for ExitCode {
    fn from(code: CliExitCode) -> ExitCode {
        ExitCode::from(code as u8)
    }
}

impl From<&CliError> for ExitCode {
    fn from(error: &CliError) -> ExitCode {
        ExitCode::from(error.exit_code())
    }
}
