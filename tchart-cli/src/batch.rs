//! Batch subcommand: per-file independent worker model.
//!
//! Each input file is processed by an independent worker that executes the
//! full pipeline — parse, font resolution, layout, render, write — without
//! waiting for any other worker to complete any stage.
//!
//! A `SharedFontCache` is shared across all workers so that each font file is
//! loaded from disk at most once across the whole batch run.
//!
//! See `docs/spec/cli.md` §並列ワーカと共有フォントキャッシュ.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::vec::IntoIter;

use tchart_core::layout::layout;
use tchart_core::parser::parse;
use tchart_core::svg::render;
use tchart_core::units::Px;

use crate::cli::{BatchArgs, BatchFormat};
use crate::error::CliError;
use crate::font::{SharedFontCache, WorkerFontContext, extract_font_families, resolve_font_path};
use crate::render::build_png_bytes;
use crate::validate::validate_font_size;

/// Entry point for the `batch` subcommand.
pub(crate) fn run_batch(args: BatchArgs) -> Result<(), CliError> {
    let font_size = validate_font_size(args.font_size)?;
    let output_dir = &args.output;
    ensure_output_dir(output_dir)?;
    check_stem_collisions(&args.inputs)?;

    let font_path = resolve_font_path(args.font.as_deref())?;
    let cache = Arc::new(SharedFontCache::new(&font_path)?);
    run_workers(&args.inputs, font_size, args.format, output_dir, &cache)
}

/// Validate that the input paths do not share any `<STEM>` that would produce
/// colliding output file names.
fn check_stem_collisions(inputs: &[PathBuf]) -> Result<(), CliError> {
    let mut seen: HashSet<String> = HashSet::new();
    for path in inputs {
        let stem = stem_of(path);
        if !seen.insert(stem.clone()) {
            return Err(CliError::StemCollision(stem));
        }
    }
    Ok(())
}

/// Extract the file stem (without extension) as a `String`.
fn stem_of(path: &Path) -> String {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("")
        .to_owned()
}

fn ensure_output_dir(dir: &Path) -> Result<(), CliError> {
    if dir.is_dir() {
        return Ok(());
    }
    std::fs::create_dir_all(dir).map_err(|error| CliError::output_write_file(dir, error))
}

/// Shared work queue used by all workers in one batch run.
struct WorkQueue {
    inputs: Mutex<IntoIter<PathBuf>>,
    /// Errors collected from each worker: `(input_path, error)`.
    errors: Mutex<Vec<(PathBuf, CliError)>>,
    /// Count of successfully written outputs.
    success_count: Mutex<usize>,
}

impl WorkQueue {
    fn new(inputs: Vec<PathBuf>) -> Self {
        Self {
            inputs: Mutex::new(inputs.into_iter()),
            errors: Mutex::new(Vec::new()),
            success_count: Mutex::new(0),
        }
    }

    fn next_input(&self) -> Option<PathBuf> {
        self.inputs
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .next()
    }

    fn record_error(&self, path: PathBuf, error: CliError) {
        self.errors
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push((path, error));
    }

    fn record_success(&self) {
        *self
            .success_count
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) += 1;
    }

    fn into_batch_result(self) -> Result<(), CliError> {
        let errors = self
            .errors
            .into_inner()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let success_count = self
            .success_count
            .into_inner()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if errors.is_empty() {
            return Ok(());
        }
        let failure_count = errors.len();
        let (first_path, first_error) = errors.into_iter().next().expect("non-empty errors");
        Err(CliError::batch_partial_failure(
            success_count,
            failure_count,
            &first_path,
            first_error,
        ))
    }
}

/// Spawn one thread per logical CPU, each pulling work from the shared queue.
///
/// Each worker processes one document at a time through the full pipeline
/// (parse → font resolution → layout → render → write to disk) before picking
/// up the next.  No inter-stage synchronisation is needed because each worker
/// owns its single in-progress document entirely.
///
/// All workers run to completion regardless of individual failures.  After the
/// scope joins, errors are aggregated and a single summary error is returned
/// when any worker failed.
fn run_workers(
    inputs: &[PathBuf],
    font_size_override: Option<f32>,
    format: BatchFormat,
    output_dir: &Path,
    cache: &Arc<SharedFontCache>,
) -> Result<(), CliError> {
    let available_parallelism = std::thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(1);
    let queue = Arc::new(WorkQueue::new(inputs.to_vec()));

    std::thread::scope(|scope| {
        for _ in 0..available_parallelism {
            let queue = Arc::clone(&queue);
            let cache = Arc::clone(cache);
            scope.spawn(move || {
                while let Some(path) = queue.next_input() {
                    match render_input(&path, font_size_override, format, output_dir, &cache) {
                        Ok(()) => queue.record_success(),
                        Err(error) => queue.record_error(path, error),
                    }
                }
            });
        }
    });

    Arc::try_unwrap(queue)
        .unwrap_or_else(|_| unreachable!("scope joined all threads"))
        .into_batch_result()
}

/// Full pipeline for one input file: parse → font resolution → layout →
/// render → write to `output_dir/<STEM>.<ext>`.
///
/// The worker holds exactly one `ChartDocument` at a time; it is dropped
/// before bytes are written to disk.
fn render_input(
    path: &Path,
    font_size_override: Option<f32>,
    format: BatchFormat,
    output_dir: &Path,
    cache: &SharedFontCache,
) -> Result<(), CliError> {
    let source =
        std::fs::read_to_string(path).map_err(|error| CliError::input_read(path, error))?;
    let mut document =
        parse(&source).map_err(|error| CliError::parse_with_file(path, source.clone(), error))?;
    if let Some(size) = font_size_override {
        document.set_font_size(Px(size));
    }

    let mut context = WorkerFontContext::new(cache);
    for family_csv in extract_font_families(&source) {
        context.add_family_csv(&family_csv);
    }

    layout(&mut document, &context)?;
    let svg_markup = render(&document, &context);
    // Drop the document before producing the output bytes so only one
    // document is live in this worker at a time.
    drop(document);

    let stem = stem_of(path);
    let extension = match format {
        BatchFormat::Svg => "svg",
        BatchFormat::Png => "png",
    };
    let output_path = output_dir.join(format!("{stem}.{extension}"));
    let bytes = match format {
        BatchFormat::Svg => svg_markup.into_bytes(),
        BatchFormat::Png => {
            let fontdb = context.build_fontdb();
            build_png_bytes(&svg_markup, &source, fontdb)?
        }
    };
    std::fs::write(&output_path, &bytes)
        .map_err(|error| CliError::output_write_file(&output_path, error))
}
