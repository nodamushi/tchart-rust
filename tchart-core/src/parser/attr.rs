//! Shared attribute parsing helpers used by multiple TCML directive parsers.
//!
//! Only generic primitive helpers (`(...)` stripping, `key=value` splitting,
//! `"..."` unquoting) live here. Type-specific parsers belong on the type
//! itself: see [`crate::style::SvgAttrList::parse`] etc.

use crate::errors::{ParseError, ParseErrorKind, SourceLocation};

/// Strip the outer `(...)` from `args`, trimming whitespace. Returns the
/// inner slice or `error_kind` (wrapped at `location`) when the parens are
/// missing. The error's `length` is sized to the trimmed args so a
/// caret/underline covers the offending span.
pub(super) fn strip_parens(
    args: &str,
    location: SourceLocation,
    error_kind: ParseErrorKind,
) -> Result<&str, ParseError> {
    let trimmed = args.trim();
    let inner = trimmed
        .strip_prefix('(')
        .and_then(|rest| rest.strip_suffix(')'))
        .ok_or_else(|| {
            let length = u32::try_from(trimmed.chars().count())
                .unwrap_or(u32::MAX)
                .max(1);
            ParseError::with_length(location, length, error_kind)
        })?;
    Ok(inner)
}

/// Split a `key=value` token (with optional whitespace around `=`) into a
/// `(key, value)` pair. Both sides are whitespace-trimmed; the value has any
/// surrounding `"..."` removed via [`unquote`].
///
/// Returns `None` when `=` is absent, when either side is empty after
/// trimming, or when the value has an unmatched leading/trailing `"`.
pub(super) fn split_key_value(token: &str) -> Option<(&str, &str)> {
    let (key, value) = token.split_once('=')?;
    let key = key.trim();
    let value = value.trim();
    if key.is_empty() || value.is_empty() {
        return None;
    }
    let value = unquote(value)?;
    Some((key, value))
}

/// Strip a single pair of surrounding ASCII double quotes from `value`.
///
/// - `"abc"` -> `Some("abc")`.
/// - `abc` -> `Some("abc")` (no quotes is fine).
/// - `"abc` / `abc"` / `"` -> `None` (mismatched quotes).
pub(super) fn unquote(value: &str) -> Option<&str> {
    let leads = value.starts_with('"');
    let trails = value.ends_with('"');
    match (leads, trails, value.len()) {
        (true, true, length) if length >= 2 => Some(&value[1..length - 1]),
        (false, false, _) => Some(value),
        _ => None,
    }
}
