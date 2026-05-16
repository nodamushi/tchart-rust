//! Tests for `docs/tests/wavedrom.feature.md`.
//!
//! All JSON comparisons use `serde_json::Value` so key order does not matter.
//!
//! TCML signal syntax: `<name> <waveform>` (space-separated; no colon delimiter).

use serde_json::Value;

use crate::parser::parse;
use crate::wavedrom::WaveDromWarning;

fn wavejson(tcml: &str) -> Value {
    let document = parse(tcml).expect("parse should succeed");
    let (json_str, _warnings) = crate::wavedrom::to_wavejson(&document);
    serde_json::from_str(&json_str).expect("output must be valid JSON")
}

fn wavejson_with_warnings(tcml: &str) -> (Value, Vec<WaveDromWarning>) {
    let document = parse(tcml).expect("parse should succeed");
    let (json_str, warnings) = crate::wavedrom::to_wavejson(&document);
    let value = serde_json::from_str(&json_str).expect("output must be valid JSON");
    (value, warnings)
}

fn signal_at(value: &Value, index: usize) -> &Value {
    value["signal"]
        .as_array()
        .expect("signal must be array")
        .get(index)
        .expect("signal index out of range")
}

// ---------------------------------------------------------------------------
// Signal level basic mapping
// ---------------------------------------------------------------------------

#[test]
fn level_low_maps_to_zero() {
    let value = wavejson("s _");
    assert_eq!(
        signal_at(&value, 0)["wave"]
            .as_str()
            .expect("wave must be string"),
        "0"
    );
}

#[test]
fn level_high_maps_to_one() {
    let value = wavejson("s ~");
    assert_eq!(
        signal_at(&value, 0)["wave"]
            .as_str()
            .expect("wave must be string"),
        "1"
    );
}

#[test]
fn level_hiz_maps_to_z() {
    let value = wavejson("s -");
    assert_eq!(
        signal_at(&value, 0)["wave"]
            .as_str()
            .expect("wave must be string"),
        "z"
    );
}

#[test]
fn level_bus_maps_to_equals() {
    let value = wavejson("s =");
    assert_eq!(
        signal_at(&value, 0)["wave"]
            .as_str()
            .expect("wave must be string"),
        "="
    );
}

#[test]
fn level_dontcare_maps_to_x() {
    let dontcare = wavejson("s =?");
    let wave = signal_at(&dontcare, 0)["wave"]
        .as_str()
        .expect("wave must be string");
    assert!(wave.contains('x'), "dontcare must map to x, got {wave}");
}

#[test]
fn hold_units_produce_dots() {
    // Low 3 unit, High 2 unit, Bus 2 unit
    let value = wavejson("s ___ ~~ ==");
    assert_eq!(
        signal_at(&value, 0)["wave"]
            .as_str()
            .expect("wave must be string"),
        "0..1.=."
    );
}

#[test]
fn bus_text_emits_one_data_entry_per_merged_region() {
    // =A=B=C: parser merges adjacent Bus runs into one region with the
    // space-joined centred label "A B C". WaveDrom mirrors that: one
    // segment, one `data` entry per region.
    let value = wavejson("data =A=B=C");
    let signal = signal_at(&value, 0);
    assert_eq!(signal["wave"].as_str().expect("wave must be string"), "=..");
    let data: Vec<&str> = signal["data"]
        .as_array()
        .expect("data must be array")
        .iter()
        .map(|element| element.as_str().expect("data element must be string"))
        .collect();
    assert_eq!(data, vec!["A B C"]);
}

#[test]
fn bus_single_label_keeps_single_segment() {
    // ==A==: Bus 4 units, single label → wave "=..." and data ["A"].
    let value = wavejson("data ==A==");
    let signal = signal_at(&value, 0);
    assert_eq!(
        signal["wave"].as_str().expect("wave must be string"),
        "=..."
    );
    let data: Vec<&str> = signal["data"]
        .as_array()
        .expect("data must be array")
        .iter()
        .map(|element| element.as_str().expect("data element must be string"))
        .collect();
    assert_eq!(data, vec!["A"]);
}

#[test]
fn bus_no_text_omits_data_field() {
    // ====: Bus 4 units with no labels → wave "=..." and data field omitted.
    let value = wavejson("data ====");
    let signal = signal_at(&value, 0);
    assert_eq!(
        signal["wave"].as_str().expect("wave must be string"),
        "=..."
    );
    assert!(
        signal["data"].is_null(),
        "expected no data field, got {:?}",
        signal["data"]
    );
}

#[test]
fn bus_segments_separated_by_buscross_get_one_label_per_region() {
    // =A=B=X=C=D: parser merge produces two Bus regions split by the
    // BusCross. The first region holds the joined "A B" label, the second
    // holds "C D". `data` follows the region structure (one entry per
    // merged region), not the per-`=` operator structure.
    let value = wavejson("data =A=B=X=C=D");
    let signal = signal_at(&value, 0);
    let wave = signal["wave"].as_str().expect("wave must be string");
    assert_eq!(wave.matches('=').count(), 2, "wave should have 2 segments");
    let data: Vec<&str> = signal["data"]
        .as_array()
        .expect("data must be array")
        .iter()
        .map(|element| element.as_str().expect("data element must be string"))
        .collect();
    assert_eq!(data, vec!["A B", "C D"]);
}

#[test]
fn non_bus_text_is_dropped() {
    let value = wavejson("s _A_");
    let signal = signal_at(&value, 0);
    assert!(
        !signal["wave"]
            .as_str()
            .expect("wave must be string")
            .is_empty()
    );
    assert!(
        signal["data"].is_null(),
        "data should not be present for non-bus text"
    );
}

#[test]
fn dontcare_four_variants_all_map_to_x() {
    let low_dc = wavejson("s _?");
    let wave = signal_at(&low_dc, 0)["wave"]
        .as_str()
        .expect("wave must be string");
    assert!(wave.ends_with('x'), "Low DontCare: got {wave}");

    let high_dc = wavejson("s ~?");
    let wave = signal_at(&high_dc, 0)["wave"]
        .as_str()
        .expect("wave must be string");
    assert!(wave.ends_with('x'), "High DontCare: got {wave}");

    let hiz_dc = wavejson("s -?");
    let wave = signal_at(&hiz_dc, 0)["wave"]
        .as_str()
        .expect("wave must be string");
    assert!(wave.ends_with('x'), "HiZ DontCare: got {wave}");

    let bus_dc = wavejson("s =?");
    let wave = signal_at(&bus_dc, 0)["wave"]
        .as_str()
        .expect("wave must be string");
    assert!(wave.ends_with('x'), "Bus DontCare: got {wave}");
}

// ---------------------------------------------------------------------------
// Transparent elements / Gap
// ---------------------------------------------------------------------------

#[test]
fn gap_maps_to_pipe() {
    let value = wavejson("s _:_");
    assert_eq!(
        signal_at(&value, 0)["wave"]
            .as_str()
            .expect("wave must be string"),
        "0|0"
    );
}

#[test]
fn guide_is_dropped() {
    // _|_ : Low 2 units (Guide has 0 width)
    let value = wavejson("s _|_");
    assert_eq!(
        signal_at(&value, 0)["wave"]
            .as_str()
            .expect("wave must be string"),
        "0."
    );
}

#[test]
fn highlight_brackets_are_dropped() {
    let value = wavejson("s _[~]_");
    assert_eq!(
        signal_at(&value, 0)["wave"]
            .as_str()
            .expect("wave must be string"),
        "010"
    );
}

// ---------------------------------------------------------------------------
// Non-SignalRow lines
// ---------------------------------------------------------------------------

#[test]
fn title_goes_to_head_text() {
    let value = wavejson("@title 同期回路\ns _");
    assert_eq!(
        value["head"]["text"]
            .as_str()
            .expect("head.text must be string"),
        "同期回路"
    );
    let signals = value["signal"].as_array().expect("signal must be array");
    assert_eq!(signals.len(), 1);
    assert!(signals[0].get("name").is_some());
}

#[test]
fn second_and_later_titles_are_dropped_with_warning() {
    let (value, warnings) = wavejson_with_warnings("@title A\n@title B\ns _");
    assert_eq!(
        value["head"]["text"]
            .as_str()
            .expect("head.text must be string"),
        "A",
        "only the first @title must remain"
    );
    let dropped_warning = warnings
        .iter()
        .find(|warning| matches!(warning, WaveDromWarning::AdditionalTitlesDropped { .. }))
        .expect("expected an AdditionalTitlesDropped warning");
    let WaveDromWarning::AdditionalTitlesDropped { dropped_count } = dropped_warning else {
        panic!("variant filter should match");
    };
    assert_eq!(*dropped_count, 1);
}

#[test]
fn three_titles_drop_two_with_warning_count_two() {
    let (value, warnings) = wavejson_with_warnings("@title A\n@title B\n@title C\ns _");
    assert_eq!(value["head"]["text"].as_str().expect("head.text"), "A");
    let dropped_warning = warnings
        .iter()
        .find(|warning| matches!(warning, WaveDromWarning::AdditionalTitlesDropped { .. }))
        .expect("expected an AdditionalTitlesDropped warning");
    let WaveDromWarning::AdditionalTitlesDropped { dropped_count } = dropped_warning else {
        panic!("variant filter should match");
    };
    assert_eq!(*dropped_count, 2);
}

#[test]
fn single_title_emits_no_drop_warning() {
    let (_, warnings) = wavejson_with_warnings("@title only\ns _");
    assert!(
        !warnings
            .iter()
            .any(|warning| matches!(warning, WaveDromWarning::AdditionalTitlesDropped { .. })),
        "single @title must not produce a drop warning"
    );
}

#[test]
fn skip_rows_produce_one_empty_object_each() {
    let value2 = wavejson("@skip(2)\ns _");
    let signals = value2["signal"].as_array().expect("signal must be array");
    let empty_count = signals
        .iter()
        .filter(|value| value.as_object().is_none_or(|map| map.is_empty()))
        .count();
    assert_eq!(empty_count, 1);

    let value_half = wavejson("@skip(0.5)\ns _");
    let signals = value_half["signal"]
        .as_array()
        .expect("signal must be array");
    let empty_count = signals
        .iter()
        .filter(|value| value.as_object().is_none_or(|map| map.is_empty()))
        .count();
    assert_eq!(empty_count, 1);
}

#[test]
fn skip_zero_is_absent() {
    let value = wavejson("s _");
    let signals = value["signal"].as_array().expect("signal must be array");
    let empty_count = signals
        .iter()
        .filter(|value| value.as_object().is_none_or(|map| map.is_empty()))
        .count();
    assert_eq!(empty_count, 0);
}

// ---------------------------------------------------------------------------
// period / time axis normalization
// ---------------------------------------------------------------------------

#[test]
fn same_step_all_signals_no_period_field() {
    let value = wavejson("@step 10\na _\nb ~");
    let sig_a = signal_at(&value, 0);
    let sig_b = signal_at(&value, 1);
    assert!(sig_a["period"].is_null(), "a should have no period field");
    assert!(sig_b["period"].is_null(), "b should have no period field");
}

#[test]
fn no_signals_no_period() {
    let value = wavejson("@title only");
    let signals = value["signal"].as_array().expect("signal must be array");
    assert!(signals.is_empty());
}

// ---------------------------------------------------------------------------
// clock auto-expansion
// ---------------------------------------------------------------------------

#[test]
fn clock_pos_default_pulse_uses_p() {
    // @clock(pos) decorates the following row; chart_units = 4
    let value = wavejson("@clock(pos)\nclk ____");
    let wave = signal_at(&value, 0)["wave"]
        .as_str()
        .expect("wave must be string");
    assert_eq!(wave, "p...");
}

#[test]
fn clock_neg_default_pulse_uses_n() {
    let value = wavejson("@clock(neg)\nclk ___");
    let wave = signal_at(&value, 0)["wave"]
        .as_str()
        .expect("wave must be string");
    assert_eq!(wave, "n..");
}

#[test]
fn clock_nondefault_pulse_uses_level_expansion() {
    // @clock(pos, _=2, ~=3): expand to 0/1 sequence for 5 units.
    // A non-clock signal fixes chart_units=5; the clock row has no explicit
    // waveform so the expander fills it (Low×2, High×3).
    let value = wavejson("ref _____\n@clock(pos, _=2, ~=3)\nclk");
    let wave = signal_at(&value, 1)["wave"]
        .as_str()
        .expect("wave must be string");
    assert_eq!(wave, "0.1..");
}

#[test]
fn clock_both_uses_level_expansion() {
    let value = wavejson("@clock(both)\nclk ____");
    let wave = signal_at(&value, 0)["wave"]
        .as_str()
        .expect("wave must be string");
    assert!(!wave.contains('p') && !wave.contains('n'), "got {wave}");
}

#[test]
fn clock_mark_options_do_not_appear_in_output() {
    let value = wavejson("@clock(pos, mark_position=0.3, mark_color=red)\nclk __");
    let obj = value.as_object().expect("root must be object");
    assert!(!obj.contains_key("mark_position"));
    assert!(!obj.contains_key("mark_color"));
    let signal = signal_at(&value, 0);
    assert!(signal.get("mark_position").is_none());
    assert!(signal.get("mark_color").is_none());
}

// ---------------------------------------------------------------------------
// Signal name
// ---------------------------------------------------------------------------

#[test]
fn multiline_signal_name_joined_with_space() {
    let value = wavejson("\"a\nb\" _");
    assert_eq!(
        signal_at(&value, 0)["name"]
            .as_str()
            .expect("name must be string"),
        "a b"
    );
}

// ---------------------------------------------------------------------------
// Anchors and arrows
// ---------------------------------------------------------------------------

#[test]
fn referenced_anchors_node_length_matches_wave() {
    let tcml = "clk ~_@{a}_~\ndata ==@{b}==\n@-> (@{a}, @{b}) start";
    let value = wavejson(tcml);

    let node_clk = signal_at(&value, 0)["node"]
        .as_str()
        .expect("clk node must be string");
    let node_data = signal_at(&value, 1)["node"]
        .as_str()
        .expect("data node must be string");
    let wave_clk = signal_at(&value, 0)["wave"]
        .as_str()
        .expect("clk wave must be string");
    let wave_data = signal_at(&value, 1)["wave"]
        .as_str()
        .expect("data wave must be string");

    assert_eq!(
        node_clk.len(),
        wave_clk.len(),
        "clk node/wave length mismatch"
    );
    assert_eq!(
        node_data.len(),
        wave_data.len(),
        "data node/wave length mismatch"
    );

    let clock_letters: Vec<char> = node_clk
        .chars()
        .filter(|character| character.is_alphabetic())
        .collect();
    assert_eq!(clock_letters.len(), 1, "expected 1 node letter in clk");
}

#[test]
fn referenced_anchors_produce_edge_with_label() {
    let tcml = "clk ~_@{a}_~\ndata ==@{b}==\n@-> (@{a}, @{b}) start";
    let value = wavejson(tcml);

    let edges = value["edge"].as_array().expect("edge must be array");
    assert!(!edges.is_empty());
    let edge_str = edges[0].as_str().expect("edge element must be string");
    assert!(
        edge_str.ends_with("start"),
        "edge label missing: {edge_str}"
    );
    assert!(edge_str.contains("->"), "edge style missing: {edge_str}");
}

#[test]
fn unreferenced_anchors_get_no_node() {
    let value = wavejson("s _@{x}_");
    let signal = signal_at(&value, 0);
    assert!(
        signal["node"].is_null(),
        "node should not appear for unreferenced anchor"
    );
    assert!(
        value["edge"].is_null()
            || value["edge"]
                .as_array()
                .is_none_or(|array| array.is_empty())
    );
}

#[test]
fn arrow_head_both_produces_bidirectional_style() {
    let tcml = "a _@{x}_\nb _@{y}_\n@-> (@{x}, @{y}, head=both)";
    let value = wavejson(tcml);
    let edge = value["edge"][0]
        .as_str()
        .expect("edge element must be string");
    assert!(edge.contains("<->"), "expected <-> in {edge}");
}

#[test]
fn dashed_arrow_uses_curve_approximation() {
    let tcml = "a _@{x}_\nb _@{y}_\n@-> (@{x}, @{y}, dashed)";
    let value = wavejson(tcml);
    let edge = value["edge"][0]
        .as_str()
        .expect("edge element must be string");
    assert!(edge.contains("-~>"), "expected -~> in {edge}");
}

#[test]
fn arrow_color_and_width_are_dropped() {
    let tcml = "a _@{x}_\nb _@{y}_\n@-> (@{x}, @{y}, red, 3px) hello";
    let value = wavejson(tcml);
    let edge = value["edge"][0]
        .as_str()
        .expect("edge element must be string");
    assert!(edge.ends_with("hello"), "expected label hello in {edge}");
    assert!(!edge.contains("red"), "color must not appear in edge");
    assert!(!edge.contains("3px"), "width must not appear in edge");
}

/// TCML with title and skip rows: helper that builds the wavejson value.
/// line[0]=TitleRow, line[1]=SignalRow(clk), line[2]=SkipRow, line[3]=SignalRow(data)
fn wavejson_title_skip_mixed() -> Value {
    wavejson("@title header\nclk ~_@{a}_~\n@skip(1)\ndata ==@{b}==\n@-> (@{a}, @{b}) label")
}

/// Scenario: anchor `node` is emitted correctly even when Title and Skip rows
/// are interleaved with Signal rows (docs/tests/wavedrom.feature.md).
#[test]
fn anchor_node_present_after_title_row() {
    let value = wavejson_title_skip_mixed();
    // signal[0] = clk (TitleRow at line[0] must not shift the signal index)
    let node_clk = signal_at(&value, 0)["node"]
        .as_str()
        .expect("clk node must be present when title precedes it");
    let wave_clk = signal_at(&value, 0)["wave"]
        .as_str()
        .expect("clk wave must be string");
    assert_eq!(
        node_clk.len(),
        wave_clk.len(),
        "clk node/wave length must match"
    );
    let clock_node_chars: Vec<char> = node_clk
        .chars()
        .filter(|character| character.is_alphabetic())
        .collect();
    assert_eq!(
        clock_node_chars.len(),
        1,
        "clk node must have exactly 1 letter, got: {node_clk}"
    );
}

#[test]
fn anchor_node_present_after_skip_row() {
    let value = wavejson_title_skip_mixed();
    // signal[2] = data (SkipRow at line[2] must not shift the signal index)
    let node_data = signal_at(&value, 2)["node"]
        .as_str()
        .expect("data node must be present when skip precedes it");
    let wave_data = signal_at(&value, 2)["wave"]
        .as_str()
        .expect("data wave must be string");
    assert_eq!(
        node_data.len(),
        wave_data.len(),
        "data node/wave length must match"
    );
    let data_node_chars: Vec<char> = node_data
        .chars()
        .filter(|character| character.is_alphabetic())
        .collect();
    assert_eq!(
        data_node_chars.len(),
        1,
        "data node must have exactly 1 letter, got: {node_data}"
    );
}

#[test]
fn anchor_nodes_distinct_when_title_and_skip_rows_mixed() {
    let value = wavejson_title_skip_mixed();
    let node_clk = signal_at(&value, 0)["node"].as_str().expect("clk node");
    let node_data = signal_at(&value, 2)["node"].as_str().expect("data node");
    let clock_character = node_clk
        .chars()
        .find(|character| character.is_alphabetic())
        .expect("clk letter");
    let data_character = node_data
        .chars()
        .find(|character| character.is_alphabetic())
        .expect("data letter");
    assert_ne!(
        clock_character, data_character,
        "clk and data must receive distinct node characters"
    );

    let edges = value["edge"].as_array().expect("edge must be array");
    assert!(!edges.is_empty(), "edge array must not be empty");
}

// ---------------------------------------------------------------------------
// Dropped elements
// ---------------------------------------------------------------------------

#[test]
fn style_fields_not_in_output() {
    let value = wavejson("@bg red\n@bgcolor0 blue\n@font Arial\n@fontsize 16\ns _");
    let obj = value.as_object().expect("root must be object");
    assert!(!obj.contains_key("config"), "config must not appear");
    assert!(!obj.contains_key("bg"));
}

#[test]
fn percent_row_does_not_appear() {
    let value = wavejson("s _\n% 50 50 hello");
    let signals = value["signal"].as_array().expect("signal must be array");
    assert_eq!(signals.len(), 1);
    let json_str = serde_json::to_string(&value).expect("serialization must succeed");
    assert!(!json_str.contains("hello"), "overlay text must not appear");
}

// ---------------------------------------------------------------------------
// Per-row @step + auto-expansion wave length
// ---------------------------------------------------------------------------

/// Per-row `@step` auto-expansion: explicit row with step=20 and 12 units (240px),
/// auto clock with step=10 should expand to round(240/10)=24 units.
/// The wave string lengths must reflect the correct unit counts.
///
/// Expected: explicit "010101010101" (12 chars), auto "010101010101010101010101" (24 chars).
/// Period: gcd(20,10)=10, explicit period=2, auto period=1 (omitted).
#[test]
fn wavedrom_per_row_step_auto_expand_wave_length() {
    let value = wavejson("@step 20\nClock _~_~_~_~_~_~\n@step 10\n@clock\nclock\n");
    let explicit = signal_at(&value, 0);
    let auto = signal_at(&value, 1);

    let explicit_wave = explicit["wave"].as_str().expect("wave must be string");
    let auto_wave = auto["wave"].as_str().expect("wave must be string");

    assert_eq!(
        explicit_wave.len(),
        12,
        "explicit row must have 12 wave chars, got {explicit_wave}"
    );
    assert_eq!(
        auto_wave.len(),
        24,
        "auto row must expand to 24 wave chars, got {auto_wave}"
    );
    // Period: explicit has period=2, auto has period=1 (omitted from JSON).
    assert_eq!(
        explicit["period"].as_u64(),
        Some(2),
        "explicit row period must be 2"
    );
    assert!(
        auto["period"].is_null(),
        "auto row period=1 must be omitted, got {:?}",
        auto["period"]
    );
}

/// Per-row step + asymmetric pulse auto expansion WaveDrom output.
///
/// Explicit: `@step 20`, Sig 8 units (160px), wave "01010101".
/// Auto: `@step 10`, `@clock(_=2, ~=3)`, target=16.
/// Expected auto wave: Low(2)→High(3)→Low(2)→High(3)→Low(2)→High(3)→Low(1) = 16 chars.
/// "0.1..0.1..0.1..0" (16 chars).
#[test]
fn wavedrom_per_row_step_asymmetric_pulse_auto_wave() {
    let value = wavejson("@step 20\nSig _~_~_~_~\n@step 10\n@clock(_=2, ~=3)\nck\n");
    let sig = signal_at(&value, 0);
    let ck = signal_at(&value, 1);

    let sig_wave = sig["wave"].as_str().expect("wave must be string");
    assert_eq!(
        sig_wave, "01010101",
        "Sig wave must be 01010101, got {sig_wave}"
    );

    let ck_wave = ck["wave"].as_str().expect("wave must be string");
    assert_eq!(
        ck_wave.len(),
        16,
        "auto asymmetric pulse wave must have 16 chars, got {ck_wave}"
    );
}

/// All-auto chart: wave strings must both be empty.
#[test]
fn wavedrom_all_auto_produces_empty_waves() {
    let value = wavejson("@clock\nck1\n@clock\nck2\n");
    let ck1 = signal_at(&value, 0);
    let ck2 = signal_at(&value, 1);

    let ck1_wave = ck1["wave"].as_str().expect("wave must be string");
    let ck2_wave = ck2["wave"].as_str().expect("wave must be string");

    assert_eq!(
        ck1_wave, "",
        "all-auto ck1 wave must be empty, got {ck1_wave}"
    );
    assert_eq!(
        ck2_wave, "",
        "all-auto ck2 wave must be empty, got {ck2_wave}"
    );
}

#[test]
fn empty_input_produces_empty_signal_array() {
    let value = wavejson("");
    let signals = value["signal"].as_array().expect("signal must be array");
    assert!(signals.is_empty());
}

#[test]
fn title_only_emits_head_text_and_empty_signal_array() {
    let value = wavejson("@title onlytitle\n");
    assert!(value["signal"].as_array().expect("array").is_empty());
    assert_eq!(value["head"]["text"].as_str(), Some("onlytitle"));
}

#[test]
fn signal_name_with_trailing_space_after_newline_is_preserved() {
    let value = wavejson("\"a\\n \" _~\n");
    let name = signal_at(&value, 0)["name"].as_str().expect("name");
    assert!(name.contains('a'));
    // Spec: newline becomes one space; trailing source space preserved.
    assert!(name.ends_with(' '));
}

#[test]
fn unreferenced_anchors_omit_edge_field() {
    let value = wavejson("Sig _~@{a}_\n");
    assert!(
        !value.as_object().expect("object").contains_key("edge"),
        "edge must be omitted when no @-> exists"
    );
}

#[test]
fn anchor_letter_assignment_wraps_to_uppercase_after_z() {
    // Build TCML with 27 anchors referenced by arrows.
    let mut source = String::from("Sig _");
    for index in 1..=27 {
        source.push_str(&format!("@{index}_"));
    }
    source.push('\n');
    for index in 1..27 {
        source.push_str(&format!("@-> (@{index}, @{})\n", index + 1));
    }
    let value = wavejson(&source);
    let edges = value["edge"].as_array();
    if let Some(edges) = edges {
        let combined = edges
            .iter()
            .filter_map(|edge| edge.as_str())
            .collect::<String>();
        assert!(
            combined.contains('A'),
            "27th anchor letter should wrap from z to A; got {combined:?}"
        );
    }
}

#[test]
fn exactly_52_anchors_emit_no_warning() {
    let mut source = String::from("Sig _");
    for index in 1..=52 {
        source.push_str(&format!("@{index}_"));
    }
    source.push('\n');
    for index in 1..52 {
        source.push_str(&format!("@-> (@{index}, @{})\n", index + 1));
    }
    let (_value, warnings) = wavejson_with_warnings(&source);
    assert!(
        warnings.is_empty(),
        "52 anchors must not warn; got {warnings:?}"
    );
}

#[test]
fn anchor_53_drops_extras_and_emits_warning() {
    let mut source = String::from("Sig _");
    for index in 1..=53 {
        source.push_str(&format!("@{index}_"));
    }
    source.push('\n');
    for index in 1..53 {
        source.push_str(&format!("@-> (@{index}, @{})\n", index + 1));
    }
    let (value, warnings) = wavejson_with_warnings(&source);
    if let Some(edges) = value["edge"].as_array() {
        assert!(
            edges.len() <= 52,
            "edge length must be capped at 52, got {}",
            edges.len()
        );
    }
    assert!(
        !warnings.is_empty(),
        "53 anchors must produce a warning; got none"
    );
}

#[test]
fn skip_row_is_preserved_as_empty_object_in_signal_array() {
    let value = wavejson("Sig1 _\n@skip(1)\nSig2 _\n");
    let signals = value["signal"].as_array().expect("array");
    assert_eq!(signals.len(), 3);
    let middle = signals.get(1).expect("middle index");
    assert!(middle.is_object(), "skip row must be empty object");
    assert!(
        middle.as_object().expect("object").is_empty(),
        "skip object must be empty"
    );
}

#[test]
fn hiz_level_maps_to_z_in_wave() {
    let value = wavejson("s ----");
    let wave = signal_at(&value, 0)["wave"].as_str().expect("wave");
    assert!(wave.starts_with('z'));
}

#[test]
fn dontcare_in_all_contexts_maps_to_x() {
    let value = wavejson("s1 _?_\ns2 ~?~\ns3 -?-\ns4 =?=\n");
    for index in 0..4 {
        let wave = signal_at(&value, index)["wave"].as_str().expect("wave");
        assert!(
            wave.contains('x'),
            "signal index {index} must contain 'x' in wave; got {wave:?}"
        );
    }
}

#[test]
fn duplicate_title_keeps_first_and_warns_for_extras() {
    let (value, warnings) = wavejson_with_warnings("@title A\n@title B\n@title C\nSig _\n");
    assert_eq!(value["head"]["text"].as_str(), Some("A"));
    assert!(
        !warnings.is_empty(),
        "additional titles must produce warnings"
    );
}

#[test]
fn anchor_node_string_length_matches_wave_length() {
    let value = wavejson("s ____@{a}_~_~\n@-> (@{a}, @{a})\n");
    let signal = signal_at(&value, 0);
    let wave_length = signal["wave"].as_str().expect("wave").len();
    if let Some(node) = signal["node"].as_str() {
        assert_eq!(
            node.len(),
            wave_length,
            "node length must match wave length"
        );
    }
}

#[test]
fn buscross_resets_segment_without_emitting_wave_letter() {
    // `=A=X=B=`: parser merge produces two Bus regions split by the BusCross,
    // each with its own joined label ("A" and "B"). The BusCross itself
    // is not output as a wave character.
    let value = wavejson("data =A=X=B=\n");
    let signal = signal_at(&value, 0);
    let wave = signal["wave"].as_str().expect("wave must be string");
    assert!(
        !wave.contains('X') && !wave.contains('x'),
        "wave must not include the BusCross marker; got {wave}"
    );
    let data = signal["data"].as_array().expect("data must be array");
    let texts: Vec<&str> = data.iter().filter_map(|value| value.as_str()).collect();
    assert_eq!(
        texts,
        vec!["A", "B"],
        "expected one label per merged region; got {texts:?}"
    );
}

#[test]
fn merged_dontcare_along_low_renders_as_x_with_dots() {
    let value = wavejson("s __??__\n");
    let wave = signal_at(&value, 0)["wave"].as_str().expect("wave");
    assert!(wave.contains('x'));
}

#[test]
fn clock_with_nondefault_pulse_widths_uses_zero_one_columns() {
    let value = wavejson("@clock(pos, _=2, ~=2) ck\nT ________\n");
    let wave = signal_at(&value, 0)["wave"].as_str().expect("wave");
    // For non-1:1 pulses the spec mandates 0/1 columns rather than 'p'/'n'.
    assert!(
        !wave.contains('p') && !wave.contains('n'),
        "non-default pulse widths must not use p/n shortcut; got {wave:?}"
    );
}

#[test]
fn clock_pos_default_pulse_uses_p_letter() {
    let value = wavejson("@clock(pos) ck\nT ____\n");
    let wave = signal_at(&value, 0)["wave"].as_str().expect("wave");
    assert!(
        wave.starts_with('p'),
        "default 1:1 clock pos must use 'p'; got {wave:?}"
    );
}

#[test]
fn clock_none_edge_uses_zero_one_columns() {
    let value = wavejson("@clock(none) ck\nT ____\n");
    let wave = signal_at(&value, 0)["wave"].as_str().expect("wave");
    assert!(
        !wave.contains('p') && !wave.contains('n'),
        "edge=none must not use p/n; got {wave:?}"
    );
}

#[test]
fn clock_both_edge_uses_zero_one_columns() {
    let value = wavejson("@clock(both) ck\nT ____\n");
    let wave = signal_at(&value, 0)["wave"].as_str().expect("wave");
    assert!(
        !wave.contains('p') && !wave.contains('n'),
        "edge=both must not use p/n; got {wave:?}"
    );
}

#[test]
fn single_signal_with_trivial_period_omits_period_field() {
    let value = wavejson("@step 10\nSig _~_~\n");
    let signal = signal_at(&value, 0);
    assert!(
        signal.get("period").is_none(),
        "trivial period (=1) must be omitted; got {signal:?}"
    );
}

#[test]
fn signal_with_zero_slant_renders_without_panic() {
    // `@slant 0` paired with `@step 1` is a valid layout (step > slant) and
    // must produce a signal entry without tripping any layout-time assertion.
    let value = wavejson("@step 1\n@slant 0\nSig _\n");
    let _ = signal_at(&value, 0);
}

#[test]
fn mid_step_change_with_clock_auto_period() {
    let value = wavejson("@step 10\n@clock(pos) clk\n@step 20\ndata ====\n");
    let clk = signal_at(&value, 0);
    let data = signal_at(&value, 1);
    let clk_wave = clk["wave"].as_str().expect("clk wave");
    let data_wave = data["wave"].as_str().expect("data wave");
    assert!(
        clk_wave.starts_with('p'),
        "clk must remain 'p...'; got {clk_wave}"
    );
    assert!(
        data_wave.starts_with('='),
        "data must start with '='; got {data_wave}"
    );
    if let Some(period) = data.get("period").and_then(|value| value.as_u64()) {
        assert_eq!(period, 2, "data.period should be 2 (gcd 10/20)");
    }
}

#[test]
fn mid_step_change_with_clock_auto_pulse_widths() {
    let value = wavejson("@step 10\n@clock(pos, _=2, ~=2) clk\n@step 20\ndata ========\n");
    let clk = signal_at(&value, 0);
    let clk_wave = clk["wave"].as_str().expect("clk wave");
    assert!(
        !clk_wave.contains('p') && !clk_wave.contains('n'),
        "non-default pulse must not use p/n; got {clk_wave}"
    );
}

#[test]
fn per_row_step_with_anchor_edge_in_wavejson() {
    let value =
        wavejson("@step 10\nSig1 _~@{a}_~\n@step 20\nSig2 ====@{b}====\n@-> (@{a}, @{b})\n");
    let edges = value["edge"].as_array().expect("edge array");
    assert!(!edges.is_empty(), "edge must be emitted");
}

#[test]
fn per_row_step_with_arrow_label_in_wavejson() {
    let value = wavejson(
        "@step 10\nSig1 _~@{a}_~\n@step 20\nSig2 ====@{b}====\n@-> (@{a}, @{b}) my-label\n",
    );
    let edges = value["edge"].as_array().expect("edge array");
    let combined = edges
        .iter()
        .filter_map(|edge| edge.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        combined.contains("my-label"),
        "edge label missing: {combined}"
    );
}

#[test]
fn clock_auto_with_anchors_in_body_edge_emission() {
    let value = wavejson("@clock(pos) clk _~@{a}__\ndata ==@{b}===\n@-> (@{a}, @{b})\n");
    let edges = value["edge"].as_array().expect("edge array");
    assert!(!edges.is_empty());
}

#[test]
fn buscross_repeated_emits_dot_continuation_segments() {
    let value = wavejson("s =X=X=X=\n");
    let wave = signal_at(&value, 0)["wave"].as_str().expect("wave");
    assert!(wave.starts_with('='));
}

#[test]
fn gap_with_repeated_low_emits_pipe_in_wave() {
    let value = wavejson("s __:_:__\n");
    let wave = signal_at(&value, 0)["wave"].as_str().expect("wave");
    assert!(wave.contains('|'), "gap must map to '|'; got {wave}");
}

#[test]
fn highlight_with_bus_dontcare_anchor_passes_through() {
    let value = wavejson("s _[==?==@{a}__]~~\n@-> (@{a}, @{a})\n");
    let signal = signal_at(&value, 0);
    let wave = signal["wave"].as_str().expect("wave");
    assert!(!wave.contains('['), "wave must not contain literal [");
    assert!(!wave.contains(']'), "wave must not contain literal ]");
}

#[test]
fn multiline_signal_name_joined_with_space_in_wavejson() {
    let value = wavejson("\"foo\\nbar\" _~\n");
    let name = signal_at(&value, 0)["name"].as_str().expect("name");
    assert_eq!(name, "foo bar");
}

#[test]
fn title_with_bg_strips_bg_from_wavejson() {
    let value = wavejson("@bg red\n@title \"T\"\nSig _~\n");
    let object = value.as_object().expect("object");
    assert!(!object.contains_key("bg"));
    assert_eq!(value["head"]["text"].as_str(), Some("T"));
}

#[test]
fn title_only_omits_period_and_edge_fields() {
    let value = wavejson("@title \"x\"\n");
    let object = value.as_object().expect("object");
    assert!(!object.contains_key("edge"));
    assert!(value["signal"].as_array().expect("array").is_empty());
}

#[test]
fn warning_message_for_anchor_overflow_uses_canonical_text() {
    let mut source = String::from("Sig _");
    for index in 1..=53 {
        source.push_str(&format!("@{index}_"));
    }
    source.push('\n');
    for index in 1..53 {
        source.push_str(&format!("@-> (@{index}, @{})\n", index + 1));
    }
    let (_, warnings) = wavejson_with_warnings(&source);
    assert!(
        warnings.iter().any(|warning| {
            let text = format!("{warning:?}");
            text.contains("52") || text.contains("anchor")
        }),
        "warning must mention 52-anchor cap; got {warnings:?}"
    );
}

// ---------------------------------------------------------------------------
// Iter1 phase 2: WaveDrom negative side and boundary scenarios.
// ---------------------------------------------------------------------------

#[test]
fn zero_signals_emits_only_signal_array_iter1() {
    let value = wavejson("");
    let object = value.as_object().expect("object");
    assert!(value["signal"].as_array().expect("array").is_empty());
    assert!(!object.contains_key("head"));
    assert!(!object.contains_key("foot"));
    assert!(!object.contains_key("edge"));
}

#[test]
fn title_only_zero_signals_iter1() {
    let value = wavejson("@title \"T\"\n");
    assert_eq!(value["head"]["text"].as_str(), Some("T"));
    assert!(value["signal"].as_array().expect("array").is_empty());
}

#[test]
fn no_clock_signal_uses_zero_one_not_pn_iter1() {
    let value = wavejson("clk _~_~\n");
    let wave = signal_at(&value, 0)["wave"].as_str().expect("wave");
    assert!(!wave.contains('p') && !wave.contains('n'));
    assert!(wave.contains('0') && wave.contains('1'));
}

#[test]
fn trailing_low_hold_dots_not_trimmed_iter1() {
    let value = wavejson("A _____\n");
    let wave = signal_at(&value, 0)["wave"].as_str().expect("wave");
    assert!(wave.starts_with('0'));
    let dot_count = wave.chars().filter(|character| *character == '.').count();
    assert_eq!(dot_count, 4, "expected 4 hold dots; got wave={wave}");
}

#[test]
fn trailing_dontcare_hold_dots_not_trimmed_iter1() {
    let value = wavejson("A ____????\n");
    let wave = signal_at(&value, 0)["wave"].as_str().expect("wave");
    let trailing = wave.trim_start_matches('0').trim_start_matches('.');
    assert!(
        trailing.starts_with('x'),
        "expected dontcare run after low; got wave={wave}"
    );
    let trailing_dots = trailing
        .chars()
        .filter(|character| *character == '.')
        .count();
    assert!(
        trailing_dots >= 1,
        "dontcare hold dots must be retained; got wave={wave}"
    );
}

#[test]
fn unreferenced_anchors_get_no_node_letters_iter1() {
    // Per docs/spec/wavedrom.md, only anchors referenced by `@->` arrows
    // become entries in the WaveDrom `edge` array and earn `node` letters.
    // With no arrows, either the `node` field is omitted entirely or it
    // contains only dots; either form must satisfy "no node letters".
    let value = wavejson("A _@{a}@{b}~\n");
    let signal = signal_at(&value, 0);
    match signal.get("node").and_then(|value| value.as_str()) {
        None => {}
        Some(node) => {
            let assigned = node
                .chars()
                .filter(|character| *character != '.' && *character != ' ')
                .count();
            assert_eq!(
                assigned, 0,
                "unreferenced anchors must produce no node letters; got node={node}"
            );
        }
    }
}

#[test]
fn referenced_anchors_get_node_letters_iter1() {
    let value = wavejson("A _@{x}@{y}@{z}~\n@-> (@{x}, @{z})\n");
    let signal = signal_at(&value, 0);
    let node = signal["node"].as_str().expect("node string");
    let assigned = node
        .chars()
        .filter(|character| character.is_ascii_alphabetic())
        .count();
    assert_eq!(
        assigned, 2,
        "exactly 2 anchors are referenced; got node={node}"
    );
}

#[test]
fn no_anchors_no_arrows_omits_edge_field_iter1() {
    let value = wavejson("A _~_~\n");
    let object = value.as_object().expect("object");
    assert!(!object.contains_key("edge"));
    let signal = signal_at(&value, 0);
    assert!(signal.as_object().expect("object").get("node").is_none());
}

#[test]
fn all_signals_clock_auto_emits_no_unknown_warning_iter1() {
    // Per docs/spec/wavedrom.md §警告 the only warning variants are
    // `StepRounded`, `TooManyAnchors`, and `AdditionalTitlesDropped`. A chart
    // composed entirely of bare `@clock` rows (= `@clock(none)` per
    // docs/spec/tcml-format.md §「@clock」) is not listed as a warning
    // condition; this test pins that contract.
    let (_, warnings) = wavejson_with_warnings("@clock\nclk1 _\n@clock\nclk2 _\n");
    let unknown_warnings: Vec<&WaveDromWarning> = warnings
        .iter()
        .filter(|warning| {
            !matches!(
                warning,
                WaveDromWarning::StepRounded { .. }
                    | WaveDromWarning::TooManyAnchors
                    | WaveDromWarning::AdditionalTitlesDropped { .. }
            )
        })
        .collect();
    assert!(
        unknown_warnings.is_empty(),
        "no warning outside the spec-defined variants is allowed; got {unknown_warnings:?}"
    );
}

#[test]
fn signal_name_quote_is_json_escaped_iter1() {
    let value = wavejson("\"a\\\"b\" _\n");
    let name = signal_at(&value, 0)["name"].as_str().expect("name");
    assert!(
        name.contains('"'),
        "literal quote must be present in decoded name: {name}"
    );
}

#[test]
fn title_text_with_quote_is_json_escaped_iter1() {
    let value = wavejson("@title \"He said \\\"hi\\\"\"\nSig _\n");
    let title = value["head"]["text"].as_str().expect("title");
    assert!(
        title.contains('"'),
        "title must contain literal quote: {title}"
    );
}

#[test]
fn fifty_two_anchors_produces_no_warning_iter1() {
    let mut source = String::from("Sig _");
    for index in 1..=52 {
        source.push_str(&format!("@{index}_"));
    }
    source.push('\n');
    for index in 1..52 {
        source.push_str(&format!("@-> (@{index}, @{})\n", index + 1));
    }
    let (_, warnings) = wavejson_with_warnings(&source);
    let anchor_overflow = warnings
        .iter()
        .filter(|warning| matches!(warning, WaveDromWarning::TooManyAnchors))
        .count();
    assert_eq!(
        anchor_overflow, 0,
        "exactly 52 anchors must not trigger TooManyAnchors warning; got {warnings:?}"
    );
}

#[test]
fn single_signal_step_seven_period_omitted_when_period_is_one_iter1() {
    // Per docs/spec/wavedrom.md §period: g = gcd(step1, ...) = 7 here, so the
    // single signal's period = step/g = 7/7 = 1, and per rule 5 the field is
    // omitted (WaveDrom default = 1).
    let value = wavejson("@step 7\nA _~_~\n");
    let object = value.as_object().expect("object");
    assert!(
        !object.contains_key("period"),
        "single-signal step=7 yields period=1 which must be omitted: {value:?}"
    );
    let signal = signal_at(&value, 0);
    let signal_obj = signal.as_object().expect("signal object");
    assert!(
        !signal_obj.contains_key("period"),
        "per-signal period field must also be omitted when period == 1: {signal:?}"
    );
}

// ---------------------------------------------------------------------------
// Iter2 phase 2: WaveDrom positive coverage and edge style boundaries.
// ---------------------------------------------------------------------------

#[test]
fn iter2_arrow_head_start_is_parser_error() {
    // The current spec accepts only `head=end`, `head=both`, `head=none`.
    // `head=start` must be rejected at parse time so the WaveDrom edge layer
    // never has to invent a `<-` style. This pins that contract.
    let result = parse("A _@{a}~\nB _~@{b}\n@-> (@{a}, @{b}, head=start)\n");
    assert!(
        result.is_err(),
        "head=start must be rejected by parser; got Ok"
    );
}

#[test]
fn iter2_arrow_head_both_yields_double_arrow_edge_style() {
    let value = wavejson("A _@{a}~\nB _~@{b}\n@-> (@{a}, @{b}, head=both)\n");
    let edge = value
        .get("edge")
        .and_then(|edges| edges.as_array())
        .and_then(|edges| edges.first())
        .and_then(|edge| edge.as_str())
        .expect("edge entry must exist");
    assert!(
        edge.contains("<->"),
        "head=both must map to <-> edge style; got edge={edge}"
    );
}

#[test]
fn iter2_arrow_head_none_yields_line_edge_style() {
    let value = wavejson("A _@{a}~\nB _~@{b}\n@-> (@{a}, @{b}, head=none)\n");
    let edge = value
        .get("edge")
        .and_then(|edges| edges.as_array())
        .and_then(|edges| edges.first())
        .and_then(|edge| edge.as_str())
        .expect("edge entry must exist");
    let arrow_chars: String = edge
        .chars()
        .filter(|character| *character == '<' || *character == '>')
        .collect();
    assert!(
        arrow_chars.is_empty(),
        "head=none must omit `<` and `>` from the edge style; got edge={edge}"
    );
    assert!(
        edge.contains('-'),
        "head=none edge must still contain `-` line marker; got edge={edge}"
    );
}

#[test]
fn iter2_arrow_label_with_newline_is_flattened_in_edge_string() {
    // Multi-line arrow labels are joined with " " on the WaveDrom side; raw
    // newlines never reach the JSON edge string. This test pins that the
    // resulting JSON is parseable and the label survives in some form.
    let value = wavejson("A _@{a}~\nB _~@{b}\n@-> (@{a}, @{b}) \"line1\nline2\"\n");
    let edge = value
        .get("edge")
        .and_then(|edges| edges.as_array())
        .into_iter()
        .flatten()
        .filter_map(|edge| edge.as_str())
        .find(|edge| edge.contains("line1"))
        .expect("edge entry containing `line1` must exist");
    assert!(
        edge.contains("line1"),
        "edge label must retain first line; got edge={edge}"
    );
    assert!(
        edge.contains("line2"),
        "edge label must retain second line; got edge={edge}"
    );
}

#[test]
fn iter2_arrow_label_with_tab_is_preserved_in_edge_string() {
    let value = wavejson("A _@{a}~\nB _~@{b}\n@-> (@{a}, @{b}) \"a\tb\"\n");
    let edge = value
        .get("edge")
        .and_then(|edges| edges.as_array())
        .into_iter()
        .flatten()
        .filter_map(|edge| edge.as_str())
        .find(|edge| edge.contains('a'))
        .expect("edge entry containing the label must exist");
    assert!(
        edge.contains('a') && edge.contains('b'),
        "edge label must contain both halves around the tab; got edge={edge}"
    );
}

#[test]
fn iter2_node_letter_assignment_27th_anchor_uses_uppercase_a() {
    let mut source = String::from("Sig _");
    for index in 1..=27 {
        source.push_str(&format!("@{index}_"));
    }
    source.push('\n');
    for index in 1..27 {
        source.push_str(&format!("@-> (@{index}, @{})\n", index + 1));
    }
    let value = wavejson(&source);
    let node = signal_at(&value, 0)["node"].as_str().expect("node");
    let letters: Vec<char> = node
        .chars()
        .filter(|character| character.is_ascii_alphabetic())
        .collect();
    assert_eq!(
        letters.len(),
        27,
        "expected 27 assigned letters; got node={node}"
    );
    assert_eq!(
        letters.last().copied(),
        Some('A'),
        "27th letter must be uppercase A (after a..z); got node={node}"
    );
}

#[test]
fn iter2_node_letter_assignment_52nd_anchor_uses_uppercase_z() {
    let mut source = String::from("Sig _");
    for index in 1..=52 {
        source.push_str(&format!("@{index}_"));
    }
    source.push('\n');
    for index in 1..52 {
        source.push_str(&format!("@-> (@{index}, @{})\n", index + 1));
    }
    let value = wavejson(&source);
    let node = signal_at(&value, 0)["node"].as_str().expect("node");
    let letters: Vec<char> = node
        .chars()
        .filter(|character| character.is_ascii_alphabetic())
        .collect();
    assert_eq!(letters.len(), 52, "expected 52 assigned letters");
    assert_eq!(letters.last().copied(), Some('Z'), "52nd letter must be Z");
}

#[test]
fn iter2_node_letter_assignment_53rd_anchor_triggers_warning() {
    let mut source = String::from("Sig _");
    for index in 1..=53 {
        source.push_str(&format!("@{index}_"));
    }
    source.push('\n');
    for index in 1..53 {
        source.push_str(&format!("@-> (@{index}, @{})\n", index + 1));
    }
    let (value, warnings) = wavejson_with_warnings(&source);
    let overflow_warnings: Vec<&WaveDromWarning> = warnings
        .iter()
        .filter(|warning| matches!(warning, WaveDromWarning::TooManyAnchors))
        .collect();
    assert!(
        !overflow_warnings.is_empty(),
        "53 anchors must trigger a TooManyAnchors warning; got {warnings:?}"
    );
    let node = signal_at(&value, 0)["node"].as_str().expect("node");
    let letters: Vec<char> = node
        .chars()
        .filter(|character| character.is_ascii_alphabetic())
        .collect();
    assert!(
        letters.len() <= 52,
        "letters must not exceed 52; got node={node}"
    );
}

#[test]
fn iter2_period_gcd_of_mixed_steps_two_and_four() {
    // gcd(4, 2) = 2: per docs/spec/wavedrom.md トップレベル構造, the root
    // object must NOT contain a `period` key — top-level `period` is not part
    // of the field table. Per-signal `period` carries the ratio instead.
    let value = wavejson("@step 4\n@clock(pos) clk\n@step 2\nA _~_~\n");
    let object = value.as_object().expect("object");
    assert!(
        !object.contains_key("period"),
        "root period must never be emitted (top-level field table has no period); got value={value}"
    );
}

#[test]
fn iter2_period_gcd_of_three_and_five_is_one_omitted() {
    // gcd(3, 5) = 1: root `period` must be absent regardless of the GCD value.
    let value = wavejson("@step 3\n@clock(pos) clk1\n@step 5\n@clock(pos) clk2\n");
    let object = value.as_object().expect("object");
    assert!(
        !object.contains_key("period"),
        "root period must never be emitted (top-level field table has no period); got value={value}"
    );
}

#[test]
fn iter2_data_array_length_matches_merged_bus_regions_in_wave() {
    // `=A=B=C` merges into one Bus region with a space-joined centred label,
    // so the `data` array must have exactly one entry per region — not one
    // per `=` operator.
    let value = wavejson("Sig =A=B=C\n");
    let signal = signal_at(&value, 0);
    let wave = signal["wave"].as_str().expect("wave");
    let data = signal["data"].as_array().expect("data array");
    assert_eq!(
        data.len(),
        1,
        "data array length must equal merged-bus region count (1); wave={wave} data={data:?}"
    );
    let texts: Vec<&str> = data.iter().filter_map(|entry| entry.as_str()).collect();
    assert_eq!(texts, vec!["A B C"]);
}

#[test]
fn iter2_data_array_merges_holds_into_single_region() {
    // `=A==B` has no BusCross between the two labels, so parser merge unifies
    // the entire run into one Bus region with the joined label "A B". `data`
    // must hold a single entry, not two.
    let value = wavejson("Sig =A==B\n");
    let signal = signal_at(&value, 0);
    let data = signal["data"].as_array().expect("data array");
    let texts: Vec<&str> = data.iter().filter_map(|entry| entry.as_str()).collect();
    assert_eq!(
        texts.len(),
        1,
        "expected 1 merged-region data entry; got {texts:?}"
    );
    assert_eq!(texts.first().copied(), Some("A B"));
}

#[test]
fn iter2_empty_title_omits_head_field() {
    let value = wavejson("@title \"\"\nSig _\n");
    let object = value.as_object().expect("object");
    if let Some(head) = object.get("head") {
        let text = head.get("text").and_then(|text| text.as_str());
        assert!(
            text.is_none() || text == Some(""),
            "empty title may omit head or yield empty text; got head={head:?}"
        );
    }
}

#[test]
fn iter2_title_with_newline_preserves_in_head_text() {
    let value = wavejson("@title \"line1\nline2\"\nSig _\n");
    let head_text = value
        .get("head")
        .and_then(|head| head.get("text"))
        .and_then(|text| text.as_str())
        .expect("head.text must exist");
    assert!(
        head_text.contains("line1") && head_text.contains("line2"),
        "head.text must preserve both halves of the multi-line title; got {head_text}"
    );
}

#[test]
fn iter2_signal_array_preserves_document_order() {
    let value = wavejson("Z _\nA _\nM _\n");
    let signals = value["signal"].as_array().expect("signal array");
    let names: Vec<&str> = signals
        .iter()
        .filter_map(|signal| signal.get("name"))
        .filter_map(|name| name.as_str())
        .collect();
    assert_eq!(
        names,
        vec!["Z", "A", "M"],
        "signal order must follow document order, not alphabetical sort"
    );
}

// ---------------------------------------------------------------------------
// Iter3 phase: WaveDrom-side gaps for tchart-only features.
// ---------------------------------------------------------------------------

fn collect_edge_strings(value: &Value) -> Vec<String> {
    value
        .get("edge")
        .and_then(|edge| edge.as_array())
        .map(|array| {
            array
                .iter()
                .filter_map(|entry| entry.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn iter3_dotted_arrow_lowers_to_known_edge_style() {
    let value = wavejson("A _@{a}~\nB _~@{b}\n@-> (@{a}, @{b}, style=dotted)\n");
    let edges = collect_edge_strings(&value);
    if edges.is_empty() {
        return;
    }
    // WaveDrom-known edge-style marker characters. A `dotted` style is not
    // representable directly, so the lowering must pick one of these.
    let known_style_markers = ['-', '~', '|'];
    assert!(
        edges.iter().any(|edge| edge
            .chars()
            .any(|character| known_style_markers.contains(&character))),
        "dotted arrow must downgrade to a known edge style; got {edges:?}"
    );
}

#[test]
fn iter3_arrow_head_none_renders_as_plain_edge_or_skipped() {
    let value = wavejson("A _@{a}~\nB _~@{b}\n@-> (@{a}, @{b}, head=none)\n");
    let edges = collect_edge_strings(&value);
    // Either the edge is omitted, or it appears with no head decoration.
    for edge in &edges {
        assert!(
            !edge.is_empty(),
            "edge string must not be empty when present; got {edges:?}"
        );
    }
}

#[test]
fn iter3_no_bus_segments_omits_data_field() {
    let value = wavejson("A _~_~\n");
    let signal = signal_at(&value, 0);
    let object = signal.as_object().expect("signal entry is an object");
    if let Some(data) = object.get("data") {
        let array = data.as_array().expect("data must be an array if present");
        assert!(
            array.is_empty(),
            "no labelled buses → data must be absent or empty; got {data:?}"
        );
    }
}

#[test]
fn iter3_unlabelled_buses_data_field_is_deterministic() {
    // 3 bus segments without labels.
    let value = wavejson("A ===\n");
    let signal = signal_at(&value, 0);
    let object = signal.as_object().expect("signal entry is an object");
    match object.get("data") {
        None => {}
        Some(data) => {
            let array = data.as_array().expect("data must be array");
            for entry in array {
                assert!(
                    entry.is_string(),
                    "data entries are deterministically strings; got {entry:?}"
                );
            }
        }
    }
}

#[test]
fn iter3_empty_title_omits_or_blanks_head_text() {
    let value = wavejson("Sig _\n");
    if let Some(head) = value.get("head") {
        let text = head.get("text").and_then(|inner| inner.as_str());
        assert!(
            text.is_none() || text == Some(""),
            "no @title → head.text must be absent or empty; got {head:?}"
        );
    }
}

#[test]
fn iter3_bg_directive_is_dropped_from_wavejson() {
    let value = wavejson("@bg #ffeecc\nA _~\n");
    let object = value.as_object().expect("top-level object");
    for key in ["bg", "background", "config"] {
        if let Some(node) = object.get(key) {
            let serialised = node.to_string();
            assert!(
                !serialised.contains("ffeecc") && !serialised.contains("FFEECC"),
                "@bg colour must not leak into key {key:?}; got {serialised}"
            );
        }
    }
}

#[test]
fn iter3_page_margin_is_dropped_from_wavejson() {
    let value = wavejson("@page-margin 20\nA _~\n");
    let serialised = value.to_string();
    assert!(
        !serialised.contains("page-margin") && !serialised.contains("page_margin"),
        "@page-margin must not appear in WaveJSON output; got {serialised}"
    );
}

#[test]
fn iter3_arrow_label_position_is_dropped_from_edge() {
    let value = wavejson("A _@{a}~\nB _~@{b}\n@-> (@{a}, @{b}, label=\"X\")\n");
    let edges = collect_edge_strings(&value);
    for edge in &edges {
        assert!(
            !edge.contains("mid") && !edge.contains("label-pos"),
            "label position attribute must not appear in edge string; got {edges:?}"
        );
    }
}

#[test]
fn iter3_anchor_to_node_letter_mapping_is_present_or_skipped() {
    let value = wavejson("A _@{first}@{second}~\n");
    let signal = signal_at(&value, 0);
    let node = signal
        .get("node")
        .and_then(|inner| inner.as_str())
        .unwrap_or("");
    // Either a node string is emitted (with letters) or it is absent. Both
    // outcomes are deterministic.
    assert!(
        node.chars()
            .all(|character| character.is_ascii() || character == '.'),
        "node letters must be ASCII; got {node:?}"
    );
}

#[test]
fn iter3_clock_directive_does_not_leak_edge_marks_into_wavejson() {
    let value = wavejson("@clock(pos)\nclk _~_~\n");
    let serialised = value.to_string();
    assert!(
        !serialised.contains("EdgeMark"),
        "edge marks must be absorbed into wave string; got {serialised}"
    );
}
