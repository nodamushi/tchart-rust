//! Parser pipeline and shared mutable state.
//!
//! # Reading order
//!
//! Skim [`Parser::parse_input`] and [`Parser::finish_into_document`] first —
//! they show the full pipeline:
//!
//! 1. [`Parser::scan_all_lines`] walks each source line and dispatches.
//! 2. [`Parser::expand_clock_signals`] expands clock-decorated rows in place.
//! 3. [`Parser::finish_into_document`] performs the anchor-resolution
//!    post-pass and builds the final [`ChartDocument`].
//!
//! Submodules contribute parsers for sub-syntaxes (`arrow`, `clock`,
//! `waveform`, ...) but never mutate `Parser` directly: they return values
//! that the methods on this file write back through `&mut self`. That way
//! every state mutation is visible in this file (Tell, don't ask).

use super::attr;
use super::clock::{ClockSpecParser, LocalMarkOptions, split_clock_args_and_inline};
use super::directive::Directive;
use super::text_quote::{QuotedToken, strip_inline_comment};
use super::waveform::{WaveformParseResult, WaveformParser};
use crate::anchor::{AnchorId, AnchorRegistry, ResolvedAnchor};
use crate::arrow::{Arrow, ArrowEnd, ArrowStyle};
use crate::clock::{ClockMarkStyle, ClockSpec};
use crate::color::Color;
use crate::defaults::{
    DEFAULT_CLOCKMARK_HEIGHT_PX, DEFAULT_CLOCKMARK_POSITION, DEFAULT_CLOCKMARK_WIDTH_PX,
    DEFAULT_TITLE_ALIGN,
};
use crate::document::{Annotations, ChartDocument, TcmlSource, TextOverlay};
use crate::errors::{ParseError, ParseErrorKind, SourceLocation};
use crate::geometry::Point;
use crate::line::RulerContribution;
use crate::line::{
    Line, LineContent, SignalDecorations, SignalGeometry, SignalRow, SkipRow, TitleRow, Waveform,
};
use crate::style::{ChartStyle, HorizontalAlign, SignalRowStyle, TitleStyle};
use crate::text::FontSpec;
use crate::text::{SignalName, UserText};
use crate::units::{Length, Px};

// =============================================================================
// Public-facing types (used outside parser/)
// =============================================================================

/// Pending anchor declaration encountered during the first parsing pass.
///
/// Re-exported through `crate::parser` for [`AnchorRegistry::build`].
#[derive(Debug, Clone)]
pub(crate) struct PendingAnchor {
    id: AnchorId,
    signal_index: usize,
    element_index: usize,
    location: SourceLocation,
}

impl PendingAnchor {
    /// Construct a pending-anchor record.
    pub(super) fn new(
        id: AnchorId,
        signal_index: usize,
        element_index: usize,
        location: SourceLocation,
    ) -> Self {
        Self {
            id,
            signal_index,
            element_index,
            location,
        }
    }

    /// Update the element index to the actual position in the final `Waveform`.
    ///
    /// Called by `WaveformParser` after `TransitionEmitter` has run, because
    /// transition injection shifts anchor indices.
    pub(super) fn set_element_index(&mut self, index: usize) {
        self.element_index = index;
    }

    /// Returns `true` when this pending anchor has the given id.
    ///
    /// Used by `reindex_anchors` to match anchors in the final waveform
    /// element list to the corresponding `PendingAnchor` record.
    pub(super) fn matches_id(&self, id: &AnchorId) -> bool {
        &self.id == id
    }

    /// Insert this anchor into `registry`. Tell-don't-ask: the registry no
    /// longer has to ask for the id, look it up, and then ask the
    /// `PendingAnchor` to manufacture an error on duplicate.
    pub(crate) fn register_into(&self, registry: &mut AnchorRegistry) -> Result<(), ParseError> {
        let resolved = ResolvedAnchor::new(Point::ZERO, self.signal_index, self.element_index);
        registry
            .try_insert_unique(self.id.clone(), resolved)
            .map_err(|()| {
                ParseError::with_length(
                    self.location,
                    anchor_token_length(&self.id),
                    ParseErrorKind::DuplicateAnchor,
                )
            })
    }
}

/// Number of source characters in the textual form of an anchor token.
/// `@{name}` → 3 + name length. `@N` → 1 + digit count.
fn anchor_token_length(id: &AnchorId) -> u32 {
    let inner = match id {
        AnchorId::Named(name) => name.char_count() + 2, // {}
        // `u32::MAX` is 10 decimal digits, so `ilog10()+1` (with 0 handled
        // explicitly) computes the digit count without allocating a `String`.
        AnchorId::Indexed(0) => 1,
        AnchorId::Indexed(value) => value.ilog10() as usize + 1,
    };
    u32::try_from(inner + 1).unwrap_or(u32::MAX) // leading '@'
}

// =============================================================================
// Parser-internal pending state
// =============================================================================

/// Pending arrow declaration awaiting anchor resolution.
#[derive(Debug, Clone)]
pub(super) struct PendingArrow {
    from: ArrowEnd,
    to: ArrowEnd,
    style: ArrowStyle,
    label: Option<UserText>,
    /// Font active at the `@->` declaration site — carried into [`Arrow`] for
    /// label rendering without a second pass.
    label_font: FontSpec,
    /// Source location of the `from` endpoint within `@->(...)` — used by
    /// [`Self::validate_endpoint`] so undefined-anchor carets sit on the
    /// specific endpoint rather than on the directive head.
    from_location: SourceLocation,
    /// Source location of the `to` endpoint within `@->(...)`.
    to_location: SourceLocation,
}

impl PendingArrow {
    /// Construct a pending-arrow record.
    pub(super) fn new(
        from: ArrowEnd,
        to: ArrowEnd,
        style: ArrowStyle,
        label: Option<UserText>,
        label_font: FontSpec,
        from_location: SourceLocation,
        to_location: SourceLocation,
    ) -> Self {
        Self {
            from,
            to,
            style,
            label,
            label_font,
            from_location,
            to_location,
        }
    }

    /// Validate both endpoints against `registry` and produce the resolved
    /// [`Arrow`]. Consumes `self` so internal fields move into the result —
    /// no clones, and the `Copy` style passes by value.
    fn into_arrow(self, registry: &AnchorRegistry) -> Result<Arrow, ParseError> {
        Self::validate_endpoint(&self.from, registry, self.from_location)?;
        Self::validate_endpoint(&self.to, registry, self.to_location)?;
        Ok(Arrow::new(
            self.from,
            self.to,
            self.style,
            self.label,
            self.label_font,
        ))
    }

    fn validate_endpoint(
        endpoint: &ArrowEnd,
        registry: &AnchorRegistry,
        location: SourceLocation,
    ) -> Result<(), ParseError> {
        if let ArrowEnd::Anchor(id) = endpoint
            && !registry.contains(id)
        {
            let display = anchor_id_display(id);
            let length = u32::try_from(display.chars().count()).unwrap_or(u32::MAX);
            return Err(ParseError::with_length(
                location,
                length,
                ParseErrorKind::UndefinedAnchor(display),
            ));
        }
        Ok(())
    }
}

/// Render the textual form of an anchor id (`@{name}` or `@N`) for use in
/// error messages and underline width calculation.
fn anchor_id_display(id: &AnchorId) -> String {
    match id {
        AnchorId::Named(name) => format!("@{{{}}}", name.as_str()),
        AnchorId::Indexed(value) => format!("@{value}"),
    }
}

/// Decorations and background that the next row should pick up.
///
/// Whenever a signal/title/skip row is appended, the per-row pickers
/// ([`take_signal_decorations`](PendingNextRow::take_signal_decorations) and
/// [`take_background`](PendingNextRow::take_background)) drain this state so
/// each row only consumes the directives that immediately preceded it.
#[derive(Debug, Clone, Default)]
struct PendingNextRow {
    /// Pending `@clock(...)` and `@signal(overline)` for the next signal row.
    signal_directive: PendingSignalDirective,
    /// Pending `@bg <color|none>` for the next row of any kind.
    background: Option<Color>,
}

impl PendingNextRow {
    /// Drain accumulated `@clock` / `@signal` directives so they apply to a
    /// single signal row only.
    fn take_signal_decorations(&mut self) -> PendingSignalDirective {
        std::mem::take(&mut self.signal_directive)
    }

    /// Drain the pending `@bg` so it applies to a single row only.
    fn take_background(&mut self) -> Option<Color> {
        self.background.take()
    }

    /// Mark the next signal row as having an `overline` decoration.
    fn mark_overline(&mut self) {
        self.signal_directive.overline = true;
    }

    /// Set the pending `@clock(...)` spec for the next signal row.
    fn set_clock(&mut self, spec: ClockSpec) {
        self.signal_directive.clock = Some(spec);
    }

    /// Replace the pending `@bg` color (`None` clears it).
    fn set_background(&mut self, color: Option<Color>) {
        self.background = color;
    }
}

/// Decoration carry-over between successive directives and their following
/// signal row (e.g. `@clock(pos)` then a clock-named signal).
#[derive(Debug, Clone, Default)]
struct PendingSignalDirective {
    clock: Option<ClockSpec>,
    overline: bool,
}

/// Global chart-wide overrides set by `@titlealign` / `@clockmark_*`.
///
/// Bundled together so [`Parser`] holds one named field for "global
/// directive defaults" rather than scattering them as siblings of the
/// per-row pending state.
#[derive(Debug, Clone, Default)]
struct GlobalChartOverrides {
    title_align: Option<HorizontalAlign>,
    clockmark: GlobalClockmarkStyle,
}

/// Global clockmark style overrides set by `@clockmark_*` directives.
#[derive(Debug, Clone, Default)]
struct GlobalClockmarkStyle {
    position: Option<f32>,
    height: Option<Px>,
    width: Option<Px>,
    color: Option<Color>,
}

// =============================================================================
// Logical line classification
// =============================================================================

/// What [`Parser::scan_one_logical_line`] decided to dispatch on. Lets the
/// dispatch be a single `match` inside [`Parser::scan_one_logical_line`].
enum LineKind<'source> {
    /// Blank or pure-comment (`//`) line — skip.
    Skip,
    /// `@<rest>` directive. `rest` is the body after the leading `@`.
    Directive(&'source str),
    /// `% x y text` overlay. `rest` is the body after the leading `%`.
    Overlay(&'source str),
    /// `"…"`-quoted signal-name line.
    QuotedSignal,
    /// Plain `name levels` signal line. The slice is the trimmed full line.
    PlainSignal(&'source str),
}

impl<'source> LineKind<'source> {
    /// Classify a source line. `raw` is the original line text; when the
    /// line does not start with `"` (the quoted-signal path, which may span
    /// multiple lines), an inline `//` comment is stripped before
    /// classification so directives, overlays, and signal rows all see the
    /// pre-comment payload only.
    fn classify(raw: &'source str) -> Self {
        if raw.trim_start().starts_with('"') {
            return Self::QuotedSignal;
        }
        let after_comment = strip_inline_comment(raw);
        let trimmed = after_comment.trim();
        if trimmed.is_empty() {
            return Self::Skip;
        }
        if let Some(rest) = trimmed.strip_prefix('@') {
            return Self::Directive(rest);
        }
        if let Some(rest) = trimmed.strip_prefix('%') {
            return Self::Overlay(rest);
        }
        Self::PlainSignal(trimmed)
    }
}

// =============================================================================
// Parser
// =============================================================================

/// `@ruler` parser state. Tracks the on/off flag and the color that the
/// next signal / `@skip` row commit will snapshot into its sidecar
/// contributions.
///
/// See `docs/spec/tcml-format.md` §「`@ruler` の詳細」.
#[derive(Debug, Clone)]
struct RulerState {
    /// `true` once `@ruler on` has been seen and not yet overridden by
    /// `@ruler off`. Initial value `true` matches the default.
    on: bool,
    /// Color currently in effect. Mutated by `@ruler_color`; read at row
    /// commit time and snapshotted into the row's contributions.
    color: Color,
}

impl Default for RulerState {
    fn default() -> Self {
        Self {
            on: true,
            color: Color::RULER_DEFAULT,
        }
    }
}

impl RulerState {
    /// Build the contribution vector this state would attach to a row
    /// committed *right now* given the row's effective `step` and
    /// `units` count. Empty when the state is `off`.
    fn donations(&self, step: Px, units: u32) -> Vec<RulerContribution> {
        if !self.on {
            return Vec::new();
        }
        RulerContribution::donations(step, units, self.color).collect()
    }
}

/// All mutable parser state.
#[derive(Debug, Clone, Default)]
pub(super) struct Parser {
    style: ChartStyle,
    lines: Vec<Line>,
    overlays: Vec<TextOverlay>,
    pending_anchors: Vec<PendingAnchor>,
    pending_arrows: Vec<PendingArrow>,
    pending_next_row: PendingNextRow,
    overrides: GlobalChartOverrides,
    ruler: RulerState,
}

impl Parser {
    // ------------------------------------------------------------------
    // Pipeline entry points
    // ------------------------------------------------------------------

    /// Run the line-by-line pass and the in-place clock expansion. Anchor
    /// resolution is deferred to [`finish_into_document`](Self::finish_into_document)
    /// so the caller can build the [`AnchorRegistry`] in between.
    ///
    /// A leading UTF-8 BOM (U+FEFF), if present, is silently skipped before
    /// line scanning per `docs/spec/tcml-format.md` §「ファイル先頭の BOM」.
    pub(super) fn parse_input(input: &str) -> Result<Self, ParseError> {
        let stripped = input.strip_prefix('\u{FEFF}').unwrap_or(input);
        let mut parser = Self::default();
        parser.scan_all_lines(stripped)?;
        parser.expand_clock_signals();
        Ok(parser)
    }

    /// Build the final document by running the anchor-resolution post-pass.
    ///
    /// This consumes the parser, builds the [`AnchorRegistry`] from all
    /// pending anchors, resolves every pending arrow against it, and packages
    /// the result with the original `source` text.
    pub(super) fn finish_into_document(
        mut self,
        source: &str,
    ) -> Result<ChartDocument, ParseError> {
        let registry = AnchorRegistry::build(&self.pending_anchors)?;
        let arrows: Vec<Arrow> = self
            .pending_arrows
            .drain(..)
            .map(|pending| pending.into_arrow(&registry))
            .collect::<Result<_, _>>()?;
        Ok(ChartDocument::new(
            self.style,
            self.lines,
            Annotations::new(self.overlays, arrows, registry),
            TcmlSource::new(source),
        ))
    }

    // ------------------------------------------------------------------
    // Stage 1: per-line scan
    // ------------------------------------------------------------------

    /// Walk every source line in `input`, dispatching each to the matching
    /// handler. A "logical line" may span multiple source lines (multi-line
    /// `"..."` quoting), so the dispatcher returns the number of source lines
    /// it consumed.
    fn scan_all_lines(&mut self, input: &str) -> Result<(), ParseError> {
        // `text_quote::QuotedToken::collect` needs random access to following
        // lines so a `Vec<&str>` is the right shape; the `enumerate`-style
        // index maintained here is the only state we need.
        let lines: Vec<&str> = input.lines().collect();
        let mut index = 0;
        while let Some(raw) = lines.get(index) {
            let consumed = self.scan_one_logical_line(raw, &lines, index)?;
            index += consumed.max(1);
        }
        Ok(())
    }

    /// Classify the line at `index` by its first non-whitespace character and
    /// route to the correct handler. Returns the number of source lines the
    /// handler consumed (always `>= 1`).
    ///
    /// `SourceLocation.column` is the **character column** (1-based, Unicode
    /// scalar values) of the first non-whitespace character of `raw` — i.e.
    /// the leading `@`, `%`, `"`, or signal-name start. Per-token columns
    /// inside the line are computed by deeper parsers from this base plus the
    /// char-offset of the token within `raw`. Tab expansion to display
    /// columns is the CLI's responsibility; core stores raw character columns.
    fn scan_one_logical_line(
        &mut self,
        raw: &str,
        lines: &[&str],
        index: usize,
    ) -> Result<usize, ParseError> {
        let line_number = line_number_for_index(index);
        let leading_chars = count_leading_whitespace_chars(raw);
        let base_column = 1 + leading_chars;
        let location = SourceLocation::new(line_number, base_column);
        match LineKind::classify(raw) {
            LineKind::Skip => Ok(1),
            LineKind::Directive(rest) => {
                self.parse_directive_line(rest, location, raw, lines, index)
            }
            LineKind::Overlay(rest) => {
                self.parse_overlay_line(rest, location, raw)?;
                Ok(1)
            }
            LineKind::QuotedSignal => self.parse_quoted_signal_line(lines, index),
            LineKind::PlainSignal(line) => {
                self.parse_unquoted_signal_line(line, location, raw)?;
                Ok(1)
            }
        }
    }

    // ------------------------------------------------------------------
    // Stage 1a: `@` directive lines
    // ------------------------------------------------------------------

    /// Dispatch an `@<name>...` line.
    ///
    /// Two shapes are recognised:
    /// 1. `@-> ...` arrow declaration (handled in-place).
    /// 2. `@<name> args...` directive — line-shaped ones (`title`, `skip`,
    ///    `clock`, `signal`) are dispatched to dedicated parsers; everything
    ///    else parses through the [`Directive`] enum.
    ///
    /// `raw_line` is the original (un-trimmed) source line. The directive
    /// argument column is computed by char-counting `rest`'s slice inside
    /// `raw_line` so that parse errors point at the offending token, not at
    /// the line-leading `@`.
    fn parse_directive_line(
        &mut self,
        rest: &str,
        location: SourceLocation,
        raw_line: &str,
        lines: &[&str],
        index: usize,
    ) -> Result<usize, ParseError> {
        if let Some(arg) = rest.strip_prefix("->") {
            let (arrow, consumed) = PendingArrow::parse(arg, location, &self.style, lines, index)?;
            self.pending_arrows.push(arrow);
            return Ok(consumed);
        }
        let (head, args) = split_directive_head(rest);
        let argument_location = location_at_substring_or_default(
            raw_line,
            args,
            location,
            line_number_for_index(index),
        );
        match head {
            "title" => self.parse_title_directive(args, argument_location, lines, index),
            "skip" => self
                .parse_skip_directive(args, argument_location)
                .map(|()| 1),
            "clock" => self
                .parse_clock_directive(args, argument_location)
                .map(|()| 1),
            "signal" => self.parse_signal_directive(args, argument_location, lines, index),
            "overline" => self
                .parse_overline_directive(args, argument_location)
                .map(|()| 1),
            _ => {
                let directive = Directive::parse(head, args, argument_location, location)?;
                self.apply_directive(directive)?;
                Ok(1)
            }
        }
    }

    /// Parse the `@overline` directive, the short alias for `@signal(overline)`
    /// defined in `docs/spec/tcml-format.md` §「@overline (alias)」. The alias
    /// takes no argument: any non-empty tail is rejected so that typos like
    /// `@overline foo` do not silently apply the decoration.
    ///
    /// Only the pending-row state is updated here; the actual signal row is
    /// committed later by [`Self::parse_waveform_and_push_row`], which runs the
    /// `step`/`slant` invariant check at row-commit time. This method does not
    /// read `step` or `slant`, so checking the invariant here would be
    /// redundant work; worse, it would also reject otherwise-valid intermediate
    /// states such as `@step 1` followed by `@slant 0` where the two fields
    /// mutate one at a time and only the final pair needs to satisfy
    /// `step <= slant`.
    fn parse_overline_directive(
        &mut self,
        args: &str,
        location: SourceLocation,
    ) -> Result<(), ParseError> {
        let trimmed = args.trim();
        if !trimmed.is_empty() {
            // Point at the offending tail rather than at the directive head
            // so the caret falls on the unexpected text.
            let leading_ws_bytes = args.len() - args.trim_start().len();
            let col_offset = args[..leading_ws_bytes].chars().count() as u32;
            let tail_location =
                SourceLocation::new(location.line(), location.column() + col_offset);
            let tail_length = u32::try_from(trimmed.chars().count()).unwrap_or(u32::MAX);
            return Err(ParseError::with_length(
                tail_location,
                tail_length,
                ParseErrorKind::InvalidOverlineSyntax(trimmed.to_owned()),
            ));
        }
        self.pending_next_row.mark_overline();
        Ok(())
    }

    /// Apply a value-only [`Directive`] to `self`.
    ///
    /// The `step <= slant` invariant is intentionally NOT checked here: pairs
    /// like `@step 1` followed by `@slant 0` mutate the two fields one at a
    /// time, and an immediate check would reject the intermediate state
    /// (`step=1`, `slant` still default 5) even though the final state is
    /// valid. The check is deferred to [`check_step_slant_invariant`], which
    /// runs just before any signal-related row is committed (see
    /// [`parse_waveform_and_push_row`] and the `@clock` / `@signal` directive
    /// parsers).
    /// Dispatch is laid out table-style via the [`dispatch_directive!`] macro
    /// so each variant stays on its own line and the mapping remains readable
    /// at a glance even as variants are added.
    fn apply_directive(&mut self, directive: Directive) -> Result<(), ParseError> {
        dispatch_directive!(self, directive);
        Ok(())
    }

    /// Returns `Err(InvalidStepSlant)` when `slant >= step` for the current
    /// layout params.
    ///
    /// Per `docs/spec/tcml-format.md` §`@slant`, the level hold portion
    /// (`step - slant`) must be positive, so `step <= slant` is rejected.
    /// Called from row-commit paths (`parse_waveform_and_push_row`,
    /// `parse_clock_directive`, `parse_signal_directive`) where the layout
    /// values are about to be observed, so each signal row is guaranteed to be
    /// laid out with a consistent state without rejecting intermediate inputs
    /// like `@step 1` before its paired `@slant 0`.
    fn check_step_slant_invariant(&self, location: SourceLocation) -> Result<(), ParseError> {
        let layout = self.style.layout();
        if layout.step() <= layout.slant() {
            return Err(ParseError::new(
                location,
                crate::errors::ParseErrorKind::InvalidStepSlant(
                    layout.step().to_f32(),
                    layout.slant().to_f32(),
                ),
            ));
        }
        Ok(())
    }

    // ------------------------------------------------------------------
    // Stage 1b: line-shaped directives
    // ------------------------------------------------------------------

    /// Parse `@title <text>` or `@title "multi\nline"` and append a
    /// title row. Returns the number of source lines consumed (1 for
    /// single-line, more for quoted multi-line).
    ///
    /// Per `docs/spec/tcml-format.md` §「@title」, an argument is required:
    /// `@title` alone (no argument at all) is a parse error. Callers that
    /// want a blank title row must write `@title ""` explicitly.
    fn parse_title_directive(
        &mut self,
        args: &str,
        location: SourceLocation,
        lines: &[&str],
        index: usize,
    ) -> Result<usize, ParseError> {
        let trimmed = args.trim();
        if trimmed.is_empty() {
            return Err(ParseError::new(
                location,
                ParseErrorKind::TitleRequiresArgument,
            ));
        }
        // Track where the *content* of `text` begins in the source line. For
        // a bare `@title abc` form that is the directive-argument location.
        // For a quoted form `@title "abc..."`, content begins one column
        // after the opening `"`. The distinction matters so per-char carets
        // (e.g. forbidden control char inside the title) land on the
        // offending char rather than on the `"`.
        let (text, text_location, consumed) = match trimmed.strip_prefix('"') {
            Some(after_open) => {
                let token = QuotedToken::collect(lines, index, after_open, location)?;
                let text_loc =
                    SourceLocation::new(location.line(), location.column().saturating_add(1));
                (token.text, text_loc, token.consumed_lines)
            }
            None => (trimmed.to_owned(), location, 1),
        };
        let user_text = UserText::parse(&text).map_err(|error| {
            let (col_off, len) = caret_for_inner(error.char_offset(), &text);
            ParseError::with_length(
                SourceLocation::new(text_location.line(), text_location.column() + col_off),
                len,
                ParseErrorKind::InvalidText(error),
            )
        })?;
        self.push_title_row(user_text);
        Ok(consumed)
    }

    /// Parse `@skip(...)` and append a skip row. A zero amount is silently
    /// ignored (no row is appended).
    fn parse_skip_directive(
        &mut self,
        args: &str,
        location: SourceLocation,
    ) -> Result<(), ParseError> {
        let inner = attr::strip_parens(args, location, ParseErrorKind::InvalidSkipSyntax)?;
        // Locate the trimmed amount inside `args` so per-token errors point at
        // the offending value rather than at `(`.
        let trimmed = inner.trim();
        let amount_location = locate_substring(args, trimmed, location);
        if let Some(length) = Length::parse_skip_amount(trimmed, amount_location)? {
            self.push_skip_row(length);
        }
        Ok(())
    }

    /// Parse `@clock(...)` attributes and stage them for the next signal row.
    ///
    /// Per the TCML spec (`docs/spec/tcml-format.md` §「@clock」), the argument
    /// list is optional. `@clock` with no parentheses behaves identically to
    /// `@clock()` and `@clock(none)` — edge defaults to `ClockEdge::None`.
    ///
    /// Two equivalent line shapes are accepted:
    /// 1. Two-line form — `@clock(...)` on its own line, then a signal row on
    ///    the next line carrying the clock name and (optionally) a partial
    ///    waveform body. This matches the examples in the spec.
    /// 2. Inline form — `@clock(...) <name> [<levels>]` on a single line.
    ///    The remainder after the closing `)` is parsed as a plain signal
    ///    line. This lets terse declarations like `@clock(pos) ck` fit on
    ///    one line.
    fn parse_clock_directive(
        &mut self,
        args: &str,
        location: SourceLocation,
    ) -> Result<(), ParseError> {
        self.check_step_slant_invariant(location)?;
        let (inner, inner_location, inline_rest) = split_clock_args_and_inline(args, location)?;
        let spec = ClockSpecParser::parse(inner, inner_location, self)?;
        self.pending_next_row.set_clock(spec);
        if !inline_rest.is_empty() {
            // Inline form does not have a separate `raw_line` available here;
            // treat the inline tail as its own line so column offsets remain
            // self-consistent within the inline row.
            self.parse_unquoted_signal_line(inline_rest, location, inline_rest)?;
        }
        Ok(())
    }

    /// Parse `@signal(...)` decorations (currently only `overline`) and stage
    /// them for the next signal row.
    ///
    /// Two equivalent line shapes are accepted, mirroring `@clock`:
    /// 1. Two-line form — `@signal(overline)` on its own line, then the target
    ///    signal row on the next line.
    /// 2. Inline form — `@signal(overline) <name> [<levels>]` on a single
    ///    line. The remainder after the closing `)` is parsed as a signal
    ///    line (quoted if it starts with `"`, otherwise plain). The signal
    ///    row receives the staged decoration.
    fn parse_signal_directive(
        &mut self,
        args: &str,
        location: SourceLocation,
        lines: &[&str],
        index: usize,
    ) -> Result<usize, ParseError> {
        self.check_step_slant_invariant(location)?;
        let (inner, inner_location, inline_rest) =
            split_signal_arguments_and_inline(args, location)?;
        self.stage_signal_attribute_tokens(inner, inner_location)?;
        if inline_rest.is_empty() {
            Ok(1)
        } else {
            self.parse_inline_signal_row(inline_rest, location, lines, index)
        }
    }

    /// Apply each comma-separated attribute token to `pending_next_row`.
    /// Empty tokens (`@signal( , overline )`) are tolerated; unknown or
    /// duplicate attributes raise [`ParseErrorKind::UnknownSignalAttribute`]
    /// pointing at the offending token (column + length of the trimmed
    /// segment).
    fn stage_signal_attribute_tokens(
        &mut self,
        inner: &str,
        inner_location: SourceLocation,
    ) -> Result<(), ParseError> {
        let mut seen_overline = false;
        let mut byte_pos = 0usize;
        for segment in inner.split(',') {
            let leading_ws_bytes =
                segment.len() - segment.trim_start_matches(ASCII_SPACE_OR_TAB).len();
            let trimmed = segment.trim();
            let token_byte_in_inner = byte_pos + leading_ws_bytes;
            let token_col_offset = inner[..token_byte_in_inner].chars().count() as u32;
            let token_location = SourceLocation::new(
                inner_location.line(),
                inner_location.column() + token_col_offset,
            );
            let token_length = u32::try_from(trimmed.chars().count()).unwrap_or(u32::MAX);
            match trimmed {
                "overline" => {
                    if seen_overline {
                        return Err(ParseError::with_length(
                            token_location,
                            token_length,
                            ParseErrorKind::UnknownSignalAttribute(trimmed.to_owned()),
                        ));
                    }
                    seen_overline = true;
                    self.pending_next_row.mark_overline();
                }
                "" => {}
                _ => {
                    return Err(ParseError::with_length(
                        token_location,
                        token_length,
                        ParseErrorKind::UnknownSignalAttribute(trimmed.to_owned()),
                    ));
                }
            }
            byte_pos += segment.len() + 1; // +1 for the ',' (no-op on last seg).
        }
        Ok(())
    }

    /// Parse the signal row attached to an inline `@signal(...) <row>` form,
    /// routing through the quoted-name scanner when the rest starts with `"`.
    fn parse_inline_signal_row(
        &mut self,
        inline_rest: &str,
        location: SourceLocation,
        lines: &[&str],
        index: usize,
    ) -> Result<usize, ParseError> {
        let Some(after_quote) = inline_rest.strip_prefix('"') else {
            // Inline form has no separate raw-line context; use the inline
            // tail itself so column offsets within it stay self-consistent.
            self.parse_unquoted_signal_line(inline_rest, location, inline_rest)?;
            return Ok(1);
        };
        let token = QuotedToken::collect(lines, index, after_quote, location)?;
        let name = SignalName::parse(&token.text).map_err(|error| {
            let (col_off, len) = caret_for_inner(error.char_offset(), &token.text);
            ParseError::with_length(
                SourceLocation::new(location.line(), location.column() + col_off),
                len,
                ParseErrorKind::InvalidName(error),
            )
        })?;
        let tail = strip_inline_comment(token.tail).trim();
        self.parse_waveform_and_push_row(name, tail, location)?;
        Ok(token.consumed_lines)
    }

    // ------------------------------------------------------------------
    // Stage 1c: `% x y text` overlay lines
    // ------------------------------------------------------------------

    /// Parse `% x y text` and append a text overlay.
    ///
    /// `raw_line` is the source line; `location` points at the leading `%`.
    /// The argument list is left in `rest` (the slice after `%`); when a
    /// coordinate fails, the column is upgraded to the first character of
    /// the offending coordinate token within `raw_line`.
    fn parse_overlay_line(
        &mut self,
        rest: &str,
        location: SourceLocation,
        raw_line: &str,
    ) -> Result<(), ParseError> {
        let trimmed = rest.trim();
        let mut parts = trimmed.splitn(3, char::is_whitespace);
        // Coordinate errors carry the column of the first coordinate token;
        // when no token is present, fall back to the `%` column.
        let coord_error_location = |token: &str| -> SourceLocation {
            location_at_substring_or_default(
                raw_line,
                token,
                location,
                line_number_for_index_from_location(location),
            )
        };
        let invalid_coord = |token: &str| {
            let length = u32::try_from(token.chars().count()).unwrap_or(u32::MAX);
            ParseError::with_length(
                coord_error_location(token),
                length,
                ParseErrorKind::InvalidOverlayCoordinate(token.to_owned()),
            )
        };
        let x_str = parts.next().ok_or_else(|| invalid_coord(""))?;
        let y_str = parts.next().ok_or_else(|| invalid_coord(""))?;
        // The text payload is optional; treat a missing third token as empty.
        let text_str = parts.next().unwrap_or("").trim();
        let x: f32 = x_str.parse().map_err(|_| invalid_coord(x_str))?;
        let y: f32 = y_str.parse().map_err(|_| invalid_coord(y_str))?;
        let text_location = locate_substring(raw_line, text_str, location);
        let text = UserText::parse(text_str).map_err(|error| {
            let (col_off, len) = caret_for_inner(error.char_offset(), text_str);
            ParseError::with_length(
                SourceLocation::new(text_location.line(), text_location.column() + col_off),
                len,
                ParseErrorKind::InvalidText(error),
            )
        })?;
        self.overlays
            .push(TextOverlay::new(Point::new_f32(x, y), text));
        Ok(())
    }

    // ------------------------------------------------------------------
    // Stage 1d: signal lines (waveform rows)
    // ------------------------------------------------------------------

    /// Parse a non-quoted signal line of the form `name <space> levels`.
    ///
    /// `raw_line` is the original (un-trimmed) source line; it is used to
    /// upgrade the levels column from "line start" to "first character of
    /// the levels run" so that waveform errors caret-align with the bad
    /// token.
    fn parse_unquoted_signal_line(
        &mut self,
        line: &str,
        location: SourceLocation,
        raw_line: &str,
    ) -> Result<(), ParseError> {
        let (raw_name, levels) = split_name_and_levels(line);
        let name_location = locate_substring(raw_line, raw_name, location);
        let name = SignalName::parse(raw_name).map_err(|error| {
            let (col_off, len) = caret_for_inner(error.char_offset(), raw_name);
            ParseError::with_length(
                SourceLocation::new(name_location.line(), name_location.column() + col_off),
                len,
                ParseErrorKind::InvalidName(error),
            )
        })?;
        let levels_location = location_at_substring_or_default(
            raw_line,
            levels,
            location,
            line_number_for_index_from_location(location),
        );
        self.parse_waveform_and_push_row(name, levels, levels_location)
    }

    /// Parse a `"..."`-quoted signal name (potentially multi-line) followed
    /// by levels on the same source line as the closing quote.
    fn parse_quoted_signal_line(
        &mut self,
        lines: &[&str],
        start: usize,
    ) -> Result<usize, ParseError> {
        let location = SourceLocation::new(line_number_for_index(start), 1);
        // Caller (`scan_one_logical_line`) only invokes this branch for a line
        // that started with `"`, so `lines[start]` is guaranteed in-range.
        // `unwrap_or` keeps the call total without `expect`.
        let first_line = lines.get(start).copied().unwrap_or("");
        let after_open = first_line
            .trim_start()
            .strip_prefix('"')
            .unwrap_or(first_line);
        let token = QuotedToken::collect(lines, start, after_open, location)?;
        let name = SignalName::parse(&token.text).map_err(|error| {
            let (col_off, len) = caret_for_inner(error.char_offset(), &token.text);
            ParseError::with_length(
                SourceLocation::new(location.line(), location.column() + col_off),
                len,
                ParseErrorKind::InvalidName(error),
            )
        })?;
        let tail = strip_inline_comment(token.tail).trim();
        self.parse_waveform_and_push_row(name, tail, location)?;
        Ok(token.consumed_lines)
    }

    /// Tokenise `levels` into a [`Waveform`] and append the resulting signal
    /// row, recording any anchors discovered along the way.
    fn parse_waveform_and_push_row(
        &mut self,
        name: SignalName,
        levels: &str,
        location: SourceLocation,
    ) -> Result<(), ParseError> {
        self.check_step_slant_invariant(location)?;
        let signal_index = self.lines.len();
        let WaveformParseResult {
            waveform,
            pending_anchors,
        } = WaveformParser::new(levels, location, signal_index).parse()?;
        self.pending_anchors.extend(pending_anchors);
        self.push_signal_row(name, waveform);
        Ok(())
    }

    // ------------------------------------------------------------------
    // Stage 2: post-pass — clock waveform expansion
    // ------------------------------------------------------------------

    /// Expand every `clock`-decorated row out to the per-row target unit count.
    ///
    /// Target units for each auto row are computed as:
    ///   `round(max_explicit_pixel_width / row.step)`
    ///
    /// where `max_explicit_pixel_width` is the maximum of `(units × step)` over
    /// all rows that are **not** empty-body clock auto rows (i.e. ordinary signal
    /// rows and partial clock rows with an existing body).
    ///
    /// When all rows are auto (no explicit rows), the maximum pixel width is 0
    /// and every auto row expands to 0 units (empty waveform).
    fn expand_clock_signals(&mut self) {
        let max_explicit_px = self.max_explicit_pixel_width();
        // Collect `(line_index, target_units)` pairs first so we do not hold
        // an immutable borrow on `self.lines` while mutating it below.
        let targets: Vec<(usize, u32)> = self
            .lines
            .iter()
            .enumerate()
            .filter_map(|(index, line)| {
                let LineContent::Signal(row) = &line.content else {
                    return None;
                };
                row.decorations().clock.as_ref()?;
                let target_units = calc_target_units(max_explicit_px, row.layout_params().step());
                Some((index, target_units))
            })
            .collect();
        for (index, target_units) in targets {
            // `index` was derived from `self.lines.iter().enumerate()` above,
            // so it is guaranteed to be in bounds.
            let line = self
                .lines
                .get_mut(index)
                .expect("index computed from self.lines enumeration is always in bounds");
            if let LineContent::Signal(row) = &mut line.content {
                row.expand_clock_row(target_units);
            }
        }
    }

    /// Maximum pixel width across all explicit signal rows.
    ///
    /// A row is "explicit" when it is either a non-clock row or a clock row that
    /// already carries a non-empty waveform body (partial clock). Empty-body auto
    /// clock rows are excluded so they do not collapse the target to zero.
    fn max_explicit_pixel_width(&self) -> Px {
        self.lines
            .iter()
            .filter_map(|line| match &line.content {
                LineContent::Signal(row) => {
                    let is_auto_clock =
                        row.decorations().clock.is_some() && row.waveform().is_empty();
                    if is_auto_clock {
                        return None;
                    }
                    let unit_count = row.waveform().level_units_total();
                    let pixel_step = row.layout_params().step();
                    Some(pixel_step * (unit_count as f32))
                }
                _ => None,
            })
            .fold(Px::ZERO, Px::max)
    }

    // ------------------------------------------------------------------
    // Row appenders (consume pending state)
    // ------------------------------------------------------------------

    /// Append a `@title` row with the current title alignment / colors.
    ///
    /// `@title` rows never donate to the `@ruler` background layer, so the
    /// row is constructed with an empty contribution vector regardless of
    /// the current `@ruler on`/`off` state.
    fn push_title_row(&mut self, text: UserText) {
        let style = TitleStyle::new(
            self.style.canvas().font().clone(),
            self.overrides.title_align.unwrap_or(DEFAULT_TITLE_ALIGN),
            self.style.default_label_style().color(),
        );
        let background = self.pending_next_row.take_background();
        self.lines.push(Line::new(
            LineContent::Title(TitleRow::new(text, style)),
            background,
        ));
    }

    /// Append a `@skip(...)` row, attaching the current `@ruler` donations
    /// when the parser state is `on`. The donation positions snapshot the
    /// current `@step` and treat the skip's `Length::Lh(N)` body as
    /// `units = floor(N)`; `Length::Px(...)` skips contribute no horizontal
    /// extent (units = 0 → a single line at `x = 0`).
    fn push_skip_row(&mut self, length: Length) {
        let background = self.pending_next_row.take_background();
        let units = skip_units_for_ruler(length);
        let ruler_contributions = self.ruler.donations(self.style.layout().step(), units);
        self.lines.push(Line::new_with_ruler_contributions(
            LineContent::Skip(SkipRow::new(length)),
            background,
            ruler_contributions,
        ));
    }

    /// Append a signal row, consuming any pending `@clock`/`@signal`
    /// directives and the pending `@bg`.
    ///
    /// A snapshot of the current `LayoutParams` is captured here so that
    /// per-row `@step`/`@slant` changes apply only to rows parsed after the
    /// directive, not retroactively to earlier rows.
    fn push_signal_row(&mut self, name: SignalName, waveform: Waveform) {
        let directive = self.pending_next_row.take_signal_decorations();
        let style = SignalRowStyle::new(
            self.style.default_signal_style().clone(),
            self.style.default_label_style().clone(),
        );
        let layout_snapshot = *self.style.layout();
        // Snapshot `@ruler` donations from the per-row layout step and the
        // current waveform unit count. Clock-expanded rows still receive
        // their commit-time snapshot here because clock expansion runs as a
        // post-pass on already-committed signal rows, leaving the
        // contributions captured at the point the user wrote the row.
        let units = waveform.level_units_total();
        let ruler_contributions = self.ruler.donations(layout_snapshot.step(), units);
        let row = SignalRow::new(
            SignalGeometry::default(),
            name,
            waveform,
            style,
            SignalDecorations::new(directive.clock, directive.overline),
            layout_snapshot,
        );
        let background = self.pending_next_row.take_background();
        self.lines.push(Line::new_with_ruler_contributions(
            LineContent::Signal(Box::new(row)),
            background,
            ruler_contributions,
        ));
    }

    // ------------------------------------------------------------------
    // Helpers shared with submodules
    // ------------------------------------------------------------------

    /// Build the effective `ClockMarkStyle` for a newly-parsed `@clock(...)`.
    ///
    /// Priority (highest wins):
    /// 1. Per-call overrides supplied by the caller (`local`).
    /// 2. Global `@clockmark_*` directives.
    /// 3. Built-in defaults.
    /// 4. `mark_color` falls back to the current `signal_color` if all else
    ///    is `None`.
    ///
    /// For `width`, when the default falls through (neither local nor global
    /// was set), the resolved value is `min(DEFAULT_CLOCKMARK_WIDTH_PX, step * 2/3)`
    /// using the current `@step`. The step-linked shrink applies only to the
    /// default path: any explicit user value (local or global) is honoured
    /// verbatim. See `docs/spec/tcml-format.md` §「`clockmark_width` の step 連動縮小」.
    pub(super) fn resolve_clock_mark_style(&self, local: &LocalMarkOptions) -> ClockMarkStyle {
        let global = &self.overrides.clockmark;
        let position = local
            .position
            .or(global.position)
            .unwrap_or(DEFAULT_CLOCKMARK_POSITION);
        let height = local
            .height
            .or(global.height)
            .unwrap_or(DEFAULT_CLOCKMARK_HEIGHT_PX);
        let width = local.width.or(global.width).unwrap_or_else(|| {
            let shrink_candidate = self.style.layout().step() * (2.0 / 3.0);
            DEFAULT_CLOCKMARK_WIDTH_PX.min(shrink_candidate)
        });
        let color = local
            .color
            .or(global.color)
            .unwrap_or(self.style.default_signal_style().color());
        ClockMarkStyle::new(position, height, width, color)
    }
}

// =============================================================================
// Directive dispatch macro
// =============================================================================

/// Variant-to-mutation table for [`Parser::apply_directive`]. Lives at module
/// scope (rather than inside the method) so the call site is a single line —
/// satisfying the `too_many_lines` lint — while every arm stays one-per-line
/// for readability.
macro_rules! dispatch_directive {
    ($parser:expr, $directive:expr) => {
        match $directive {
            Directive::FontSize(value) => $parser.style.set_font_size(value),
            Directive::LineHeight(value) => $parser.style.set_line_height_ratio(value),
            Directive::CapWidth(value) => $parser.style.set_capwidth(value),
            Directive::NamePad(value) => $parser.style.set_name_padding(value),
            // `@scale` multiplies the SVG-root `width`/`height` attributes
            // only; internal layout coordinates remain at 1.0. The validated
            // value is stored on `CanvasStyle` and read at SVG-render time.
            Directive::Scale(value) => $parser.style.set_scale(value),
            Directive::PageMargin(value) => $parser.style.set_page_margin(value),
            Directive::Step(value) => $parser.style.set_step(value),
            Directive::Slant(value) => $parser.style.set_slant(value),
            Directive::SignalGap(value) => $parser.style.set_h_space(value),
            Directive::Font(value) => $parser.style.set_font_family(value),
            Directive::SignalColor(value) => $parser.style.set_signal_color(value),
            Directive::SignalWidth(value) => $parser.style.set_signal_width(value),
            Directive::GuideColor(value) => $parser.style.set_guide_color(value),
            Directive::GuideWidth(value) => $parser.style.set_guide_width(value),
            Directive::Bg(value) => $parser.pending_next_row.set_background(value),
            Directive::BgColor0(value) => $parser.style.set_bgcolor0(value),
            Directive::BgColor1(value) => $parser.style.set_bgcolor1(value),
            Directive::HighlightStyle(value) => $parser.style.set_highlight_attrs(value),
            Directive::DontcareColor(value) => $parser.style.set_dontcare_color(value),
            Directive::TitleAlign(value) => $parser.overrides.title_align = Some(value),
            Directive::ClockmarkPosition(value) => {
                $parser.overrides.clockmark.position = Some(value)
            }
            Directive::ClockmarkHeight(value) => $parser.overrides.clockmark.height = Some(value),
            Directive::ClockmarkWidth(value) => $parser.overrides.clockmark.width = Some(value),
            Directive::ClockmarkColor(value) => $parser.overrides.clockmark.color = Some(value),
            Directive::OverlineGap(value) => $parser.style.set_overline_gap(value),
            Directive::OverlineThickness(value) => $parser.style.set_overline_thickness(value),
            Directive::Ruler(value) => $parser.ruler.on = value,
            Directive::RulerColor(value) => $parser.ruler.color = value,
        }
    };
}
use dispatch_directive;

// =============================================================================
// Free helpers (intentionally not on `Parser` — they are pure, location-free)
// =============================================================================

/// Split `@<head>(args)` or `@<head> args` into the canonical
/// `(head, args)` pair.
///
/// The head terminates at the first whitespace or `(`, whichever comes
/// first. Looking only at the prefix prevents a `(` inside an argument
/// value (e.g. `@highlight_style fill="rgb(255, 128, 0)"`) from being
/// mistaken for the start of a parenthesised argument list.
fn split_directive_head(rest: &str) -> (&str, &str) {
    // Only ASCII space and tab are treated as separators here, matching the
    // split criterion below and `split_name_and_levels`. Unicode whitespace
    // (NBSP, U+3000, ...) inside an argument list is preserved verbatim so
    // downstream parsers can decide what to do with it.
    let head_end = rest
        .find(|character: char| character == '(' || ASCII_SPACE_OR_TAB.contains(&character))
        .unwrap_or(rest.len());
    let head = rest[..head_end].trim_matches(ASCII_SPACE_OR_TAB);
    let args = rest[head_end..].trim_matches(ASCII_SPACE_OR_TAB);
    (head, args)
}

/// ASCII space and tab — the only characters treated as field separators in
/// signal lines (`docs/spec/tcml-format.md` §「信号名」) and directive heads.
pub(super) const ASCII_SPACE_OR_TAB: &[char] = &[' ', '\t'];

/// Split `@signal(...)` arguments into the parenthesised attribute list and
/// the optional inline signal-line tail.
///
/// Accepts three shapes:
/// - empty (two-line form without parens: `@signal\n<signal row>`) → `("", "")`
/// - `(<attrs>)` with no trailing content → `(<attrs>, "")`
/// - `(<attrs>) <inline signal row>` → `(<attrs>, <inline rest>)`
///
/// Missing closing `)` yields [`ParseErrorKind::UnknownSignalAttribute`] —
/// the same error kind used for unknown attribute names — so that any
/// malformed `(` payload routes through one well-known parse error.
fn split_signal_arguments_and_inline(
    arguments: &str,
    location: SourceLocation,
) -> Result<(&str, SourceLocation, &str), ParseError> {
    let leading_ws_bytes = arguments.len() - arguments.trim_start_matches(ASCII_SPACE_OR_TAB).len();
    let trimmed = arguments.trim_matches(ASCII_SPACE_OR_TAB);
    if trimmed.is_empty() {
        return Ok(("", location, ""));
    }
    let Some(rest_after_open) = trimmed.strip_prefix('(') else {
        return Err(ParseError::with_length(
            location,
            u32::try_from(trimmed.chars().count()).unwrap_or(u32::MAX),
            ParseErrorKind::UnknownSignalAttribute(trimmed.to_owned()),
        ));
    };
    // Source column of `inner[0]` = column of `arguments[0]` + chars before
    // `inner` in `arguments` (leading whitespace + the opening `(`).
    let inner_byte_offset = leading_ws_bytes + 1;
    let inner_col_offset = arguments[..inner_byte_offset].chars().count() as u32;
    let inner_location = SourceLocation::new(location.line(), location.column() + inner_col_offset);
    let close = rest_after_open.find(')').ok_or_else(|| {
        ParseError::with_length(
            location,
            u32::try_from(trimmed.chars().count()).unwrap_or(u32::MAX),
            ParseErrorKind::UnknownSignalAttribute(trimmed.to_owned()),
        )
    })?;
    let inner = &rest_after_open[..close];
    let tail = rest_after_open[close + 1..].trim_matches(ASCII_SPACE_OR_TAB);
    Ok((inner, inner_location, tail))
}

fn split_name_and_levels(line: &str) -> (&str, &str) {
    // Only ASCII space and tab separate the signal name from the level
    // string. Other Unicode whitespace (NBSP, U+3000 ideographic space,
    // ZWSP, ...) is preserved as part of the signal name.
    // See `docs/spec/tcml-format.md` §「信号名」.
    // `splitn(2, _)` always yields at least one element on a non-empty
    // input; on an empty string it also yields one empty slice. The
    // `expect` documents that contract instead of papering over it with
    // `unwrap_or`.
    let mut parts = line.splitn(2, |character: char| ASCII_SPACE_OR_TAB.contains(&character));
    // `splitn` cuts at the first separator, so `name` never carries a leading
    // or trailing ASCII space/tab — only `rest` needs trimming to swallow
    // runs of separators between the name and the level string.
    let name = parts
        .next()
        .expect("splitn always yields at least one item");
    let rest = parts.next().unwrap_or("");
    (name, rest.trim_matches(ASCII_SPACE_OR_TAB))
}

/// 1-based line number for a 0-based source-line `index`. Saturates at
/// [`u32::MAX`] for adversarial inputs.
fn line_number_for_index(index: usize) -> u32 {
    u32::try_from(index.saturating_add(1)).unwrap_or(u32::MAX)
}

/// Recover a line number from a [`SourceLocation`]. Used by helpers that
/// need to upgrade only the column field without losing the line.
fn line_number_for_index_from_location(location: SourceLocation) -> u32 {
    location.line()
}

/// Count the number of leading ASCII space/tab characters in `raw`. Used to
/// place [`SourceLocation::column`] at the first non-whitespace character of
/// a source line.
///
/// Counts in **characters**, not bytes — though for ASCII space/tab the two
/// happen to coincide. The function only considers the small fixed leading
/// whitespace set; Unicode whitespace earlier in a directive is not stripped
/// by the trimmer and so is not counted here either.
fn count_leading_whitespace_chars(raw: &str) -> u32 {
    let count = raw
        .chars()
        .take_while(|character| ASCII_SPACE_OR_TAB.contains(character))
        .count();
    u32::try_from(count).unwrap_or(u32::MAX)
}

/// Compute the 1-based character column of `substring`'s start within
/// `haystack`. `substring` must be a slice produced from `haystack`
/// (pointer-derived); the function uses pointer arithmetic to determine the
/// byte offset and then char-counts the prefix.
///
/// Returns `None` when `substring`'s pointer falls outside `haystack`'s
/// byte range — defensive against accidental misuse.
fn char_column_of_substring(haystack: &str, substring: &str) -> Option<u32> {
    let haystack_start = haystack.as_ptr() as usize;
    let substring_start = substring.as_ptr() as usize;
    if substring_start < haystack_start {
        return None;
    }
    let byte_offset = substring_start - haystack_start;
    if byte_offset > haystack.len() {
        return None;
    }
    let prefix = haystack.get(..byte_offset)?;
    let character_count = prefix.chars().count();
    u32::try_from(character_count + 1).ok()
}

/// Compute the source caret (column offset and length) for an inner-error
/// payload such as [`NameError`] / [`TextError`] / [`ColorError`].
///
/// `inner_offset` is the char-offset returned by the inner error's
/// `char_offset()` method (or equivalent) — `Some(N)` means "the issue is
/// at the N-th character of the value", `None` means "the value as a whole
/// is the issue (empty, not-found, etc.)".
///
/// Returns `(column_offset, length)` relative to the value's start column:
/// for an offset-bearing error, `(offset, 1)` so the underline covers a
/// single character; otherwise `(0, value.chars().count())` so the
/// underline spans the whole value.
fn caret_for_inner(inner_offset: Option<u32>, value: &str) -> (u32, u32) {
    match inner_offset {
        Some(offset) => (offset, 1),
        None => (0, u32::try_from(value.chars().count()).unwrap_or(u32::MAX)),
    }
}

/// Locate `inner` (a derived `&str` slice of `outer`) and return a
/// [`SourceLocation`] whose column = `outer_location.column` + char count
/// of `outer` bytes before `inner`. Falls back to `outer_location` if the
/// pointer math indicates `inner` is not a slice of `outer`.
fn locate_substring(outer: &str, inner: &str, outer_location: SourceLocation) -> SourceLocation {
    let outer_start = outer.as_ptr() as usize;
    let inner_start = inner.as_ptr() as usize;
    if inner_start < outer_start {
        return outer_location;
    }
    let byte_offset = inner_start - outer_start;
    if byte_offset > outer.len() {
        return outer_location;
    }
    let Some(prefix) = outer.get(..byte_offset) else {
        return outer_location;
    };
    let col_offset = u32::try_from(prefix.chars().count()).unwrap_or(0);
    SourceLocation::new(outer_location.line(), outer_location.column() + col_offset)
}

/// Build a [`SourceLocation`] whose column points at the start of
/// `substring` within `raw_line`. Falls back to `fallback` when `substring`
/// is empty or not a slice of `raw_line` (e.g. inline pseudo-line forms).
fn location_at_substring_or_default(
    raw_line: &str,
    substring: &str,
    fallback: SourceLocation,
    line: u32,
) -> SourceLocation {
    if substring.is_empty() {
        return fallback;
    }
    match char_column_of_substring(raw_line, substring) {
        Some(column) => SourceLocation::new(line, column),
        None => fallback,
    }
}

/// Convert a `@skip(...)` amount into the integer "unit count" used by
/// `@ruler` donations.
///
/// Per `docs/spec/tcml-format.md` §「`@ruler` の詳細」, the spec only writes
/// concrete unit counts for `@skip(N)` (an integer Lh value). For
/// non-integer Lh and for `Length::Px(...)` skips — which have no notion of
/// a unit grid — the donation collapses to `units = 0` so the skip
/// contributes a single line at `x = 0` rather than producing a meaningless
/// fractional grid.
fn skip_units_for_ruler(length: Length) -> u32 {
    match length {
        Length::Lh(value) if value.is_finite() && value >= 0.0 => value.floor() as u32,
        _ => 0,
    }
}

/// Compute the target unit count for a clock auto row given the maximum
/// explicit pixel width across the chart and this row's own step.
///
/// Formula: `round(max_px / row_step)`.
///
/// Returns 0 when `max_px` is zero (all-auto chart) or `row_step` is zero
/// (degenerate — should not occur after the invariant check).
fn calc_target_units(max_explicit_px: Px, row_step: Px) -> u32 {
    let max_pixel_width = max_explicit_px.to_f32();
    let row_step_f32 = row_step.to_f32();
    if row_step_f32 <= 0.0 || max_pixel_width <= 0.0 {
        return 0;
    }
    (max_pixel_width / row_step_f32).round() as u32
}
