//! Torrent search domain.

pub mod ports;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Number of results shown per page in the search popup.
pub const SEARCH_PAGE_SIZE: usize = 21;

/// A search result from the user-configured JSON API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub name: String,
    pub info_hash: String,
    pub seeders: u64,
    pub leechers: u64,
    pub size: u64,
    pub username: String,
}

fn default_name() -> String {
    "name".into()
}
fn default_info_hash() -> String {
    "info_hash".into()
}
fn default_seeders() -> String {
    "seeders".into()
}
fn default_leechers() -> String {
    "leechers".into()
}
fn default_size() -> String {
    "size".into()
}
fn default_username() -> String {
    "username".into()
}

/// Maps the user-configured API response fields to the fields twatch
/// cares about.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMap {
    #[serde(default = "default_name")]
    pub name: String,
    #[serde(default = "default_info_hash")]
    pub info_hash: String,
    #[serde(default = "default_seeders")]
    pub seeders: String,
    #[serde(default = "default_leechers")]
    pub leechers: String,
    #[serde(default = "default_size")]
    pub size: String,
    #[serde(default = "default_username")]
    pub username: String,
}

impl Default for FieldMap {
    fn default() -> Self {
        Self {
            name: default_name(),
            info_hash: default_info_hash(),
            seeders: default_seeders(),
            leechers: default_leechers(),
            size: default_size(),
            username: default_username(),
        }
    }
}

/// Search API configuration.
///
/// A pure value object owned by the domain layer; the persistence
/// functions ([`SearchConfig::load`], [`SearchConfig::save`]) are
/// implemented by the config layer.  Runtime state (such as the
/// current page) never lives here — it is passed to the search port
/// per request.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchConfig {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub field_map: FieldMap,
}
