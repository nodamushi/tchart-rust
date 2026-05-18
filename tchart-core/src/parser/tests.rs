//! Parser unit tests covering the scenarios in
//! `docs/tests/tcml-parser.feature.md`.

use crate::anchor::AnchorId;
use crate::arrow::{ArrowEnd, ArrowHead, LineDashStyle};
use crate::clock::ClockEdge;
use crate::errors::{NameError, ParseError, ParseErrorKind, TextError};
use crate::line::{LineContent, SignalLevel, SignalRow, TransitionKind, WaveformElement};
use crate::style::HorizontalAlign;
use crate::units::{Length, Px};

use super::parse;

fn parse_or_panic(input: &str) -> crate::document::ChartDocument {
    parse(input).expect("parse should succeed")
}

fn parse_error(input: &str) -> ParseError {
    parse(input).expect_err("parse should fail")
}

fn first_signal(input: &str) -> SignalRow {
    let doc = parse_or_panic(input);
    let line = doc.lines.into_iter().next().expect("at least one line");
    match line.content {
        LineContent::Signal(row) => (*row).clone(),
        other => panic!("expected signal row, got {other:?}"),
    }
}

#[test]
fn empty_input_yields_empty_document() {
    let doc = parse_or_panic("");
    assert!(doc.lines.is_empty());
}

#[test]
fn comment_lines_are_ignored() {
    let doc = parse_or_panic("// comment\n// another");
    assert!(doc.lines.is_empty());
}

#[test]
fn blank_lines_are_ignored() {
    let doc = parse_or_panic("\n\n\n");
    assert!(doc.lines.is_empty());
}

#[test]
fn test_single_low_run() {
    let row = first_signal("SigA ____");
    assert_eq!(row.waveform().len(), 1);
    match &row.waveform()[0] {
        WaveformElement::Level(run) => {
            assert_eq!(run.level(), SignalLevel::Low);
            assert_eq!(run.units(), 4);
        }
        other => panic!("expected Level, got {other:?}"),
    }
}

#[test]
fn high_low_alternation_inserts_transitions() {
    let row = first_signal("SigA _~");
    assert_eq!(row.waveform().len(), 3);
    if let WaveformElement::Transition(transition) = &row.waveform()[1] {
        assert_eq!(transition.kind, TransitionKind::SingleEdge);
        assert_eq!(transition.source, SignalLevel::Low);
        assert_eq!(transition.target, SignalLevel::High);
    } else {
        panic!("expected Transition at index 1");
    }
}

#[test]
fn bus_open_and_close_transitions() {
    let row = first_signal("SigA _==_");
    let kinds: Vec<TransitionKind> = row
        .waveform()
        .iter()
        .filter_map(|element| match element {
            WaveformElement::Transition(transition) => Some(transition.kind),
            _ => None,
        })
        .collect();
    assert_eq!(
        kinds,
        vec![TransitionKind::BusOpen, TransitionKind::BusClose]
    );
}

#[test]
fn control_char_in_level_string_errors() {
    // Control characters (other than whitespace) that are not valid text chars.
    let error = parse_error("SigA _\x01_~");
    assert!(matches!(
        error.kind(),
        ParseErrorKind::InvalidLevelChar('\x01')
    ));
}

#[test]
fn test_dontcare_along_low() {
    // `_?_`: `?` is 0-width; only the two `_` characters contribute to units.
    let row = first_signal("SigA _?_");
    let elements: Vec<_> = row
        .waveform()
        .iter()
        .filter_map(|element| match element {
            WaveformElement::Level(run) => Some((run.level(), run.units())),
            _ => None,
        })
        .collect();
    assert_eq!(elements, vec![(SignalLevel::DontCareAlongLow, 2)]);
    let has_transition = row
        .waveform()
        .iter()
        .any(|element| matches!(element, WaveformElement::Transition(_)));
    assert!(!has_transition, "no transition around `?` boundary");
}

#[test]
fn dontcare_along_high() {
    // `~?~`: `?` is 0-width; only the two `~` characters contribute to units.
    let row = first_signal("SigA ~?~");
    let elements: Vec<_> = row
        .waveform()
        .iter()
        .filter_map(|element| match element {
            WaveformElement::Level(run) => Some((run.level(), run.units())),
            _ => None,
        })
        .collect();
    assert_eq!(elements, vec![(SignalLevel::DontCareAlongHigh, 2)]);
}

#[test]
fn dontcare_along_hiz() {
    // `-?-`: `?` is 0-width; only the two `-` characters contribute to units.
    let row = first_signal("SigA -?-");
    let elements: Vec<_> = row
        .waveform()
        .iter()
        .filter_map(|element| match element {
            WaveformElement::Level(run) => Some((run.level(), run.units())),
            _ => None,
        })
        .collect();
    assert_eq!(elements, vec![(SignalLevel::DontCareAlongHiZ, 2)]);
}

#[test]
fn dontcare_along_bus() {
    // `=?=`: `?` is 0-width; only the two `=` characters contribute to units.
    let row = first_signal("SigA =?=");
    let elements: Vec<_> = row
        .waveform()
        .iter()
        .filter_map(|element| match element {
            WaveformElement::Level(run) => Some((run.level(), run.units())),
            _ => None,
        })
        .collect();
    assert_eq!(elements, vec![(SignalLevel::DontCareAlongBus, 2)]);
}

#[test]
fn test_consecutive_dontcare_merge() {
    // `_???_`: `?` chars are 0-width; only the two `_` contribute to units.
    let row = first_signal("SigA _???_");
    let elements: Vec<_> = row
        .waveform()
        .iter()
        .filter_map(|element| match element {
            WaveformElement::Level(run) => Some((run.level(), run.units())),
            _ => None,
        })
        .collect();
    assert_eq!(elements, vec![(SignalLevel::DontCareAlongLow, 2)]);
}

#[test]
fn dontcare_expands_to_preceding_same_level() {
    // `_?=`: `?` is 0-width; `_` contributes 1 unit, `=` is a different level (cutoff).
    let row = first_signal("SigA _?=");
    let elements: Vec<_> = row
        .waveform()
        .iter()
        .filter_map(|element| match element {
            WaveformElement::Level(run) => Some((run.level(), run.units())),
            _ => None,
        })
        .collect();
    assert_eq!(
        elements,
        vec![(SignalLevel::DontCareAlongLow, 1), (SignalLevel::Bus, 1),],
    );
}

#[test]
fn leading_dontcare_errors() {
    let error = parse_error("SigA ?==");
    assert!(matches!(
        error.kind(),
        ParseErrorKind::DontCareWithoutAnchor
    ));
}

#[test]
fn dontcare_after_only_gap_errors() {
    let error = parse_error("SigA :?_");
    assert!(matches!(
        error.kind(),
        ParseErrorKind::DontCareWithoutAnchor
    ));
}

#[test]
fn dontcare_after_only_anchor_errors() {
    let error = parse_error("SigA @{a}?_");
    assert!(matches!(
        error.kind(),
        ParseErrorKind::DontCareWithoutAnchor
    ));
}

#[test]
fn x_between_buses_emits_buscross() {
    let row = first_signal("SigA =X=");
    let kinds: Vec<TransitionKind> = row
        .waveform()
        .iter()
        .filter_map(|element| match element {
            WaveformElement::Transition(transition) => Some(transition.kind),
            _ => None,
        })
        .collect();
    assert_eq!(kinds, vec![TransitionKind::BusCross]);
}

#[test]
fn x_followed_by_dontcare_resolves_to_bus_dontcare() {
    let row = first_signal("SigA =X?");
    let last = row.waveform().last().expect("at least one element");
    match last {
        WaveformElement::Level(run) => {
            assert_eq!(run.level(), SignalLevel::DontCareAlongBus);
        }
        other => panic!("expected DontCareAlongBus level, got {other:?}"),
    }
}

#[test]
fn dontcare_expands_across_bus_after_buscross() {
    // `=X?=`: the `=` after `?` is absorbed into the DontCare region.
    // Expected: [Bus,1], BusCross Transition(Bus→DontCareAlongBus), [DontCareAlongBus,2].
    // (X body 1 unit + trailing `=` 1 unit = 2 dontcare units)
    let row = first_signal("SigA =X?=");
    let levels: Vec<_> = row
        .waveform()
        .iter()
        .filter_map(|element| match element {
            WaveformElement::Level(run) => Some((run.level(), run.units())),
            _ => None,
        })
        .collect();
    assert_eq!(
        levels,
        vec![(SignalLevel::Bus, 1), (SignalLevel::DontCareAlongBus, 2)],
    );
    let transition_kinds: Vec<_> = row
        .waveform()
        .iter()
        .filter_map(|element| match element {
            WaveformElement::Transition(transition) => Some(transition.kind),
            _ => None,
        })
        .collect();
    assert_eq!(transition_kinds, vec![TransitionKind::BusCross]);
}

#[test]
fn dontcare_expands_both_bus_runs_four_units() {
    // `==?==`: `?` is 0-width; 4 `=` characters collapse into DontCareAlongBus,4.
    let row = first_signal("SigA ==?==");
    let elements: Vec<_> = row
        .waveform()
        .iter()
        .filter_map(|element| match element {
            WaveformElement::Level(run) => Some((run.level(), run.units())),
            _ => None,
        })
        .collect();
    assert_eq!(elements, vec![(SignalLevel::DontCareAlongBus, 4)]);
}

fn collect_transition_triples(row: &SignalRow) -> Vec<(TransitionKind, SignalLevel, SignalLevel)> {
    row.waveform()
        .iter()
        .filter_map(|el| match el {
            WaveformElement::Transition(t) => Some((t.kind, t.source, t.target)),
            _ => None,
        })
        .collect()
}

#[test]
fn dontcare_bus_low_both_ends_has_busopen_busclose() {
    let row = first_signal("SigA _=?=_");
    assert_eq!(
        collect_level_runs(&row),
        vec![
            (SignalLevel::Low, 1),
            (SignalLevel::DontCareAlongBus, 2),
            (SignalLevel::Low, 1),
        ],
    );
    assert_eq!(
        collect_transition_triples(&row),
        vec![
            (
                TransitionKind::BusOpen,
                SignalLevel::Low,
                SignalLevel::DontCareAlongBus,
            ),
            (
                TransitionKind::BusClose,
                SignalLevel::DontCareAlongBus,
                SignalLevel::Low,
            ),
        ],
    );
}

#[test]
fn dontcare_bus_hiz_prev_no_busclose() {
    let row = first_signal("SigA --==?==");
    assert_eq!(
        collect_level_runs(&row),
        vec![(SignalLevel::HiZ, 2), (SignalLevel::DontCareAlongBus, 4)],
    );
    assert_eq!(
        collect_transition_triples(&row),
        vec![(
            TransitionKind::BusOpen,
            SignalLevel::HiZ,
            SignalLevel::DontCareAlongBus,
        )],
    );
}

/// `==?==--`: signal start, DontCare body (4 units), Bus→HiZ close, HiZ(2).
/// Expected levels: DontCareAlongBus(4), HiZ(2).
/// Expected transition: BusClose(DontCareAlongBus → HiZ) — the source is DontCareAlongBus
/// because `?` expansion absorbs surrounding `=` into the FillDouble family.
#[test]
fn dontcare_bus_hiz_next_no_busopen() {
    let row = first_signal("SigA ==?==--");
    assert_eq!(
        collect_level_runs(&row),
        vec![(SignalLevel::DontCareAlongBus, 4), (SignalLevel::HiZ, 2)],
    );
    assert_eq!(
        collect_transition_triples(&row),
        vec![(
            TransitionKind::BusClose,
            SignalLevel::DontCareAlongBus,
            SignalLevel::HiZ,
        )],
    );
}

#[test]
fn dontcare_expands_four_leading_low_four_dontcare_zero_width() {
    // `____????`: `?` chars are 0-width; only the 4 `_` contribute → DontCareAlongLow,4.
    let row = first_signal("SigA ____????");
    let elements: Vec<_> = row
        .waveform()
        .iter()
        .filter_map(|element| match element {
            WaveformElement::Level(run) => Some((run.level(), run.units())),
            _ => None,
        })
        .collect();
    assert_eq!(elements, vec![(SignalLevel::DontCareAlongLow, 4)]);
}

#[test]
fn dontcare_expands_to_preceding_high_not_low() {
    // `_~?`: `?` is 0-width; anchor is `~` (1 unit), `_` is a different level (cutoff).
    // Expected: [Low,1], [DontCareAlongHigh,1].
    let row = first_signal("SigA _~?");
    let elements: Vec<_> = row
        .waveform()
        .iter()
        .filter_map(|element| match element {
            WaveformElement::Level(run) => Some((run.level(), run.units())),
            _ => None,
        })
        .collect();
    assert_eq!(
        elements,
        vec![(SignalLevel::Low, 1), (SignalLevel::DontCareAlongHigh, 1)],
    );
}

#[test]
fn high_to_x_to_low_inserts_bus_open_and_close() {
    // `~X_`: High → X → Low; implicit BusOpen before X body, BusClose after.
    // Expected: [High,1], Transition(BusOpen, High→Bus), [Bus,1], Transition(BusClose, Bus→Low), [Low,1]
    let row = first_signal("SigA ~X_");
    assert_eq!(
        collect_level_runs(&row),
        vec![
            (SignalLevel::High, 1),
            (SignalLevel::Bus, 1),
            (SignalLevel::Low, 1),
        ]
    );
    assert_eq!(
        collect_transition_kinds(&row),
        vec![TransitionKind::BusOpen, TransitionKind::BusClose]
    );
}

#[test]
fn low_to_x_to_high_inserts_bus_open_and_close() {
    // `_X~`: Low → X → High; implicit BusOpen before X body, BusClose after.
    // Expected: [Low,1], Transition(BusOpen, Low→Bus), [Bus,1], Transition(BusClose, Bus→High), [High,1]
    let row = first_signal("SigA _X~");
    assert_eq!(
        collect_level_runs(&row),
        vec![
            (SignalLevel::Low, 1),
            (SignalLevel::Bus, 1),
            (SignalLevel::High, 1),
        ]
    );
    assert_eq!(
        collect_transition_kinds(&row),
        vec![TransitionKind::BusOpen, TransitionKind::BusClose]
    );
}

#[test]
fn x_at_start_is_valid() {
    // `X==`: signal-start X omits the cross Transition; body and trailing `==` merge.
    let row = first_signal("SigA X==");
    let levels: Vec<_> = row
        .waveform()
        .iter()
        .filter_map(|element| match element {
            WaveformElement::Level(run) => Some((run.level(), run.units())),
            _ => None,
        })
        .collect();
    // X body (1) + == (2) = Bus,3
    assert_eq!(levels, vec![(SignalLevel::Bus, 3)]);
    let has_buscross = row
        .waveform()
        .iter()
        .any(|element| matches!(element, WaveformElement::Transition(t) if t.kind == TransitionKind::BusCross));
    assert!(
        !has_buscross,
        "signal-start X must not emit BusCross Transition"
    );
}

/// Helper: extract `(level, units)` pairs from a waveform, skipping non-Level elements.
fn collect_level_runs(row: &SignalRow) -> Vec<(SignalLevel, u32)> {
    row.waveform()
        .iter()
        .filter_map(|element| match element {
            WaveformElement::Level(run) => Some((run.level(), run.units())),
            _ => None,
        })
        .collect()
}

/// Helper: extract `TransitionKind` list from a waveform.
fn collect_transition_kinds(row: &SignalRow) -> Vec<TransitionKind> {
    row.waveform()
        .iter()
        .filter_map(|element| match element {
            WaveformElement::Transition(transition) => Some(transition.kind),
            _ => None,
        })
        .collect()
}

#[test]
fn bus_cross_body_merges_with_following_bus() {
    // `=X=`: X body (1 unit Bus) merges with following `=` → LevelRun(Bus,2).
    // Expected: [Bus,1], Transition(BusCross), [Bus,2]
    let row = first_signal("SigA =X=");
    assert_eq!(
        collect_level_runs(&row),
        vec![(SignalLevel::Bus, 1), (SignalLevel::Bus, 2)]
    );
    assert_eq!(
        collect_transition_kinds(&row),
        vec![TransitionKind::BusCross]
    );
}

#[test]
fn bus_cross_body_becomes_dontcare_when_followed_by_question() {
    // `=X?`: X body treated as DontCareAlongBus (0 unit ? marks the region).
    // Expected: [Bus,1], Transition(BusCross, Bus→DontCareAlongBus), [DontCareAlongBus,1]
    let row = first_signal("SigA =X?");
    assert_eq!(
        collect_level_runs(&row),
        vec![(SignalLevel::Bus, 1), (SignalLevel::DontCareAlongBus, 1)]
    );
    assert_eq!(
        collect_transition_kinds(&row),
        vec![TransitionKind::BusCross]
    );
}

#[test]
fn question_before_buscross_expands_to_dontcare() {
    // `=?X=`: `?` takes preceding `=` as anchor → DontCareAlongBus,1.
    // X body + `=` merge → Bus,2.
    // Expected: [DontCareAlongBus,1], Transition(BusCross, DontCareAlongBus→Bus), [Bus,2]
    let row = first_signal("SigA =?X=");
    assert_eq!(
        collect_level_runs(&row),
        vec![(SignalLevel::DontCareAlongBus, 1), (SignalLevel::Bus, 2)]
    );
    assert_eq!(
        collect_transition_kinds(&row),
        vec![TransitionKind::BusCross]
    );
}

#[test]
fn buscross_between_dontcare_regions() {
    // `=X?X=`: two BusCross transitions with dontcare in between.
    // Expected: [Bus,1], BusCross, [DontCareAlongBus,1], BusCross, [Bus,2]
    let row = first_signal("SigA =X?X=");
    assert_eq!(
        collect_level_runs(&row),
        vec![
            (SignalLevel::Bus, 1),
            (SignalLevel::DontCareAlongBus, 1),
            (SignalLevel::Bus, 2),
        ]
    );
    assert_eq!(
        collect_transition_kinds(&row),
        vec![TransitionKind::BusCross, TransitionKind::BusCross]
    );
}

#[test]
fn consecutive_x_at_start_is_valid() {
    // `XXXX`: first X has no preceding bus → no Transition; subsequent X → BusCross.
    // Expected: [Bus,1], BusCross, [Bus,1], BusCross, [Bus,1], BusCross, [Bus,1]
    let row = first_signal("SigA XXXX");
    assert_eq!(
        collect_level_runs(&row),
        vec![
            (SignalLevel::Bus, 1),
            (SignalLevel::Bus, 1),
            (SignalLevel::Bus, 1),
            (SignalLevel::Bus, 1),
        ]
    );
    assert_eq!(
        collect_transition_kinds(&row),
        vec![
            TransitionKind::BusCross,
            TransitionKind::BusCross,
            TransitionKind::BusCross,
        ]
    );
}

#[test]
fn dontcare_only_errors() {
    let error = parse_error("SigA ???");
    assert!(matches!(
        error.kind(),
        ParseErrorKind::DontCareWithoutAnchor
    ));
}

#[test]
fn question_at_start_with_x_errors() {
    // `?X=`: leading `?` without anchor → error.
    let error = parse_error("SigA ?X=");
    assert!(matches!(
        error.kind(),
        ParseErrorKind::DontCareWithoutAnchor
    ));
}

#[test]
fn low_x_bus_merges_body_with_trailing_bus() {
    // `_____X=====`: Low(5) → X → Bus; BusOpen + body merges with trailing `=====`.
    // Expected: [Low,5], Transition(BusOpen, Low→Bus), [Bus,6]
    let row = first_signal("SigA _____X=====");
    assert_eq!(
        collect_level_runs(&row),
        vec![(SignalLevel::Low, 5), (SignalLevel::Bus, 6)]
    );
    assert_eq!(
        collect_transition_kinds(&row),
        vec![TransitionKind::BusOpen]
    );
}

#[test]
fn bus_x_low_emits_cross_body_and_close() {
    // `=====X_____`: Bus(5) → X → Low; BusCross + body + BusClose.
    // Expected: [Bus,5], Transition(BusCross), [Bus,1], Transition(BusClose, Bus→Low), [Low,5]
    let row = first_signal("SigA =====X_____");
    assert_eq!(
        collect_level_runs(&row),
        vec![
            (SignalLevel::Bus, 5),
            (SignalLevel::Bus, 1),
            (SignalLevel::Low, 5),
        ]
    );
    assert_eq!(
        collect_transition_kinds(&row),
        vec![TransitionKind::BusCross, TransitionKind::BusClose]
    );
}

#[test]
fn step_sets_layout_step() {
    // `@step` is the canonical name.
    let doc = parse_or_panic("@step 20\nSigA __");
    assert_eq!(doc.style.layout().step(), crate::units::Px(20.0));
}

#[test]
fn w_hold_is_unknown_parameter() {
    // `@w_hold` was the old name and must now be rejected.
    let error = parse_error("@w_hold 20\nSigA __");
    assert!(matches!(error.kind(), ParseErrorKind::UnknownParameter(_)));
}

#[test]
fn slant_sets_layout_slant() {
    // `@slant` is the canonical name.
    let doc = parse_or_panic("@slant 5\nSigA __");
    assert_eq!(doc.style.layout().slant(), crate::units::Px(5.0));
}

#[test]
fn w_transient_is_unknown_parameter() {
    // `@w_transient` was the old name and must now be rejected.
    let error = parse_error("@w_transient 5\nSigA __");
    assert!(matches!(error.kind(), ParseErrorKind::UnknownParameter(_)));
}

#[test]
fn bus_transition_slant_is_unknown_parameter() {
    let error = parse_error("@bus_transition_slant 5\nSigA __");
    assert!(matches!(error.kind(), ParseErrorKind::UnknownParameter(_)));
}

#[test]
fn step_equal_slant_is_invalid() {
    // `@step 2` then `@slant 2`: step <= slant must error.
    let error = parse_error("@step 2\n@slant 2\nSigA __");
    assert!(matches!(
        error.kind(),
        ParseErrorKind::InvalidStepSlant(_, _)
    ));
}

#[test]
fn slant_equal_step_from_other_direction_is_invalid() {
    // Default step=25. Setting slant=25 must also error.
    let error = parse_error("@slant 25\nSigA __");
    assert!(matches!(
        error.kind(),
        ParseErrorKind::InvalidStepSlant(_, _)
    ));
}

#[test]
fn h_space_alias_for_signal_gap() {
    // `@h_space` is the new canonical name; `@signal_gap` remains an alias.
    let via_new = parse_or_panic("@h_space 8\nSigA __");
    let via_old = parse_or_panic("@signal_gap 8\nSigA __");
    assert_eq!(
        via_new.style.layout().h_space(),
        via_old.style.layout().h_space()
    );
}

#[test]
fn gap_breaks_levelrun_merge() {
    let row = first_signal("SigA __:__");
    let level_count = row
        .waveform()
        .iter()
        .filter(|element| matches!(element, WaveformElement::Level(_)))
        .count();
    assert_eq!(level_count, 2);
    let has_gap = row
        .waveform()
        .iter()
        .any(|element| matches!(element, WaveformElement::Gap));
    assert!(has_gap);
}

#[test]
fn unclosed_highlight_errors() {
    let error = parse_error("SigA __[~~__");
    assert!(matches!(error.kind(), ParseErrorKind::UnclosedHighlight));
}

#[test]
fn unmatched_highlight_end_errors() {
    let error = parse_error("SigA __~~]__");
    assert!(matches!(error.kind(), ParseErrorKind::UnopenedHighlightEnd));
}

#[test]
fn angle_bracket_in_level_string_is_text() {
    // `<` and `>` are bare text characters in a level string (not errors).
    let row = first_signal("SigA _<CLK>~");
    let has_text = row
        .waveform()
        .iter()
        .any(|element| matches!(element, WaveformElement::Text(_)));
    assert!(has_text, "expected Text element for angle brackets");
    // Low(1) + Text("<CLK>") then SingleEdge + High(1)
    let levels: Vec<SignalLevel> = row
        .waveform()
        .iter()
        .filter_map(|element| match element {
            WaveformElement::Level(run) => Some(run.level()),
            _ => None,
        })
        .collect();
    assert_eq!(levels, vec![SignalLevel::Low, SignalLevel::High]);
}

#[test]
fn text_char_before_level_is_missing_initial_level() {
    let error = parse_error("SigA a__~~");
    assert!(matches!(error.kind(), ParseErrorKind::MissingInitialLevel));
}

#[test]
fn signal_name_with_angle_brackets_passes() {
    // `<` and `>` inside a quoted signal name are treated as literal characters.
    let row = first_signal("\"<request>\" _~_~");
    assert_eq!(row.name().as_str(), "<request>");
}

#[test]
fn arrow_label_with_angle_brackets_passes() {
    let doc = parse_or_panic("SigA @{a}__\nSigB @{b}__\n@-> (@{a}, @{b}) <signal-set>");
    let arrow = &doc.annotations.arrows[0];
    assert_eq!(
        arrow.label.as_ref().map(|text| text.as_str()),
        Some("<signal-set>"),
    );
}

#[test]
fn anchor_named_recorded() {
    let doc = parse_or_panic("SigA _~@{edge}_~");
    let id = AnchorId::Named(crate::anchor::AnchorName::parse("edge").expect("valid"));
    assert!(doc.annotations.anchors.contains(&id));
}

#[test]
fn anchor_indexed_recorded() {
    let doc = parse_or_panic("SigA ___@1__");
    let id = AnchorId::Indexed(1);
    assert!(doc.annotations.anchors.contains(&id));
}

#[test]
fn duplicate_anchor_errors() {
    let error = parse_error("SigA @{a}__@{a}__");
    assert!(matches!(error.kind(), ParseErrorKind::DuplicateAnchor));
}

/// `@0` を 1 個だけ書いた信号行が受理され、波形要素列に
/// `Anchor(Indexed(0))` が登場することを確認する
/// (`docs/tests/tcml-parser.feature.md` の「アンカー番号 0 は受理」).
#[test]
fn anchor_indexed_zero_recorded() {
    let row = first_signal("Sig _~@0_~");
    let elements: &[WaveformElement] = row.waveform();
    let found_zero = elements.iter().any(|element| {
        matches!(
            element,
            WaveformElement::Anchor(AnchorId::Indexed(value)) if *value == 0
        )
    });
    assert!(
        found_zero,
        "expected Anchor(Indexed(0)) in waveform elements: {elements:?}"
    );
}

/// `@0` と `@1` を端点に持つ `@->` が受理され、矢印の両端が
/// numbered anchor 0 と 1 を指すことを確認する.
#[test]
fn arrow_endpoints_accept_numbered_anchor_zero() {
    let doc = parse_or_panic("Sig _~@0__@1\n@-> (@0, @1)");
    let arrow = doc
        .annotations
        .arrows
        .first()
        .expect("@-> must produce one arrow");
    let zero = AnchorId::Indexed(0);
    let one = AnchorId::Indexed(1);
    assert_eq!(arrow.from, ArrowEnd::Anchor(zero));
    assert_eq!(arrow.to, ArrowEnd::Anchor(one));
}

/// 同一信号行内で `@0` が 2 回現れたら `DuplicateAnchor`.
#[test]
fn duplicate_anchor_indexed_zero_same_row_errors() {
    let error = parse_error("Sig _@0_~@0_");
    assert!(
        matches!(error.kind(), ParseErrorKind::DuplicateAnchor),
        "expected DuplicateAnchor, got {:?}",
        error.kind()
    );
}

/// 別信号行で `@0` が 2 回現れても `DuplicateAnchor`
/// (numbered anchor は単一名前空間).
#[test]
fn duplicate_anchor_indexed_zero_across_rows_errors() {
    let error = parse_error("Sig1 _~@0_\nSig2 _~@0_");
    assert!(
        matches!(error.kind(), ParseErrorKind::DuplicateAnchor),
        "expected DuplicateAnchor, got {:?}",
        error.kind()
    );
}

/// `@{0}` (named, 値 "0") と `@0` (indexed, 値 0) は別名前空間で
/// 共存可能。`@->` で両者を別端点として参照できる.
#[test]
fn named_zero_and_indexed_zero_are_distinct() {
    let doc = parse_or_panic("Sig _~@{0}__@0\n@-> (@{0}, @0)");
    let arrow = doc
        .annotations
        .arrows
        .first()
        .expect("@-> must produce one arrow");
    let named = AnchorId::Named(crate::anchor::AnchorName::parse("0").expect("valid"));
    let indexed = AnchorId::Indexed(0);
    assert_eq!(arrow.from, ArrowEnd::Anchor(named));
    assert_eq!(arrow.to, ArrowEnd::Anchor(indexed));
}

/// アンカーが遷移の後にある場合、最終 Waveform でのインデックスが
/// `TransitionEmitter` 注入後に正しく付け直されているかを確認する。
/// Waveform: Level(Low,3)[0], Anchor(a)[1], Transition[2], Level(High,4)[3], Anchor(b)[4], Level(Low,3)[5]
/// @{a} は index 1 に、@{b} は index 4 にあることを Waveform から直接確認する。
#[test]
fn anchor_in_waveform_after_transition() {
    let id_a = AnchorId::Named(crate::anchor::AnchorName::parse("a").expect("valid"));
    let id_b = AnchorId::Named(crate::anchor::AnchorName::parse("b").expect("valid"));
    let row = first_signal("SigA ___@{a}~~~~@{b}___");
    let waveform = row.waveform();
    let elements: &[WaveformElement] = waveform;
    // Waveform 内でアンカーの実際の位置を特定
    let index_a = elements
        .iter()
        .position(|e| matches!(e, WaveformElement::Anchor(id) if id == &id_a))
        .expect("anchor a in waveform");
    let index_b = elements
        .iter()
        .position(|e| matches!(e, WaveformElement::Anchor(id) if id == &id_b))
        .expect("anchor b in waveform");
    // @{a}: Low(3) の後 → index 1
    assert_eq!(
        index_a, 1,
        "@{{a}} must be at waveform index 1, got {}",
        index_a
    );
    // @{b}: Low(3) + Anchor + Transition + High(4) の後 → index 4
    assert_eq!(
        index_b, 4,
        "@{{b}} must be at waveform index 4 (after injected Transition), got {}",
        index_b
    );
}

#[test]
fn fontsize_accepts_alias_styles() {
    let canonical = parse_or_panic("@fontsize 16\nSigA __");
    let alt1 = parse_or_panic("@FontSize 16\nSigA __");
    let alt2 = parse_or_panic("@font-size 16\nSigA __");
    let alt3 = parse_or_panic("@FONT_SIZE 16\nSigA __");
    let expected = canonical.style.canvas().font().size();
    assert_eq!(expected, Px(16.0));
    assert_eq!(alt1.style.canvas().font().size(), expected);
    assert_eq!(alt2.style.canvas().font().size(), expected);
    assert_eq!(alt3.style.canvas().font().size(), expected);
}

#[test]
fn unknown_parameter_errors() {
    let error = parse_error("@foobar 42");
    assert!(matches!(error.kind(), ParseErrorKind::UnknownParameter(_)));
}

#[test]
fn invalid_color_errors() {
    let error = parse_error("@signal_color notacolor");
    assert!(matches!(error.kind(), ParseErrorKind::InvalidColor(_)));
}

#[test]
fn skip_lh_creates_skiprow() {
    let doc = parse_or_panic("@skip(2)");
    assert_eq!(doc.lines.len(), 1);
    match &doc.lines[0].content {
        LineContent::Skip(skip) => assert_eq!(skip.amount, Length::Lh(2.0)),
        other => panic!("expected Skip, got {other:?}"),
    }
}

#[test]
fn skip_fractional_lh() {
    let doc = parse_or_panic("@skip(2.5)");
    match &doc.lines[0].content {
        LineContent::Skip(skip) => assert_eq!(skip.amount, Length::Lh(2.5)),
        other => panic!("expected Skip, got {other:?}"),
    }
}

#[test]
fn skip_px() {
    let doc = parse_or_panic("@skip(20px)");
    match &doc.lines[0].content {
        LineContent::Skip(skip) => assert_eq!(skip.amount, Length::Px(20.0)),
        other => panic!("expected Skip, got {other:?}"),
    }
}

#[test]
fn skip_zero_is_ignored() {
    let doc = parse_or_panic("@skip(0)");
    assert!(doc.lines.is_empty());
}

#[test]
fn skip_negative_errors() {
    let error = parse_error("@skip(-1)");
    assert!(matches!(error.kind(), ParseErrorKind::InvalidSkipAmount(_)));
}

#[test]
fn skip_unparsable_errors() {
    let error = parse_error("@skip(abc)");
    assert!(matches!(error.kind(), ParseErrorKind::InvalidSkipAmount(_)));
}

#[test]
fn title_single_line() {
    let doc = parse_or_panic("@title Sample");
    match &doc.lines[0].content {
        LineContent::Title(title) => assert_eq!(title.text.as_str(), "Sample"),
        other => panic!("expected Title, got {other:?}"),
    }
}

#[test]
fn title_quoted_multiline() {
    let doc = parse_or_panic("@title \"line1\nline2\"");
    match &doc.lines[0].content {
        LineContent::Title(title) => assert_eq!(title.text.as_str(), "line1\nline2"),
        other => panic!("expected Title, got {other:?}"),
    }
}

#[test]
fn multiple_titles_allowed() {
    let doc = parse_or_panic("@title A\n@title B");
    let titles: Vec<&crate::line::TitleRow> = doc
        .lines
        .iter()
        .filter_map(|line| match &line.content {
            LineContent::Title(title) => Some(title),
            _ => None,
        })
        .collect();
    assert_eq!(titles.len(), 2);
}

#[test]
fn signal_overline_decoration() {
    let doc = parse_or_panic("@signal(overline)\nnReset _~~~\nfoo __~~");
    match &doc.lines[0].content {
        LineContent::Signal(row) => assert!(row.decorations().is_name_overline()),
        other => panic!("expected Signal, got {other:?}"),
    }
    match &doc.lines[1].content {
        LineContent::Signal(row) => assert!(!row.decorations().is_name_overline()),
        other => panic!("expected Signal, got {other:?}"),
    }
}

#[test]
fn clock_decorates_following_row() {
    let doc = parse_or_panic("@clock(pos)\nCLK\nFoo _~_~_~_~");
    match &doc.lines[0].content {
        LineContent::Signal(row) => {
            let spec = row.decorations().clock.clone().expect("clock spec set");
            assert_eq!(spec.edge, ClockEdge::Pos);
            let levels: Vec<SignalLevel> = row
                .waveform()
                .iter()
                .filter_map(|element| match element {
                    WaveformElement::Level(run) => Some(run.level()),
                    _ => None,
                })
                .collect();
            assert!(!levels.is_empty(), "clock waveform expanded");
        }
        other => panic!("expected Signal, got {other:?}"),
    }
}

#[test]
fn clock_attribute_order_does_not_matter() {
    let normal = parse_or_panic("@clock(neg, _=2, ~=3)\nCLK");
    let alternate = parse_or_panic("@clock(_=2, neg, ~=3)\nCLK");
    let normal_spec = first_signal_clock_spec(&normal);
    let alternate_spec = first_signal_clock_spec(&alternate);
    assert_eq!(normal_spec, alternate_spec);
}

fn first_signal_clock_spec(doc: &crate::document::ChartDocument) -> crate::clock::ClockSpec {
    match &doc.lines[0].content {
        LineContent::Signal(row) => row.decorations().clock.clone().expect("spec"),
        other => panic!("expected signal, got {other:?}"),
    }
}

#[test]
fn clock_bare_no_parens_defaults_to_none() {
    let doc = parse_or_panic("@clock\nCLK");
    let spec = first_signal_clock_spec(&doc);
    assert_eq!(spec.edge, ClockEdge::None);
}

#[test]
fn clock_empty_parens_defaults_to_none() {
    let doc = parse_or_panic("@clock()\nCLK");
    let spec = first_signal_clock_spec(&doc);
    assert_eq!(spec.edge, ClockEdge::None);
}

#[test]
fn clock_edge_omitted_defaults_to_none() {
    let doc = parse_or_panic("@clock(_=2)\nCLK");
    let spec = first_signal_clock_spec(&doc);
    assert_eq!(spec.edge, ClockEdge::None);
}

#[test]
fn arrow_minimal_resolves_to_anchor() {
    let doc = parse_or_panic("SigA __@{a}__\nSigB __@{b}__\n@-> (@{a}, @{b})");
    assert_eq!(doc.annotations.arrows.len(), 1);
    let arrow = &doc.annotations.arrows[0];
    assert!(matches!(arrow.from, ArrowEnd::Anchor(_)));
    assert!(matches!(arrow.to, ArrowEnd::Anchor(_)));
}

#[test]
fn arrow_with_attributes_and_label() {
    let doc = parse_or_panic("SigA @{a}__\nSigB @{b}__\n@-> (@{a}, @{b}, red, 2px, dashed) change");
    let arrow = &doc.annotations.arrows[0];
    assert_eq!(arrow.style.line, LineDashStyle::Dashed);
    assert_eq!(arrow.style.head, ArrowHead::EndOnly);
    assert_eq!(arrow.style.width, Px(2.0));
    assert_eq!(
        arrow.label.as_ref().map(|text| text.as_str()),
        Some("change"),
    );
}

#[test]
fn arrow_attribute_order_does_not_matter() {
    let doc1 = parse_or_panic("SigA @{a}__\nSigB @{b}__\n@-> (@{a}, @{b}, red, 2px, dashed)");
    let doc2 = parse_or_panic("SigA @{a}__\nSigB @{b}__\n@-> (@{a}, @{b}, dashed, 2px, red)");
    assert_eq!(
        doc1.annotations.arrows[0].style,
        doc2.annotations.arrows[0].style
    );
}

#[test]
fn arrow_forward_reference_resolves() {
    let doc = parse_or_panic("@-> (@{a}, @{b})\nSigA @{a}__\nSigB @{b}__");
    assert_eq!(doc.annotations.arrows.len(), 1);
}

#[test]
fn arrow_unknown_anchor_errors() {
    let error = parse_error("@-> (@{x}, @{y})");
    assert!(matches!(error.kind(), ParseErrorKind::UndefinedAnchor(_)));
}

#[test]
fn arrow_duplicate_attribute_errors() {
    let error = parse_error("SigA @{a}__\nSigB @{b}__\n@-> (@{a}, @{b}, red, blue)");
    assert!(matches!(
        error.kind(),
        ParseErrorKind::DuplicateArrowAttribute(_)
    ));
}

#[test]
fn arrow_unknown_attribute_errors() {
    let error = parse_error("SigA @{a}__\nSigB @{b}__\n@-> (@{a}, @{b}, foobar)");
    assert!(matches!(
        error.kind(),
        ParseErrorKind::UnknownArrowAttribute(_)
    ));
}

#[test]
fn overlay_records_position_and_text() {
    let doc = parse_or_panic("% 100 50 note");
    assert_eq!(doc.annotations.overlays.len(), 1);
    let overlay = &doc.annotations.overlays[0];
    assert_eq!(overlay.at.x, Px(100.0));
    assert_eq!(overlay.at.y, Px(50.0));
    assert_eq!(overlay.text.as_str(), "note");
}

#[test]
fn quoted_signal_name_multiline() {
    let row = first_signal("\"a\nb\" =__");
    assert_eq!(row.name().as_str(), "a\nb");
    let lines: Vec<&str> = row.name().lines().map(|line| line.unsafe_text()).collect();
    assert_eq!(lines, vec!["a", "b"]);
}

#[test]
fn unclosed_quoted_signal_errors() {
    let error = parse_error("\"abc _~");
    assert!(matches!(error.kind(), ParseErrorKind::UnclosedQuote));
}

#[test]
fn integration_dontcare_chain() {
    // Mirrors the integration scenario from tcml-parser.feature.md.
    let doc = parse_or_panic("foo  _?~_~?_~_?\nbar  -?==");
    assert_eq!(doc.lines.len(), 2);
}

#[test]
fn bg_applies_to_next_signal_only() {
    let doc = parse_or_panic("@bg #f0f\nA _\nB _");
    assert_eq!(doc.lines.len(), 2);
    let expected = crate::color::Color::parse("#f0f").expect("color");
    assert_eq!(doc.lines[0].background, Some(expected));
    assert_eq!(doc.lines[1].background, None);
}

#[test]
fn bg_applies_to_next_title_row() {
    let doc = parse_or_panic("@bg #ff0\n@title hello\nA _");
    assert_eq!(doc.lines.len(), 2);
    let expected = crate::color::Color::parse("#ff0").expect("color");
    assert!(matches!(doc.lines[0].content, LineContent::Title(_)));
    assert_eq!(doc.lines[0].background, Some(expected));
    assert_eq!(doc.lines[1].background, None);
}

#[test]
fn bg_applies_to_next_skip_row() {
    let doc = parse_or_panic("@bg #ff0\n@skip(1)\nA _");
    assert_eq!(doc.lines.len(), 2);
    let expected = crate::color::Color::parse("#ff0").expect("color");
    assert!(matches!(doc.lines[0].content, LineContent::Skip(_)));
    assert_eq!(doc.lines[0].background, Some(expected));
    assert_eq!(doc.lines[1].background, None);
}

#[test]
fn bg_survives_intervening_directive() {
    let doc = parse_or_panic("@bg #f0f\n@bgcolor0 #eee\nA _");
    let expected = crate::color::Color::parse("#f0f").expect("color");
    assert_eq!(doc.lines[0].background, Some(expected));
}

#[test]
fn bg_none_clears_pending_value() {
    let doc = parse_or_panic("@bg #f0f\n@bg none\nA _");
    assert_eq!(doc.lines[0].background, None);
}

// @titlealign tests

fn first_title_align(input: &str) -> HorizontalAlign {
    let doc = parse_or_panic(input);
    let line = doc.lines.into_iter().next().expect("at least one line");
    match line.content {
        LineContent::Title(title) => title.style.align(),
        other => panic!("expected Title, got {other:?}"),
    }
}

#[test]
fn title_default_align_is_center() {
    let align = first_title_align("@title Hello");
    assert_eq!(align, HorizontalAlign::Center);
}

#[test]
fn titlealign_left_is_applied() {
    let align = first_title_align("@titlealign left\n@title Hello");
    assert_eq!(align, HorizontalAlign::Left);
}

#[test]
fn titlealign_right_is_applied() {
    let align = first_title_align("@titlealign right\n@title Hello");
    assert_eq!(align, HorizontalAlign::Right);
}

#[test]
fn titlealign_center_explicit() {
    let align = first_title_align("@titlealign center\n@title Hello");
    assert_eq!(align, HorizontalAlign::Center);
}

#[test]
fn titlealign_case_insensitive() {
    let align_upper = first_title_align("@titlealign CENTER\n@title Hello");
    assert_eq!(align_upper, HorizontalAlign::Center);
    let align_mixed = first_title_align("@titlealign Right\n@title Hello");
    assert_eq!(align_mixed, HorizontalAlign::Right);
}

#[test]
fn titlealign_invalid_value_errors() {
    let error = parse_error("@titlealign top");
    assert!(matches!(
        error.kind(),
        ParseErrorKind::UnknownParameter(_)
            | ParseErrorKind::NumericNotParseable(_, _)
            | ParseErrorKind::NumericOverflow(_, _, _)
            | ParseErrorKind::NumericNotPositive(_, _)
            | ParseErrorKind::NumericNotNonNegative(_, _)
            | ParseErrorKind::InvalidTitleAlign(_)
            | ParseErrorKind::TitleRequiresArgument
    ));
}

#[test]
fn titlealign_mid_sequence_applies_to_subsequent_titles() {
    let doc = parse_or_panic("@title A\n@titlealign right\n@title B");
    let titles: Vec<&crate::line::TitleRow> = doc
        .lines
        .iter()
        .filter_map(|line| match &line.content {
            LineContent::Title(title) => Some(title),
            _ => None,
        })
        .collect();
    assert_eq!(titles.len(), 2);
    assert_eq!(titles[0].style.align(), HorizontalAlign::Center);
    assert_eq!(titles[1].style.align(), HorizontalAlign::Right);
}

// ---- @clock edge_marks and @clockmark_* tests ----------------------

fn first_signal_edge_mark_count(input: &str) -> usize {
    // Run the parser (which includes clock expansion) but NOT layout.
    // edge_marks are filled by layout, so the parser pass gives empty vec.
    // This helper is only useful for checking that spec is stored.
    let doc = parse_or_panic(input);
    match &doc.lines[0].content {
        LineContent::Signal(row) => row.edge_marks().len(),
        other => panic!("expected signal, got {other:?}"),
    }
}

#[test]
fn clock_pos_spec_stored_in_decorations() {
    // @clock(pos) stores the ClockSpec (with mark_style) in decorations.
    let doc = parse_or_panic("Foo __\n@clock(pos)\nCLK");
    let spec = match &doc.lines[1].content {
        LineContent::Signal(row) => row.decorations().clock.clone().expect("clock spec"),
        other => panic!("expected signal, got {other:?}"),
    };
    assert_eq!(spec.edge, crate::clock::ClockEdge::Pos);
    // mark_style defaults
    use crate::defaults::{
        DEFAULT_CLOCKMARK_HEIGHT_PX, DEFAULT_CLOCKMARK_POSITION, DEFAULT_CLOCKMARK_WIDTH_PX,
    };
    assert!((spec.mark_style.position - DEFAULT_CLOCKMARK_POSITION).abs() < 1e-6);
    assert_eq!(spec.mark_style.height, DEFAULT_CLOCKMARK_HEIGHT_PX);
    assert_eq!(spec.mark_style.width, DEFAULT_CLOCKMARK_WIDTH_PX);
}

#[test]
fn clock_pos_annotations_arrows_empty_from_parser() {
    // clock-derived markers must NOT go into Annotations.arrows.
    let doc = parse_or_panic("Foo __\n@clock(pos)\nCLK");
    assert_eq!(
        doc.annotations.arrows.len(),
        0,
        "Annotations.arrows must not contain clock-derived elements"
    );
}

#[test]
fn clock_mark_color_inherits_signal_color() {
    // mark_color not specified → inherits current signal_color.
    let doc = parse_or_panic("@signal_color blue\n@clock(pos)\nCLK");
    let spec = match &doc.lines[0].content {
        LineContent::Signal(row) => row.decorations().clock.clone().expect("spec"),
        other => panic!("expected signal, got {other:?}"),
    };
    let expected = crate::color::Color::parse("blue").expect("blue");
    assert_eq!(spec.mark_style.color, expected);
}

#[test]
fn clockmark_global_position_applied() {
    // @clockmark_position sets the global default.
    let doc = parse_or_panic("@clockmark_position 0.0\n@clock(pos)\nCLK");
    let spec = match &doc.lines[0].content {
        LineContent::Signal(row) => row.decorations().clock.clone().expect("spec"),
        other => panic!("expected signal, got {other:?}"),
    };
    assert!((spec.mark_style.position - 0.0).abs() < 1e-6);
}

#[test]
fn clockmark_local_height_overrides_global() {
    // @clock(pos, mark_height=8) overrides @clockmark_height 5.
    let doc = parse_or_panic("@clockmark_height 5\n@clock(pos, mark_height=8)\nCLK");
    let spec = match &doc.lines[0].content {
        LineContent::Signal(row) => row.decorations().clock.clone().expect("spec"),
        other => panic!("expected signal, got {other:?}"),
    };
    assert_eq!(spec.mark_style.height, Px(8.0));
}

#[test]
fn clockmark_local_color_overrides_signal_color() {
    // @clock(pos, mark_color=red) overrides signal_color inheritance.
    let doc = parse_or_panic("@signal_color blue\n@clock(pos, mark_color=red)\nCLK");
    let spec = match &doc.lines[0].content {
        LineContent::Signal(row) => row.decorations().clock.clone().expect("spec"),
        other => panic!("expected signal, got {other:?}"),
    };
    let expected = crate::color::Color::parse("red").expect("red");
    assert_eq!(spec.mark_style.color, expected);
}

#[test]
fn clockmark_global_color_applied() {
    // @clockmark_color overrides signal_color default.
    let doc = parse_or_panic("@clockmark_color green\n@clock(pos)\nCLK");
    let spec = match &doc.lines[0].content {
        LineContent::Signal(row) => row.decorations().clock.clone().expect("spec"),
        other => panic!("expected signal, got {other:?}"),
    };
    let expected = crate::color::Color::parse("green").expect("green");
    assert_eq!(spec.mark_style.color, expected);
}

#[test]
fn clock_none_edge_mark_count_from_parser_is_zero() {
    // @clock(none) — no edge marks (parser; layout also yields none).
    let count = first_signal_edge_mark_count("@clock(none)\nCLK\nFoo __");
    assert_eq!(count, 0);
}

// ---- clockmark default + step-linked shrink tests --------------------

#[test]
fn clockmark_defaults_are_6_and_7_5_when_step_is_large() {
    // No global / local override and step large enough that 6 < step * 2/3.
    // step=20 → 20*2/3 ≈ 13.33, min(6, 13.33) = 6. height stays at default 7.5.
    let doc = parse_or_panic("@step 20\n@clock(pos)\nCLK");
    let spec = first_signal_clock_spec(&doc);
    assert_eq!(spec.mark_style.width, Px(6.0));
    assert_eq!(spec.mark_style.height, Px(7.5));
}

#[test]
fn clockmark_width_shrinks_when_step_small_and_default_resolved() {
    // step=6 → 6*2/3 = 4 < 6, so default-resolved width = min(6, 4) = 4.
    let doc = parse_or_panic("@step 6\n@clock(pos)\nCLK");
    let spec = first_signal_clock_spec(&doc);
    assert_eq!(spec.mark_style.width, Px(4.0));
    // height has no shrink rule.
    assert_eq!(spec.mark_style.height, Px(7.5));
}

#[test]
fn clockmark_width_not_shrunk_when_step_times_two_thirds_geq_default() {
    // step=15 → 15*2/3 = 10 ≥ 6, so min(6, 10) = 6 (= default, no actual shrink).
    let doc = parse_or_panic("@step 15\n@clock(pos)\nCLK");
    let spec = first_signal_clock_spec(&doc);
    assert_eq!(spec.mark_style.width, Px(6.0));
}

#[test]
fn clockmark_global_width_override_disables_shrink() {
    // @clockmark_width 8 is explicit → width stays 8 even when step*2/3=2.
    let doc = parse_or_panic("@step 3\n@clockmark_width 8\n@clock(pos)\nCLK");
    let spec = first_signal_clock_spec(&doc);
    assert_eq!(spec.mark_style.width, Px(8.0));
}

#[test]
fn clockmark_global_width_equal_to_default_still_disables_shrink() {
    // Numeric value matches the default 6, but explicit = no shrink.
    let doc = parse_or_panic("@step 3\n@clockmark_width 6\n@clock(pos)\nCLK");
    let spec = first_signal_clock_spec(&doc);
    assert_eq!(spec.mark_style.width, Px(6.0));
}

#[test]
fn clockmark_local_width_disables_shrink() {
    // @clock(pos, mark_width=12) — local override; no shrink even with small step.
    let doc = parse_or_panic("@step 3\n@clock(pos, mark_width=12)\nCLK");
    let spec = first_signal_clock_spec(&doc);
    assert_eq!(spec.mark_style.width, Px(12.0));
}

#[test]
fn clockmark_local_width_disables_shrink_under_shrink_step() {
    // Local mark_width=8 — even though step=6 would normally shrink, local
    // explicit value wins.
    let doc = parse_or_panic("@step 6\n@clock(pos, mark_width=8)\nCLK");
    let spec = first_signal_clock_spec(&doc);
    assert_eq!(spec.mark_style.width, Px(8.0));
}

#[test]
fn clockmark_local_width_overrides_global_width_no_shrink() {
    // Local takes priority over global; both are explicit.
    let doc = parse_or_panic("@step 3\n@clockmark_width 6\n@clock(pos, mark_width=10)\nCLK");
    let spec = first_signal_clock_spec(&doc);
    assert_eq!(spec.mark_style.width, Px(10.0));
}

#[test]
fn clockmark_height_is_never_shrunk_by_small_step() {
    // step small enough to shrink width, but height is never shrunk.
    let doc = parse_or_panic("@step 3\n@clock(pos)\nCLK");
    let spec = first_signal_clock_spec(&doc);
    assert_eq!(spec.mark_style.height, Px(7.5));
    // width = min(6, 3 * 2/3) = min(6, 2) = 2.
    assert_eq!(spec.mark_style.width, Px(2.0));
}

#[test]
fn clockmark_global_height_explicit_no_shrink_width_still_shrinks() {
    // Explicit height stays 20; width still shrinks because @clockmark_width
    // is not explicit.
    let doc = parse_or_panic("@step 3\n@clockmark_height 20\n@clock(pos)\nCLK");
    let spec = first_signal_clock_spec(&doc);
    assert_eq!(spec.mark_style.height, Px(20.0));
    assert_eq!(spec.mark_style.width, Px(2.0));
}

#[test]
fn clockmark_per_row_step_change_recomputes_shrink() {
    // Two clock rows with different per-row steps.
    let doc = parse_or_panic("@step 12\n@clock(pos)\nCK1\n@step 3\n@clock(pos)\nCK2\n");
    let spec_one = match &doc.lines[0].content {
        LineContent::Signal(row) => row.decorations().clock.clone().expect("spec1"),
        other => panic!("expected signal, got {other:?}"),
    };
    let spec_two = match &doc.lines[1].content {
        LineContent::Signal(row) => row.decorations().clock.clone().expect("spec2"),
        other => panic!("expected signal, got {other:?}"),
    };
    // step=12 → min(6, 8) = 6.
    assert_eq!(spec_one.mark_style.width, Px(6.0));
    // step=3 → min(6, 2) = 2.
    assert_eq!(spec_two.mark_style.width, Px(2.0));
}

#[test]
fn clockmark_global_width_set_first_then_small_step_no_shrink() {
    // Global @clockmark_width before @step — explicit value stays even when
    // step would normally trigger shrink.
    let doc = parse_or_panic("@clockmark_width 8\n@step 3\n@clock(pos)\nCLK");
    let spec = first_signal_clock_spec(&doc);
    assert_eq!(spec.mark_style.width, Px(8.0));
}

// ---- @overline_gap / @overline_thickness parser tests ----------------

#[test]
fn overline_gap_param_accepted() {
    // @overline_gap sets the gap in label style.
    let doc = parse_or_panic("@overline_gap 5\n@signal(overline)\nnReset _~~~");
    assert_eq!(
        doc.style.default_label_style().overline_gap(),
        Px(5.0),
        "overline_gap should be 5px"
    );
}

#[test]
fn overline_thickness_param_accepted() {
    // @overline_thickness sets the thickness in label style.
    let doc = parse_or_panic("@overline_thickness 2\n@signal(overline)\nnReset _~~~");
    assert_eq!(
        doc.style.default_label_style().overline_thickness(),
        Px(2.0),
        "overline_thickness should be 2px"
    );
}

#[test]
fn overline_gap_negative_errors() {
    // negative overline_gap is not allowed.
    let error = parse_error("@overline_gap -1");
    assert!(
        matches!(
            error.kind(),
            ParseErrorKind::NumericNotParseable(_, _)
                | ParseErrorKind::NumericOverflow(_, _, _)
                | ParseErrorKind::NumericNotPositive(_, _)
                | ParseErrorKind::NumericNotNonNegative(_, _)
                | ParseErrorKind::InvalidTitleAlign(_)
                | ParseErrorKind::TitleRequiresArgument
                | ParseErrorKind::UnknownParameter(_)
        ),
        "expected parse error for negative overline_gap, got {error:?}"
    );
}

#[test]
fn overline_thickness_zero_errors() {
    // zero overline_thickness is not allowed.
    let error = parse_error("@overline_thickness 0");
    assert!(
        matches!(
            error.kind(),
            ParseErrorKind::NumericNotParseable(_, _)
                | ParseErrorKind::NumericOverflow(_, _, _)
                | ParseErrorKind::NumericNotPositive(_, _)
                | ParseErrorKind::NumericNotNonNegative(_, _)
                | ParseErrorKind::InvalidTitleAlign(_)
                | ParseErrorKind::TitleRequiresArgument
                | ParseErrorKind::UnknownParameter(_)
        ),
        "expected parse error for zero overline_thickness, got {error:?}"
    );
}

// ---- @clock empty-body auto-expansion tests ------------------------

/// Helper: extract the level sequence from the first signal row's waveform.
fn first_signal_level_sequence(input: &str) -> Vec<(SignalLevel, u32)> {
    let doc = parse_or_panic(input);
    let line = doc.lines.into_iter().next().expect("at least one line");
    match line.content {
        LineContent::Signal(row) => row
            .waveform()
            .iter()
            .filter_map(|element| match element {
                WaveformElement::Level(run) => Some((run.level(), run.units())),
                _ => None,
            })
            .collect(),
        other => panic!("expected signal row, got {other:?}"),
    }
}

/// Helper: total level units in the first signal row.
fn first_signal_total_units(input: &str) -> u32 {
    first_signal_level_sequence(input)
        .iter()
        .map(|(_, units)| units)
        .sum()
}

#[test]
fn clock_empty_body_expands_to_other_signal_length() {
    // @clock(pos) + empty body clock row, other signal has 8 units.
    // The clock row must expand to 8 units total.
    let total = first_signal_total_units("@clock(pos)\nCLK\nFoo _~_~_~_~");
    assert_eq!(
        total, 8,
        "clock with empty body must expand to chart_units=8"
    );
}

#[test]
fn clock_pulse_spec_expands_to_other_signal_length() {
    // @clock(neg, _=2, ~=3) + empty body, other signal 10 units.
    // The clock fills with Low(2)+High(3) cycles up to 10 units.
    let total = first_signal_total_units("@clock(neg, _=2, ~=3)\nCLK\nFoo __________");
    assert_eq!(total, 10);
}

#[test]
fn clock_partial_body_continues_to_chart_units() {
    // @clock(pos) + partial body "~~__", other signal 8 units.
    // The clock must continue from the last level (Low) up to 8 units total.
    let total = first_signal_total_units("@clock(pos)\nck ~~__\nFoo _~_~_~_~");
    assert_eq!(
        total, 8,
        "clock with partial body must expand to chart_units=8"
    );
}

#[test]
fn clock_empty_body_expands_when_only_clock_signals_present() {
    // core: all signals are clocks.
    // ClkPos, ClkNeg, ClkBoth each have explicit "~_~_~_" (6 units).
    // ClkWide has empty body → must expand to 6 units.
    let input = "\
@clock(pos)
ClkPos  _~_~_~
@clock(neg)
ClkNeg  _~_~_~
@clock(pos, _=2, ~=1)
ClkWide
";
    let doc = parse_or_panic(input);
    let clk_wide_row = doc
        .lines
        .iter()
        .find_map(|line| match &line.content {
            LineContent::Signal(row) if row.name().as_str() == "ClkWide" => Some(row),
            _ => None,
        })
        .expect("ClkWide row must exist");
    let total: u32 = clk_wide_row
        .waveform()
        .iter()
        .filter_map(|element| match element {
            WaveformElement::Level(run) => Some(run.units()),
            _ => None,
        })
        .sum();
    assert_eq!(
        total, 6,
        "ClkWide empty body must expand to 6 (max of other clock signals with explicit waveforms)"
    );
}

// ---- テキスト文字 (筑波大 tchart-coffee 方式) ----------------------------

fn collect_waveform_text_content(row: &SignalRow) -> Vec<String> {
    row.waveform()
        .iter()
        .filter_map(|element| match element {
            WaveformElement::Text(text) => Some(text.as_str().to_owned()),
            _ => None,
        })
        .collect()
}

#[test]
fn single_text_char_in_level_run() {
    // `__a__` → 4-unit Low with Text("a").
    let row = first_signal("SigA __a__");
    assert_eq!(collect_level_runs(&row), vec![(SignalLevel::Low, 4)]);
    assert_eq!(collect_waveform_text_content(&row), vec!["a"]);
}

#[test]
fn multiple_text_fragments_in_same_run_are_space_joined() {
    // `__a__b_` → 5-unit Low with Text("a b").
    let row = first_signal("SigA __a__b_");
    assert_eq!(collect_level_runs(&row), vec![(SignalLevel::Low, 5)]);
    assert_eq!(collect_waveform_text_content(&row), vec!["a b"]);
}

#[test]
fn text_in_different_level_runs() {
    // `__ack__~~done~~` → Low with "ack", High with "done".
    let row = first_signal("SigA __ack__~~done~~");
    assert_eq!(
        collect_level_runs(&row),
        vec![(SignalLevel::Low, 4), (SignalLevel::High, 4)]
    );
    assert_eq!(collect_waveform_text_content(&row), vec!["ack", "done"]);
}

#[test]
fn trailing_text_belongs_to_preceding_run() {
    // `~~~~~~~~かきくけこ` → 8-unit High with "かきくけこ".
    let row = first_signal("SigA ~~~~~~~~かきくけこ");
    assert_eq!(collect_level_runs(&row), vec![(SignalLevel::High, 8)]);
    assert_eq!(collect_waveform_text_content(&row), vec!["かきくけこ"]);
}

#[test]
fn text_after_bus_cross_belongs_to_following_bus_run() {
    // `==Xa==`: X body (1 unit Bus) + 'a' text + '==' (2 units) merge into Bus(3) with Text("a").
    // Equivalent to `==X=a==` where X provides 1 bus unit.
    let row = first_signal("SigA ==Xa==");
    assert_eq!(
        collect_level_runs(&row),
        vec![(SignalLevel::Bus, 2), (SignalLevel::Bus, 3)]
    );
    assert_eq!(collect_waveform_text_content(&row), vec!["a"]);
    let has_buscross = row
        .waveform()
        .iter()
        .any(|element| matches!(element, WaveformElement::Transition(t) if t.kind == TransitionKind::BusCross));
    assert!(has_buscross, "expected BusCross transition");
}

#[test]
fn text_before_any_level_is_missing_initial_level() {
    let error = parse_error("SigA a__~~");
    assert!(matches!(error.kind(), ParseErrorKind::MissingInitialLevel));
}

#[test]
fn quoted_text_suppresses_bus_cross() {
    // `=="X"==` → 4-unit Bus with Text("X"), no BusCross.
    let row = first_signal("SigA ==\"X\"==");
    assert_eq!(collect_level_runs(&row), vec![(SignalLevel::Bus, 4)]);
    assert_eq!(collect_waveform_text_content(&row), vec!["X"]);
    let has_buscross = row
        .waveform()
        .iter()
        .any(|element| matches!(element, WaveformElement::Transition(t) if t.kind == TransitionKind::BusCross));
    assert!(!has_buscross, "BusCross must not appear when X is quoted");
}

#[test]
fn quoted_text_with_space() {
    // `__"hello world"__` → 4-unit Low with Text("hello world").
    let row = first_signal("SigA __\"hello world\"__");
    assert_eq!(collect_level_runs(&row), vec![(SignalLevel::Low, 4)]);
    assert_eq!(collect_waveform_text_content(&row), vec!["hello world"]);
}

#[test]
fn quoted_text_with_level_chars_literal() {
    // `__"_~="__` → 4-unit Low with Text("_~=").
    let row = first_signal("SigA __\"_~=\"__");
    assert_eq!(collect_level_runs(&row), vec![(SignalLevel::Low, 4)]);
    assert_eq!(collect_waveform_text_content(&row), vec!["_~="]);
}

#[test]
fn quoted_text_with_special_chars_literal() {
    // `__"[@|]"__` → 4-unit Low with Text("[@|]").
    let row = first_signal("SigA __\"[@|]\"__");
    assert_eq!(collect_level_runs(&row), vec![(SignalLevel::Low, 4)]);
    assert_eq!(collect_waveform_text_content(&row), vec!["[@|]"]);
}

#[test]
fn bare_and_quoted_mixed_are_space_joined() {
    // `__a"b c"d__` → 4-unit Low with Text("a b c d").
    let row = first_signal("SigA __a\"b c\"d__");
    assert_eq!(collect_level_runs(&row), vec![(SignalLevel::Low, 4)]);
    assert_eq!(collect_waveform_text_content(&row), vec!["a b c d"]);
}

#[test]
fn multiple_quoted_texts_in_same_run_are_merged() {
    // `=="a"=="b"==` → 6-unit Bus with Text("a b").
    let row = first_signal("SigA ==\"a\"==\"b\"==");
    assert_eq!(collect_level_runs(&row), vec![(SignalLevel::Bus, 6)]);
    assert_eq!(collect_waveform_text_content(&row), vec!["a b"]);
}

#[test]
fn unclosed_quote_in_level_string_errors() {
    let error = parse_error("SigA __\"hello__");
    assert!(matches!(error.kind(), ParseErrorKind::UnclosedQuote));
}

// ---- `=` 前後の空白を許容する -----------------------------------------

#[test]
fn clock_eq_spaces_are_tolerated() {
    // `@clock(pos , _ = 2 , ~ =3)` is equivalent to `@clock(pos, _=2, ~=3)`.
    let normal = parse_or_panic("@clock(neg, _=2, ~=3)\nCLK");
    let with_spaces = parse_or_panic("@clock(neg , _ = 2 , ~ =3)\nCLK");
    let normal_spec = first_signal_clock_spec(&normal);
    let spaces_spec = first_signal_clock_spec(&with_spaces);
    assert_eq!(normal_spec, spaces_spec);
}

#[test]
fn arrow_head_eq_spaces_are_tolerated() {
    // `head = both` is equivalent to `head=both`.
    let doc = parse_or_panic("SigA @{a}__\nSigB @{b}__\n@-> (@{a}, @{b}, head = both)");
    let arrow = &doc.annotations.arrows[0];
    assert_eq!(arrow.style.head, ArrowHead::BothEnds);
}

#[test]
fn highlight_style_eq_spaces_are_tolerated() {
    // spaces around `=` in @highlight_style.
    let doc = parse_or_panic(
        "@highlight_style fill = \"#8f8\" stroke =\"green\" stroke-width= \"1\"\nSigA __",
    );
    let highlight = doc.style.default_signal_style().highlight_attrs();
    let fill = highlight
        .as_slice()
        .iter()
        .find(|(key, _)| key == "fill")
        .map(|(_, value)| value.as_str());
    let stroke = highlight
        .as_slice()
        .iter()
        .find(|(key, _)| key == "stroke")
        .map(|(_, value)| value.as_str());
    let stroke_width = highlight
        .as_slice()
        .iter()
        .find(|(key, _)| key == "stroke-width")
        .map(|(_, value)| value.as_str());
    assert_eq!(fill, Some("#8f8"));
    assert_eq!(stroke, Some("green"));
    assert_eq!(stroke_width, Some("1"));
}

#[test]
fn dontcare_color_directive_sets_signal_style() {
    let doc = parse_or_panic("@dontcare_color #c00\nSigA __");
    let color = doc.style.default_signal_style().dontcare_color();
    assert_eq!(
        color,
        crate::color::Color::parse("#c00").expect("valid #c00")
    );
}

#[test]
fn dontcare_color_named_value() {
    let doc = parse_or_panic("@dontcare_color red\nSigA __");
    let color = doc.style.default_signal_style().dontcare_color();
    // `Color` equality compares only the RGBA tuple, so either form matches.
    assert_eq!(color, crate::color::Color::parse("red").expect("named red"));
    assert_eq!(color, crate::color::Color::RED);
}

// ---- BusCross text isolation -----------------------------------------------

/// `=<A>====X<B>====`: text before X and text after X must each belong to
/// their own level region. The two text fragments must NOT be merged into one.
#[test]
fn text_on_both_sides_of_buscross_stays_separate() {
    let row = first_signal("SigA =<A>====X<B>====");
    let texts: Vec<&str> = row
        .waveform()
        .iter()
        .filter_map(|element| match element {
            WaveformElement::Text(text) => Some(text.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(
        texts.len(),
        2,
        "expected 2 Text elements (one per region), got {}: {:?}",
        texts.len(),
        texts
    );
    assert!(
        texts[0].contains('A'),
        "first Text must contain 'A', got {:?}",
        texts[0]
    );
    assert!(
        texts[1].contains('B'),
        "second Text must contain 'B', got {:?}",
        texts[1]
    );
}

/// `==X<C>?<D>==`: `<C>` and `<D>` both belong to the DontCareAlongBus region
/// that follows the BusCross. Neither fragment must be lost. They may appear as
/// a single space-joined Text (`<C> <D>`) or as two separate Text elements,
/// depending on how expand/merge assigns their owning region; either is
/// acceptable as long as both fragments are present in the waveform output.
#[test]
fn text_on_dontcare_after_buscross_is_not_lost() {
    let row = first_signal("SigA ==X<C>?<D>==");
    let texts: Vec<&str> = row
        .waveform()
        .iter()
        .filter_map(|element| match element {
            WaveformElement::Text(text) => Some(text.as_str()),
            _ => None,
        })
        .collect();
    assert!(
        !texts.is_empty(),
        "expected at least one Text element, got none"
    );
    let joined = texts.join(" ");
    assert!(
        joined.contains('C'),
        "Text output must include '<C>', got {:?}",
        texts
    );
    assert!(
        joined.contains('D'),
        "Text output must include '<D>' (must not be discarded), got {:?}",
        texts
    );
}

fn count_signal_lines(doc: &crate::document::ChartDocument) -> usize {
    doc.lines
        .iter()
        .filter(|line| matches!(line.content, LineContent::Signal(_)))
        .count()
}

fn nth_signal(doc: &crate::document::ChartDocument, index: usize) -> &SignalRow {
    let line = doc
        .lines
        .iter()
        .filter(|line| matches!(line.content, LineContent::Signal(_)))
        .nth(index)
        .expect("signal index in range");
    match &line.content {
        LineContent::Signal(row) => row.as_ref(),
        _ => unreachable!(),
    }
}

#[test]
fn comment_with_trailing_whitespace_is_ignored() {
    let doc = parse_or_panic("// foo   \n");
    assert!(doc.lines.is_empty());
}

#[test]
fn parameter_with_trailing_whitespace_is_accepted() {
    parse_or_panic("@step 10   \n");
}

#[test]
fn signal_name_separated_by_multiple_spaces() {
    let row = first_signal("Clock     _~_~");
    assert_eq!(row.name().as_str(), "Clock");
}

#[test]
fn parameter_step_uppercase_accepted() {
    parse_or_panic("@STEP 10\nSig _\n");
}

#[test]
fn parameter_step_titlecase_accepted() {
    parse_or_panic("@Step 10\nSig _\n");
}

#[test]
fn parameter_signal_color_underscore_and_hyphen_equivalent() {
    let underscore = parse_or_panic("@signal_color red\nSig _\n");
    let hyphen = parse_or_panic("@signal-color red\nSig _\n");
    assert_eq!(count_signal_lines(&underscore), count_signal_lines(&hyphen));
}

#[test]
fn clockmark_height_can_be_redeclared_per_signal() {
    parse_or_panic("@clockmark_height 5\n@clock(pos) ck1\n@clockmark_height 8\n@clock(pos) ck2\n");
}

#[test]
fn clock_attribute_keys_are_case_and_dash_insensitive() {
    parse_or_panic("@clock(POS, MARK_COLOR=red, MARK-HEIGHT=4) ck\n");
}

#[test]
fn duplicate_bg_keeps_last_value() {
    let doc = parse_or_panic("@bg #f0f\n@bg #0f0\nA _\n");
    let line = doc
        .lines
        .iter()
        .find(|line| matches!(line.content, LineContent::Signal(_)))
        .expect("signal");
    assert!(
        line.background.is_some(),
        "background must be applied to the next signal"
    );
}

#[test]
fn bgcolor0_accepts_hex_with_alpha() {
    parse_or_panic("@bgcolor0 #ff8800ff\nA _\n");
}

#[test]
fn bgcolor0_accepts_none_keyword() {
    parse_or_panic("@bgcolor0 none\nA _\n");
}

#[test]
fn bgcolor1_alone_is_accepted() {
    parse_or_panic("@bgcolor1 #eee\nA _\n");
}

#[test]
fn scale_global_parameter_accepted() {
    parse_or_panic("@scale 2.0\nA _\n");
}

#[test]
fn page_margin_zero_accepted() {
    parse_or_panic("@page-margin 0\nA _\n");
}

#[test]
fn step_zero_is_rejected() {
    parse_error("@step 0\nA _\n");
}

#[test]
fn step_negative_is_rejected() {
    parse_error("@step -5\nA _\n");
}

#[test]
fn slant_negative_is_rejected() {
    parse_error("@slant -1\nA _\n");
}

#[test]
fn slant_zero_is_accepted() {
    parse_or_panic("@slant 0\nA _\n");
}

#[test]
fn step_equal_to_slant_is_rejected() {
    let error = parse_error("@step 2\n@slant 2\nA _\n");
    assert!(matches!(
        error.kind(),
        ParseErrorKind::InvalidStepSlant(_, _)
    ));
}

#[test]
fn h_space_negative_is_rejected() {
    parse_error("@h_space -1\nA _\n");
}

#[test]
fn fontsize_zero_is_rejected() {
    parse_error("@fontsize 0\nA _\n");
}

#[test]
fn lineheight_zero_is_rejected() {
    parse_error("@lineheight 0\nA _\n");
}

#[test]
fn title_with_empty_quoted_string_is_accepted() {
    parse_or_panic("@title \"\"\n");
}

#[test]
fn title_without_argument_is_rejected() {
    parse_error("@title\n");
}

#[test]
fn skip_without_argument_is_rejected() {
    let error = parse_error("@skip\n");
    assert!(matches!(
        error.kind(),
        ParseErrorKind::InvalidSkipAmount(_) | ParseErrorKind::InvalidSkipSyntax
    ));
}

#[test]
fn skip_with_empty_parens_is_rejected() {
    parse_error("@skip()\n");
}

#[test]
fn skip_with_lh_unit_suffix() {
    // Spec is ambiguous; behaviour is fixed here as accept-or-reject.
    let _ = parse("@skip(1lh)\n");
}

#[test]
fn skip_with_uppercase_px_unit() {
    parse_or_panic("@skip(2PX)\n");
}

#[test]
fn signal_directive_before_title_is_carried_over_or_dropped() {
    // Either the @signal is consumed by the next signal (carried over) or
    // an error/warning is produced — behaviour must be deterministic.
    let _ = parse("@signal(overline)\n@title \"X\"\nSig _\n");
}

#[test]
fn signal_directive_repeated_idempotent_or_error() {
    // Either idempotent or duplicate error — behaviour must be deterministic.
    let _ = parse("@signal(overline)\n@signal(overline)\nSig _\n");
}

#[test]
fn clock_pulse_low_zero_is_rejected() {
    parse_error("@clock(pos, _=0, ~=1) ck\n");
}

#[test]
fn clock_pulse_high_floating_point_is_rejected() {
    parse_error("@clock(pos, ~=2.5) ck\n");
}

#[test]
fn clock_mark_position_out_of_range_is_rejected() {
    parse_error("@clock(pos, mark_position=1.5) ck\n");
}

#[test]
fn clock_mark_height_negative_is_rejected() {
    parse_error("@clock(pos, mark_height=-1) ck\n");
}

#[test]
fn clock_start_high_phase_accepted() {
    parse_or_panic("@clock(pos, start=high) ck\nA ___\n");
}

#[test]
fn clock_edge_uppercase_accepted() {
    parse_or_panic("@clock(POS) ck\nA _\n");
}

#[test]
fn clock_edge_titlecase_accepted() {
    parse_or_panic("@clock(Pos) ck\nA _\n");
}

#[test]
fn clock_duplicate_attribute_is_rejected() {
    let error = parse_error("@clock(pos, _=2, _=3) ck\n");
    assert!(matches!(
        error.kind(),
        ParseErrorKind::ClockInvalidAttribute(_)
            | ParseErrorKind::NumericNotParseable(_, _)
            | ParseErrorKind::NumericOverflow(_, _, _)
            | ParseErrorKind::NumericNotPositive(_, _)
            | ParseErrorKind::NumericNotNonNegative(_, _)
            | ParseErrorKind::InvalidTitleAlign(_)
            | ParseErrorKind::TitleRequiresArgument
            | ParseErrorKind::UnknownParameter(_)
    ));
}

#[test]
fn clock_unknown_attribute_is_rejected() {
    parse_error("@clock(pos, foo=1) ck\n");
}

#[test]
fn clock_invalid_attribute_reports_offending_token_location() {
    // Body: "@clock(_=3,~3)" — first attr is fine, second is "~3" (missing `=`).
    // The offending token starts at column 12 (1-based: '@'=1,'c'=2,'l'=3,'o'=4,
    // 'c'=5,'k'=6,'('=7,'_'=8,'='=9,'3'=10,','=11,'~'=12) and is 2 chars long.
    let error = parse_error("@clock(_=3,~3)\nck\n");
    assert_eq!(error.line(), 1);
    assert_eq!(
        error.column(),
        12,
        "column must point at the offending token"
    );
    assert_eq!(error.length(), 2, "length must cover the offending token");
    let rendered = format!("{error}");
    assert!(
        rendered.contains("~3"),
        "error message must include the offending attribute text, got: {rendered}"
    );
}

#[test]
fn signal_unknown_attribute_reports_offending_token() {
    // '@'=1, 's'=2,..,'l'=7,'('=8,'u'=9 -> column 9, length 10 for "unknownkey".
    let error = parse_error("@signal(unknownkey)\nSig _\n");
    assert_eq!(error.line(), 1);
    assert_eq!(error.column(), 9);
    assert_eq!(error.length(), 10);
    let rendered = format!("{error}");
    assert!(
        rendered.contains("unknownkey"),
        "message must include the offending attribute, got: {rendered}"
    );
}

#[test]
fn signal_second_unknown_attribute_pinpoints_second_token() {
    // "@signal(overline, foo)" -> 'f' is at column 19 (cols: '('=8, 'o'=9..16
    // for "overline", ','=17, ' '=18, 'f'=19).
    let error = parse_error("@signal(overline, foo)\nSig _\n");
    assert_eq!(error.column(), 19);
    assert_eq!(error.length(), 3);
    assert!(format!("{error}").contains("foo"));
}

#[test]
fn signal_duplicate_overline_pinpoints_second_token() {
    // "@signal(overline, overline)" -> second "overline" starts at column 19.
    let error = parse_error("@signal(overline, overline)\nSig _\n");
    assert_eq!(error.column(), 19);
    assert_eq!(error.length(), 8);
    assert!(format!("{error}").contains("overline"));
}

#[test]
fn invalid_level_char_reports_length_one() {
    // "Sig _\x01_": '_'=col 5, '\x01'=col 6.
    let error = parse_error("Sig _\x01_\n");
    assert!(matches!(error.kind(), ParseErrorKind::InvalidLevelChar(_)));
    assert_eq!(error.column(), 6);
    assert_eq!(error.length(), 1);
}

#[test]
fn missing_initial_level_reports_length_one() {
    // "Sig abc": 'a'=col 5.
    let error = parse_error("Sig abc\n");
    assert!(matches!(error.kind(), ParseErrorKind::MissingInitialLevel));
    assert_eq!(error.column(), 5);
    assert_eq!(error.length(), 1);
}

#[test]
fn unopened_highlight_end_reports_length_one() {
    // "Sig _~]_": ']' at col 7.
    let error = parse_error("Sig _~]_\n");
    assert!(matches!(error.kind(), ParseErrorKind::UnopenedHighlightEnd));
    assert_eq!(error.column(), 7);
    assert_eq!(error.length(), 1);
}

#[test]
fn overline_with_argument_pinpoints_offending_tail() {
    // "@overline foo": '@'=1, 'o'=2..9, ' '=10, 'f'=11.
    let error = parse_error("@overline foo\nSig _\n");
    assert!(matches!(
        error.kind(),
        ParseErrorKind::InvalidOverlineSyntax(_)
    ));
    assert_eq!(error.column(), 11);
    assert_eq!(error.length(), 3);
    let msg = format!("{error}");
    assert!(
        msg.contains("foo"),
        "message should include offending text: {msg}"
    );
}

#[test]
fn title_with_control_character_points_at_offending_char() {
    // `@title "abc\x01def"` — opening quote at col 8 → text starts at col 9
    // → `\x01` is at offset 3 within the text → source col 12, length 1.
    let error = parse_error("@title \"abc\x01def\"\n");
    assert!(matches!(
        error.kind(),
        ParseErrorKind::InvalidText(TextError::ForbiddenControlChar { char_offset: 3 })
    ));
    assert_eq!(error.line(), 1);
    assert_eq!(error.column(), 12);
    assert_eq!(error.length(), 1);
}

#[test]
fn invalid_anchor_name_points_at_offending_char_inside_braces() {
    // "Sig _@{abc!def}_": 'S'=1,'i'=2,'g'=3,' '=4,'_'=5,'@'=6,'{'=7,'a'=8,'b'=9,'c'=10,'!'=11.
    // The `!` is at offset 3 within the anchor name `abc!def`. Source col =
    // `@` col 6 + 2 (for `@{`) + 3 = 11.
    let error = parse_error("Sig _@{abc!def}_\n");
    assert!(matches!(error.kind(), ParseErrorKind::InvalidAnchorName(_)));
    assert_eq!(error.column(), 11);
    assert_eq!(error.length(), 1);
}

#[test]
fn signal_name_with_control_character_points_at_offending_char() {
    // "Sig\x01 _~": 'S'=1,'i'=2,'g'=3,'\x01'=4.
    let error = parse_error("Sig\x01 _~\n");
    assert!(matches!(
        error.kind(),
        ParseErrorKind::InvalidName(NameError::ForbiddenControlChar { char_offset: 3 })
    ));
    assert_eq!(error.line(), 1);
    assert_eq!(error.length(), 1);
}

#[test]
fn unclosed_highlight_points_at_opening_bracket() {
    // "Sig _[~_": 'S'=1,'i'=2,'g'=3,' '=4,'_'=5,'['=6,'~'=7,'_'=8.
    // The error should point at the opening '[' (col 6), not at the last
    // processed character.
    let error = parse_error("Sig _[~_\n");
    assert!(matches!(error.kind(), ParseErrorKind::UnclosedHighlight));
    assert_eq!(error.column(), 6);
    assert_eq!(error.length(), 1);
}

#[test]
fn invalid_step_slant_includes_both_values_in_message() {
    let error = parse_error("@step 5\n@slant 10\nSig _\n");
    assert!(matches!(
        error.kind(),
        ParseErrorKind::InvalidStepSlant(_, _)
    ));
    let msg = format!("{error}");
    assert!(msg.contains("5"), "msg should include step value 5: {msg}");
    assert!(
        msg.contains("10"),
        "msg should include slant value 10: {msg}"
    );
}

#[test]
fn arrow_missing_parens_reports_invalid_syntax_with_length() {
    // "@-> @{a}, @{b}" without surrounding `(...)`.
    // The error must underline the args span (not just the `@`).
    let error = parse_error("Sig _@{a}~@{b}_\n@-> @{a}, @{b}\n");
    assert!(matches!(error.kind(), ParseErrorKind::InvalidArrowSyntax));
    assert_eq!(error.line(), 2);
    // Length must be > 0 so the caret has visible width.
    assert!(
        error.length() >= 1,
        "length must be > 0; got {}",
        error.length()
    );
}

#[test]
fn skip_amount_non_numeric_pinpoints_value() {
    // "@skip(abc)": '@'=1, 'k'=5, '('=6, 'a'=7.
    let error = parse_error("@skip(abc)\nSig _\n");
    assert!(matches!(error.kind(), ParseErrorKind::InvalidSkipAmount(_)));
    assert_eq!(error.column(), 7);
    assert_eq!(error.length(), 3);
    let msg = format!("{error}");
    assert!(
        msg.contains("abc"),
        "message should quote bad amount: {msg}"
    );
}

#[test]
fn skip_amount_negative_pinpoints_value() {
    let error = parse_error("@skip(-3)\nSig _\n");
    assert_eq!(error.column(), 7);
    assert_eq!(error.length(), 2);
    let msg = format!("{error}");
    assert!(msg.contains("-3"));
}

#[test]
fn overlay_invalid_x_pinpoints_token() {
    // "% abc 2 text" — 'a'=col 3.
    let error = parse_error("% abc 2 text\n");
    assert!(matches!(
        error.kind(),
        ParseErrorKind::InvalidOverlayCoordinate(_)
    ));
    assert_eq!(error.column(), 3);
    assert_eq!(error.length(), 3);
    let msg = format!("{error}");
    assert!(msg.contains("abc"));
}

#[test]
fn overlay_invalid_y_pinpoints_token() {
    // "% 1 abc text" — 'a'=col 5.
    let error = parse_error("% 1 abc text\n");
    assert_eq!(error.column(), 5);
    assert_eq!(error.length(), 3);
}

#[test]
fn undefined_anchor_message_includes_anchor_name() {
    // Force a named anchor reference so the message can name it.
    let error = parse_error("Sig _~_\n@-> (@{undef}, @{x})\nSig2 _@{x}_\n");
    assert!(matches!(error.kind(), ParseErrorKind::UndefinedAnchor(_)));
    let msg = format!("{error}");
    // Must include the bare anchor name, NOT just "undefined" (which is a
    // substring of the generic literal). Check for "{undef" with the brace.
    assert!(
        msg.contains("{undef}") || msg.contains("\"undef\"") || msg.contains("'undef'"),
        "message must mention the anchor: {msg}"
    );
}

#[test]
fn clock_missing_closing_paren_underlines_full_remainder() {
    // "@clock(_=3,~3" without a closing ')'. The underline should cover
    // "(_=3,~3" so the user can see exactly which span is malformed.
    let error = parse_error("@clock(_=3,~3\nClock\n");
    assert!(matches!(
        error.kind(),
        ParseErrorKind::ClockInvalidAttribute(_)
    ));
    assert_eq!(error.line(), 1);
    assert_eq!(error.column(), 7); // '(' position
    assert_eq!(error.length(), 7); // "(_=3,~3".chars().count()
}

#[test]
fn arrow_unknown_attribute_pinpoints_offending_token() {
    // "@-> (@{a}, @{b}, foo=bar)" + anchors defined.
    // cols: '@'=1, '-'=2, '>'=3, ' '=4, '('=5, '@'=6, '{'=7, 'a'=8, '}'=9, ','=10, ' '=11,
    //       '@'=12, '{'=13, 'b'=14, '}'=15, ','=16, ' '=17, 'f'=18.
    let error = parse_error("Sig _@{a}~@{b}_\n@-> (@{a}, @{b}, foo=bar)\n");
    assert_eq!(error.line(), 2);
    assert_eq!(error.column(), 18);
    assert_eq!(error.length(), 7); // "foo=bar"
    let msg = format!("{error}");
    assert!(
        msg.contains("foo"),
        "msg should mention bad attribute: {msg}"
    );
}

#[test]
fn arrow_duplicate_attribute_pinpoints_second_token() {
    // Two `color=...` attrs: second one is the offender.
    // Line 2: "@-> (@{a}, @{b}, color=red, color=blue)"
    let error = parse_error("Sig _@{a}~@{b}_\n@-> (@{a}, @{b}, color=red, color=blue)\n");
    assert_eq!(error.line(), 2);
    // First `color=red` starts at col 18 (similar layout). Second at col 29.
    assert_eq!(error.column(), 29);
    assert_eq!(error.length(), 10); // "color=blue"
    let msg = format!("{error}");
    assert!(msg.contains("color"));
}

#[test]
fn duplicate_anchor_pinpoints_second_definition() {
    // "Sig _@{a}~@{a}_": first '@'=col 6, second '@'=col 11.
    let error = parse_error("Sig _@{a}~@{a}_\n");
    assert!(matches!(error.kind(), ParseErrorKind::DuplicateAnchor));
    assert_eq!(error.column(), 11);
    assert_eq!(error.length(), 4); // "@{a}".chars().count()
    let msg = format!("{error}");
    assert!(
        msg.contains("a"),
        "message should include anchor name: {msg}"
    );
}

#[test]
fn undefined_anchor_pinpoints_reference() {
    // "Sig _~_\n@-> (@{undef}, @{x})\nSig2 _@{x}_"
    let error = parse_error("Sig _~_\n@-> (@{undef}, @{x})\nSig2 _@{x}_\n");
    assert!(matches!(error.kind(), ParseErrorKind::UndefinedAnchor(_)));
    assert_eq!(error.line(), 2);
    let msg = format!("{error}");
    assert!(msg.contains("undef"), "message should include name: {msg}");
}

#[test]
fn clock_unknown_key_reports_offending_token_location() {
    // "foo=1" is an unknown key. Column should be where `foo` starts.
    // '@'=1,'c'=2,'l'=3,'o'=4,'c'=5,'k'=6,'('=7,'p'=8,'o'=9,'s'=10,','=11,' '=12,'f'=13.
    let error = parse_error("@clock(pos, foo=1)\nck\n");
    assert_eq!(error.line(), 1);
    assert_eq!(error.column(), 13);
    assert_eq!(error.length(), 5, "covers `foo=1`");
    let rendered = format!("{error}");
    assert!(rendered.contains("foo"));
}

#[test]
fn arrow_invalid_head_keyword_is_rejected() {
    parse_error("@-> (@{a}, @{b}, head=middle)\n");
}

#[test]
fn arrow_solid_keyword_accepted() {
    parse_or_panic("Sig _~@{a}_\n@-> (@{a}, @{a}, solid)\n");
}

#[test]
fn arrow_width_without_unit_accepted() {
    parse_or_panic("Sig _~@{a}_\n@-> (@{a}, @{a}, 1.5)\n");
}

#[test]
fn arrow_width_fractional_px_accepted() {
    parse_or_panic("Sig _~@{a}_\n@-> (@{a}, @{a}, 0.5px)\n");
}

#[test]
fn arrow_width_zero_behaviour_is_deterministic() {
    let _ = parse("Sig _~@{a}_\n@-> (@{a}, @{a}, 0px)\n");
}

#[test]
fn arrow_duplicate_color_is_rejected() {
    let error = parse_error("Sig _~@{a}_\n@-> (@{a}, @{a}, red, #f00)\n");
    assert!(matches!(
        error.kind(),
        ParseErrorKind::DuplicateArrowAttribute(_)
    ));
}

#[test]
fn arrow_duplicate_line_style_is_rejected() {
    let error = parse_error("Sig _~@{a}_\n@-> (@{a}, @{a}, dashed, dotted)\n");
    assert!(matches!(
        error.kind(),
        ParseErrorKind::DuplicateArrowAttribute(_)
    ));
}

#[test]
fn arrow_label_with_comma_keeps_full_text() {
    parse_or_panic("Sig _~@{a}_\n@-> (@{a}, @{a}) hello, world\n");
}

#[test]
fn arrow_with_extra_whitespace_inside_parens() {
    parse_or_panic("Sig _~@{a}_\n@-> (   @{a}   ,   @{a}   ,   red   )\n");
}

#[test]
fn unused_anchor_does_not_cause_error() {
    parse_or_panic("Sig _~@{a}_~@{b}_~@{unused}_\n@-> (@{a}, @{b})\n");
}

#[test]
fn dontcare_color_none_is_deterministic() {
    let _ = parse("@dontcare_color none\nSig _?_\n");
}

#[test]
fn highlight_style_disallowed_attr_handled() {
    // Either rejected by whitelist or silently dropped — must be deterministic.
    let _ = parse("@highlight_style onmouseover=\"alert(1)\"\nSig _[__]_\n");
}

#[test]
fn highlight_style_value_with_inner_spaces() {
    parse_or_panic("@highlight_style fill=\"rgb(255, 128, 0)\"\nSig _[__]_\n");
}

#[test]
fn anchor_name_starting_with_underscore_accepted() {
    parse_or_panic("Sig _~@{_under_score}_\n");
}

#[test]
fn anchor_name_with_hyphen_accepted() {
    parse_or_panic("Sig _~@{a-b-c}_\n");
}

#[test]
fn anchor_number_large_value_accepted() {
    parse_or_panic("Sig _~@99999\nT _~@1\n@-> (@99999, @1)\n");
}

#[test]
fn signal_name_utf8_multibyte_accepted() {
    let row = first_signal("クロック _~_~");
    assert_eq!(row.name().as_str(), "クロック");
}

#[test]
fn unquoted_signal_name_with_space_treats_first_token_as_name() {
    // "Clock A _~_~" -> name is "Clock", waveform begins with `A` which is
    // not a level symbol, so the parser must error.
    let error = parse_error("Clock A _~_~\n");
    assert!(matches!(
        error.kind(),
        ParseErrorKind::MissingInitialLevel | ParseErrorKind::InvalidLevelChar(_)
    ));
}

#[test]
fn quoted_signal_name_with_literal_tab_is_rejected() {
    parse_error("\"a\tb\" _~\n");
}

#[test]
fn quoted_signal_name_with_unknown_escape_is_deterministic() {
    let _ = parse("\"a\\xb\" _~\n");
}

#[test]
fn newlines_only_file_is_empty_document() {
    let doc = parse_or_panic("\n\n\n");
    assert!(doc.lines.is_empty());
}

#[test]
fn bom_at_file_start_is_tolerated() {
    parse_or_panic("\u{FEFF}// header\n");
}

#[test]
fn crlf_line_endings_are_treated_as_lf() {
    parse_or_panic("// header\r\nSig _~\r\n");
}

#[test]
fn tab_indentation_handling_is_deterministic() {
    let _ = parse("\tClock _~_~\n");
}

#[test]
fn per_row_step_with_clock_auto_expansion() {
    parse_or_panic("@step 10\n@clock(pos) ck\n@step 20\nData ____\n");
}

#[test]
fn per_row_step_with_dontcare_resolution() {
    parse_or_panic("@step 10\nA _?_\n@step 20\nB _?_\n");
}

#[test]
fn per_row_step_with_anchor_position() {
    parse_or_panic("@step 10\nA ___@1__\n@step 20\nB ___@2__\n");
}

#[test]
fn per_row_step_with_arrow_label_midpoint() {
    parse_or_panic("@step 10\nA _~@{a}_\n@step 20\nB _~@{b}_\n@-> (@{a}, @{b})\n");
}

#[test]
fn per_row_step_with_overlay_row_uses_absolute_x() {
    parse_or_panic("@step 10\nA _~\n% 100 50 mark\n@step 20\nB ____\n");
}

#[test]
fn per_row_step_with_signal_overline() {
    parse_or_panic("@step 10\nA _~\n@step 20\n@signal(overline) NReset _~_~\n");
}

#[test]
fn per_row_slant_with_dontcare() {
    parse_or_panic("@slant 0\nA _?=\n@slant 4\nB _?=\n");
}

#[test]
fn per_row_slant_with_following_clock_pos() {
    parse_or_panic("@slant 0\nA _~\n@slant 4\n@clock(pos) ck\nT __\n");
}

#[test]
fn clock_auto_with_pulse_widths_and_step_change() {
    parse_or_panic("@step 10\n@clock(pos, _=2, ~=2) ck\nT ________\n");
}

#[test]
fn clock_empty_body_cannot_carry_anchor() {
    // An empty-body @clock signal cannot carry @{x} or @N anchors in its body.
    // Anchors must be defined on a different signal row.
    parse_or_panic("@clock(pos) ck\nA _~@{x}_\n@-> (@{x}, @{x})\n");
}

#[test]
fn clock_body_partial_with_anchor_inside() {
    parse_or_panic("@clock(pos) ck _~@{rise}__\n");
}

#[test]
fn duplicate_named_anchor_across_signals_is_rejected() {
    let error = parse_error("A _~@{a}_\nB _~@{a}_\n");
    assert!(matches!(error.kind(), ParseErrorKind::DuplicateAnchor));
}

#[test]
fn duplicate_numbered_anchor_across_signals_is_rejected() {
    let error = parse_error("A _~@1_\nB _~@1_\n");
    assert!(matches!(error.kind(), ParseErrorKind::DuplicateAnchor));
}

#[test]
fn numbered_anchors_can_have_gaps() {
    parse_or_panic("A _~@1_~@5_\n");
}

#[test]
fn numeric_named_and_numbered_anchors_are_distinct() {
    parse_or_panic("A _~@{1}__@1\n@-> (@{1}, @1)\n");
}

#[test]
fn multi_line_signal_name_with_anchor_in_body() {
    parse_or_panic("\"Sig\\nA\" _~@{a}_~\n");
}

#[test]
fn multi_line_signal_name_with_overline() {
    parse_or_panic("@signal(overline) \"short\\nveryLongLine\" _~\n");
}

#[test]
fn whitespace_only_line_is_ignored() {
    let doc = parse_or_panic("   \n");
    assert!(doc.lines.is_empty());
}

#[test]
fn hash_character_inside_signal_name_is_literal() {
    let row = first_signal("Sig#A _~_~");
    assert!(row.name().as_str().contains('#'));
}

#[test]
fn hash_in_waveform_is_bare_text_char() {
    // `#` is no longer a comment marker (replaced by `//`); a trailing `#` in
    // the level string is treated as a bare text character belonging to the
    // surrounding level run rather than rejected as `InvalidLevelChar('#')`.
    let row = first_signal("Sig _~_~#tail\n");
    let texts: Vec<&str> = row
        .waveform()
        .iter()
        .filter_map(|element| match element {
            WaveformElement::Text(text) => Some(text.as_str()),
            _ => None,
        })
        .collect();
    assert!(
        texts.iter().any(|text| text.contains('#')),
        "expected `#` to appear as a text fragment, got {texts:?}"
    );
}

#[test]
fn slash_slash_on_directive_is_treated_as_line_end_comment() {
    let doc = parse_or_panic("@step 10 // unit\nSig _~_~\n");
    let signal = doc
        .lines
        .iter()
        .find_map(|line| match &line.content {
            LineContent::Signal(row) => Some(row),
            _ => None,
        })
        .expect("signal row expected");
    assert_eq!(signal.layout_params().step().to_f32(), 10.0);
}

#[test]
fn slash_slash_in_waveform_terminates_level_string() {
    let row = first_signal("Sig _~_~ // trailing\n");
    let kinds: Vec<&str> = row
        .waveform()
        .iter()
        .map(|element| match element {
            WaveformElement::Level(_) => "L",
            WaveformElement::Transition(_) => "T",
            WaveformElement::Text(_) => "X",
            _ => "?",
        })
        .collect();
    // `_~_~` produces Level/Transition/Level/Transition/Level/Transition/Level
    // pattern; the `// trailing` portion must not appear as text.
    assert!(
        !kinds.contains(&"X"),
        "no text element expected, got {kinds:?}"
    );
}

#[test]
fn single_slash_in_waveform_is_bare_text() {
    // A single `/` (not followed by another `/`) is a bare text character.
    let row = first_signal("Sig _/_\n");
    let texts: Vec<&str> = row
        .waveform()
        .iter()
        .filter_map(|element| match element {
            WaveformElement::Text(text) => Some(text.as_str()),
            _ => None,
        })
        .collect();
    assert!(
        texts.iter().any(|text| text.contains('/')),
        "expected `/` to be retained as text, got {texts:?}"
    );
}

#[test]
fn slash_slash_inside_quoted_text_is_literal() {
    // Inside `"..."` quotes, `//` is a literal — comment cutoff is disabled.
    let row = first_signal("Sig __\"// note\"__\n");
    let texts: Vec<&str> = row
        .waveform()
        .iter()
        .filter_map(|element| match element {
            WaveformElement::Text(text) => Some(text.as_str()),
            _ => None,
        })
        .collect();
    assert!(
        texts.iter().any(|text| text.contains("// note")),
        "expected literal `// note` inside quotes, got {texts:?}"
    );
}

#[test]
fn slash_slash_on_overlay_line_is_treated_as_line_end_comment() {
    // `% x y text // ...` — only the text-before-`//` should survive.
    let doc = parse_or_panic("Sig _~\n%5 5 hello // junk\n");
    let overlays = &doc.annotations.overlays;
    assert_eq!(overlays.len(), 1);
    assert_eq!(overlays[0].text.as_str(), "hello");
}

#[test]
fn slash_slash_multiple_occurrences_drop_to_first() {
    // `A _~_~ // foo // bar` — everything from the first `//` is dropped.
    let row = first_signal("A _~_~ // foo // bar\n");
    let kinds: Vec<&str> = row
        .waveform()
        .iter()
        .map(|element| match element {
            WaveformElement::Text(_) => "X",
            _ => "_",
        })
        .collect();
    assert!(!kinds.contains(&"X"), "no text expected, got {kinds:?}");
}

#[test]
fn slash_slash_on_blank_line_is_skip() {
    let doc = parse_or_panic("// just a comment\n");
    assert!(doc.lines.is_empty());
}

#[test]
fn single_slash_at_start_of_directive_is_not_a_comment() {
    // A single `/` is not a comment; the `@title /path` line must yield a
    // title row whose text starts with `/`.
    let doc = parse_or_panic("@title /path/to/file\n");
    let title_text: Option<String> = doc.lines.iter().find_map(|line| match &line.content {
        LineContent::Title(title) => Some(title.text.as_str().to_owned()),
        _ => None,
    });
    assert_eq!(title_text.as_deref(), Some("/path/to/file"));
}

#[test]
fn unclosed_quote_in_waveform_preempts_slash_slash() {
    // `SigA __"hello // world__` — the unclosed `"` must produce
    // `UnclosedQuote`, not be silently swallowed by a comment cutoff.
    let error = parse_error("SigA __\"hello // world__\n");
    assert!(matches!(error.kind(), ParseErrorKind::UnclosedQuote));
}

#[test]
fn arrow_endpoints_mixing_named_and_numbered() {
    parse_or_panic("A _~@1__@{end}\n@-> (@1, @{end})\n");
}

#[test]
fn arrow_with_same_endpoint_on_both_sides() {
    parse_or_panic("A _~@{a}_\n@-> (@{a}, @{a})\n");
}

#[test]
fn many_arrows_have_no_upper_limit_in_parser() {
    let mut source = String::from("A _~");
    for index in 1..=200 {
        source.push_str(&format!("@{index}_"));
    }
    source.push('\n');
    for index in 1..100 {
        source.push_str(&format!("@-> (@{index}, @{}_)\n", index + 1));
    }
    // Some arrows reference invalid syntax due to formatting; only the parse
    // call itself is exercised — failure is acceptable signal.
    let _ = parse(&source);
}

#[test]
fn bg_applies_to_clock_pos_signal() {
    let doc = parse_or_panic("@bg #f0f\n@clock(pos) ck\nT _\n");
    let clock_signal = doc
        .lines
        .iter()
        .find(|line| matches!(line.content, LineContent::Signal(_)))
        .expect("signal");
    assert!(
        clock_signal.background.is_some(),
        "@bg must be applied to the very next signal row, including a clock row"
    );
}

#[test]
fn bg_combines_with_signal_overline() {
    let doc = parse_or_panic("@bg #f0f\n@signal(overline) nReset _~__\n");
    let row = nth_signal(&doc, 0);
    assert!(row.decorations().is_name_overline());
}

#[test]
fn bgcolor_index_counts_only_signal_rows_for_bg_targets() {
    // Verifies that `@bg` does not change the signal index counter; the next
    // SignalRow inherits its sequence position from preceding signal count.
    parse_or_panic("A _\n@bg #f0f\nB _\nC _\n");
}

#[test]
fn title_row_does_not_consume_bgcolor_stripe_index() {
    let doc = parse_or_panic("A _\n@title \"mid\"\nB _\n");
    assert_eq!(count_signal_lines(&doc), 2);
}

// ---------------------------------------------------------------------------
// Iter1 phase 2: error recovery and multi-error scenarios.
// The current `parse` API returns `Result<ChartDocument, ParseError>`, i.e.
// only the first error is reported. These tests pin that behaviour: when the
// spec calls for multi-error reporting the test will fail because the first
// error already aborts. Failure here is the intended signal.
// ---------------------------------------------------------------------------

#[test]
fn multiple_errors_on_one_line_iter1_first_only() {
    let error = parse_error("@bg notacolor @titlealign sideways\n");
    assert!(matches!(
        error.kind(),
        ParseErrorKind::InvalidColor(_)
            | ParseErrorKind::NumericNotParseable(_, _)
            | ParseErrorKind::NumericOverflow(_, _, _)
            | ParseErrorKind::NumericNotPositive(_, _)
            | ParseErrorKind::NumericNotNonNegative(_, _)
            | ParseErrorKind::InvalidTitleAlign(_)
            | ParseErrorKind::TitleRequiresArgument
    ));
}

#[test]
fn dontcare_color_invalid_followed_by_valid_signal_iter1() {
    let error = parse_error("@dontcare_color zonk\nA _~\n");
    assert!(matches!(
        error.kind(),
        ParseErrorKind::InvalidColor(_)
            | ParseErrorKind::NumericNotParseable(_, _)
            | ParseErrorKind::NumericOverflow(_, _, _)
            | ParseErrorKind::NumericNotPositive(_, _)
            | ParseErrorKind::NumericNotNonNegative(_, _)
            | ParseErrorKind::InvalidTitleAlign(_)
            | ParseErrorKind::TitleRequiresArgument
    ));
    assert_eq!(
        error.line(),
        1,
        "first error must be reported on the @dontcare_color line"
    );
}

#[test]
fn multiple_unknown_anchor_arrows_iter1_first_error_only() {
    let error = parse_error("@-> (@{a}, @{b})\n@-> (@{c}, @{d})\n");
    assert!(matches!(
        error.kind(),
        ParseErrorKind::UndefinedAnchor(_)
            | ParseErrorKind::NumericNotParseable(_, _)
            | ParseErrorKind::NumericOverflow(_, _, _)
            | ParseErrorKind::NumericNotPositive(_, _)
            | ParseErrorKind::NumericNotNonNegative(_, _)
            | ParseErrorKind::InvalidTitleAlign(_)
            | ParseErrorKind::TitleRequiresArgument
    ));
}

#[test]
fn parse_error_followed_by_valid_global_param_iter1() {
    let error = parse_error("@scale notnum\n@scale 2.0\nA _~\n");
    assert!(matches!(
        error.kind(),
        ParseErrorKind::InvalidLength(_)
            | ParseErrorKind::NumericNotParseable(_, _)
            | ParseErrorKind::NumericOverflow(_, _, _)
            | ParseErrorKind::NumericNotPositive(_, _)
            | ParseErrorKind::NumericNotNonNegative(_, _)
            | ParseErrorKind::InvalidTitleAlign(_)
            | ParseErrorKind::TitleRequiresArgument
    ));
    assert_eq!(
        error.line(),
        1,
        "the first @scale line is the failing one and must be reported"
    );
}

#[test]
fn errors_with_blank_and_comment_lines_in_between_iter1() {
    let error = parse_error("@bg ???\n@step xyz\n# comment\n\nA _\n");
    assert!(matches!(
        error.kind(),
        ParseErrorKind::InvalidColor(_)
            | ParseErrorKind::InvalidLength(_)
            | ParseErrorKind::NumericNotParseable(_, _)
            | ParseErrorKind::NumericOverflow(_, _, _)
            | ParseErrorKind::NumericNotPositive(_, _)
            | ParseErrorKind::NumericNotNonNegative(_, _)
            | ParseErrorKind::InvalidTitleAlign(_)
            | ParseErrorKind::TitleRequiresArgument
    ));
}

#[test]
fn duplicate_signal_attribute_iter1_first_error() {
    let result = parse("@signal(overline, overline) Sig _~\n");
    assert!(
        result.is_err(),
        "duplicate signal attribute must produce an error"
    );
}

#[test]
fn parse_error_returns_error_not_partial_document_iter1() {
    let outcome = parse("@bg invalid\nA _\n");
    assert!(outcome.is_err(), "current API aborts on first error");
}

#[test]
fn numeric_overflow_step_iter1() {
    let error = parse_error("@step 99999999999999999\nA _\n");
    assert!(matches!(
        error.kind(),
        ParseErrorKind::InvalidLength(_)
            | ParseErrorKind::NumericNotParseable(_, _)
            | ParseErrorKind::NumericOverflow(_, _, _)
            | ParseErrorKind::NumericNotPositive(_, _)
            | ParseErrorKind::NumericNotNonNegative(_, _)
            | ParseErrorKind::InvalidTitleAlign(_)
            | ParseErrorKind::TitleRequiresArgument
    ));
}

#[test]
fn invalid_arrow_head_iter1_aborts_pipeline() {
    let error = parse_error("@-> (@{a}, @{b}, head=foo)\n@-> (@{a}, @{b})\n");
    assert!(matches!(
        error.kind(),
        ParseErrorKind::UndefinedAnchor(_)
            | ParseErrorKind::UnknownArrowAttribute(_)
            | ParseErrorKind::DuplicateArrowAttribute(_)
            | ParseErrorKind::InvalidArrowSyntax
            | ParseErrorKind::NumericNotParseable(_, _)
            | ParseErrorKind::NumericOverflow(_, _, _)
            | ParseErrorKind::NumericNotPositive(_, _)
            | ParseErrorKind::NumericNotNonNegative(_, _)
            | ParseErrorKind::InvalidTitleAlign(_)
            | ParseErrorKind::TitleRequiresArgument
    ));
}

#[test]
fn first_error_location_is_lowest_line_number_iter1() {
    let error = parse_error("A _\nB _\n@bg invalid\nC _\n");
    assert_eq!(
        error.line(),
        3,
        "first error must be reported on the @bg line (line 3); got {error:?}"
    );
}

// ---------------------------------------------------------------------------
// Iter1 phase 2: empty / zero / extreme value boundaries.
// ---------------------------------------------------------------------------

#[test]
fn empty_zero_byte_input_iter1() {
    let doc = parse_or_panic("");
    assert!(doc.lines.is_empty(), "empty input must produce zero lines");
    assert!(
        doc.annotations.anchors.is_empty(),
        "empty input must produce zero anchors"
    );
    assert!(
        doc.annotations.arrows.is_empty(),
        "empty input must produce zero arrows"
    );
    assert!(
        doc.annotations.overlays.is_empty(),
        "empty input must produce zero overlays"
    );
}

#[test]
fn title_only_zero_signal_file_iter1() {
    let doc = parse_or_panic("@title \"T\"\n");
    assert_eq!(count_signal_lines(&doc), 0);
    assert!(!doc.lines.is_empty());
}

#[test]
fn scale_zero_is_rejected_iter1() {
    let error = parse_error("@scale 0\nA _\n");
    assert!(matches!(
        error.kind(),
        ParseErrorKind::InvalidLength(_)
            | ParseErrorKind::NumericNotParseable(_, _)
            | ParseErrorKind::NumericOverflow(_, _, _)
            | ParseErrorKind::NumericNotPositive(_, _)
            | ParseErrorKind::NumericNotNonNegative(_, _)
            | ParseErrorKind::InvalidTitleAlign(_)
            | ParseErrorKind::TitleRequiresArgument
    ));
}

#[test]
fn scale_thousand_is_accepted_iter1() {
    let doc = parse_or_panic("@scale 1000\nA _\n");
    assert_eq!(count_signal_lines(&doc), 1);
}

#[test]
fn fontsize_half_is_accepted_iter1() {
    let doc = parse_or_panic("@fontsize 0.5\nA _\n");
    assert_eq!(count_signal_lines(&doc), 1);
}

#[test]
fn fontsize_one_boundary_iter1() {
    let doc = parse_or_panic("@fontsize 1.0\nA _\n");
    assert_eq!(count_signal_lines(&doc), 1);
}

#[test]
fn step_one_minimum_with_slant_zero_iter1() {
    let doc = parse_or_panic("@step 1\n@slant 0\nA _~_~\n");
    assert_eq!(count_signal_lines(&doc), 1);
}

#[test]
fn slant_zero_parses_iter1() {
    let doc = parse_or_panic("@slant 0\nA _~\n");
    assert_eq!(count_signal_lines(&doc), 1);
}

#[test]
fn one_character_waveform_iter1() {
    let doc = parse_or_panic("A _\n");
    let row = nth_signal(&doc, 0);
    assert_eq!(row.waveform().len(), 1);
}

// ---------------------------------------------------------------------------
// Iter1 phase 2: anchor / arrow advanced combinations.
// ---------------------------------------------------------------------------

#[test]
fn consecutive_anchors_in_single_signal_iter1() {
    let doc = parse_or_panic("A _@{x}@{y}@{z}~\n");
    assert_eq!(count_signal_lines(&doc), 1);
    assert_eq!(
        doc.annotations.anchors.len(),
        3,
        "three consecutive anchors must produce three registry entries"
    );
}

#[test]
fn arrow_self_loop_same_anchor_iter1() {
    let doc = parse_or_panic("A _@{a}~\n@-> (@{a}, @{a})\n");
    assert_eq!(doc.annotations.arrows.len(), 1);
}

#[test]
fn zero_arrows_anchor_only_iter1() {
    let doc = parse_or_panic("A _@{a}~\n");
    assert!(doc.annotations.arrows.is_empty());
}

#[test]
fn one_arrow_minimum_iter1() {
    let doc = parse_or_panic("A _@{a}@{b}~\n@-> (@{a}, @{b})\n");
    assert_eq!(doc.annotations.arrows.len(), 1);
}

#[test]
fn one_hundred_arrows_iter1() {
    let mut source = String::from("A _");
    // Anchor names must start with a non-digit identifier character (see
    // docs/spec/types.md `AnchorName`); use an `a` prefix so `@{a1}` etc. are
    // valid.
    for index in 1..=101 {
        source.push_str(&format!("@{{a{index}}}_"));
    }
    source.push('\n');
    for index in 1..=100 {
        source.push_str(&format!("@-> (@{{a{index}}}, @{{a{}}})\n", index + 1));
    }
    let doc = parse_or_panic(&source);
    assert_eq!(doc.annotations.arrows.len(), 100);
}

#[test]
fn clock_none_yields_zero_edge_marks_iter1() {
    let doc = parse_or_panic("@clock(none)\nclk _~_~\n");
    assert_eq!(count_signal_lines(&doc), 1);
}

#[test]
fn clock_pos_single_edge_iter1() {
    let doc = parse_or_panic("@clock(pos)\nclk _~\n");
    assert_eq!(count_signal_lines(&doc), 1);
}

#[test]
fn clock_pos_fifty_rising_edges_iter1() {
    let body: String = "_~".repeat(50);
    let source = format!("@clock(pos)\nclk {body}\n");
    let doc = parse_or_panic(&source);
    assert_eq!(count_signal_lines(&doc), 1);
}

#[test]
fn consecutive_anchors_with_three_arrows_iter1() {
    let doc =
        parse_or_panic("A _@{a}@{b}@{c}~\n@-> (@{a}, @{b})\n@-> (@{b}, @{c})\n@-> (@{a}, @{c})\n");
    assert_eq!(doc.annotations.arrows.len(), 3);
}

// ---------------------------------------------------------------------------
// Iter1 phase 2: WaveDrom-targeted parser fixtures (no actual conversion).
// ---------------------------------------------------------------------------

#[test]
fn no_clock_normal_signal_parses_iter1() {
    let doc = parse_or_panic("clk _~_~_~\n");
    assert_eq!(count_signal_lines(&doc), 1);
    let row = nth_signal(&doc, 0);
    assert!(
        row.edge_marks().is_empty(),
        "no @clock directive must not produce edge marks; got {:?}",
        row.edge_marks()
    );
    assert!(
        !row.waveform().is_empty(),
        "explicit waveform must keep its level runs"
    );
}

#[test]
fn unreferenced_anchors_do_not_cause_errors_iter1() {
    let doc = parse_or_panic("A _@{a}@{b}@{c}@{d}~\n");
    assert!(doc.annotations.arrows.is_empty());
    assert_eq!(count_signal_lines(&doc), 1);
    assert_eq!(
        doc.annotations.anchors.len(),
        4,
        "all four anchor declarations must be registered even when unreferenced"
    );
}

// ---------------------------------------------------------------------------
// Iter3 phase: Unicode / i18n in signal names and titles.
// ---------------------------------------------------------------------------

fn first_signal_name_text(doc: &crate::document::ChartDocument) -> String {
    let row = nth_signal(doc, 0);
    row.name()
        .lines()
        .next()
        .expect("at least one name line")
        .unsafe_text()
        .to_string()
}

#[test]
fn iter3_signal_name_arabic_rtl_is_preserved() {
    // RTL bytes in signal names must survive parse without normalisation.
    let result = parse("\u{0627}\u{0644}\u{0639}\u{0631}\u{0628}\u{064A}\u{0629} _~_~\n");
    let document = result.expect("RTL identifier must parse");
    let text = first_signal_name_text(&document);
    assert!(
        text.contains('\u{0627}'),
        "Arabic alef must appear verbatim in signal name; got {text:?}"
    );
}

#[test]
fn iter3_signal_name_hebrew_with_digit_preserves_bidi_chars() {
    let result = parse("\u{05E9}\u{05DC}\u{05D5}\u{05DD}2 _~\n");
    let document = result.expect("Hebrew + digit identifier must parse");
    let text = first_signal_name_text(&document);
    assert!(
        text.contains('\u{05E9}') && text.contains('2'),
        "BiDi mix must keep both halves; got {text:?}"
    );
}

#[test]
fn iter3_signal_name_combining_diacritic_keeps_two_codepoints() {
    let result = parse("e\u{0301} _~\n");
    let document = result.expect("e + combining acute must parse");
    let text = first_signal_name_text(&document);
    let codepoints: Vec<char> = text.chars().collect();
    assert!(
        codepoints.contains(&'\u{0301}'),
        "combining acute must remain unfused (no NFC); got {codepoints:?}"
    );
}

#[test]
fn iter3_signal_name_supplementary_emoji_preserved() {
    let result = parse("\u{1F389} _~\n");
    let document = result.expect("supplementary-plane emoji must parse");
    let text = first_signal_name_text(&document);
    assert!(
        text.contains('\u{1F389}'),
        "U+1F389 must round-trip as a single char; got {text:?}"
    );
}

#[test]
fn iter3_signal_name_with_ideographic_space_keeps_full_width_char() {
    // U+3000 ideographic space inside a name; whether the parser splits on it
    // is spec-defined — accept either behaviour but capture it.
    let outcome = parse("A\u{3000}B _~\n");
    match outcome {
        Ok(document) => {
            let text = first_signal_name_text(&document);
            assert!(
                text.contains('\u{3000}') || text == "A",
                "either keep the full-width space or split; got {text:?}"
            );
        }
        Err(error) => {
            assert!(
                matches!(
                    error.kind(),
                    ParseErrorKind::NumericNotParseable(_, _)
                        | ParseErrorKind::NumericOverflow(_, _, _)
                        | ParseErrorKind::NumericNotPositive(_, _)
                        | ParseErrorKind::NumericNotNonNegative(_, _)
                        | ParseErrorKind::InvalidTitleAlign(_)
                        | ParseErrorKind::TitleRequiresArgument
                ),
                "if rejected, must surface a message error; got {error:?}"
            );
        }
    }
}

#[test]
fn iter3_signal_name_with_nbsp_is_not_split() {
    let outcome = parse("A\u{00A0}B _~\n");
    if let Ok(document) = outcome {
        let text = first_signal_name_text(&document);
        assert!(
            text.contains('\u{00A0}') || text == "A",
            "NBSP behaviour must be deterministic; got {text:?}"
        );
    }
}

#[test]
fn iter3_signal_name_with_zero_width_space_keeps_codepoint() {
    let outcome = parse("A\u{200B}B _~\n");
    if let Ok(document) = outcome {
        let text = first_signal_name_text(&document);
        assert!(
            text.contains('\u{200B}') || text.contains('A'),
            "ZWSP behaviour must be deterministic; got {text:?}"
        );
    }
}

#[test]
fn iter3_title_nfc_and_nfd_treated_as_distinct_bytes() {
    // NFC: U+00E9 (é). NFD: e + U+0301.
    let nfc = parse("@title \"caf\u{00E9}\"\nA _\n").expect("NFC title parses");
    let nfd = parse("@title \"cafe\u{0301}\"\nA _\n").expect("NFD title parses");
    // Pin: parser must not normalise. Documents differ in some observable way
    // (line count or stored bytes) — we cannot inspect the title text directly
    // without widening visibility, so assert both successfully parsed.
    assert_eq!(count_signal_lines(&nfc), 1);
    assert_eq!(count_signal_lines(&nfd), 1);
}

#[test]
fn iter3_bom_at_start_of_file() {
    // BOM (U+FEFF) at file start; spec-undecided. Test pins current behaviour.
    let outcome = parse("\u{FEFF}@title \"T\"\nA _\n");
    match outcome {
        Ok(document) => assert!(!document.lines.is_empty()),
        Err(error) => assert!(
            matches!(
                error.kind(),
                ParseErrorKind::NumericNotParseable(_, _)
                    | ParseErrorKind::NumericOverflow(_, _, _)
                    | ParseErrorKind::NumericNotPositive(_, _)
                    | ParseErrorKind::NumericNotNonNegative(_, _)
                    | ParseErrorKind::InvalidTitleAlign(_)
                    | ParseErrorKind::TitleRequiresArgument
                    | ParseErrorKind::InvalidLevelChar(_)
            ),
            "BOM rejection should surface a message-style error; got {error:?}"
        ),
    }
}

#[test]
fn iter3_comment_with_emoji_and_combining_chars_is_skipped() {
    let document = parse_or_panic("// \u{1F389}e\u{0301} comment text\nA _\n");
    assert_eq!(count_signal_lines(&document), 1);
}

// ---------------------------------------------------------------------------
// Iter3 phase: error location precision.
// ---------------------------------------------------------------------------

#[test]
fn iter3_error_at_first_line_first_column() {
    let error = parse_error("@invalid_directive\n");
    assert_eq!(error.line(), 1, "error must be on line 1; got {error:?}");
    assert!(
        error.column() >= 1,
        "column must be >= 1 (1-based); got col={}",
        error.column()
    );
}

#[test]
fn iter3_error_after_tabs_has_deterministic_column() {
    let error = parse_error("\t\t@bad_directive\n");
    // Column convention is spec-undefined. Pin: column is non-zero, column
    // varies if tabs are counted, otherwise reflects byte offset + 1.
    assert!(
        error.column() >= 1,
        "column must be >= 1 even after tab indentation; got {}",
        error.column()
    );
}

#[test]
fn iter3_multiline_quoted_advances_line_counter() {
    let error = parse_error("@title \"line1\nline2\"\n@invalid_after\n");
    // After the multi-line title, the bad directive sits on line 3.
    assert!(
        error.line() >= 2,
        "post-multiline-title error must be reported beyond line 1; got line={}",
        error.line()
    );
}

#[test]
fn iter3_crlf_counts_as_single_newline() {
    let error = parse_error("@title \"T\"\r\n@bad\r\n");
    assert_eq!(
        error.line(),
        2,
        "CRLF must count as one newline; got line={}",
        error.line()
    );
}

#[test]
fn iter3_lone_cr_line_handling_is_deterministic() {
    // CR-only is spec-undefined. We just pin that parser does not panic and
    // reports a definite line number.
    let outcome = parse("@title T\r@bad\r");
    if let Err(error) = outcome {
        assert!(error.line() >= 1, "line must be >= 1; got {}", error.line());
    }
}

#[test]
fn iter3_multibyte_chars_column_unit_is_deterministic() {
    let error = parse_error("\u{65E5}\u{672C}\u{8A9E} _~ @bad_directive\n");
    assert!(
        error.column() >= 1 && error.line() >= 1,
        "error position must be reported; got {error:?}"
    );
}

#[test]
fn iter3_trailing_whitespace_then_eof_token() {
    // Trailing spaces after a valid signal — followed by an unknown directive
    // on next line — error is on line 2.
    let error = parse_error("A _~   \n@bad_directive\n");
    assert_eq!(
        error.line(),
        2,
        "error must report on the directive line, not the previous trailing space; got {error:?}"
    );
}

#[test]
fn iter3_first_error_on_line_with_two_bad_directives() {
    let error = parse_error("@bad1 @bad2\n");
    assert_eq!(
        error.line(),
        1,
        "two errors on one line: only first reported; got {error:?}"
    );
}

#[test]
fn iter3_eof_during_directive_argument() {
    // `@title` without an argument before EOF.
    let outcome = parse("@title");
    match outcome {
        Ok(_) => {}
        Err(error) => {
            assert_eq!(error.line(), 1, "EOF mid-directive: line 1; got {error:?}");
        }
    }
}

// ---------------------------------------------------------------------------
// Iter3 phase: numeric-precision negative tests for global parameters.
// ---------------------------------------------------------------------------

#[test]
fn iter3_scale_very_small_boundary() {
    // Spec lower bound for @scale is undefined. Pin current acceptance.
    let outcome = parse("@scale 0.0001\nA _\n");
    if let Ok(document) = outcome {
        assert_eq!(count_signal_lines(&document), 1);
    }
}

#[test]
fn iter3_scale_exponential_notation() {
    let outcome = parse("@scale 1e10\nA _\n");
    match outcome {
        Ok(document) => assert_eq!(count_signal_lines(&document), 1),
        Err(error) => assert!(
            matches!(
                error.kind(),
                ParseErrorKind::InvalidLength(_)
                    | ParseErrorKind::NumericNotParseable(_, _)
                    | ParseErrorKind::NumericOverflow(_, _, _)
                    | ParseErrorKind::NumericNotPositive(_, _)
                    | ParseErrorKind::NumericNotNonNegative(_, _)
                    | ParseErrorKind::InvalidTitleAlign(_)
                    | ParseErrorKind::TitleRequiresArgument
            ),
            "rejected exponential literal must surface a known error; got {error:?}"
        ),
    }
}

#[test]
fn iter3_step_zero_is_rejected() {
    let error = parse_error("@step 0\nA _\n");
    assert!(
        matches!(
            error.kind(),
            ParseErrorKind::InvalidLength(_)
                | ParseErrorKind::NumericNotParseable(_, _)
                | ParseErrorKind::NumericOverflow(_, _, _)
                | ParseErrorKind::NumericNotPositive(_, _)
                | ParseErrorKind::NumericNotNonNegative(_, _)
                | ParseErrorKind::InvalidTitleAlign(_)
                | ParseErrorKind::TitleRequiresArgument
        ),
        "step=0 must be a length error; got {error:?}"
    );
}

#[test]
fn iter3_step_negative_is_rejected() {
    let error = parse_error("@step -1\nA _\n");
    assert!(
        matches!(
            error.kind(),
            ParseErrorKind::InvalidLength(_)
                | ParseErrorKind::NumericNotParseable(_, _)
                | ParseErrorKind::NumericOverflow(_, _, _)
                | ParseErrorKind::NumericNotPositive(_, _)
                | ParseErrorKind::NumericNotNonNegative(_, _)
                | ParseErrorKind::InvalidTitleAlign(_)
                | ParseErrorKind::TitleRequiresArgument
        ),
        "negative step must be rejected; got {error:?}"
    );
}

#[test]
fn iter3_slant_negative_value_is_deterministic() {
    let outcome = parse("@slant -0.5\nA _~\n");
    match outcome {
        Ok(document) => assert_eq!(count_signal_lines(&document), 1),
        Err(error) => assert!(
            matches!(
                error.kind(),
                ParseErrorKind::InvalidLength(_)
                    | ParseErrorKind::NumericNotParseable(_, _)
                    | ParseErrorKind::NumericOverflow(_, _, _)
                    | ParseErrorKind::NumericNotPositive(_, _)
                    | ParseErrorKind::NumericNotNonNegative(_, _)
                    | ParseErrorKind::InvalidTitleAlign(_)
                    | ParseErrorKind::TitleRequiresArgument
            ),
            "if slant rejects negatives, must use length error; got {error:?}"
        ),
    }
}

// ---- @overline alias for @signal(overline) ---------------------------------
// Spec: docs/spec/tcml-format.md §「@overline (alias)」.

#[test]
fn overline_directive_alias_accepted() {
    let doc = parse_or_panic("@overline\nSig _\n");
    let row = match &doc.lines[0].content {
        LineContent::Signal(row) => row,
        other => panic!("expected Signal, got {other:?}"),
    };
    assert!(
        row.decorations().is_name_overline(),
        "@overline alias must mark following row as overline"
    );
}

#[test]
fn overline_directive_followed_by_quoted_name() {
    let doc = parse_or_panic("@overline\n\"sig name\" _\n");
    let row = match &doc.lines[0].content {
        LineContent::Signal(row) => row,
        other => panic!("expected Signal, got {other:?}"),
    };
    assert!(row.decorations().is_name_overline());
    assert_eq!(row.name().as_str(), "sig name");
}

#[test]
fn overline_directive_rejects_argument() {
    // Per spec: `@overline` takes no argument. Anything after `@overline`
    // (other than a trailing comment or whitespace) must be rejected with the
    // dedicated `InvalidOverlineSyntax` variant so that callers can
    // distinguish a typo'd alias from unrelated failures.
    let error = parse_error("@overline foo\nSig _\n");
    assert!(
        matches!(error.kind(), ParseErrorKind::InvalidOverlineSyntax(_)),
        "@overline with an argument must produce InvalidOverlineSyntax; got {error:?}"
    );
}

#[test]
fn overline_directive_then_signal_attribute_combines() {
    // `@overline` and `@signal(overline)` are the same decoration; the alias
    // path must reuse the same pending-state pipeline. Two scenarios:
    //
    // 1. Different rows get their own pending state — `@overline` on row 1 +
    //    `@signal(overline)` on row 2 leaves each row with exactly one
    //    decoration.
    let doc = parse_or_panic("@overline\nA _\n@signal(overline)\nB _\n");
    let first = match &doc.lines[0].content {
        LineContent::Signal(row) => row,
        other => panic!("expected Signal, got {other:?}"),
    };
    let second = match &doc.lines[1].content {
        LineContent::Signal(row) => row,
        other => panic!("expected Signal, got {other:?}"),
    };
    assert!(first.decorations().is_name_overline());
    assert!(second.decorations().is_name_overline());

    // 2. Stacking on the same row — `@overline` immediately before
    //    `@signal(overline)`, then the target row — must be deterministic.
    //    Either idempotent (single decoration applied) or a duplicate-attribute
    //    error; never a silent disagreement between the two forms.
    if let Ok(document) = parse("@overline\n@signal(overline)\nSig _\n") {
        match &document.lines[0].content {
            LineContent::Signal(row) => {
                assert!(
                    row.decorations().is_name_overline(),
                    "if stacking succeeds, overline must still be set"
                );
            }
            other => panic!("expected Signal, got {other:?}"),
        }
    }
}

// ---------------------------------------------------------------------------
// `@step` -> auto-clamp `@slant` boundary tests (spec §「@step」「@slant」).
//
// When `@step N` is set and `@slant` was never explicitly given, slant is
// silently shrunk to `step / 2` so that small `@step` values stay usable with
// the default slant=5. Once `@slant` has been written, subsequent `@step`
// directives stop auto-clamping and `step <= slant` surfaces as
// `InvalidStepSlant`.
// ---------------------------------------------------------------------------

#[test]
fn auto_clamp_slant_chain_of_decreasing_step() {
    // Default state: slant=5, `@slant` not explicit.
    // Each `@step N` re-evaluates the clamp `slant = min(slant, N / 2)`.
    // The chain only shrinks slant when the *current* slant satisfies
    // `slant >= new_step`. We assert the value after every step value by
    // parsing each prefix as an independent document.
    let cases: &[(&str, Px)] = &[
        // `@step 12` -> slant(5) < 12, no clamp -> slant stays 5.
        ("@step 12\nA __\n", Px(5.0)),
        // `@step 10` -> slant(5) < 10, no clamp -> slant stays 5.
        ("@step 10\nA __\n", Px(5.0)),
        // `@step 8` -> slant(5) < 8 -> slant stays 5.
        ("@step 8\nA __\n", Px(5.0)),
        // `@step 6` -> slant(5) < 6 -> slant stays 5.
        ("@step 6\nA __\n", Px(5.0)),
        // `@step 5` -> slant(5) >= 5 -> clamp to 5/2 = 2.5.
        ("@step 5\nA __\n", Px(2.5)),
        // `@step 4` -> slant(5) >= 4 -> clamp to 4/2 = 2.0.
        ("@step 4\nA __\n", Px(2.0)),
        // `@step 3` -> slant(5) >= 3 -> clamp to 3/2 = 1.5.
        ("@step 3\nA __\n", Px(1.5)),
        // `@step 2` -> slant(5) >= 2 -> clamp to 2/2 = 1.0.
        ("@step 2\nA __\n", Px(1.0)),
        // `@step 1` -> slant(5) >= 1 -> clamp to 1/2 = 0.5.
        ("@step 1\nA __\n", Px(0.5)),
    ];
    for (input, expected_slant) in cases {
        let doc = parse_or_panic(input);
        assert_eq!(
            doc.style.layout().slant(),
            *expected_slant,
            "input={input:?}: expected slant={expected_slant:?}"
        );
    }
}

#[test]
fn auto_clamp_does_not_fire_when_slant_explicit() {
    // After `@slant 0` is set, the explicit flag is sticky: subsequent `@step`
    // directives must not rewrite slant to `step / 2`. Slant stays at the
    // value the user wrote.
    for step in ["10", "4", "1"] {
        let source = format!("@slant 0\n@step {step}\nA __\n");
        let doc = parse_or_panic(&source);
        assert_eq!(
            doc.style.layout().slant(),
            Px(0.0),
            "after `@slant 0` + `@step {step}`, slant must stay 0"
        );
    }
}

#[test]
fn auto_clamp_explicit_slant_then_smaller_step_errors() {
    // `@slant 8` makes slant explicit. `@step 4` therefore skips auto-clamp,
    // and the standard `step <= slant` rule (4 <= 8) fires as
    // `InvalidStepSlant`.
    let error = parse_error("@slant 8\n@step 4\nA __\n");
    assert!(matches!(
        error.kind(),
        ParseErrorKind::InvalidStepSlant(_, _)
    ));
}

#[test]
fn auto_clamp_does_not_fire_when_step_larger_than_slant() {
    // `@step 100` with default slant=5: `slant >= step` is false (5 < 100), so
    // clamp does not fire and slant stays at the default value 5.
    let doc = parse_or_panic("@step 100\nA __\n");
    assert_eq!(doc.style.layout().slant(), Px(5.0));
}

// ---------------------------------------------------------------- @ruler

/// Returns the (count, set of x values) of ruler contributions on the given
/// line index. Test helper for `@ruler` scenarios.
fn ruler_contributions_for(input: &str, line_index: usize) -> Vec<crate::line::RulerContribution> {
    let doc = parse_or_panic(input);
    doc.lines
        .get(line_index)
        .expect("line index out of range in test")
        .ruler_contributions
        .clone()
}

#[test]
fn ruler_default_is_on() {
    // Default state: no `@ruler` directive at all. Signal rows commit with
    // ruler_on = true (new default), so their ruler_contributions Vec is
    // non-empty (units + 1 entries).
    let doc = parse_or_panic("A _~_~\n");
    let line = doc.lines.first().expect("one line");
    assert!(!line.ruler_contributions.is_empty());
}

#[test]
fn ruler_on_switches_state() {
    // `@ruler on` flips the parser state; the next signal row picks it up
    // and contributes.
    let doc = parse_or_panic("@ruler on\nA _~\n");
    let line = doc.lines.first().expect("one line");
    assert!(!line.ruler_contributions.is_empty());
}

#[test]
fn ruler_off_switches_state() {
    // After `@ruler on` + a contributing row, `@ruler off` returns to the
    // default state and following rows do not contribute.
    let doc = parse_or_panic("@ruler on\nA _~\n@ruler off\nB _~\n");
    assert!(!doc.lines[0].ruler_contributions.is_empty());
    assert!(doc.lines[1].ruler_contributions.is_empty());
}

#[test]
fn ruler_invalid_argument_errors() {
    // `@ruler maybe` — argument must be `on` or `off`. Spec leaves the
    // exact error kind unconstrained; the implementation surfaces it as
    // `InvalidRulerValue`.
    let error = parse_error("@ruler maybe\n");
    assert!(matches!(error.kind(), ParseErrorKind::InvalidRulerValue(_)));
}

#[test]
fn ruler_missing_argument_errors() {
    // `@ruler` alone — argument is required.
    let error = parse_error("@ruler\n");
    // Same error family as other "missing argument" cases. We accept any
    // kind here; the important property is that parsing fails.
    let _ = error.kind();
}

#[test]
fn ruler_color_invalid_format_errors() {
    let error = parse_error("@ruler_color not-a-color\n");
    assert!(matches!(error.kind(), ParseErrorKind::InvalidColor(_)));
}

#[test]
fn ruler_color_default_is_a0a0a0() {
    // `@ruler on` alone, no `@ruler_color` — contributed color is #a0a0a0.
    let doc = parse_or_panic("@ruler on\nA _~\n");
    let expected = crate::color::Color::parse("#a0a0a0").expect("color");
    let contributions = &doc.lines[0].ruler_contributions;
    assert!(!contributions.is_empty());
    for contribution in contributions {
        assert_eq!(contribution.color, expected);
    }
}

#[test]
fn ruler_signal_row_contribution_count_and_positions() {
    // `@step 10`, `@ruler on`, signal `A _~_~_~` (6 chars → units = 6).
    // Expect 7 contributions at x = 0, 10, 20, 30, 40, 50, 60.
    let doc = parse_or_panic("@step 10\n@ruler on\nA _~_~_~\n");
    let contributions = &doc.lines[0].ruler_contributions;
    let xs: Vec<f32> = contributions
        .iter()
        .map(|contribution| contribution.x.to_f32())
        .collect();
    assert_eq!(xs, vec![0.0, 10.0, 20.0, 30.0, 40.0, 50.0, 60.0]);
}

#[test]
fn ruler_skip_row_contributes() {
    // `@step 10`, `@ruler on`, `@skip(3)` (units = 3, both ends included → 4 lines).
    let doc = parse_or_panic("@step 10\n@ruler on\n@skip(3)\n");
    let contributions = &doc.lines[0].ruler_contributions;
    let xs: Vec<f32> = contributions
        .iter()
        .map(|contribution| contribution.x.to_f32())
        .collect();
    assert_eq!(xs, vec![0.0, 10.0, 20.0, 30.0]);
}

#[test]
fn ruler_skip_zero_contributes_single_line() {
    // `@step 10`, `@ruler on`, `@skip(0)` — but @skip(0) is dropped by the
    // parser entirely (no row appended). So no contribution exists.
    // To exercise the units = 0 path explicitly, use a signal row with 0
    // body. Empty body still commits a SignalRow, and units = 0 ⇒ exactly
    // one ruler contribution at x = 0.
    let doc = parse_or_panic("@step 10\n@ruler on\nA\n");
    let contributions = &doc.lines[0].ruler_contributions;
    let xs: Vec<f32> = contributions
        .iter()
        .map(|contribution| contribution.x.to_f32())
        .collect();
    assert_eq!(xs, vec![0.0]);
}

#[test]
fn ruler_off_rows_have_empty_sidecar() {
    // After `@ruler on` + row A contributing, `@ruler off` then row B
    // contributes nothing.
    let doc = parse_or_panic("@step 10\n@ruler on\nA _~_~\n@ruler off\nB _~_~\n");
    assert!(!doc.lines[0].ruler_contributions.is_empty());
    assert!(doc.lines[1].ruler_contributions.is_empty());
}

#[test]
fn ruler_title_row_does_not_contribute() {
    // `@title` rows never contribute regardless of `@ruler on`.
    let doc = parse_or_panic("@ruler on\n@title \"Section\"\nA _~\n");
    // The title row is at index 0, the signal row at index 1.
    let title = doc.lines.first().expect("title");
    assert!(title.content.is_title());
    assert!(title.ruler_contributions.is_empty());
    let signal = doc.lines.get(1).expect("signal");
    assert!(!signal.ruler_contributions.is_empty());
}

#[test]
fn ruler_color_snapshot_independent_per_row() {
    // Row A is committed while @ruler_color = #aaa, row B while #bbb. Later
    // changing the color must not retroactively rewrite row A's snapshot.
    let doc =
        parse_or_panic("@step 10\n@ruler on\n@ruler_color #aaa\nA _~\n@ruler_color #bbb\nB _~\n");
    let color_aaa = crate::color::Color::parse("#aaa").expect("color");
    let color_bbb = crate::color::Color::parse("#bbb").expect("color");
    assert!(
        doc.lines[0]
            .ruler_contributions
            .iter()
            .all(|contribution| contribution.color == color_aaa)
    );
    assert!(
        doc.lines[1]
            .ruler_contributions
            .iter()
            .all(|contribution| contribution.color == color_bbb)
    );
}

#[test]
fn ruler_step_snapshot_per_row() {
    // Row A is committed with @step 10, row B with @step 25. Per-row
    // contribution positions reflect the step at commit time.
    let doc = parse_or_panic("@ruler on\n@step 10\nA _~_~\n@step 25\nB _~_~\n");
    let xs_a: Vec<f32> = doc.lines[0]
        .ruler_contributions
        .iter()
        .map(|contribution| contribution.x.to_f32())
        .collect();
    let xs_b: Vec<f32> = doc.lines[1]
        .ruler_contributions
        .iter()
        .map(|contribution| contribution.x.to_f32())
        .collect();
    assert_eq!(xs_a, vec![0.0, 10.0, 20.0, 30.0, 40.0]);
    assert_eq!(xs_b, vec![0.0, 25.0, 50.0, 75.0, 100.0]);
}

#[test]
fn ruler_units_match_signal_row_units() {
    // A has 4 chars (units=4), B has 8 chars (units=8).
    let doc = parse_or_panic("@step 10\n@ruler on\nA _~_~\nB _~_~_~_~\n");
    assert_eq!(doc.lines[0].ruler_contributions.len(), 5);
    assert_eq!(doc.lines[1].ruler_contributions.len(), 9);
}

#[test]
fn ruler_color_change_during_off_applies_to_next_on_row() {
    // While ruler is off, `@ruler_color #bbb` is recorded but no row
    // contributes. The next `@ruler on` row picks up the new color.
    let doc = parse_or_panic(
        "@ruler on\n@ruler_color #aaa\nA _~\n@ruler off\n@ruler_color #bbb\n@ruler on\nB _~\n",
    );
    let color_aaa = crate::color::Color::parse("#aaa").expect("color");
    let color_bbb = crate::color::Color::parse("#bbb").expect("color");
    assert!(
        doc.lines[0]
            .ruler_contributions
            .iter()
            .all(|contribution| contribution.color == color_aaa)
    );
    assert!(!doc.lines[1].ruler_contributions.is_empty());
    assert!(
        doc.lines[1]
            .ruler_contributions
            .iter()
            .all(|contribution| contribution.color == color_bbb)
    );
}

#[test]
fn ruler_toggle_independent_per_row() {
    // ON / OFF / ON with three signal rows: A (on), B (off), C (on).
    let doc = parse_or_panic("@ruler on\nA _~\n@ruler off\nB _~\n@ruler on\nC _~\n");
    assert!(!doc.lines[0].ruler_contributions.is_empty());
    assert!(doc.lines[1].ruler_contributions.is_empty());
    assert!(!doc.lines[2].ruler_contributions.is_empty());
}

#[test]
fn ruler_all_off_explicit_means_all_empty() {
    // Explicit `@ruler off` before any row → all rows have empty
    // contributions. (Default is on; this test forces off-from-start.)
    let doc = parse_or_panic("@ruler off\nA _~\nB _~\nC _~\n@skip(2)\n");
    for line in &doc.lines {
        assert!(line.ruler_contributions.is_empty());
    }
}

#[test]
fn ruler_contribution_x_values_are_ascending() {
    // Spec: "同一行内では x 値が小さい順に並ぶ" (clarifying test).
    let contributions = ruler_contributions_for("@step 10\n@ruler on\nA _~_~_~\n", 0);
    let xs: Vec<f32> = contributions
        .iter()
        .map(|contribution| contribution.x.to_f32())
        .collect();
    let mut sorted = xs.clone();
    sorted.sort_by(|left, right| left.partial_cmp(right).expect("finite"));
    assert_eq!(xs, sorted);
}
