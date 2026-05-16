//! Insert `WaveformElement::Transition` between adjacent level runs and
//! emit `BusCross` transitions for `X` markers.
//!
//! The peekable token iterator is owned by [`TransitionEmitter`] rather
//! than being passed `&mut` between free functions.

use std::iter::Peekable;
use std::vec::IntoIter;

use crate::errors::ParseError;
use crate::line::{LevelRun, LevelShape, SignalLevel, Transition, TransitionKind, WaveformElement};
use crate::text::UserText;

use super::waveform::ResolvedToken;

/// State machine that walks a [`ResolvedToken`] list and produces the
/// transition-resolved [`WaveformElement`] list.
///
/// Text fragments carried by each `Level` token are emitted as
/// [`WaveformElement::Text`] immediately after the corresponding `Level`
/// element (spec: `docs/spec/types.md` §6.4).
pub(super) struct TransitionEmitter {
    tokens: Peekable<IntoIter<ResolvedToken>>,
    output: Vec<WaveformElement>,
    last_level: Option<SignalLevel>,
    /// Latched `true` when `emit_buscross` pushes a Transition but cannot yet
    /// set `preceded_by_transition` on the body `LevelRun` (because that run
    /// has not been popped yet).  Cleared by `emit_level` after it consumes it.
    pending_transition_for_next_level: bool,
}

impl TransitionEmitter {
    pub(super) fn new(tokens: Vec<ResolvedToken>) -> Self {
        let capacity = tokens.len();
        Self {
            tokens: tokens.into_iter().peekable(),
            output: Vec::with_capacity(capacity),
            last_level: None,
            pending_transition_for_next_level: false,
        }
    }

    /// Drain the entire input, returning the transition-resolved element list.
    pub(super) fn emit(mut self) -> Result<Vec<WaveformElement>, ParseError> {
        while let Some(token) = self.tokens.next() {
            match token {
                ResolvedToken::Level(run, fragments) => self.emit_level(run, fragments),
                ResolvedToken::BusCross => self.emit_buscross(),
                ResolvedToken::Element(element) => self.output.push(element),
            }
        }
        Ok(self.output)
    }

    /// Emit a level run, inserting a transition from the previous level when
    /// appropriate, then emitting a `Text` element when `fragments` is non-empty.
    fn emit_level(&mut self, mut run: LevelRun, fragments: Vec<String>) {
        let current = run.level();
        // A `BusCross` call may have latched `pending_transition_for_next_level`
        // when it already pushed a Transition that will precede this level.
        let pending = std::mem::replace(&mut self.pending_transition_for_next_level, false);
        let mut auto_transition = false;
        if !pending
            && let Some(previous) = self.last_level
            && let Some(kind) = classify_transition_kind(previous, current)
        {
            self.output
                .push(WaveformElement::Transition(Transition::new(
                    previous, current, kind, None,
                )));
            auto_transition = true;
        }
        if pending || auto_transition {
            run.mark_preceded_by_transition();
        }
        self.output.push(WaveformElement::Level(run));
        if let Some(text) = join_text_fragments(&fragments) {
            self.output.push(WaveformElement::Text(text));
        }
        self.last_level = Some(current);
    }

    /// Emit a `BusCross` (`X`) marker.
    ///
    /// The tokenizer has already appended a `LevelRun(Bus, 1)` body immediately
    /// after every `BusCross` token, so the next resolved token is always a bus-
    /// family level (Bus or DontCareAlongBus after `?` resolution).
    ///
    /// Behaviour depends on the preceding level:
    ///
    /// - No preceding level (signal start / after Gap): no `Transition` is emitted.
    ///   `last_level` is updated to the body level so subsequent elements connect.
    /// - Preceding level is Bus/DontCareAlongBus: emit `Transition(BusCross)`.
    /// - Preceding level is non-bus (Low/High/HiZ): emit `Transition(BusOpen)`.
    ///   This implicitly opens a bus region before the X body.
    ///
    /// After the body level run has been emitted, when the *following* token is a
    /// non-bus level (or absent), `Transition(BusClose)` is injected between the
    /// body and the next level. That injection is handled at the start of the next
    /// `emit_level` call via the normal `classify_transition_kind` path — no
    /// special post-processing is required here because `BusClose` is already the
    /// result of `Double/FillDouble → Single` in that function.
    fn emit_buscross(&mut self) {
        // The body LevelRun(Bus, 1) is always the next token.
        let next_level = self.peek_next_level().unwrap_or(SignalLevel::Bus);

        match self.last_level {
            None => {
                // Signal start: suppress Transition entirely; body LevelRun follows.
                self.last_level = Some(next_level);
            }
            Some(from) if from.is_bus_family() => {
                // Normal BusCross between bus-family levels.
                self.output
                    .push(WaveformElement::Transition(Transition::new(
                        from,
                        next_level,
                        TransitionKind::BusCross,
                        None,
                    )));
                // The body LevelRun will be emitted by the next `emit_level` call.
                // Latch so it picks up `preceded_by_transition = true`.
                self.pending_transition_for_next_level = true;
                self.last_level = Some(next_level);
            }
            Some(from) => {
                // Non-bus predecessor: insert implicit BusOpen instead of BusCross.
                self.output
                    .push(WaveformElement::Transition(Transition::new(
                        from,
                        next_level,
                        TransitionKind::BusOpen,
                        None,
                    )));
                self.pending_transition_for_next_level = true;
                self.last_level = Some(next_level);
            }
        }
    }

    /// Peek at the level of the next `Level` token, if any.
    fn peek_next_level(&mut self) -> Option<SignalLevel> {
        match self.tokens.peek()? {
            ResolvedToken::Level(run, _) => Some(run.level()),
            _ => None,
        }
    }
}

/// Join text fragments into a single space-separated [`UserText`].
///
/// Returns `None` when `fragments` is empty or all the entries are empty
/// strings. Returns `None` also if the joined value somehow fails
/// `UserText::parse`; in practice this cannot happen because the tokenizer
/// already rejected forbidden control characters before any fragment reached
/// this function, but we handle the case defensively rather than panic.
fn join_text_fragments(fragments: &[String]) -> Option<UserText> {
    let mut joined = String::new();
    for fragment in fragments.iter().filter(|fragment| !fragment.is_empty()) {
        if !joined.is_empty() {
            joined.push(' ');
        }
        joined.push_str(fragment);
    }
    if joined.is_empty() {
        return None;
    }
    UserText::parse(&joined).ok()
}

fn classify_transition_kind(from: SignalLevel, to: SignalLevel) -> Option<TransitionKind> {
    if from == to {
        return None;
    }
    match (from.into_shape(), to.into_shape()) {
        (LevelShape::Single, LevelShape::Single) => Some(TransitionKind::SingleEdge),
        // Single -> Double/FillDouble: opening into a bus-family region.
        // FillDouble (`DontCareAlongBus`) absorbs surrounding `=` runs during
        // DontCare expansion, so the Single->FillDouble boundary needs the same
        // BusOpen transition as Single->Double.
        (LevelShape::Single, LevelShape::Double | LevelShape::FillDouble) => {
            Some(TransitionKind::BusOpen)
        }
        // Double/FillDouble -> Single: closing out of a bus-family region.
        (LevelShape::Double | LevelShape::FillDouble, LevelShape::Single) => {
            Some(TransitionKind::BusClose)
        }
        // Bus<->Bus or FillSingle pairs do not produce an automatic transition.
        _ => None,
    }
}
