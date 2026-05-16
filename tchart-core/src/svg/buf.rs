//! Output buffer and trust-boundary helpers for SVG serialization.
//!
//! See `docs/spec/svg-rendering.md` "セキュリティ: 型による信頼境界". Static
//! literals enter through [`SvgBuf::write_literal`] /
//! [`SvgBuf::write_static_attribute`]; user values must implement
//! [`UserValue`] and pass through [`SvgBuf::write_escaped`] /
//! [`SvgBuf::write_user_attribute`] which apply XML escaping.

use crate::color::Color;
use crate::geometry::Size;
use crate::text::{FontFamily, SignalName, UnsafeLineText, UserText};
use crate::units::Px;

/// Sealed trait for values originating from user-supplied TCML.
///
/// Implementations write themselves into an [`SvgBuf`] with XML escaping.
pub(super) trait UserValue {
    /// Append the escaped representation of this value to `buf`.
    fn write_escaped(&self, buf: &mut SvgBuf);
}

/// Trait implemented by domain types that know how to serialize themselves
/// into an [`SvgBuf`].
///
/// The operation goes through this trait so that `SvgBuf` exposes a single
/// `&mut self` write method instead of accepting a fistful of free
/// `fn render_*(buf: &mut SvgBuf, ...)` functions scattered across modules.
pub(super) trait WriteSvgOn {
    /// Append this value's SVG representation into `target`.
    fn write_svg_on(&self, target: &mut SvgBuf);
}

/// Pre-escaped SVG fragment originating from another [`SvgBuf`] sub-buffer.
///
/// Concatenating raw `&str` into the main SVG buffer would create a trust
/// hole; only fragments previously rendered through this API may be appended
/// verbatim. The newtype contains that boundary in the type system.
pub(super) struct AlreadyEscapedSvgFragment<'a>(&'a str);

impl<'a> AlreadyEscapedSvgFragment<'a> {
    /// Wrap a sub-buffer's contents.
    ///
    /// The lifetime ties the borrow to the originating [`SvgBuf`].
    pub(super) fn from_buf(buffer: &'a SvgBuf) -> Self {
        Self(buffer.as_str())
    }

    /// Borrow the underlying pre-escaped fragment.
    pub(super) fn as_str(&self) -> &str {
        self.0
    }
}

/// A growable SVG output buffer.
#[derive(Debug, Default)]
pub(super) struct SvgBuf {
    inner: String,
}

impl SvgBuf {
    /// Create an empty buffer.
    pub(super) fn new() -> Self {
        Self::default()
    }

    /// Append a static literal (tag names, fixed attribute values).
    pub(super) fn write_literal(&mut self, literal: &'static str) {
        self.inner.push_str(literal);
    }

    /// Append the content of another rendered SVG fragment.
    ///
    /// The [`AlreadyEscapedSvgFragment`] newtype enforces in the type system
    /// that the caller has produced `fragment` through this same buffer API.
    pub(super) fn write_fragment(&mut self, fragment: AlreadyEscapedSvgFragment<'_>) {
        self.inner.push_str(fragment.as_str());
    }

    /// Render `source` into this buffer. Single entry point so `SvgBuf` does
    /// not need to know every concrete type.
    pub(super) fn write<T: WriteSvgOn + ?Sized>(&mut self, source: &T) {
        source.write_svg_on(self);
    }

    /// Open `<g class="...">` / run `body` / close `</g>`, omitting the wrapper
    /// entirely when `body` writes nothing.
    ///
    /// `docs/spec/svg-rendering.md` mandates that a `<g class="...">` whose
    /// contents are empty must not be emitted (no empty `<g
    /// class="..."></g>` tag either). Centralising the check here keeps every
    /// layer call site at one line and avoids per-layer `if` branches.
    pub(super) fn write_layer<F>(&mut self, class: &'static str, body: F)
    where
        F: FnOnce(&mut SvgBuf),
    {
        let before_open = self.inner.len();
        self.write_literal("<g class=\"");
        self.write_literal(class);
        self.write_literal("\">");
        let after_open = self.inner.len();
        body(self);
        if self.inner.len() == after_open {
            self.inner.truncate(before_open);
        } else {
            self.write_literal("</g>");
        }
    }

    /// Wrap an already-rendered sub-buffer in `<g class="...">…</g>` and append.
    ///
    /// Empty sub-buffers produce no output (spec: see [`Self::write_layer`]).
    pub(super) fn write_layer_buffer(&mut self, class: &'static str, body: &SvgBuf) {
        if body.as_str().is_empty() {
            return;
        }
        self.write_literal("<g class=\"");
        self.write_literal(class);
        self.write_literal("\">");
        self.write_fragment(AlreadyEscapedSvgFragment::from_buf(body));
        self.write_literal("</g>");
    }

    /// Open the `<svg>` root with display and internal (logical) dimensions,
    /// run `body`, then close.
    ///
    /// `display_size` is what goes into `width=` / `height=` (post-`@scale`).
    /// `internal_size` is what goes into `viewBox="0 0 W H"` (pre-`@scale`,
    /// the logical coordinate system used by every inner element). Emitting
    /// both is required so the SVG viewport actually scales the contents;
    /// without `viewBox` the canvas would just be bigger with the drawing
    /// pinned at its original size to the top-left.
    /// See `docs/spec/svg-rendering.md` §「ルート width / height / viewBox と @scale」.
    pub(super) fn write_svg_root<F>(&mut self, display_size: Size, internal_size: Size, body: F)
    where
        F: FnOnce(&mut SvgBuf),
    {
        self.write_literal(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:tchart=\"http://tchart-rust/1.0\"",
        );
        self.write_px_attribute("width", display_size.width);
        self.write_px_attribute("height", display_size.height);
        self.write_view_box_attribute(internal_size);
        self.write_char('>');
        body(self);
        self.write_literal("</svg>");
    }

    /// Write `viewBox="0 0 W H"` using the pre-`@scale` logical dimensions.
    fn write_view_box_attribute(&mut self, internal_size: Size) {
        self.write_attribute_prefix("viewBox");
        self.write_literal("0 0 ");
        self.push_px(internal_size.width);
        self.write_char(' ');
        self.push_px(internal_size.height);
        self.write_char('"');
    }

    /// Append a single character (no escaping).
    pub(super) fn write_char(&mut self, character: char) {
        self.inner.push(character);
    }

    /// Append a line break.
    pub(super) fn write_linebreak(&mut self) {
        self.inner.push('\n');
    }

    /// Append the escaped form of a [`UserValue`].
    pub(super) fn write_escaped<T: UserValue + ?Sized>(&mut self, value: &T) {
        value.write_escaped(self);
    }

    /// Write a `name="value"` pair where the value is XML-escaped.
    pub(super) fn write_user_attribute<T: UserValue + ?Sized>(
        &mut self,
        name: &'static str,
        value: &T,
    ) {
        self.write_attribute_prefix(name);
        value.write_escaped(self);
        self.write_char('"');
    }

    /// Write a `name="value"` pair using a static literal value.
    pub(super) fn write_static_attribute(&mut self, name: &'static str, value: &'static str) {
        self.write_attribute_prefix(name);
        self.write_literal(value);
        self.write_char('"');
    }

    fn write_attribute_prefix(&mut self, name: &'static str) {
        self.write_char(' ');
        self.write_literal(name);
        self.write_literal("=\"");
    }

    /// Push raw escaped characters from a `&str` (used by [`UserValue`] impls).
    pub(super) fn write_escaped_str(&mut self, value: &str) {
        for character in value.chars() {
            match character {
                '<' => self.inner.push_str("&lt;"),
                '>' => self.inner.push_str("&gt;"),
                '&' => self.inner.push_str("&amp;"),
                '"' => self.inner.push_str("&quot;"),
                '\'' => self.inner.push_str("&apos;"),
                other => self.inner.push(other),
            }
        }
    }

    /// Write a `Px` numeric attribute value (3 decimal places, trimmed).
    pub(super) fn write_px_attribute(&mut self, name: &'static str, value: Px) {
        self.write_attribute_prefix(name);
        self.push_px(value);
        self.write_char('"');
    }

    /// Append a [`Px`] numeric value at the current cursor.
    pub(super) fn write_px(&mut self, value: Px) {
        self.push_px(value);
    }

    /// Append a [`DontcareHatchPatternId`] at the current cursor.
    ///
    /// The id's `Display` form is the fixed-format string `dontcare-hatch-N`,
    /// safe to write without escaping.
    pub(super) fn write_dontcare_id(&mut self, id: crate::svg::waveform::DontcareHatchPatternId) {
        use std::fmt::Write;
        let _ = write!(self.inner, "{id}");
    }

    /// Consume the buffer, returning the underlying string.
    pub(super) fn finish(self) -> String {
        self.inner
    }

    /// Borrow the current contents.
    pub(super) fn as_str(&self) -> &str {
        &self.inner
    }

    fn push_px(&mut self, value: Px) {
        let formatted = format!("{:.3}", value.to_f32());
        let trimmed = trim_trailing_zeros(&formatted);
        self.inner.push_str(trimmed);
    }
}

fn trim_trailing_zeros(text: &str) -> &str {
    if !text.contains('.') {
        return text;
    }
    let trimmed = text.trim_end_matches('0');
    trimmed.trim_end_matches('.')
}

impl UserValue for str {
    fn write_escaped(&self, buf: &mut SvgBuf) {
        buf.write_escaped_str(self);
    }
}

impl UserValue for String {
    fn write_escaped(&self, buf: &mut SvgBuf) {
        buf.write_escaped_str(self);
    }
}

impl UserValue for UnsafeLineText<'_> {
    fn write_escaped(&self, buf: &mut SvgBuf) {
        buf.write_escaped_str(self.unsafe_text());
    }
}

impl UserValue for SignalName {
    fn write_escaped(&self, buf: &mut SvgBuf) {
        write_lines_joined(self.lines(), buf);
    }
}

impl UserValue for UserText {
    fn write_escaped(&self, buf: &mut SvgBuf) {
        write_lines_joined(self.lines(), buf);
    }
}

impl UserValue for FontFamily {
    fn write_escaped(&self, buf: &mut SvgBuf) {
        self.as_unsafe_line().write_escaped(buf);
    }
}

/// Escape each line through the SVG escape API and rejoin with literal `\n`.
///
/// Multi-line `UserText` / `SignalName` values reach this helper when their
/// `UserValue` impl is invoked from a context that treats the value as a
/// single blob (e.g. attribute defaults). The line iterator is the only
/// production path that yields `UnsafeLineText`, so this helper is the sole
/// place where the line separator is re-introduced — and it stays as `\n`
/// (the normalized internal representation), never `\r\n`.
fn write_lines_joined<'text, I>(lines: I, buf: &mut SvgBuf)
where
    I: Iterator<Item = UnsafeLineText<'text>>,
{
    let mut first = true;
    for line in lines {
        if !first {
            buf.write_linebreak();
        }
        first = false;
        line.write_escaped(buf);
    }
}

impl UserValue for Color {
    fn write_escaped(&self, buf: &mut SvgBuf) {
        buf.write_escaped_str(&self.to_css_string());
    }
}
