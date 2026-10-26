//! Cover-art resolution for torrent titles.
//!
//! Providers are intentionally keyless and best-effort:
//! - IMDb suggestions for films and TV
//! - AniList for anime
//! - TVmaze as a TV fallback
//! - Open Library for books
//!
//! Results are cached in `$XDG_CONFIG_HOME/twatch/covers.json` for 30 days.
//! Covers are decorative: every provider failure degrades to `url: None` and
//! the UI falls back to a file-type icon.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use futures_util::stream::{self, StreamExt};
use regex::Regex;
use serde::{Deserialize, Serialize};

const CACHE_TTL_SECS: u64 = 60 * 60 * 24 * 30;
const NEGATIVE_TTL_SECS: u64 = 60 * 60;
const CACHE_VERSION: &str = "v3";
const MAX_QUERY_LEN: usize = 300;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(8);
const CONCURRENCY: usize = 4;

#[derive(Debug, Clone)]
pub struct CoverQuery {
    pub title: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cover {
    /// The original request title, so a batch client can map responses back.
    pub query: String,
    /// The matched title from the provider.
    pub title: String,
    pub url: Option<String>,
    pub provider: String,
    pub year: Option<u32>,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedCover {
    cover: Cover,
    fetched_at: u64,
}

#[derive(Debug, Default, Deserialize)]
struct ImdbResponse {
    #[serde(default)]
    d: Vec<ImdbItem>,
}

#[derive(Debug, Deserialize)]
struct ImdbItem {
    l: String,
    #[serde(default)]
    y: Option<u32>,
    #[serde(default)]
    q: Option<String>,
    #[serde(default)]
    i: Option<ImdbImage>,
}

#[derive(Debug, Deserialize)]
struct ImdbImage {
    #[serde(rename = "imageUrl")]
    image_url: String,
}

#[derive(Debug, Default, Deserialize)]
struct AniListResponse {
    #[serde(default)]
    data: Option<AniListData>,
}

#[derive(Debug, Deserialize)]
struct AniListData {
    #[serde(rename = "Page")]
    page: AniListPage,
}

#[derive(Debug, Deserialize)]
struct AniListPage {
    #[serde(default)]
    media: Vec<AniListMedia>,
}

#[derive(Debug, Deserialize)]
struct AniListMedia {
    title: AniListTitle,
    #[serde(rename = "coverImage")]
    cover_image: Option<AniListCover>,
    #[serde(rename = "startDate")]
    start_date: Option<AniListDate>,
}

#[derive(Debug, Deserialize)]
struct AniListTitle {
    romaji: Option<String>,
    english: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AniListCover {
    #[serde(rename = "extraLarge")]
    extra_large: Option<String>,
    large: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AniListDate {
    year: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct TvMazeResult {
    show: TvMazeShow,
}

#[derive(Debug, Deserialize)]
struct TvMazeShow {
    name: String,
    #[serde(default)]
    premiered: Option<String>,
    #[serde(default)]
    image: Option<TvMazeImage>,
}

#[derive(Debug, Deserialize)]
struct TvMazeImage {
    #[serde(default)]
    medium: Option<String>,
    #[serde(default)]
    original: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct OpenLibraryResponse {
    #[serde(default)]
    docs: Vec<OpenLibraryDoc>,
}

#[derive(Debug, Deserialize)]
struct OpenLibraryDoc {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    cover_i: Option<u64>,
    #[serde(default)]
    first_publish_year: Option<u32>,
}

pub struct CoverService {
    client: reqwest::Client,
    cache: RwLock<HashMap<String, CachedCover>>,
    cache_path: Option<PathBuf>,
}

impl CoverService {
    #[must_use]
    pub fn new(cache_path: PathBuf) -> Self {
        let cache = load_cache(&cache_path);
        let client = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .user_agent("twatch/0.1 (+https://github.com/0Risotto/twatch)")
            .build()
            .unwrap_or_else(|error| {
                tracing::warn!("Cover HTTP client fallback: {error}");
                reqwest::Client::new()
            });
        Self { client, cache: RwLock::new(cache), cache_path: Some(cache_path) }
    }

    /// No network and no persistence, for tests.
    #[must_use]
    pub fn ephemeral() -> Self {
        Self {
            client: reqwest::Client::new(),
            cache: RwLock::new(HashMap::new()),
            cache_path: None,
        }
    }

    /// Resolve one title.  Never returns an error: missing covers are `url: None`.
    pub async fn resolve(&self, query: CoverQuery) -> Cover {
        let title = query.title.trim();
        if title.is_empty() || title.len() > MAX_QUERY_LEN {
            return none_cover(&query);
        }

        let cache_key = format!("{CACHE_VERSION}|{}", normalize(title));
        if let Some(cached) = self.cached(&cache_key) {
            return cached;
        }

        let (clean, year) = clean_title(title);
        if clean.is_empty() {
            return none_cover(&query);
        }

        let lower = title.to_lowercase();
        let kind = query.kind.to_lowercase();
        let is_book = kind == "ebook" || looks_book(&lower);
        let is_anime = looks_anime(title, &lower);
        let is_tv = kind == "tv" || looks_tv(&lower);

        let mut cover = none_cover(&query);
        if is_book {
            if let Some(found) = self.open_library(&clean).await {
                cover = found;
            }
        } else if is_anime || is_tv {
            // Anime and series often have a better poster on AniList. IMDb is
            // still the fallback for live-action TV.
            if let Some(found) = self.anilist(&clean).await {
                cover = found;
            }
        }

        if cover.url.is_none() && !is_book {
            if let Some(found) = self.imdb(&clean, year, is_tv).await {
                cover = found;
            }
        }
        if cover.url.is_none() && is_tv {
            if let Some(found) = self.tvmaze(&clean).await {
                cover = found;
            }
        }
        if cover.url.is_none() && is_book {
            if let Some(found) = self.imdb(&clean, year, false).await {
                cover = found;
            }
        }

        cover.query = query.title.clone();
        cover.kind = query.kind.clone();
        self.store(cache_key, cover.clone());
        cover
    }

    /// Resolve a batch while keeping provider traffic bounded.
    pub async fn resolve_many(&self, queries: &[CoverQuery]) -> Vec<Cover> {
        let covers: Vec<Cover> = stream::iter(queries.iter().cloned())
            .map(|query| async move { self.resolve(query).await })
            .buffer_unordered(CONCURRENCY)
            .collect()
            .await;
        self.persist();
        covers
    }

    fn cached(&self, key: &str) -> Option<Cover> {
        let now = now_secs();
        let cache = self.cache.read().unwrap_or_else(|e| e.into_inner());
        let entry = cache.get(key)?;
        let ttl = if entry.cover.url.is_some() { CACHE_TTL_SECS } else { NEGATIVE_TTL_SECS };
        if now.saturating_sub(entry.fetched_at) > ttl {
            return None;
        }
        Some(entry.cover.clone())
    }

    fn store(&self, key: String, cover: Cover) {
        let mut cache = self.cache.write().unwrap_or_else(|e| e.into_inner());
        cache.insert(key, CachedCover { cover, fetched_at: now_secs() });
    }

    fn persist(&self) {
        let Some(path) = &self.cache_path else {
            return;
        };
        let cache = self.cache.read().unwrap_or_else(|e| e.into_inner());
        let Ok(json) = serde_json::to_string_pretty(&*cache) else {
            return;
        };
        drop(cache);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let tmp = path.with_extension("tmp");
        if std::fs::write(&tmp, json).is_ok() {
            let _ = std::fs::rename(&tmp, path);
        }
    }

    async fn imdb(&self, clean: &str, year: Option<u32>, prefer_tv: bool) -> Option<Cover> {
        let mut url = reqwest::Url::parse("https://v3.sg.media-imdb.com/suggestion/x/").ok()?;
        url.path_segments_mut().ok()?.push(&format!("{clean}.json"));
        let response = self.client.get(url).send().await.ok()?.error_for_status().ok()?;
        let body: ImdbResponse = response.json().await.ok()?;

        let mut fallback: Option<Cover> = None;
        for item in body.d {
            let Some(image) = item.i else { continue };
            let kind = item.q.as_deref().unwrap_or("");
            let cover = Cover {
                query: clean.to_owned(),
                title: item.l,
                url: Some(image.image_url),
                provider: "imdb".into(),
                year: item.y.or(year),
                kind: if kind.contains("TV") { "tv".into() } else { "movie".into() },
            };
            if prefer_tv && kind.contains("TV") {
                return Some(cover);
            }
            if !prefer_tv && matches!(kind, "feature" | "tvMovie" | "short" | "video") {
                return Some(cover);
            }
            if fallback.is_none() {
                fallback = Some(cover);
            }
            if !prefer_tv && fallback.is_some() {
                return fallback;
            }
        }
        fallback
    }

    async fn anilist(&self, clean: &str) -> Option<Cover> {
        const QUERY: &str = r#"query ($search: String) {
          Page(perPage: 4) {
            media(search: $search, type: ANIME) {
              title { romaji english }
              coverImage { extraLarge large }
              startDate { year }
            }
          }
        }"#;
        let response = self
            .client
            .post("https://graphql.anilist.co")
            .json(&serde_json::json!({ "query": QUERY, "variables": { "search": clean } }))
            .send()
            .await
            .ok()?
            .error_for_status()
            .ok()?;
        let body: AniListResponse = response.json().await.ok()?;
        let media = body.data?.page.media.into_iter().find(|media| {
            let has_cover = media
                .cover_image
                .as_ref()
                .is_some_and(|cover| cover.extra_large.is_some() || cover.large.is_some());
            if !has_cover {
                return false;
            }
            let candidate = media
                .title
                .english
                .as_deref()
                .or(media.title.romaji.as_deref())
                .unwrap_or_default();
            similarity(&normalize(clean), &normalize(candidate)) >= 0.5
        })?;
        let cover = media.cover_image?;
        Some(Cover {
            query: clean.to_owned(),
            title: media.title.english.or(media.title.romaji).unwrap_or_else(|| clean.to_owned()),
            url: cover.extra_large.or(cover.large),
            provider: "anilist".into(),
            year: media.start_date.and_then(|date| date.year),
            kind: "anime".into(),
        })
    }

    async fn tvmaze(&self, clean: &str) -> Option<Cover> {
        let response = self
            .client
            .get("https://api.tvmaze.com/search/shows")
            .query(&[("q", clean)])
            .send()
            .await
            .ok()?
            .error_for_status()
            .ok()?;
        let results: Vec<TvMazeResult> = response.json().await.ok()?;
        let show = results.into_iter().next()?.show;
        let image = show.image?;
        let year = show
            .premiered
            .as_deref()
            .and_then(|date| date.get(..4))
            .and_then(|year| year.parse().ok());
        Some(Cover {
            query: clean.to_owned(),
            title: show.name,
            url: image.original.or(image.medium),
            provider: "tvmaze".into(),
            year,
            kind: "tv".into(),
        })
    }

    async fn open_library(&self, clean: &str) -> Option<Cover> {
        let response = self
            .client
            .get("https://openlibrary.org/search.json")
            .query(&[("q", clean), ("limit", "1")])
            .send()
            .await
            .ok()?
            .error_for_status()
            .ok()?;
        let body: OpenLibraryResponse = response.json().await.ok()?;
        let doc = body.docs.into_iter().next()?;
        let cover_id = doc.cover_i?;
        Some(Cover {
            query: clean.to_owned(),
            title: doc.title.unwrap_or_else(|| clean.to_owned()),
            url: Some(format!("https://covers.openlibrary.org/b/id/{cover_id}-L.jpg")),
            provider: "openlibrary".into(),
            year: doc.first_publish_year,
            kind: "ebook".into(),
        })
    }
}

fn none_cover(query: &CoverQuery) -> Cover {
    Cover {
        query: query.title.clone(),
        title: query.title.clone(),
        url: None,
        provider: "none".into(),
        year: None,
        kind: query.kind.clone(),
    }
}

fn load_cache(path: &PathBuf) -> HashMap<String, CachedCover> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|contents| serde_json::from_str(&contents).ok())
        .unwrap_or_default()
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

fn normalize(value: &str) -> String {
    value
        .to_lowercase()
        .chars()
        .map(|character| if character.is_alphanumeric() { character } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn similarity(query: &str, candidate: &str) -> f64 {
    if query == candidate {
        return 1.0;
    }
    if candidate.contains(query) || query.contains(candidate) {
        return 0.9;
    }
    let query_tokens: Vec<&str> = query.split_whitespace().collect();
    if query_tokens.is_empty() {
        return 0.0;
    }
    let candidate_tokens: Vec<&str> = candidate.split_whitespace().collect();
    let common = query_tokens.iter().filter(|token| candidate_tokens.contains(token)).count();
    common as f64 / query_tokens.len() as f64
}

fn looks_tv(lower: &str) -> bool {
    static TV: OnceLock<Regex> = OnceLock::new();
    TV.get_or_init(|| {
        Regex::new(r"(?i)\b(s\d{1,2}(e\d{1,3})?|season\s*\d+|complete series|episode\s*\d+)\b")
            .expect("valid TV regex")
    })
    .is_match(lower)
}

fn looks_book(lower: &str) -> bool {
    lower.contains("ebook")
        || lower.contains("epub")
        || lower.contains("mobi")
        || lower.contains(" pdf")
        || lower.contains("novel")
        || lower.contains("bible")
        || lower.contains("handbook")
}

fn looks_anime(raw: &str, lower: &str) -> bool {
    lower.contains("anime")
        || lower.contains("subsplease")
        || lower.contains("erai-raws")
        || lower.contains("horriblesubs")
        || lower.contains("nyaa")
        || lower.contains("[judas]")
        || lower.contains("jpn")
        || lower.contains("japanese")
        || lower.contains("dual audio")
        || raw.chars().any(|c| matches!(c, '\u{3040}'..='\u{30ff}' | '\u{4e00}'..='\u{9fff}'))
}

fn clean_title(raw: &str) -> (String, Option<u32>) {
    static YEAR: OnceLock<Regex> = OnceLock::new();
    static BRACKETS: OnceLock<Regex> = OnceLock::new();
    static JUNK: OnceLock<Regex> = OnceLock::new();
    static SERIES: OnceLock<Regex> = OnceLock::new();
    static SPACES: OnceLock<Regex> = OnceLock::new();

    let year_re = YEAR.get_or_init(|| Regex::new(r"\b(19|20)\d{2}\b").expect("year regex"));
    let brackets =
        BRACKETS.get_or_init(|| Regex::new(r"[\[\(\{][^\]\)\}]*[\]\)\}]").expect("brackets regex"));
    let junk = JUNK.get_or_init(|| {
        Regex::new(
            r"(?i)\b(2160p|1080p|720p|480p|web[- .]?dl|webrip|bluray|blu-ray|bdrip|brrip|hdtv|dvdrip|remux|x264|x265|h\.?264|h\.?265|hevc|avc|aac\d?|ac3|dts|ddp?5\.1|10bit|8bit|hdr|sdr|proper|repack|extended|unrated|imax|multi|dual audio|subbed|dubbed|complete|playweb|sensei|ntb|tgx|eztv|rarbg|yify|yts|criterion|cr|season\s*\d+|s\d{1,2}(e\d{1,3})?|episode\s*\d+|mkv|mp4|avi|mov|m4v|webm|flv|wmv|m2ts|iso|srt|ass|vtt|pdf|epub|mobi|azw3?|zip|rar|7z|tar|gz|flac|mp3|aac|wav|jpg|jpeg|png|webp|gif|www\.\S+|\S+\.(com|org|net|to|io|me|cc|tv|xyz|info|club|site|ru|ch|li|se|is))\b",
        )
        .expect("junk regex")
    });
    let series = SERIES.get_or_init(|| {
        Regex::new(r"(?i)\b(s\d{1,2}(e\d{1,3})?(\s*-\s*s?\d{1,2})?|season\s*\d+|episode\s*\d+)\b")
            .expect("series regex")
    });
    let spaces = SPACES.get_or_init(|| Regex::new(r"\s+").expect("spaces regex"));

    let year = year_re.find(raw).and_then(|matched| matched.as_str().parse().ok());
    let without_brackets = brackets.replace_all(raw, " ");
    // For a series, everything after the season/episode marker is episode
    // metadata, not part of the show title.  This is what lets AniList/TVmaze
    // resolve "Jujutsu Kaisen S03E01 Execution..." to the right show.
    let head = series
        .find(&without_brackets)
        .map_or_else(|| without_brackets.as_ref(), |matched| &without_brackets[..matched.start()]);
    let without_junk = junk.replace_all(head, " ");
    let without_year = year_re.replace_all(&without_junk, " ");
    let cleaned = without_year.replace(['.', '_', '-'], " ");
    let cleaned = spaces.replace_all(&cleaned, " ").trim().to_owned();
    (cleaned.trim_matches(|c: char| !c.is_alphanumeric()).to_owned(), year)
}

#[cfg(test)]
mod tests {
    use super::{clean_title, looks_book, looks_tv};

    #[test]
    fn cleans_release_titles() {
        let (title, year) =
            clean_title("Perfume.The.Story.of.a.Murderer.2006.1080p.BluRay.x265-HEVC.mkv");
        assert_eq!(title, "Perfume The Story of a Murderer");
        assert_eq!(year, Some(2006));
    }

    #[test]
    fn cleans_anime_series() {
        let (title, _) = clean_title(
            "Jujutsu Kaisen S03E01 Execution REPACK 1080p CR WEB-DL AAC2 0 H 264-playWEB",
        );
        assert_eq!(title, "Jujutsu Kaisen");
    }

    #[tokio::test]
    #[ignore = "live provider test"]
    async fn live_anilist_resolves() {
        let service = super::CoverService::ephemeral();
        let cover = service
            .resolve(super::CoverQuery {
                title:
                    "Jujutsu Kaisen S03E01 Execution REPACK 1080p CR WEB-DL AAC2 0 H 264-playWEB"
                        .into(),
                kind: "tv".into(),
            })
            .await;
        eprintln!("{:?}", cover);
        assert!(cover.url.is_some());
    }

    #[test]
    fn detects_tv_and_books() {
        assert!(looks_tv("planet earth s01e02 2160p"));
        assert!(looks_tv("modern family season 1"));
        assert!(looks_book("ubuntu linux bible pdf"));
    }
}
