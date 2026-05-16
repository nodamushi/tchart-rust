//! Clock-edge triangle marker synthesis.
//!
//! After stacking resolves all row geometries, this pass walks every
//! clock-decorated row and pushes one `EdgeMark` per matching transition
//! into `SignalRow.edge_marks`. The actual logic lives on
//! [`Line::fill_clock_edge_marks`] — a method on `Line`, not a free
//! function taking `&mut Line`.
//!
//! The clock-derived triangle markers live separately from `@->` arrows
//! and are rendered as `<polygon>` in the waveforms layer; they are not
//! pushed into `Annotations.arrows`.

use crate::line::Line;

/// Fill `SignalRow.edge_marks` for every clock-decorated row.
///
/// Must be called **after** `Line::stack_lines` so that `signal_box`
/// geometry is resolved.
pub(super) fn emit_clock_edge_marks(lines: &mut [Line]) {
    for line in lines.iter_mut() {
        line.fill_clock_edge_marks();
    }
}
