//! SignalRow → WaveDrom signal object conversion.
//!
//! See `docs/spec/wavedrom.md` §信号オブジェクト and §wave 文字列マッピング.

use serde_json::{Map, Value};

use crate::clock::{ClockEdge, ClockPulse};
use crate::line::{LevelRun, SignalLevel, SignalRow, WaveformElement};
use crate::text::UserText;

use super::node::{self, NodeMap};

/// Build the WaveDrom signal JSON object for one [`SignalRow`].
pub(super) fn build_signal_object(
    row: &SignalRow,
    name: &str,
    period: Option<u32>,
    node_map: &NodeMap,
) -> Value {
    let mut object = Map::new();
    object.insert("name".to_owned(), Value::String(name.to_owned()));

    let wave_and_data = match row.decorations().clock.as_ref() {
        Some(spec) => build_clock_wave(row, spec),
        None => NormalWaveBuilder::new().build(row),
    };
    let wave_len = wave_and_data.wave.len();
    object.insert("wave".to_owned(), Value::String(wave_and_data.wave));
    if !wave_and_data.data.is_empty() {
        object.insert(
            "data".to_owned(),
            Value::Array(wave_and_data.data.into_iter().map(Value::String).collect()),
        );
    }
    if let Some(node_string) = node::build_node_string(row, node_map, wave_len) {
        object.insert("node".to_owned(), Value::String(node_string));
    }
    if let Some(period_value) = period {
        object.insert(
            "period".to_owned(),
            Value::Number(serde_json::Number::from(period_value)),
        );
    }

    Value::Object(object)
}

/// Result of scanning a waveform: the wave string and data items.
struct WaveAndData {
    wave: String,
    data: Vec<String>,
}

/// Build wave string for a `@clock`-decorated signal row.
fn build_clock_wave(row: &SignalRow, spec: &crate::clock::ClockSpec) -> WaveAndData {
    let ClockPulse {
        low_units,
        high_units,
    } = spec.pulse;
    if low_units.get() == 1 && high_units.get() == 1 {
        match spec.edge {
            ClockEdge::Pos => return build_repeating_clock_wave(row, 'p'),
            ClockEdge::Neg => return build_repeating_clock_wave(row, 'n'),
            _ => {}
        }
    }
    NormalWaveBuilder::new().build(row)
}

/// Build a `p...` or `n..` clock wave using the chart_units from the waveform.
fn build_repeating_clock_wave(row: &SignalRow, marker: char) -> WaveAndData {
    let total_units = row.waveform().level_units_total();
    if total_units == 0 {
        return WaveAndData {
            wave: String::new(),
            data: Vec::new(),
        };
    }
    let mut wave = String::with_capacity(total_units as usize);
    wave.push(marker);
    for _ in 1..total_units {
        wave.push('.');
    }
    WaveAndData {
        wave,
        data: Vec::new(),
    }
}

/// Stateful builder for normal (non repeating-clock) wave strings.
///
/// `bus_segment_data` records one entry per emitted Bus segment (one `=`
/// character starting a new region). Text elements append into the current
/// segment's entry, joined with single spaces — the parser already merged
/// adjacent same-level runs and joined their text fragments with spaces, so
/// the WaveDrom centred label matches the TCML semantics of one centred
/// label per Bus region.
struct NormalWaveBuilder {
    wave: String,
    bus_segment_data: Vec<String>,
    last_level: Option<SignalLevel>,
}

impl NormalWaveBuilder {
    fn new() -> Self {
        Self {
            wave: String::new(),
            bus_segment_data: Vec::new(),
            last_level: None,
        }
    }

    fn build(mut self, row: &SignalRow) -> WaveAndData {
        for element in row.waveform().iter() {
            self.consume(element);
        }
        let data = if self.bus_segment_data.iter().any(|entry| !entry.is_empty()) {
            self.bus_segment_data
        } else {
            Vec::new()
        };
        WaveAndData {
            wave: self.wave,
            data,
        }
    }

    fn consume(&mut self, element: &WaveformElement) {
        match element {
            WaveformElement::Level(run) => self.consume_level(run),
            // Reset `last_level` so the next Level always re-emits its
            // character (and, for Bus, opens a new `data` slot in
            // `consume_level`). This is what makes `BusCross`-separated Bus
            // segments each get their own data slot — e.g. `=A=B=X=C=D` →
            // `=..` + Transition(BusCross) + `=..` produces two slots.
            WaveformElement::Transition(_) => self.last_level = None,
            WaveformElement::Gap => {
                self.wave.push('|');
                self.last_level = None;
            }
            WaveformElement::Guide
            | WaveformElement::HighlightStart
            | WaveformElement::HighlightEnd
            | WaveformElement::Anchor(_) => {}
            WaveformElement::Text(text) => self.consume_text(text),
        }
    }

    fn consume_level(&mut self, run: &LevelRun) {
        let level = run.level();
        let units = run.units();
        let is_continuation = self.last_level == Some(level);
        if is_continuation {
            for _ in 0..units {
                self.wave.push('.');
            }
        } else {
            self.wave.push(level_char(level));
            for _ in 1..units {
                self.wave.push('.');
            }
            // A new Bus segment opens a fresh data slot. The slot stays empty
            // when no Text element follows; an all-empty `data` array is
            // collapsed in `NormalWaveBuilder::build` so WaveDrom omits the
            // field entirely. The slot count is kept aligned with `=`
            // segments because `consume`'s Transition arm resets `last_level`
            // to `None`, ensuring every BusCross-separated Bus run reaches
            // this branch and pushes its own slot.
            if level == SignalLevel::Bus {
                self.bus_segment_data.push(String::new());
            }
        }
        self.last_level = Some(level);
    }

    fn consume_text(&mut self, text: &UserText) {
        if self.last_level != Some(SignalLevel::Bus) {
            // WaveDrom cannot label `0`/`1`/`z`/`x` segments; drop silently.
            return;
        }
        let Some(entry) = self.bus_segment_data.last_mut() else {
            return;
        };
        for line in text.lines() {
            let trimmed = line.unsafe_text().trim();
            if trimmed.is_empty() {
                continue;
            }
            if !entry.is_empty() {
                entry.push(' ');
            }
            entry.push_str(trimmed);
        }
    }
}

/// Map a [`SignalLevel`] to its WaveDrom character.
fn level_char(level: SignalLevel) -> char {
    match level {
        SignalLevel::Low => '0',
        SignalLevel::High => '1',
        SignalLevel::HiZ => 'z',
        SignalLevel::Bus => '=',
        SignalLevel::DontCareAlongLow
        | SignalLevel::DontCareAlongHigh
        | SignalLevel::DontCareAlongHiZ
        | SignalLevel::DontCareAlongBus => 'x',
    }
}
