//! Single-document parse → layout → SVG/PNG pipeline.
//!
//! This module provides the convenience wrapper used by the `svg` and `png`
//! subcommands, which are "worker 1, input 1" degenerations of the batch model.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use tchart_core::layout::layout;
use tchart_core::parser::parse;
use tchart_core::svg::render;
use tchart_core::units::Px;

use crate::error::{CliError, PngEncodeStage};
use crate::font::{FontEntry, SharedFontCache, WorkerFontContext, extract_font_families};

/// Heuristic capacity bonus for the iTXt-augmented PNG buffer.
const ITXT_HEADER_OVERHEAD: usize = 64;

/// PNG `tchart-source` iTXt chunk keyword.
const PNG_SOURCE_KEYWORD: &str = "tchart-source";

/// Combined parse + layout + SVG-render result.
///
/// Retains the font cache and the CSS generic assignments resolved during
/// rendering so that PNG rasterisation uses the same font data without
/// re-reading files.
pub(crate) struct Rendered {
    svg_content: String,
    /// Original TCML source, kept for PNG iTXt embedding.
    source: String,
    /// Font cache used during layout.
    cache: SharedFontCache,
    /// CSS generic → fontdb family name resolved during this render pass.
    generic_assignments: HashMap<String, String>,
    /// Fonts resolved for this document (subset of the shared cache).
    resolved_entries: Vec<Arc<FontEntry>>,
}

impl Rendered {
    /// Consume the SVG markup as a UTF-8 byte buffer.
    pub(crate) fn into_svg_bytes(self) -> Vec<u8> {
        self.svg_content.into_bytes()
    }

    /// Rasterise the SVG to a PNG byte buffer, embedding the original TCML
    /// `source` in a `tchart-source` iTXt chunk.
    pub(crate) fn into_png_bytes(self) -> Result<Vec<u8>, CliError> {
        let fontdb = self
            .cache
            .build_fontdb_for_document(&self.resolved_entries, &self.generic_assignments);
        build_png_bytes(&self.svg_content, &self.source, fontdb)
    }
}

/// Parse, layout, and render a single TCML document.
///
/// Creates a `SharedFontCache` seeded with the default font, resolves all
/// `@font` families from the source, runs layout and SVG render, then returns
/// the result.  This is the "worker 1, input 1" degeneration of the batch
/// worker model defined in `docs/spec/cli.md` §単一サブコマンドの挙動.
pub(crate) fn render_single(
    input_path: &Path,
    source: String,
    font_path: &Path,
    font_size_override: Option<f32>,
) -> Result<Rendered, CliError> {
    let cache = SharedFontCache::new(font_path)?;
    let mut document = parse(&source)
        .map_err(|error| CliError::parse_with_file(input_path, source.clone(), error))?;
    if let Some(size) = font_size_override {
        document.set_font_size(Px(size));
    }
    let mut context = WorkerFontContext::new(&cache);
    for family_csv in extract_font_families(&source) {
        context.add_family_csv(&family_csv);
    }
    layout(&mut document, &context)?;
    let svg_content = render(&document, &context);
    let (generic_assignments, resolved_entries) = context.into_parts();
    Ok(Rendered {
        svg_content,
        source,
        cache,
        generic_assignments,
        resolved_entries,
    })
}

/// Rasterise `svg_markup` to PNG and embed `tcml_source` in a `tchart-source`
/// iTXt chunk. The `fontdb` database must be pre-built with all fonts needed
/// by the SVG so that `resvg` resolves family names to the same font files
/// used during layout.
pub(crate) fn build_png_bytes(
    svg_markup: &str,
    tcml_source: &str,
    fontdb: fontdb::Database,
) -> Result<Vec<u8>, CliError> {
    let pixmap = render_svg_to_pixmap(svg_markup, fontdb)?;
    let encoded_png_bytes = pixmap.encode_png().map_err(|error| {
        CliError::output_write_png_stage(PngEncodeStage::PngEncode, error.to_string())
    })?;
    embed_itxt(&encoded_png_bytes, tcml_source)
}

fn render_svg_to_pixmap(
    svg_markup: &str,
    fontdb: fontdb::Database,
) -> Result<tiny_skia::Pixmap, CliError> {
    let options = usvg::Options {
        fontdb: Arc::new(fontdb),
        ..usvg::Options::default()
    };
    let tree = usvg::Tree::from_str(svg_markup, &options).map_err(|error| {
        CliError::output_write_png_stage(PngEncodeStage::UsvgParse, format!("usvg parse: {error}"))
    })?;
    let size = tree.size();
    let dimensions = SvgPixelDimensions::from_floats(size.width(), size.height())?;
    let mut pixmap =
        tiny_skia::Pixmap::new(dimensions.width, dimensions.height).ok_or_else(|| {
            CliError::output_write_png_stage(
                PngEncodeStage::PixmapAlloc,
                format!(
                    "pixmap allocation failed ({}x{})",
                    dimensions.width, dimensions.height
                ),
            )
        })?;
    resvg::render(
        &tree,
        tiny_skia::Transform::identity(),
        &mut pixmap.as_mut(),
    );
    Ok(pixmap)
}

/// Pixel dimensions for the rasterised SVG, validated to be non-zero `u32` values.
struct SvgPixelDimensions {
    width: u32,
    height: u32,
}

impl SvgPixelDimensions {
    fn from_floats(width: f32, height: f32) -> Result<Self, CliError> {
        let width = Self::ceil_to_u32(width)?;
        let height = Self::ceil_to_u32(height)?;
        if width == 0 || height == 0 {
            return Err(CliError::output_write_png_stage(
                PngEncodeStage::SvgSize,
                format!("rendered SVG has zero size ({width}x{height})"),
            ));
        }
        Ok(SvgPixelDimensions { width, height })
    }

    fn ceil_to_u32(value: f32) -> Result<u32, CliError> {
        if !value.is_finite() || value < 0.0 {
            return Err(CliError::output_write_png_stage(
                PngEncodeStage::SvgSize,
                format!("rendered SVG has invalid size: {value}"),
            ));
        }
        let ceiling = value.ceil();
        Ok(if ceiling >= u32::MAX as f32 {
            u32::MAX
        } else {
            ceiling as u32
        })
    }
}

/// Decoded pixel buffer for a freshly rasterised PNG.
struct DecodedPng {
    width: u32,
    height: u32,
    color: png::ColorType,
    depth: png::BitDepth,
    pixels: Vec<u8>,
}

impl DecodedPng {
    fn decode(png_bytes: &[u8]) -> Result<Self, CliError> {
        let decoder = png::Decoder::new(std::io::Cursor::new(png_bytes));
        let mut reader = decoder.read_info().map_err(|error| {
            CliError::output_write_png_stage(PngEncodeStage::PngDecode, error.to_string())
        })?;
        let (width, height, color, depth) = {
            let info = reader.info();
            (info.width, info.height, info.color_type, info.bit_depth)
        };
        let mut pixels = vec![0u8; reader.output_buffer_size()];
        reader.next_frame(&mut pixels).map_err(|error| {
            CliError::output_write_png_stage(PngEncodeStage::PngDecodeFrame, error.to_string())
        })?;
        Ok(DecodedPng {
            width,
            height,
            color,
            depth,
            pixels,
        })
    }
}

fn embed_itxt(png_bytes: &[u8], tcml_source: &str) -> Result<Vec<u8>, CliError> {
    let decoded = DecodedPng::decode(png_bytes)?;
    let mut output = Vec::with_capacity(png_bytes.len() + tcml_source.len() + ITXT_HEADER_OVERHEAD);
    {
        let mut encoder = png::Encoder::new(&mut output, decoded.width, decoded.height);
        encoder.set_color(decoded.color);
        encoder.set_depth(decoded.depth);
        encoder
            .add_itxt_chunk(PNG_SOURCE_KEYWORD.to_owned(), tcml_source.to_owned())
            .map_err(|error| {
                CliError::output_write_png_stage(PngEncodeStage::Itxt, error.to_string())
            })?;
        let mut writer = encoder.write_header().map_err(|error| {
            CliError::output_write_png_stage(PngEncodeStage::PngHeader, error.to_string())
        })?;
        writer.write_image_data(&decoded.pixels).map_err(|error| {
            CliError::output_write_png_stage(PngEncodeStage::PngData, error.to_string())
        })?;
    }
    Ok(output)
}
