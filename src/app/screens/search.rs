//! The search popup screen.

use super::registry;
use super::{Screen, Transition};
use crate::app::App;
use crate::domain::common::fuzzy_match;
use crate::domain::common::paging::Page;
use crate::domain::common::percent_encode;
use crate::domain::search::{SEARCH_PAGE_SIZE, SearchConfig, SearchResult};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::any::Any;

/// View model for [`SearchScreen`].
pub struct ViewModel<'a> {
    pub query: &'a str,
    pub fetched_query: &'a str,
    pub busy: bool,
    pub page: usize,
    pub items: Vec<&'a SearchResult>,
    pub page_total: usize,
    pub filtering: bool,
    pub selected: usize,
    pub config_open: bool,
    pub config_input: &'a str,
    pub has_config: bool,
}

/// Searches a user-configured API; results can be added as magnets.
pub struct SearchScreen {
    query: String,
    fetched_query: String,
    selected: usize,
    busy: bool,
    page: usize,
    all_results: Vec<SearchResult>,
    config: SearchConfig,
    config_open: bool,
    config_input: String,
}

impl SearchScreen {
    #[must_use]
    pub const fn new(config: SearchConfig) -> Self {
        Self {
            query: String::new(),
            fetched_query: String::new(),
            selected: 0,
            busy: false,
            page: 0,
            all_results: Vec::new(),
            config,
            config_open: false,
            config_input: String::new(),
        }
    }

    /// Apply results delivered by the search background flow.
    pub fn apply_results(&mut self, results: Vec<SearchResult>) {
        self.busy = false;
        self.all_results = results;
        self.selected = 0;
        self.page = 0;
    }

    /// True while the live query narrows the fetched results.
    fn filtering(&self) -> bool {
        !self.query.is_empty() && self.query != self.fetched_query
    }

    /// All results passing the current live filter, in list order.
    fn filtered(&self) -> Vec<&SearchResult> {
        if self.filtering() {
            self.all_results.iter().filter(|r| fuzzy_match(&self.query, &r.name)).collect()
        } else {
            self.all_results.iter().collect()
        }
    }

    /// The current page of the filtered list plus the total page count.
    /// Selection, navigation, and Enter all operate on this single
    /// view so the highlighted row is always the acted-on row.
    fn visible_page(&self) -> (Vec<&SearchResult>, usize) {
        let filtered = self.filtered();
        let page = Page::new(&filtered, self.page, SEARCH_PAGE_SIZE);
        (page.items().to_vec(), page.total_pages())
    }

    fn clamp_selection(&mut self) {
        let max = self.visible_page().0.len().saturating_sub(1);
        if self.selected > max {
            self.selected = max;
        }
    }

    fn move_up(&mut self) {
        let len = self.visible_page().0.len();
        if len == 0 {
            return;
        }
        if self.selected > 0 {
            self.selected -= 1;
        } else {
            self.selected = len - 1;
        }
    }

    fn move_down(&mut self) {
        let len = self.visible_page().0.len();
        if len == 0 {
            return;
        }
        if self.selected < len - 1 {
            self.selected += 1;
        } else {
            self.selected = 0;
        }
    }

    /// Fetch a query (or select a result when it was already fetched).
    fn enter(&mut self, app: &mut App) -> Option<Transition> {
        let items = self.visible_page().0;
        let select_result = !items.is_empty() && self.query == self.fetched_query;

        if select_result {
            let (hash, name) =
                items.get(self.selected).map(|r| (r.info_hash.clone(), r.name.clone()))?;
            let magnet = format!("magnet:?xt=urn:btih:{hash}&dn={}", percent_encode(&name));
            app.status_message = format!("Ready: {name}");
            return Some(Transition::Push(registry::input(magnet)));
        }

        let query = self.query.clone();
        if !query.is_empty() && !self.busy {
            self.busy = true;
            self.page = 0;
            self.fetched_query = query.clone();
            self.all_results.clear();
            self.selected = 0;
            app.enqueue_search(query, 0);
        }
        None
    }

    fn handle_input(&mut self, app: &mut App, key: KeyEvent) -> Option<Transition> {
        if self.config_open {
            return self.handle_config_input(app, key.code);
        }
        let code = key.code;
        match code {
            KeyCode::Esc => Some(Transition::Pop),
            KeyCode::Enter => self.enter(app),
            KeyCode::Up => {
                self.move_up();
                None
            }
            KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.move_up();
                None
            }
            KeyCode::Down => {
                self.move_down();
                None
            }
            KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.move_down();
                None
            }
            KeyCode::Left => {
                let (_, total_pages) = self.visible_page();
                if self.page > 0 && total_pages > 0 {
                    self.page -= 1;
                    self.selected = 0;
                }
                None
            }
            KeyCode::Right => {
                let (_, total_pages) = self.visible_page();
                if self.page + 1 < total_pages {
                    self.page += 1;
                    self.selected = 0;
                }
                None
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.query.clear();
                self.fetched_query.clear();
                self.all_results.clear();
                self.selected = 0;
                self.page = 0;
                None
            }
            KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.config_open = true;
                self.config_input = self.config.url.clone();
                None
            }
            KeyCode::Backspace => {
                self.query.pop();
                self.clamp_selection();
                None
            }
            KeyCode::Char(ch) if self.query.len() < 256 => {
                self.query.push(ch);
                self.clamp_selection();
                None
            }
            _ => None,
        }
    }

    fn handle_config_input(&mut self, app: &mut App, code: KeyCode) -> Option<Transition> {
        match code {
            KeyCode::Esc => {
                self.config_open = false;
                None
            }
            KeyCode::Enter => {
                let url = self.config_input.trim().to_string();
                self.config.url = url;
                self.config_open = false;
                if let Err(e) = self.config.save(&app.config.config_dir) {
                    app.set_error(format!("Search API set, but failed to persist: {e}"));
                }
                app.search_config = self.config.clone();
                app.status_message = "Search API configured".into();
                None
            }
            KeyCode::Backspace => {
                self.config_input.pop();
                None
            }
            KeyCode::Char(c) if self.config_input.len() < 2048 => {
                self.config_input.push(c);
                None
            }
            _ => None,
        }
    }
}

impl Screen for SearchScreen {
    fn handle_key(&mut self, app: &mut App, key: KeyEvent) -> Option<Transition> {
        self.handle_input(app, key)
    }

    fn render(&self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect, app: &App) {
        crate::ui::search_popup::render(frame, area, app, self.view(app));
    }

    fn is_modal(&self) -> bool {
        true
    }

    fn renders_full_frame(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl SearchScreen {
    /// Build the view model rendered by the presentation layer.
    #[must_use]
    pub fn view<'a>(&'a self, _app: &'a App) -> ViewModel<'a> {
        let (items, _) = self.visible_page();
        let page_total = items.len();

        ViewModel {
            query: &self.query,
            fetched_query: &self.fetched_query,
            busy: self.busy,
            page: self.page,
            selected: self.selected.min(page_total.saturating_sub(1)),
            items,
            page_total,
            filtering: self.filtering(),
            config_open: self.config_open,
            config_input: &self.config_input,
            has_config: !self.config.url.is_empty(),
        }
    }
}
