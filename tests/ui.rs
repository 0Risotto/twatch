//! Presentation-layer tests: which screens are visible when rendering.

use crossterm::event::KeyEvent;
use ratatui::Frame;
use std::any::Any;
use twatch::app::App;
use twatch::app::screens::{Screen, Transition};
use twatch::ui::base_screen_idx;

struct FakeScreen {
    modal: bool,
}

impl Screen for FakeScreen {
    fn handle_key(&mut self, _app: &mut App, _key: KeyEvent) -> Option<Transition> {
        None
    }

    fn render(&self, _frame: &mut Frame, _area: ratatui::layout::Rect, _app: &App) {}

    fn is_modal(&self) -> bool {
        self.modal
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[test]
fn base_is_topmost_non_modal() {
    let welcome = FakeScreen { modal: false };
    let history = FakeScreen { modal: false };
    let popup = FakeScreen { modal: true };

    // Single base screen.
    let stack: Vec<&dyn Screen> = vec![&welcome];
    assert_eq!(base_screen_idx(&stack), Some(0));

    // Non-modal on top hides the screen below it.
    let stack: Vec<&dyn Screen> = vec![&welcome, &history];
    assert_eq!(base_screen_idx(&stack), Some(1));

    // A modal on top keeps the base below visible.
    let stack: Vec<&dyn Screen> = vec![&welcome, &history, &popup];
    assert_eq!(base_screen_idx(&stack), Some(1));

    // A lone modal still exposes the root welcome below it.
    let stack: Vec<&dyn Screen> = vec![&welcome, &popup];
    assert_eq!(base_screen_idx(&stack), Some(0));
}
