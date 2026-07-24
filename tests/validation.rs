use twatch::model::validate_torrent_input;

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
