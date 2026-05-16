//! PNG iTXt 抽出 / 埋込 (target-agnostic、wasm から薄くラップして公開)。
//!
//! 詳細は `docs/spec/web.md` の `extract_tcml_source_from_png` /
//! `embed_tcml_source_in_png` を参照。
//!
//! tchart-cli 側 (`tchart-cli/src/render.rs::embed_itxt`) と同様のことを
//! `png` クレートで行うが、そちらとは独立した実装としてコード重複を
//! 許容している (core 経由しないことの帰結)。

const TCHART_SOURCE_KEYWORD: &str = "tchart-source";

/// PNG iTXt チャンクのうち keyword `tchart-source` の本文を返す。
/// 該当チャンクが無い、入力が PNG として不正、等の場合は `None`。
/// 圧縮されていれば `png` クレートが復元する。
pub fn extract_tcml_source_from_png(bytes: &[u8]) -> Option<String> {
    let decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    let reader = decoder.read_info().ok()?;
    let info = reader.info();
    let chunk = info
        .utf8_text
        .iter()
        .find(|chunk| chunk.keyword == TCHART_SOURCE_KEYWORD)?;
    chunk.get_text().ok()
}

/// PNG バイト列に keyword `tchart-source` の iTXt チャンクとして `source` を
/// 埋め込んだ新しい PNG バイト列を返す。元の画像データ (IHDR / IDAT / IEND) は
/// 保持する。入力が PNG として不正な場合は `Err`。
pub fn embed_tcml_source_in_png(bytes: &[u8], source: &str) -> Result<Vec<u8>, String> {
    let decoded = decode_png(bytes).map_err(|error| format!("decode: {error}"))?;
    let mut output = Vec::with_capacity(bytes.len() + source.len() + 64);
    {
        let mut encoder = png::Encoder::new(&mut output, decoded.width, decoded.height);
        encoder.set_color(decoded.color);
        encoder.set_depth(decoded.depth);
        encoder
            .add_itxt_chunk(TCHART_SOURCE_KEYWORD.to_owned(), source.to_owned())
            .map_err(|error| format!("itxt: {error}"))?;
        let mut writer = encoder
            .write_header()
            .map_err(|error| format!("header: {error}"))?;
        writer
            .write_image_data(&decoded.pixels)
            .map_err(|error| format!("data: {error}"))?;
    }
    Ok(output)
}

struct DecodedPng {
    width: u32,
    height: u32,
    color: png::ColorType,
    depth: png::BitDepth,
    pixels: Vec<u8>,
}

fn decode_png(bytes: &[u8]) -> Result<DecodedPng, png::DecodingError> {
    let decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    let mut reader = decoder.read_info()?;
    let (width, height, color, depth) = {
        let info = reader.info();
        (info.width, info.height, info.color_type, info.bit_depth)
    };
    let mut pixels = vec![0u8; reader.output_buffer_size()];
    reader.next_frame(&mut pixels)?;
    Ok(DecodedPng {
        width,
        height,
        color,
        depth,
        pixels,
    })
}
