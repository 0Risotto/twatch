//! Zero-copy pagination over a slice.
//!
//! A single source of truth for page-slicing math so callers never
//! duplicate `div_ceil`/clamp logic (or magic page sizes).

/// A window of `page_size` items inside a larger collection.
#[derive(Debug, Clone, Copy)]
pub struct Page<'a, T> {
    items: &'a [T],
    index: usize,
    total_pages: usize,
}

impl<'a, T> Page<'a, T> {
    /// Create the page at `index` (0-based) over `all` items.
    ///
    /// An out-of-range index produces an empty page.
    #[must_use]
    pub fn new(all: &'a [T], index: usize, page_size: usize) -> Self {
        let page_size = page_size.max(1);
        let total_pages = if all.is_empty() { 0 } else { all.len().div_ceil(page_size) };
        let start = index.min(total_pages.saturating_sub(1)).saturating_mul(page_size);
        let end = start.saturating_add(page_size).min(all.len());
        Self { items: &all[start..end], index, total_pages }
    }

    /// The items on this page.
    #[must_use]
    pub const fn items(&self) -> &'a [T] {
        self.items
    }

    /// This page's 0-based index.
    #[must_use]
    pub const fn index(&self) -> usize {
        self.index
    }

    /// Total number of pages for the underlying collection.
    #[must_use]
    pub const fn total_pages(&self) -> usize {
        self.total_pages
    }

    /// True when there is a page before this one.
    #[must_use]
    pub const fn can_go_back(&self) -> bool {
        self.index > 0
    }

    /// True when there is a page after this one.
    #[must_use]
    pub const fn can_go_forward(&self) -> bool {
        self.index + 1 < self.total_pages
    }
}
