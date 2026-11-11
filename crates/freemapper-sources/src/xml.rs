//! What the XML readers share.

/// The character an XML entity reference (`amp`, `#228`, `#xE4`) stands for.
pub fn resolve_entity(name: &str) -> Option<char> {
    match name {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        _ => name
            .strip_prefix("#x")
            .and_then(|hex| u32::from_str_radix(hex, 16).ok())
            .or_else(|| name.strip_prefix('#').and_then(|dec| dec.parse().ok()))
            .and_then(char::from_u32),
    }
}
