pub(super) fn valid_key(key: &str) -> bool {
    !key.is_empty()
        && key.len() <= 160
        && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

pub(super) fn valid_file(file: &str) -> bool {
    !file.is_empty()
        && !file.contains("..")
        && !file.contains('/')
        && !file.contains('\\')
        && (file.ends_with(".m3u8") || file.ends_with(".ts"))
}
