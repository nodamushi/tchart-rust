//! WaveDrom (WaveJSON) export for TCML chart documents.
//!
//! See `docs/spec/wavedrom.md` for the full mapping specification.
//!
//! # Usage
//!
//! ```rust,ignore
//! let (json, warnings) = tchart_core::wavedrom::to_wavejson(&document);
//! for warning in &warnings {
//!     eprintln!("{warning}");
//! }
//! println!("{json}");
//! ```

mod edge;
mod node;
mod period;
mod signal;
pub(crate) mod warning;

#[cfg(test)]
mod tests;

use serde_json::{Map, Value};

use crate::document::ChartDocument;
use crate::line::{LineContent, SignalRow};

pub use warning::WaveDromWarning;

/// Convert a [`ChartDocument`] to a WaveJSON strict JSON string.
///
/// Returns `(json_string, warnings)`. Warnings should be printed to stderr by
/// the caller; they do not affect the validity of the output JSON.
///
/// See `docs/spec/wavedrom.md`.
pub fn to_wavejson(document: &ChartDocument) -> (String, Vec<WaveDromWarning>) {
    let (node_map, node_warning) = node::build_node_map(document);
    let (root, mut warnings) = build_root(document, &node_map);
    if let Some(warning) = node_warning {
        warnings.insert(0, warning);
    }
    let json = serde_json::to_string(&root).expect("serde_json serialization cannot fail");
    (json, warnings)
}

/// Build the top-level WaveJSON [`Value`] together with the warnings it
/// accumulated while walking the document.
fn build_root(document: &ChartDocument, node_map: &node::NodeMap) -> (Value, Vec<WaveDromWarning>) {
    let SignalArrayResult {
        signal_array,
        mut warnings,
    } = build_signal_array(document, node_map);

    let mut root = Map::new();
    root.insert("signal".to_owned(), Value::Array(signal_array));
    let HeadEntry { entry, warning } = head_entry(document);
    if let Some((key, value)) = entry {
        root.insert(key, value);
    }
    if let Some(warning) = warning {
        warnings.push(warning);
    }
    if let Some((key, value)) = edge_entry(document, node_map) {
        root.insert(key, value);
    }
    (Value::Object(root), warnings)
}

/// `(head.text, warning)` pair returned by [`head_entry`].
///
/// `entry` is the `"head"` key/value to splice into the root object when the
/// document has a `@title`. `warning` carries the "additional titles dropped"
/// signal so the caller can append it to the warning list without the
/// function touching that list directly.
struct HeadEntry {
    entry: Option<(String, Value)>,
    warning: Option<WaveDromWarning>,
}

/// Produce the `head.text` object when the document has a `@title`, and the
/// "additional titles dropped" warning when later titles were ignored.
fn head_entry(document: &ChartDocument) -> HeadEntry {
    let (head_text, warning) = collect_head_text(document);
    let entry = head_text.map(|text| {
        let mut head = Map::new();
        head.insert("text".to_owned(), Value::String(text));
        ("head".to_owned(), Value::Object(head))
    });
    HeadEntry { entry, warning }
}

/// Produce the `edge` array entry when the document contains any `@->` arrows.
fn edge_entry(document: &ChartDocument, node_map: &node::NodeMap) -> Option<(String, Value)> {
    let edges = edge::build_edges(document, node_map);
    (!edges.is_empty()).then(|| {
        (
            "edge".to_owned(),
            Value::Array(edges.into_iter().map(Value::String).collect()),
        )
    })
}

/// Bundled result of [`build_signal_array`] so callers can read the signal
/// list and any warnings without inspecting a tuple's positional fields.
/// Holds a single computed result, no inter-field invariants, no methods
/// beyond field reads.
struct SignalArrayResult {
    signal_array: Vec<Value>,
    warnings: Vec<WaveDromWarning>,
}

/// Build the JSON `signal` array from all document lines.
fn build_signal_array(document: &ChartDocument, node_map: &node::NodeMap) -> SignalArrayResult {
    let (step_integers, warnings) = collect_step_integers(document);
    let period_divisor = period::compute_divisor(&step_integers);
    let signal_index_by_line = build_signal_index_map(document);
    let signal_array = convert_lines_to_values(
        document,
        &step_integers,
        period_divisor,
        node_map,
        &signal_index_by_line,
    );
    SignalArrayResult {
        signal_array,
        warnings,
    }
}

/// Collect rounded step integers for every signal row (in document order).
///
/// Returns `(step_integers, warnings)`. Each signal that required rounding
/// contributes at most one [`WaveDromWarning::StepRounded`] entry.
///
/// Per-row `@step` snapshots are used so that a chart with rows at different
/// step values produces the correct `period` ratios in the WaveDrom output.
fn collect_step_integers(document: &ChartDocument) -> (Vec<u32>, Vec<WaveDromWarning>) {
    let (integers, maybe_warnings): (Vec<u32>, Vec<Option<WaveDromWarning>>) = document
        .lines
        .iter()
        .filter_map(|line| {
            if let LineContent::Signal(row) = &line.content {
                let name = row.name().flatten_to_string();
                let row_step = row.layout_params().step();
                Some(period::round_step(row_step, &name))
            } else {
                None
            }
        })
        .unzip();
    let warnings: Vec<WaveDromWarning> = maybe_warnings.into_iter().flatten().collect();
    (integers, warnings)
}

/// Map each document line to its 0-based signal index (`None` for non-signal lines).
///
/// The resulting `Vec` is parallel to `document.lines`. State is threaded via
/// `Iterator::scan` so the closure has no captured mutable variable.
fn build_signal_index_map(document: &ChartDocument) -> Vec<Option<usize>> {
    document
        .lines
        .iter()
        .scan(0usize, |count, line| {
            if matches!(&line.content, LineContent::Signal(_)) {
                let index = *count;
                *count += 1;
                Some(Some(index))
            } else {
                Some(None)
            }
        })
        .collect()
}

/// Walk document lines and produce the JSON `signal` array.
fn convert_lines_to_values(
    document: &ChartDocument,
    step_integers: &[u32],
    period_divisor: u32,
    node_map: &node::NodeMap,
    signal_index_by_line: &[Option<usize>],
) -> Vec<Value> {
    document
        .lines
        .iter()
        .zip(signal_index_by_line.iter())
        .filter_map(|(line, signal_index_opt)| match &line.content {
            LineContent::Signal(row) => {
                let Some(signal_index) = signal_index_opt else {
                    unreachable!(
                        "build_signal_index_map assigns Some for every Signal line; \
                         signal_index_by_line is built from the same document"
                    );
                };
                Some(build_signal_value(
                    row,
                    step_integers[*signal_index],
                    period_divisor,
                    node_map,
                ))
            }
            LineContent::Skip(_) => Some(Value::Object(Map::new())),
            LineContent::Title(_) => None,
        })
        .collect()
}

/// Build the JSON object for one signal row.
fn build_signal_value(
    row: &SignalRow,
    step_integer: u32,
    period_divisor: u32,
    node_map: &node::NodeMap,
) -> Value {
    let period_value = period::signal_period(step_integer, period_divisor);
    let name = row.name().flatten_to_string();
    signal::build_signal_object(row, &name, period_value, node_map)
}

/// Return the text of the first `@title` row in the document along with a
/// warning if additional `@title` rows were present and dropped. WaveDrom's
/// `head.text` is a single string, so any title rows past the first are
/// information loss; we surface that via [`WaveDromWarning::AdditionalTitlesDropped`]
/// instead of silently discarding them. See `docs/spec/wavedrom.md` §警告.
fn collect_head_text(document: &ChartDocument) -> (Option<String>, Option<WaveDromWarning>) {
    let mut titles = document
        .lines
        .iter()
        .filter(|line| matches!(line.content, LineContent::Title(_)));
    let Some(first_line) = titles.next() else {
        return (None, None);
    };
    let LineContent::Title(first_title) = &first_line.content else {
        unreachable!("filter ensures Title variant");
    };
    let head_text = title_row_text(first_title);
    let dropped_count = titles.count();
    let warning =
        (dropped_count > 0).then_some(WaveDromWarning::AdditionalTitlesDropped { dropped_count });
    (Some(head_text), warning)
}

/// Join a `TitleRow`'s internal text lines (from `"..."`-quoted multi-line
/// titles) with `\n`. Single-line titles return their text unchanged.
fn title_row_text(title: &crate::line::TitleRow) -> String {
    let mut text_lines = title.text.lines().map(|line| line.unsafe_text());
    let Some(first) = text_lines.next() else {
        return String::new();
    };
    text_lines.fold(first.to_owned(), |mut accumulated, item| {
        accumulated.push('\n');
        accumulated.push_str(item);
        accumulated
    })
}
