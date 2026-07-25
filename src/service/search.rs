//! Default [`SearchService`](crate::traits::SearchService) implementation.
//!
//! Queries a user-configured JSON API via reqwest and maps the response
//! using a configurable field map.

use crate::config::search::SearchConfig;
use crate::model::SearchResult;
use crate::traits::SearchService;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde_json::Value;
use shaku::Component;

#[derive(Component)]
#[shaku(interface = SearchService)]
pub struct SearchServiceImpl;

#[async_trait]
impl SearchService for SearchServiceImpl {
    async fn search(&self, query: &str, config: &SearchConfig) -> Result<Vec<SearchResult>> {
        if config.url.is_empty() {
            return Ok(Vec::new());
        }

        let url = config
            .url
            .replace("{query}", &urlencoding(query))
            .replace("{page}", &config.page.to_string());

        let mut req = reqwest::Client::new().get(&url);

        for (key, value) in &config.headers {
            req = req.header(key.as_str(), value.as_str());
        }

        let resp = req.send().await.context("Search request failed")?;
        let body = resp.text().await.context("Failed to read response")?;

        let json: Value =
            serde_json::from_str(&body).context("Search response is not valid JSON")?;

        let array = match &json {
            Value::Array(arr) => arr.clone(),
            _ => {
                return Err(anyhow::anyhow!("Expected a JSON array, got {}", json));
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

                Some(SearchResult { name, info_hash, seeders, leechers, size, username })
            })
            .collect();

        Ok(results)
    }
}

fn string_val(item: &Value, field: &str) -> Option<String> {
    item.get(field).and_then(|v| v.as_str()).map(|s| s.to_string())
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
