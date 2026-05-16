//! Pure SVG-side TCML extractor (no DOM dependency).

const OPEN: &str = "<tchart:source>";
const CLOSE: &str = "</tchart:source>";

/// Find `<tchart:source>...</tchart:source>` in `svg`, XML-unescape the
/// inside text and return it. Returns `None` when the marker is missing or
/// unclosed.
pub fn extract_tcml_source(svg: &str) -> Option<String> {
    let start = svg.find(OPEN)? + OPEN.len();
    let end_relative = svg[start..].find(CLOSE)?;
    Some(unescape_xml(&svg[start..start + end_relative]))
}

fn unescape_xml(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(character) = chars.next() {
        if character != '&' {
            output.push(character);
            continue;
        }
        match read_entity(&mut chars) {
            Some(entity) => append_entity(&mut output, &entity),
            None => output.push('&'),
        }
    }
    output
}

fn read_entity<I: Iterator<Item = char>>(chars: &mut std::iter::Peekable<I>) -> Option<String> {
    let mut entity = String::new();
    for _ in 0..6 {
        match chars.next()? {
            ';' => return Some(entity),
            other => entity.push(other),
        }
    }
    None
}

fn append_entity(output: &mut String, entity: &str) {
    match entity {
        "lt" => output.push('<'),
        "gt" => output.push('>'),
        "amp" => output.push('&'),
        "quot" => output.push('"'),
        "apos" => output.push('\''),
        other => {
            output.push('&');
            output.push_str(other);
            output.push(';');
        }
    }
}
