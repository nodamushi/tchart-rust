//! Row-level default styles and per-row composite style.

use crate::style::label::LabelStyle;
use crate::style::signal::SignalStyle;
use crate::units::Px;

/// Default styles used for newly constructed rows when no per-row override
/// is supplied.
///
/// See `docs/spec/types.md` §4.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct DefaultRowStyles {
    signal: SignalStyle,
    label: LabelStyle,
}

impl DefaultRowStyles {
    /// Default style for signal rows.
    pub(crate) fn signal(&self) -> &SignalStyle {
        &self.signal
    }

    /// Default style for signal-row labels.
    pub(crate) fn label(&self) -> &LabelStyle {
        &self.label
    }

    /// Mutable access to the signal default style (used by the parser).
    pub(super) fn signal_mut(&mut self) -> &mut SignalStyle {
        &mut self.signal
    }

    /// Mutable access to the label default style (used by the parser).
    pub(super) fn label_mut(&mut self) -> &mut LabelStyle {
        &mut self.label
    }

    /// Update the label default font family.
    pub(super) fn set_label_font_family(&mut self, family: crate::text::FontFamily) {
        self.label.set_font_family(family);
    }
}

/// Composite style attached to a single signal row.
///
/// See `docs/spec/types.md` §3.2.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SignalRowStyle {
    signal: SignalStyle,
    label: LabelStyle,
}

impl SignalRowStyle {
    /// Construct a [`SignalRowStyle`] from its two component styles.
    pub(crate) fn new(signal: SignalStyle, label: LabelStyle) -> Self {
        Self { signal, label }
    }

    /// Stroke / fill style for the waveform.
    pub(crate) fn signal(&self) -> &SignalStyle {
        &self.signal
    }

    /// Style applied to the signal name label.
    pub(crate) fn label(&self) -> &LabelStyle {
        &self.label
    }

    /// Propagate a CLI/WASM `--font-size` override into the per-row label
    /// snapshot. SignalStyle has no font of its own, so only the label is
    /// updated.
    pub(crate) fn set_font_size(&mut self, size: Px) {
        self.label.set_font_size(size);
    }
}
