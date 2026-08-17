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
