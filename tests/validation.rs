use twatch::model::{fuzzy_match, validate_torrent_input};

#[test]
fn magnet_link_passes() {
    assert!(validate_torrent_input("magnet:?xt=urn:btih:abc").is_ok());
}

#[test]
fn http_url_passes() {
    assert!(validate_torrent_input("http://example.com/file.torrent").is_ok());
}

#[test]
fn https_url_passes() {
    assert!(validate_torrent_input("https://example.com/file.torrent").is_ok());
}

#[test]
fn empty_string_fails() {
    assert!(validate_torrent_input("").is_err());
    assert!(validate_torrent_input("   ").is_err());
}

#[test]
fn arbitrary_text_fails() {
    assert!(validate_torrent_input("hello world").is_err());
    assert!(validate_torrent_input("ftp://example.com").is_err());
}

#[test]
fn whitespace_is_trimmed() {
    assert!(validate_torrent_input("  magnet:?xt=urn:btih:abc  ").is_ok());
}

#[test]
fn fuzzy_exact_match() {
    assert!(fuzzy_match("bunny", "Big Buck Bunny"));
}

#[test]
fn fuzzy_case_insensitive() {
    assert!(fuzzy_match("BUNNY", "Big Buck Bunny"));
}

#[test]
fn fuzzy_subsequence() {
    assert!(fuzzy_match("bb", "Big Buck Bunny"));
}

#[test]
fn fuzzy_no_match() {
    assert!(!fuzzy_match("xyz", "Big Buck Bunny"));
}

#[test]
fn fuzzy_empty_query() {
    assert!(fuzzy_match("", "anything"));
}

#[test]
fn fuzzy_empty_target() {
    assert!(!fuzzy_match("a", ""));
}

#[test]
fn fuzzy_btih() {
    assert!(fuzzy_match("btih", "magnet:?xt=urn:btih:abcdef1234567890"));
}

#[test]
fn fuzzy_extension() {
    assert!(fuzzy_match("mkv", "season1/episode05.mkv"));
}
