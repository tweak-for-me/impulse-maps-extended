//! Hand-written scanning of markup; no HTML parser.

/// The text between the first `start` and the next `end` after it.
pub fn between<'a>(text: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let from = text.find(start)? + start.len();
    let len = text[from..].find(end)?;
    Some(&text[from..from + len])
}

/// A URL path segment that is safe to put in a request: what a slug is made of.
pub fn is_slug(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

pub fn unescape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find('&') {
        out.push_str(&rest[..at]);
        rest = &rest[at..];
        let entity = rest.find(';').filter(|&end| end <= 10).and_then(|end| Some((decode(&rest[1..end])?, end)));
        if let Some((c, end)) = entity {
            out.push(c);
            rest = &rest[end + 1..];
        } else {
            out.push('&');
            rest = &rest[1..];
        }
    }
    out.push_str(rest);
    out
}

fn decode(name: &str) -> Option<char> {
    let code = match name {
        "amp" => return Some('&'),
        "lt" => return Some('<'),
        "gt" => return Some('>'),
        "quot" => return Some('"'),
        "apos" => return Some('\''),
        _ => name.strip_prefix('#')?,
    };
    let number = match code.strip_prefix(['x', 'X']) {
        Some(hex) => u32::from_str_radix(hex, 16),
        None => code.parse(),
    };
    char::from_u32(number.ok()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entities_decode_and_unknown_ones_stay() {
        assert_eq!(unescape("Assassin&#039;s &amp; &#x41; &bogus; a&b"), "Assassin's & A &bogus; a&b");
    }

    #[test]
    fn between_stops_at_the_first_end_after_the_start() {
        assert_eq!(between("<h4>a</h4><h4>b</h4>", "<h4>", "</h4>"), Some("a"));
        assert_eq!(between("<h4>a", "<h4>", "</h4>"), None);
    }

    #[test]
    fn a_slug_is_a_safe_path_segment() {
        assert!(is_slug("night-city_2"));
        assert!(!is_slug("") && !is_slug("a/b") && !is_slug("a?b"));
    }
}
