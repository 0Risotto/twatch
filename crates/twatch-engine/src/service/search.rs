//! Default [`SearchPort`](twatch_core::domain::search::ports::SearchPort) implementation.
//!
//! Queries a user-configured JSON API via reqwest and maps the response
//! using a configurable field map.

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde_json::Value;
use shaku::{Component, Module, ModuleBuildContext};
use std::time::Duration;
use twatch_core::domain::search::ports::SearchPort;
use twatch_core::domain::search::{SearchConfig, SearchResult};

/// Give up on a hung search API after this long.
const SEARCH_TIMEOUT: Duration = Duration::from_secs(15);

/// Truncate oversized error payloads before they reach the status bar.
const MAX_ERROR_PREVIEW: usize = 200;

pub struct SearchServiceImpl {
    client: reqwest::Client,
}

impl Default for SearchServiceImpl {
    fn default() -> Self {
        let client =
            reqwest::Client::builder().timeout(SEARCH_TIMEOUT).build().unwrap_or_else(|e| {
                tracing::error!("Failed to build HTTP client, using defaults: {e}");
                reqwest::Client::new()
            });
        Self { client }
    }
}

impl<M: Module> Component<M> for SearchServiceImpl {
    type Interface = dyn SearchPort;
    type Parameters = ();

    fn build(
        _context: &mut ModuleBuildContext<M>,
        _params: Self::Parameters,
    ) -> Box<Self::Interface> {
        Box::new(Self::default())
    }
}

#[async_trait]
impl SearchPort for SearchServiceImpl {
    async fn search(
        &self,
        query: &str,
        config: &SearchConfig,
        page: u32,
    ) -> Result<Vec<SearchResult>> {
        if config.url.is_empty() {
            return Ok(Vec::new());
        }

        let url =
            config.url.replace("{query}", &urlencoding(query)).replace("{page}", &page.to_string());

        let mut req = self.client.get(&url);

        for (key, value) in &config.headers {
            req = req.header(key.as_str(), value.as_str());
        }

        let resp = req.send().await.context("Search request failed")?;
        if !resp.status().is_success() {
            return Err(anyhow::anyhow!("Search API returned HTTP {}", resp.status()));
        }
        let body = resp.text().await.context("Failed to read response")?;

        let json: Value =
            serde_json::from_str(&body).context("Search response is not valid JSON")?;

        let array = match &json {
            Value::Array(arr) => arr.clone(),
            _ => {
                let mut preview = json.to_string();
                if preview.len() > MAX_ERROR_PREVIEW {
                    preview.truncate(MAX_ERROR_PREVIEW);
                    preview.push('…');
                }
                return Err(anyhow::anyhow!("Expected a JSON array, got {preview}"));
            }
        };

        let fm = &config.field_map;
        let results: Vec<SearchResult> = array
            .iter()
            .filter_map(|item| {
                let name = string_val(item, &fm.name)?;
                let info_hash = string_val(item, &fm.info_hash)?;
                let seeders = int_val(item, &fm.seeders).unwrap_or(0);
                let leechers = int_val(item, &fm.leechers).unwrap_or(0);
                let size = int_val(item, &fm.size).unwrap_or(0);
                let username = string_val(item, &fm.username).unwrap_or_default();
                if is_blocked_category(
                    &category_values(item, &fm.category),
                    &config.blocked_categories,
                ) {
                    return None;
                }

                Some(SearchResult { name, info_hash, seeders, leechers, size, username })
            })
            .collect();

        Ok(results)
    }
}

fn string_val(item: &Value, field: &str) -> Option<String> {
    item.get(field).and_then(|v| v.as_str()).map(|s| s.to_string())
}

fn category_values(item: &Value, field: &str) -> Vec<String> {
    let Some(value) = item.get(field) else {
        return Vec::new();
    };
    match value {
        Value::Array(values) => values.iter().filter_map(scalar_string).collect(),
        other => scalar_string(other).into_iter().collect(),
    }
}

fn scalar_string(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.clone()),
        Value::Number(value) => Some(value.to_string()),
        _ => None,
    }
}

fn is_blocked_category(values: &[String], rules: &[String]) -> bool {
    values.iter().any(|value| {
        let value = value.trim().to_lowercase();
        let numeric = value.parse::<u64>().ok();
        rules.iter().any(|rule| rule_matches(&value, numeric, rule))
    })
}

fn rule_matches(value: &str, numeric: Option<u64>, rule: &str) -> bool {
    let rule = rule.trim().to_lowercase();
    if rule.is_empty() {
        return false;
    }
    if let Some((start, end)) = parse_range(&rule) {
        return numeric.is_some_and(|category| (start..=end).contains(&category));
    }
    if let Ok(category) = rule.parse::<u64>() {
        return numeric == Some(category);
    }
    value.contains(&rule)
}

fn parse_range(rule: &str) -> Option<(u64, u64)> {
    let (start, end) = rule.split_once('-')?;
    let start = start.trim().parse().ok()?;
    let end = end.trim().parse().ok()?;
    (start <= end).then_some((start, end))
}

fn int_val(item: &Value, field: &str) -> Option<u64> {
    let v = item.get(field)?;
    if let Some(n) = v.as_u64() {
        return Some(n);
    }
    v.as_str().and_then(|s| s.parse().ok())
}

fn urlencoding(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for &b in s.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            b' ' => out.push('+'),
            _ => {
                out.push('%');
                out.push(hex(b >> 4));
                out.push(hex(b & 0x0F));
            }
        }
    }
    out
}

const fn hex(n: u8) -> char {
    match n {
        0..=9 => (b'0' + n) as char,
        _ => (b'A' + (n - 10)) as char,
    }
}

#[cfg(test)]
mod tests {
    use super::{category_values, int_val, is_blocked_category};
    use serde_json::json;

    #[test]
    fn category_values_accept_strings_numbers_and_arrays() {
        assert_eq!(int_val(&json!({"category": "505"}), "category"), Some(505));
        assert_eq!(int_val(&json!({"category": 505}), "category"), Some(505));
        assert_eq!(int_val(&json!({}), "category"), None);
        assert_eq!(
            category_values(&json!({"category": ["video", 505]}), "category"),
            vec!["video", "505"]
        );
    }

    #[test]
    fn category_rules_accept_numbers_ranges_and_words() {
        assert!(is_blocked_category(&["700".into()], &["700".into()]));
        assert!(is_blocked_category(&["750".into()], &["700-799".into()]));
        assert!(!is_blocked_category(&["699".into()], &["700-799".into()]));
        assert!(is_blocked_category(&["blocked-tag".into()], &["blocked".into()]));
        assert!(!is_blocked_category(&["video".into()], &["blocked".into()]));
        assert!(!is_blocked_category(&["700".into()], &[]));
    }
}
