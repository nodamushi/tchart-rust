//! Extract embedded TCML source from a tchart-generated SVG or PNG file.
//!
//! See `docs/spec/cli.md` §extract.
//!
//! The SVG side relies on tchart's own output convention: a single
//! `<tchart:source>...</tchart:source>` element with XML-escaped contents.
//! The extractor therefore scans for that exact tag pair as bytes rather than
//! parsing the SVG as XML — this keeps the extractor zero-dependency and
//! tolerant of resvg/usvg-shaped output, but it relies on the input being a
//! tchart-produced SVG. Hand-written SVGs that use the `<tchart:source>` name
//! for unrelated content are not supported.

use std::path::Path;
use std::str::Chars;

use crate::error::{CliError, ExtractError};

const SVG_OPEN: &str = "<tchart:source>";
const SVG_CLOSE: &str = "</tchart:source>";
const PNG_KEY: &str = "tchart-source";
const PNG_MAGIC: &[u8; 8] = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

/// Container the input file uses for its embedded payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Container {
    Svg,
    Png,
}

impl Container {
    /// Identify the container by sniffing the file's magic bytes. Anything
    /// that does not match the PNG signature is assumed to be SVG; that
    /// matches the spec which only documents SVG and PNG inputs.
    fn detect(bytes: &[u8]) -> Container {
        if bytes.starts_with(PNG_MAGIC) {
            Container::Png
        } else {
            Container::Svg
        }
    }
}

/// Read `path`, detect SVG vs PNG by magic bytes, and return the embedded
/// TCML source verbatim.
///
/// The returned bytes mirror what was embedded by the SVG/PNG renderer; in
/// particular the trailing newline is preserved as-is. Inputs with no
/// trailing LF round-trip byte-for-byte and inputs with a trailing LF keep
/// it. Spec: `docs/spec/cli.md` §round-trip 完全一致 / `docs/tests/cli.feature.md`
/// §SVG round-trip.
pub(crate) fn read_embedded_source(path: &Path) -> Result<String, CliError> {
    let bytes = std::fs::read(path).map_err(|source| CliError::input_read(path, source))?;
    let source = match Container::detect(&bytes) {
        Container::Svg => extract_from_svg(&bytes)?,
        Container::Png => extract_from_png(&bytes)?,
    };
    Ok(source)
}

fn extract_from_svg(bytes: &[u8]) -> Result<String, ExtractError> {
    let text = std::str::from_utf8(bytes).map_err(|_| ExtractError::SvgNotUtf8)?;
    let start = text
        .find(SVG_OPEN)
        .ok_or(ExtractError::SvgNoSourceElement)?
        + SVG_OPEN.len();
    let end_relative = text[start..]
        .find(SVG_CLOSE)
        .ok_or(ExtractError::SvgUnclosedSource)?;
    Ok(unescape_xml(&text[start..start + end_relative]))
}

fn extract_from_png(bytes: &[u8]) -> Result<String, ExtractError> {
    let decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    let reader = decoder
        .read_info()
        .map_err(|error| ExtractError::PngInvalid(error.to_string()))?;
    let info = reader.info();
    // Prefer the latin1 tEXt chunk when present; it requires no additional
    // decoding step. In practice tchart always writes an iTXt (UTF-8) chunk,
    // but round-tripping a file that was processed by other tools may produce
    // a latin1 chunk instead. Return it first to match the PNG spec ordering.
    if let Some(text) = info
        .uncompressed_latin1_text
        .iter()
        .find(|entry| entry.keyword == PNG_KEY)
        .map(|entry| entry.text.clone())
    {
        return Ok(text);
    }
    info.utf8_text
        .iter()
        .find(|entry| entry.keyword == PNG_KEY)
        .ok_or(ExtractError::PngNoSourceChunk)?
        .get_text()
        .map_err(|error| ExtractError::PngItxtDecode(error.to_string()))
}

/// The five XML entities that tchart's SVG renderer ever emits. Variant
/// names use the XML entity short names (`lt` / `gt` / `amp` / `quot` /
/// `apos`) verbatim — those are the canonical XML names rather than ad-hoc
/// abbreviations.
#[derive(Clone, Copy, PartialEq, Eq)]
enum XmlEntity {
    Lt,
    Gt,
    Amp,
    Quot,
    Apos,
}

impl XmlEntity {
    /// Longest entity name in [`XmlEntity`] is `apos` (4 bytes).
    const MAX_NAME_LEN: usize = 4;

    /// Look up an entity by its XML name (e.g. `"lt"` → [`XmlEntity::Lt`]).
    fn from_name(name: &str) -> Option<XmlEntity> {
        match name {
            "lt" => Some(XmlEntity::Lt),
            "gt" => Some(XmlEntity::Gt),
            "amp" => Some(XmlEntity::Amp),
            "quot" => Some(XmlEntity::Quot),
            "apos" => Some(XmlEntity::Apos),
            _ => None,
        }
    }

    fn to_char(self) -> char {
        match self {
            XmlEntity::Lt => '<',
            XmlEntity::Gt => '>',
            XmlEntity::Amp => '&',
            XmlEntity::Quot => '"',
            XmlEntity::Apos => '\'',
        }
    }
}

fn unescape_xml(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars();
    while let Some(character) = chars.next() {
        if character != '&' {
            output.push(character);
            continue;
        }
        match decode_known_entity(chars.clone()) {
            Some((entity, consumed)) => {
                output.push(entity.to_char());
                // Advance the live iterator past the consumed chars.
                // `consumed` is always >= 1 (name + ';'), so nth(consumed - 1)
                // is always valid.
                chars.nth(consumed - 1);
            }
            None => output.push('&'),
        }
    }
    output
}

/// Try to decode the next entity from the remaining chars after a `&`.
/// Returns the decoded entity and the number of `char`s consumed (including
/// the trailing `;`). Returning the count rather than mutating a `&mut Chars`
/// keeps `unescape_xml` in charge of advancing its own iterator.
fn decode_known_entity(remaining: Chars<'_>) -> Option<(XmlEntity, usize)> {
    let mut name = String::new();
    for (index, character) in remaining.take(XmlEntity::MAX_NAME_LEN + 1).enumerate() {
        if character == ';' {
            return XmlEntity::from_name(&name).map(|entity| (entity, index + 1));
        }
        name.push(character);
    }
    None
}
