//! Validated anchor names — `[A-Za-z0-9_][A-Za-z0-9_-]*`.

/// A validated anchor name (`[A-Za-z0-9_][A-Za-z0-9_-]*`).
///
/// Pure-numeric names such as `1` are valid; the `{ }` syntax distinguishes
/// named anchors from numbered anchors (`@N`) so `@{1}` and `@1` are not the
/// same anchor.
///
/// See `docs/spec/types.md` §2.1 and `docs/spec/tcml-format.md` §「アンカー」.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct AnchorName(String);

/// Errors produced by [`AnchorName::parse`].
///
/// `InvalidChar` carries the 0-based char offset of the offending character
/// within the parsed name so the caller can compute an exact source column.
/// `InvalidLeadingChar` always sits at offset 0 by construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub(crate) enum AnchorNameError {
    /// The name was empty.
    #[error("anchor name is empty")]
    Empty,
    /// The first character was not `[A-Za-z0-9_]`.
    #[error("anchor name must start with [A-Za-z0-9_]")]
    InvalidLeadingChar,
    /// A subsequent character was not `[A-Za-z0-9_-]`.
    #[error("anchor name contains an invalid character")]
    InvalidChar { char_offset: u32 },
}

impl AnchorNameError {
    /// 0-based char offset within the parsed name where the caret should
    /// land, or `None` when the error spans the whole name (empty).
    pub(crate) fn char_offset(&self) -> Option<u32> {
        match self {
            Self::Empty => None,
            Self::InvalidLeadingChar => Some(0),
            Self::InvalidChar { char_offset } => Some(*char_offset),
        }
    }
}

impl AnchorName {
    /// Parse an anchor name. Allowed pattern: `[A-Za-z0-9_][A-Za-z0-9_-]*`.
    pub(crate) fn parse(input: &str) -> Result<Self, AnchorNameError> {
        let mut chars = input.chars().enumerate();
        let (_, first) = chars.next().ok_or(AnchorNameError::Empty)?;
        if !Self::is_leading_char(first) {
            return Err(AnchorNameError::InvalidLeadingChar);
        }
        for (index, character) in chars {
            if !Self::is_tail_char(character) {
                return Err(AnchorNameError::InvalidChar {
                    char_offset: u32::try_from(index).unwrap_or(u32::MAX),
                });
            }
        }
        Ok(Self(input.to_owned()))
    }

    /// Borrow the underlying string slice. Production callers use this to
    /// render the name in error messages (`@{name}`) and to compute caret /
    /// underline widths for anchor-related parse errors.
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }

    /// Number of source characters in the name. Production callers use this
    /// to size the caret/underline drawn over `@{name}` tokens when emitting
    /// duplicate/undefined anchor errors.
    pub(crate) fn char_count(&self) -> usize {
        self.0.chars().count()
    }

    fn is_leading_char(character: char) -> bool {
        character.is_ascii_alphanumeric() || character == '_'
    }

    fn is_tail_char(character: char) -> bool {
        character.is_ascii_alphanumeric() || character == '_' || character == '-'
    }
}
