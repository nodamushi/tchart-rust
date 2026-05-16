//! Two-pass waveform-tag parser.
//!
//! A waveform tag is the right-hand side of a signal line: the level run
//! string `_~=X[?:|]"text"a@anchor`. This module owns its full tokenisation
//! and assembly pipeline.
//!
//! 1. [`WaveformParser::parse`] tokenises the raw level string into
//!    [`RawElement`]s, deferring `?` to a placeholder
//!    `LevelRun { level: None, .. }` and recording every `@anchor` it sees.
//! 2. Don't-care inheritance is resolved and adjacent equal runs are merged.
//! 3. Text fragments accumulated per level run are joined with spaces and
//!    emitted as [`WaveformElement::Text`] elements immediately after their
//!    owning [`LevelRun`] by the transition-emitter pass.
//! 4. Transition elements are injected by
//!    [`super::transition::TransitionEmitter`].

use std::iter::Peekable;
use std::str::CharIndices;

use crate::anchor::AnchorId;
use crate::errors::{ParseError, ParseErrorKind, SourceLocation};
use crate::line::{LevelRun, SignalLevel, Waveform, WaveformElement};

use super::anchor::AnchorScanner;
use super::state::PendingAnchor;
use super::transition::TransitionEmitter;

/// Owning parser for a single waveform-tag string.
///
/// The full pipeline lives on this type so `Waveform`-returning logic does
/// not appear as a free function.
pub(super) struct WaveformParser<'source> {
    levels: &'source str,
    location: SourceLocation,
    signal_index: usize,
}

/// Final output of [`WaveformParser::parse`]. A named struct so call sites
/// can refer to each field by name.
pub(super) struct WaveformParseResult {
    pub(super) waveform: Waveform,
    pub(super) pending_anchors: Vec<PendingAnchor>,
}

impl<'source> WaveformParser<'source> {
    pub(super) fn new(levels: &'source str, location: SourceLocation, signal_index: usize) -> Self {
        Self {
            levels,
            location,
            signal_index,
        }
    }

    pub(super) fn parse(self) -> Result<WaveformParseResult, ParseError> {
        let TokenizeOutput {
            elements,
            mut pending_anchors,
        } = Tokenizer::new(self.levels, self.location, self.signal_index).run()?;
        let resolved = resolve_dontcare_inheritance(&elements, self.location)?;
        let merged = merge_adjacent_levels(resolved);
        let with_transitions = TransitionEmitter::new(merged).emit()?;
        // TransitionEmitter may inject Transition elements between existing
        // elements, shifting anchor indices. Recompute each anchor's
        // element_index by scanning the final waveform element list.
        reindex_anchors(&with_transitions, &mut pending_anchors);
        Ok(WaveformParseResult {
            waveform: Waveform::from(with_transitions),
            pending_anchors,
        })
    }
}

// =============================================================================
// First pass: raw tokenisation
// =============================================================================

/// One token produced by the first parsing pass.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum RawElement {
    /// A run of one signal level. `level=None` represents a pending `?`.
    LevelRun(RawLevel),
    /// Bus crossing marker (`X`).
    BusCross,
    /// Gap (`:`).
    Gap,
    /// Guide (`|`).
    Guide,
    /// Highlight start (`[`).
    HighlightStart,
    /// Highlight end (`]`).
    HighlightEnd,
    /// Anchor marker.
    Anchor(AnchorId),
    /// One bare (unquoted) text character.
    BareChar(char),
    /// A `"..."` quoted text literal.
    QuotedText(String),
}

/// Level information emitted by the first pass.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct RawLevel {
    /// Resolved level when known, or `None` for a pending `?`.
    level: Option<SignalLevel>,
    /// Number of step units this run covers.
    units: u32,
    /// Column of the first level character of this run (1-based, char
    /// units). Used by the don't-care resolver to caret-align a
    /// `DontCareWithoutAnchor` error directly under the originating `?`.
    /// `0` means "not tracked" (e.g. synthesised level runs from `X`).
    first_char_column: u32,
}

impl RawLevel {
    /// Pick the [`SourceLocation`] to attach to a `DontCareWithoutAnchor`
    /// error originating from this run. Prefers the per-run
    /// `first_char_column` when tracked; otherwise falls back to the
    /// row-start `fallback`.
    fn dontcare_location(&self, fallback: SourceLocation) -> SourceLocation {
        if self.first_char_column == 0 {
            fallback
        } else {
            SourceLocation::new(fallback.line(), self.first_char_column)
        }
    }
}

/// One of the level-bearing characters that [`Tokenizer::push_level`]
/// understands. Encoding the closed set as an enum makes the dispatch total
/// (no `unreachable!` arm needed).
#[derive(Debug, Clone, Copy)]
enum LevelChar {
    Low,
    High,
    HiZ,
    Bus,
    DontCare,
}

impl LevelChar {
    fn from_dispatch(character: char) -> Option<Self> {
        match character {
            '_' => Some(Self::Low),
            '~' => Some(Self::High),
            '-' => Some(Self::HiZ),
            '=' => Some(Self::Bus),
            '?' => Some(Self::DontCare),
            _ => None,
        }
    }

    fn into_signal_level(self) -> Option<SignalLevel> {
        match self {
            Self::Low => Some(SignalLevel::Low),
            Self::High => Some(SignalLevel::High),
            Self::HiZ => Some(SignalLevel::HiZ),
            Self::Bus => Some(SignalLevel::Bus),
            Self::DontCare => None,
        }
    }
}

/// Output bundle assembled by [`Tokenizer::run`].
struct TokenizeOutput {
    elements: Vec<RawElement>,
    pending_anchors: Vec<PendingAnchor>,
}

/// Character-by-character tokenizer for waveform tags.
///
/// Separates the input cursor (`chars`/`source`) from output accumulation
/// ([`TokenizeOutput`]) and from temporary scanning state
/// (`highlight_open`). Call [`Self::run`] once.
///
/// `levels_start_column` is the 1-based char-column at which `source` begins
/// in the original TCML line. Every per-token error column is computed as
/// `levels_start_column + char_offset_of_token_in_source`. Char offsets are
/// counted in Unicode scalar values (not bytes), matching
/// `docs/spec/tcml-format.md` §位置情報の必須化.
struct Tokenizer<'source> {
    chars: Peekable<CharIndices<'source>>,
    source: &'source str,
    location: SourceLocation,
    /// Column at which `source` (the level string) begins in the TCML line.
    levels_start_column: u32,
    /// Column of the most recently consumed character (1-based, char units).
    /// Used by error constructors so they point at the offending token, not
    /// at the start of the line.
    last_char_column: u32,
    signal_index: usize,
    output: TokenizeOutput,
    highlight_open: bool,
    /// Column of the opening `[` when `highlight_open` is true, so the
    /// `UnclosedHighlight` error caret can sit on the `[` rather than on
    /// whatever the last processed character happens to be.
    highlight_open_column: u32,
    /// Whether any level symbol or `X` has been seen yet (for MissingInitialLevel check).
    seen_level: bool,
}

impl<'source> Tokenizer<'source> {
    fn new(source: &'source str, location: SourceLocation, signal_index: usize) -> Self {
        let levels_start_column = location.column();
        Self {
            chars: source.char_indices().peekable(),
            source,
            location,
            levels_start_column,
            last_char_column: levels_start_column,
            signal_index,
            output: TokenizeOutput {
                elements: Vec::new(),
                pending_anchors: Vec::new(),
            },
            highlight_open: false,
            highlight_open_column: 0,
            seen_level: false,
        }
    }

    /// Build a [`SourceLocation`] pointing at the column of the most
    /// recently consumed character. Used by error constructors so each
    /// kind's caret aligns with the offending token.
    fn location_at_last_char(&self) -> SourceLocation {
        SourceLocation::new(self.location.line(), self.last_char_column)
    }

    /// Convert a byte index inside `self.source` into a 1-based char column
    /// in the original TCML line. Used by [`Self::consume_quoted_text`] to
    /// record the column of the opening `"` rather than of the line start.
    fn column_at_source_byte(&self, byte_index: usize) -> u32 {
        let prefix = self.source.get(..byte_index).unwrap_or("");
        let chars_before = u32::try_from(prefix.chars().count()).unwrap_or(u32::MAX);
        self.levels_start_column.saturating_add(chars_before)
    }

    /// Walk every character, dispatch by syntactic role, and yield the
    /// accumulated output. Errors when a `[` highlight is left unclosed.
    fn run(mut self) -> Result<TokenizeOutput, ParseError> {
        while let Some((byte_index, character)) = self.chars.next() {
            self.last_char_column = self.column_at_source_byte(byte_index);
            self.dispatch_char(byte_index, character)?;
        }
        if self.highlight_open {
            return Err(ParseError::with_length(
                SourceLocation::new(self.location.line(), self.highlight_open_column),
                1,
                ParseErrorKind::UnclosedHighlight,
            ));
        }
        Ok(self.output)
    }

    fn dispatch_char(&mut self, byte_index: usize, character: char) -> Result<(), ParseError> {
        if let Some(level) = LevelChar::from_dispatch(character) {
            self.push_level(level);
            return Ok(());
        }
        match character {
            'X' => {
                self.push_bus_cross();
                Ok(())
            }
            ':' => {
                self.push_simple(RawElement::Gap);
                Ok(())
            }
            '|' => {
                self.push_simple(RawElement::Guide);
                Ok(())
            }
            '[' => self.open_highlight(),
            ']' => self.close_highlight(),
            '@' => self.consume_anchor(),
            '"' => self.consume_quoted_text(byte_index),
            other if other.is_whitespace() => Ok(()),
            other if other.is_control() => Err(ParseError::with_length(
                self.location_at_last_char(),
                1,
                ParseErrorKind::InvalidLevelChar(other),
            )),
            other => self.push_bare_text_char(other),
        }
    }

    fn push_level(&mut self, level_char: LevelChar) {
        self.seen_level = true;
        let level = level_char.into_signal_level();
        // `?` is a zero-width marker: it contributes 0 units to the run.
        let units = if level.is_none() { 0 } else { 1 };
        if let Some(RawElement::LevelRun(run)) = self.output.elements.last_mut()
            && run.level == level
        {
            run.units += units;
            return;
        }
        self.output.elements.push(RawElement::LevelRun(RawLevel {
            level,
            units,
            first_char_column: self.last_char_column,
        }));
    }

    fn push_bus_cross(&mut self) {
        self.seen_level = true;
        // `X` emits a BusCross marker followed immediately by a Bus body run of
        // 1 unit (the "new value" region). When there is no preceding bus level
        // (signal start), TransitionEmitter will suppress the Transition element;
        // the body LevelRun remains and is later merged with the following `=`
        // run during `merge_adjacent_levels`.
        self.output.elements.push(RawElement::BusCross);
        self.output.elements.push(RawElement::LevelRun(RawLevel {
            level: Some(SignalLevel::Bus),
            units: 1,
            // Synthesised run; the `X` column is `self.last_char_column` —
            // close enough for caret rendering, though this run is never the
            // origin of a `DontCareWithoutAnchor` error.
            first_char_column: self.last_char_column,
        }));
    }

    fn push_simple(&mut self, element: RawElement) {
        self.output.elements.push(element);
    }

    fn open_highlight(&mut self) -> Result<(), ParseError> {
        if self.highlight_open {
            // Nested `[`: point at the inner `[`.
            return Err(ParseError::with_length(
                self.location_at_last_char(),
                1,
                ParseErrorKind::UnclosedHighlight,
            ));
        }
        self.highlight_open = true;
        self.highlight_open_column = self.last_char_column;
        self.output.elements.push(RawElement::HighlightStart);
        Ok(())
    }

    fn close_highlight(&mut self) -> Result<(), ParseError> {
        if !self.highlight_open {
            return Err(ParseError::with_length(
                self.location_at_last_char(),
                1,
                ParseErrorKind::UnopenedHighlightEnd,
            ));
        }
        self.highlight_open = false;
        self.output.elements.push(RawElement::HighlightEnd);
        Ok(())
    }

    fn consume_anchor(&mut self) -> Result<(), ParseError> {
        let anchor_location = self.location_at_last_char();
        let id = AnchorScanner::new(&mut self.chars, self.source, anchor_location).consume_id()?;
        let element_index = self.output.elements.len();
        self.output.pending_anchors.push(PendingAnchor::new(
            id.clone(),
            self.signal_index,
            element_index,
            anchor_location,
        ));
        self.output.elements.push(RawElement::Anchor(id));
        Ok(())
    }

    /// Push one bare (unquoted) text character as a `BareChar`.
    ///
    /// A bare text character before any level symbol is rejected with
    /// `MissingInitialLevel`.
    fn push_bare_text_char(&mut self, character: char) -> Result<(), ParseError> {
        if !self.seen_level {
            return Err(ParseError::with_length(
                self.location_at_last_char(),
                1,
                ParseErrorKind::MissingInitialLevel,
            ));
        }
        self.output.elements.push(RawElement::BareChar(character));
        Ok(())
    }

    /// Consume a `"..."` quoted text literal and push it as a `QuotedText`.
    ///
    /// The quote that triggered this call has already been consumed. Returns
    /// `UnclosedQuote` when the closing `"` is never found in `source` — in
    /// which case the error column points at the opening `"` so the caret
    /// renders directly under it.
    fn consume_quoted_text(&mut self, quote_byte_index: usize) -> Result<(), ParseError> {
        if !self.seen_level {
            return Err(ParseError::new(
                self.location_at_last_char(),
                ParseErrorKind::MissingInitialLevel,
            ));
        }
        // Column of the opening `"` for the unclosed-quote error case.
        let quote_location = SourceLocation::new(
            self.location.line(),
            self.column_at_source_byte(quote_byte_index),
        );
        let text = collect_quoted_literal(&mut self.chars, quote_location)?;
        self.output.elements.push(RawElement::QuotedText(text));
        Ok(())
    }
}

/// Consume characters from `chars` until an unescaped `"` is found.
///
/// The opening `"` has already been consumed by the caller. Returns the
/// unescaped content between the quotes, or `UnclosedQuote` when the iterator
/// runs out first.
///
/// Escape rules inside `"..."`: `\"` → `"`, `\\` → `\`, `\n` → newline.
/// Unknown `\X` sequences are preserved verbatim (`\` + `X`).
fn collect_quoted_literal(
    chars: &mut Peekable<CharIndices<'_>>,
    location: SourceLocation,
) -> Result<String, ParseError> {
    let mut text = String::new();
    loop {
        match chars.next() {
            None => {
                return Err(ParseError::new(location, ParseErrorKind::UnclosedQuote));
            }
            Some((_, '"')) => return Ok(text),
            Some((_, '\\')) => match chars.next() {
                Some((_, '"')) => text.push('"'),
                Some((_, '\\')) => text.push('\\'),
                Some((_, 'n')) => text.push('\n'),
                Some((_, other)) => {
                    text.push('\\');
                    text.push(other);
                }
                None => text.push('\\'),
            },
            Some((_, character)) => text.push(character),
        }
    }
}

// =============================================================================
// Second pass: don't-care resolution + merge
// =============================================================================

/// Token list after `?` resolution. `BusCross` is preserved separately so
/// the transition pass can choose `from`/`to`.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum ResolvedToken {
    /// One level run (after `?` resolution), with text fragments attached.
    Level(LevelRun, Vec<String>),
    /// `X` marker that will produce a `Transition(BusCross)`.
    BusCross,
    /// A waveform element that does not interact with transition synthesis.
    Element(WaveformElement),
}

/// Walk every raw token left-to-right and resolve every `?` placeholder by
/// inheriting the most recent prior level's shape. Then expand each resolved
/// DontCare run to absorb adjacent same-kind level runs in both directions
/// (stopped by a `BusCross`, `Gap`, a different level, or the waveform boundary).
fn resolve_dontcare_inheritance(
    raw: &[RawElement],
    location: SourceLocation,
) -> Result<Vec<ResolvedToken>, ParseError> {
    let mut resolver = DontcareResolver::with_capacity(raw.len());
    for element in raw {
        resolver.consume(element, location)?;
    }
    let mut resolved = resolver.finish();
    expand_dontcare_ranges(&mut resolved);
    Ok(resolved)
}

/// State machine carrying the partially-resolved token list and the buffer of
/// pending text fragments that have not yet been attached to a level run.
///
/// Holding both fields on one struct lets the dispatch arms in
/// [`Self::consume`] read like an inlined `match` while still expressing the
/// text-flush and bare-char-append operations as `&mut self` methods, instead
/// of free functions that take two `&mut` arguments.
struct DontcareResolver {
    resolved: Vec<ResolvedToken>,
    pending_text: Vec<String>,
    /// `true` when the last raw token consumed was a `QuotedText`.
    /// Consulted by the next [`RawElement::BareChar`] to decide whether the
    /// character continues the current fragment or starts a fresh one.
    after_quoted: bool,
}

impl DontcareResolver {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            resolved: Vec::with_capacity(capacity),
            pending_text: Vec::new(),
            after_quoted: false,
        }
    }

    /// Drain any trailing pending text onto the last level and yield the
    /// resolved-token list.
    fn finish(mut self) -> Vec<ResolvedToken> {
        self.flush_pending_text_backward();
        self.resolved
    }

    /// Dispatch on the raw element kind. Inlined `match` directly here — the
    /// per-arm logic is the only call site for each branch and adding a
    /// wrapper would be a pointless single-dispatch indirection.
    fn consume(
        &mut self,
        element: &RawElement,
        location: SourceLocation,
    ) -> Result<(), ParseError> {
        match element {
            RawElement::BareChar(character) => self.append_bare_char(*character),
            RawElement::QuotedText(fragment) => {
                self.pending_text.push(fragment.clone());
                self.after_quoted = true;
                return Ok(());
            }
            RawElement::LevelRun(run) => self.push_resolved_level(run, location)?,
            RawElement::BusCross => self.push_simple(ResolvedToken::BusCross),
            RawElement::Gap => self.push_element(WaveformElement::Gap),
            RawElement::Guide => self.push_element(WaveformElement::Guide),
            RawElement::HighlightStart => self.push_element(WaveformElement::HighlightStart),
            RawElement::HighlightEnd => self.push_element(WaveformElement::HighlightEnd),
            RawElement::Anchor(id) => self.push_element(WaveformElement::Anchor(id.clone())),
        }
        self.after_quoted = false;
        Ok(())
    }

    fn push_element(&mut self, element: WaveformElement) {
        self.push_simple(ResolvedToken::Element(element));
    }

    fn push_simple(&mut self, token: ResolvedToken) {
        self.flush_pending_text_backward();
        self.resolved.push(token);
    }

    /// Resolve and push one `LevelRun` raw element as a `ResolvedToken::Level`.
    ///
    /// Spec rule 4: text immediately after a `BusCross` belongs to the
    /// destination level, not the preceding level. When the last resolved
    /// token is a `BusCross`, pending text is consumed and attached to the
    /// new level; otherwise it is flushed onto the previous level first.
    ///
    /// When `run.level` is `None` (a pending `?`), the
    /// `DontCareWithoutAnchor` error column is taken from
    /// `run.first_char_column` so the caret lands directly under the `?`
    /// rather than at the start of the level string.
    fn push_resolved_level(
        &mut self,
        run: &RawLevel,
        location: SourceLocation,
    ) -> Result<(), ParseError> {
        let after_cross = matches!(self.resolved.last(), Some(ResolvedToken::BusCross));
        let fragments = if after_cross {
            std::mem::take(&mut self.pending_text)
        } else {
            self.flush_pending_text_backward();
            Vec::new()
        };
        let level = match run.level {
            Some(value) => value,
            None => inherit_dontcare_shape(&self.resolved, run.dontcare_location(location))?,
        };
        self.resolved.push(ResolvedToken::Level(
            LevelRun::new(level, run.units),
            fragments,
        ));
        Ok(())
    }

    /// Append a bare character to the pending-text buffer.
    ///
    /// Consecutive bare characters concatenate into one fragment so that
    /// `ack` (three `BareChar` tokens) becomes the single fragment `"ack"`.
    /// After a quoted literal, a fresh fragment is started instead of
    /// appending to the existing last fragment.
    fn append_bare_char(&mut self, character: char) {
        if !self.after_quoted
            && let Some(last) = self.pending_text.last_mut()
        {
            last.push(character);
            return;
        }
        self.pending_text.push(character.to_string());
    }

    /// Flush pending text onto the last `Level` token in `resolved`, walking
    /// backwards. Scanning stops at the first `BusCross` (text after a cross
    /// must not be attached to a level that precedes it). When there is no
    /// preceding `Level` the text is discarded — the tokenizer already
    /// emitted `MissingInitialLevel` for the leading-bare-text case.
    fn flush_pending_text_backward(&mut self) {
        if self.pending_text.is_empty() {
            return;
        }
        for token in self.resolved.iter_mut().rev() {
            match token {
                ResolvedToken::Level(_, fragments) => {
                    fragments.append(&mut self.pending_text);
                    return;
                }
                ResolvedToken::BusCross => break,
                ResolvedToken::Element(_) => {}
            }
        }
        self.pending_text.clear();
    }
}

/// For every DontCare level run, walk backward and forward absorbing adjacent
/// level runs that share the same underlying level kind into the DontCare shape.
///
/// Transparent elements (`Anchor`, `Guide`, `HighlightStart`, `HighlightEnd`)
/// are skipped without breaking the expansion. `BusCross`, `Gap`, a different
/// level, and the waveform boundary all terminate expansion in that direction.
fn expand_dontcare_ranges(tokens: &mut [ResolvedToken]) {
    let length = tokens.len();
    for index in 0..length {
        let dontcare_level = match &tokens[index] {
            ResolvedToken::Level(run, _) if run.level().is_dontcare() => run.level(),
            _ => continue,
        };
        absorb_same_kind_levels_backward(tokens, index, dontcare_level);
        absorb_same_kind_levels_forward(tokens, index, dontcare_level);
    }
}

/// Walk from `start_index - 1` downward, converting same-kind level runs to
/// `dontcare_level`. Transparent tokens are skipped. Any other token stops the walk.
/// Text fragments on absorbed runs are preserved so that labels survive the conversion.
fn absorb_same_kind_levels_backward(
    tokens: &mut [ResolvedToken],
    start_index: usize,
    dontcare_level: SignalLevel,
) {
    let mut index = start_index;
    loop {
        if index == 0 {
            break;
        }
        index -= 1;
        if !try_absorb_same_kind_at(tokens, index, dontcare_level) {
            break;
        }
    }
}

/// Walk from `start_index + 1` upward, converting same-kind level runs to
/// `dontcare_level`. Transparent tokens are skipped. Any other token stops the walk.
/// Text fragments on absorbed runs are preserved so that labels survive the conversion.
fn absorb_same_kind_levels_forward(
    tokens: &mut [ResolvedToken],
    start_index: usize,
    dontcare_level: SignalLevel,
) {
    let length = tokens.len();
    let mut index = start_index;
    loop {
        index += 1;
        if index >= length {
            break;
        }
        if !try_absorb_same_kind_at(tokens, index, dontcare_level) {
            break;
        }
    }
}

/// Try to convert `tokens[index]` into a `dontcare_level` Level (absorbing
/// it). Returns `true` to continue walking, `false` to stop. Transparent
/// elements are passed through (`true`); same-kind levels are rewritten to
/// `dontcare_level` while preserving fragments (`true`); every other shape
/// halts the walk (`false`).
fn try_absorb_same_kind_at(
    tokens: &mut [ResolvedToken],
    index: usize,
    dontcare_level: SignalLevel,
) -> bool {
    match &tokens[index] {
        ResolvedToken::Element(
            WaveformElement::Anchor(_)
            | WaveformElement::Guide
            | WaveformElement::HighlightStart
            | WaveformElement::HighlightEnd,
        ) => true,
        ResolvedToken::Level(run, _) if run.level().into_dontcare_along() == dontcare_level => {
            let units = run.units();
            let fragments = match std::mem::replace(&mut tokens[index], ResolvedToken::BusCross) {
                ResolvedToken::Level(_, frags) => frags,
                other => {
                    tokens[index] = other;
                    Vec::new()
                }
            };
            tokens[index] = ResolvedToken::Level(LevelRun::new(dontcare_level, units), fragments);
            true
        }
        _ => false,
    }
}

/// Resolve a `?` token by inheriting the most recent prior level's shape.
/// Errors when there is no prior level to inherit from.
fn inherit_dontcare_shape(
    accumulated: &[ResolvedToken],
    location: SourceLocation,
) -> Result<SignalLevel, ParseError> {
    for token in accumulated.iter().rev() {
        match token {
            ResolvedToken::Level(run, _) => return Ok(run.level().into_dontcare_along()),
            ResolvedToken::BusCross => return Ok(SignalLevel::DontCareAlongBus),
            ResolvedToken::Element(_) => {}
        }
    }
    Err(ParseError::with_length(
        location,
        1,
        ParseErrorKind::DontCareWithoutAnchor,
    ))
}

/// Coalesce neighbouring `Level` tokens that share the same level.
///
/// Adjacent text fragments are joined with a space by [`TransitionEmitter`]
/// when the final `WaveformElement::Text` is emitted; this pass only
/// concatenates the per-run unit counts and fragment lists.
fn merge_adjacent_levels(tokens: Vec<ResolvedToken>) -> Vec<ResolvedToken> {
    let mut accumulator: Vec<ResolvedToken> = Vec::with_capacity(tokens.len());
    for token in tokens {
        let ResolvedToken::Level(new_run, mut new_frags) = token else {
            accumulator.push(token);
            continue;
        };
        if let Some(ResolvedToken::Level(prev_run, prev_frags)) = accumulator.last_mut()
            && prev_run.level() == new_run.level()
        {
            prev_run.extend_units(new_run.units());
            prev_frags.append(&mut new_frags);
            continue;
        }
        accumulator.push(ResolvedToken::Level(new_run, new_frags));
    }
    accumulator
}

/// Correct the `element_index` stored in each `PendingAnchor` to match the
/// anchor's actual position in `final_elements`.
///
/// `TransitionEmitter` injects `Transition` elements between level runs, which
/// shifts the indices of anchors that follow a transition boundary. The
/// tokenizer recorded provisional indices based on the raw element list; this
/// function replaces those with the true indices from the final waveform.
fn reindex_anchors(final_elements: &[WaveformElement], pending_anchors: &mut [PendingAnchor]) {
    for (index, element) in final_elements.iter().enumerate() {
        let WaveformElement::Anchor(id) = element else {
            continue;
        };
        for anchor in pending_anchors.iter_mut() {
            if anchor.matches_id(id) {
                anchor.set_element_index(index);
                break;
            }
        }
    }
}
