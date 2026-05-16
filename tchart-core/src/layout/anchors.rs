//! Resolve `AnchorRegistry` entries to absolute `Px` coordinates and rewrite
//! `Arrow` endpoints accordingly.
//!
//! See `docs/spec/types.md` §3.2.x and §5.

use crate::anchor::AnchorRegistry;
use crate::arrow::{Arrow, ArrowEnd};
use crate::geometry::Point;
use crate::layout::errors::LayoutError;
use crate::line::{Line, LineContent, SignalLevel, SignalRow, WaveformElement};
use crate::style::LayoutParams;
use crate::units::Px;

/// Resolve every `Inline` anchor to its `(x, y)` chart-coordinate position.
///
/// Anchor x coordinates are computed using each signal row's own
/// `layout_params` snapshot so that per-row `@step` / `@slant` directives
/// affect the anchors on the rows that follow them, not the chart-wide
/// final value.
pub(super) fn resolve_inline_anchors(registry: &mut AnchorRegistry, lines: &[Line]) {
    for resolved in registry.iter_resolved_mut() {
        let signal_index = resolved.signal_index;
        let element_index = resolved.element_index;
        let Some(row) = find_signal_row_at(lines, signal_index) else {
            continue;
        };
        if let Some(point) = compute_anchor_point(lines, signal_index, row, element_index) {
            resolved.set_position(point);
        }
    }
}

fn find_signal_row_at(lines: &[Line], index: usize) -> Option<&SignalRow> {
    let line = lines.get(index)?;
    match &line.content {
        LineContent::Signal(row) => Some(row),
        _ => None,
    }
}

fn compute_anchor_point(
    lines: &[Line],
    signal_index: usize,
    row: &SignalRow,
    element_index: usize,
) -> Option<Point> {
    let line = lines.get(signal_index)?;
    let signal_box = row.geometry().signal_box;
    let signal_origin = line.bounding_box.origin + signal_box.origin;
    let cumulative_x = compute_cumulative_width(row.waveform(), element_index, row.layout_params());
    let level = previous_level(row.waveform(), element_index);
    let y_offset = compute_anchor_y_offset(level, signal_box.size.height);
    Some(Point {
        x: signal_origin.x + cumulative_x,
        y: signal_origin.y + y_offset,
    })
}

fn compute_cumulative_width(
    elements: &[WaveformElement],
    upto_exclusive: usize,
    params: &LayoutParams,
) -> Px {
    params.sum_element_widths(&elements[..upto_exclusive.min(elements.len())])
}

fn previous_level(elements: &[WaveformElement], upto_exclusive: usize) -> Option<SignalLevel> {
    elements
        .iter()
        .take(upto_exclusive)
        .rev()
        .find_map(|element| match element {
            WaveformElement::Level(run) => Some(run.level()),
            _ => None,
        })
}

fn compute_anchor_y_offset(level: Option<SignalLevel>, body_height: Px) -> Px {
    match level {
        Some(SignalLevel::Low | SignalLevel::DontCareAlongLow) => body_height,
        Some(SignalLevel::High | SignalLevel::DontCareAlongHigh) => Px::ZERO,
        Some(
            SignalLevel::HiZ
            | SignalLevel::DontCareAlongHiZ
            | SignalLevel::Bus
            | SignalLevel::DontCareAlongBus,
        )
        | None => body_height * 0.5,
    }
}

/// Replace each `ArrowEnd::Anchor` with `ArrowEnd::Absolute` using the
/// resolved registry, returning [`LayoutError::UnresolvedAnchor`] on miss.
pub(super) fn rewrite_arrows(
    arrows: &mut [Arrow],
    registry: &AnchorRegistry,
) -> Result<(), LayoutError> {
    for arrow in arrows.iter_mut() {
        let resolved_from = resolve_endpoint(&arrow.from, registry)?;
        let resolved_to = resolve_endpoint(&arrow.to, registry)?;
        arrow.set_from(resolved_from);
        arrow.set_to(resolved_to);
    }
    Ok(())
}

fn resolve_endpoint(end: &ArrowEnd, registry: &AnchorRegistry) -> Result<ArrowEnd, LayoutError> {
    match end {
        ArrowEnd::Absolute(point) => Ok(ArrowEnd::Absolute(*point)),
        ArrowEnd::Anchor(id) => {
            let point = registry
                .lookup_position(id)
                .ok_or(LayoutError::UnresolvedAnchor)?;
            Ok(ArrowEnd::Absolute(point))
        }
    }
}
