//! Domain-agnostic utilities shared across domains.

pub mod paging;
pub mod spawner;

/// Subsequence fuzzy match.  Returns `true` when every character in
/// `query` appears in `target` in order (case-insensitive).
#[must_use]
pub fn fuzzy_match(query: &str, target: &str) -> bool {
    let query_lower = query.to_lowercase();
    let mut qchars = query_lower.chars().peekable();
    let target_lower = target.to_lowercase();
    for c in target_lower.chars() {
        if qchars.peek() == Some(&c) {
            qchars.next();
        }
    }
    qchars.peek().is_none()
}

/// Percent-encode every byte outside the RFC 3986 unreserved set.
///
/// Used for magnet link display names and similar URL components
/// where space must stay `%20` (not `+`, which means space only in
/// form-encoded query strings).
#[must_use]
pub fn percent_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for &b in input.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => {
                out.push('%');
                out.push(hex_digit(b >> 4));
                out.push(hex_digit(b & 0x0F));
            }
        }
    }
    out
}

const fn hex_digit(nibble: u8) -> char {
    match nibble {
        0..=9 => (b'0' + nibble) as char,
        _ => (b'A' + (nibble - 10)) as char,
    }
}

/// Validate a user-supplied torrent URL / magnet link.
///
/// Returns `Err(message)` if the input is empty or does not look like
/// a recognised torrent identifier.
pub fn validate_torrent_input(input: &str) -> Result<(), &'static str> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("URL cannot be empty");
    }
    if trimmed.starts_with("magnet:") {
        return Ok(());
    }
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return Ok(());
    }
    Err("Must be a magnet link or an http(s) URL")
}
