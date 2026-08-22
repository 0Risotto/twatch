//! `InputState` cursor/editing behaviour, including multibyte safety.

use proptest::prelude::*;
use twatch::app::state::{InputState, MAX_INPUT_LEN};

#[test]
fn new_starts_cursor_at_end() {
    let input = InputState::new("abc");
    assert_eq!(input.value, "abc");
    assert_eq!(input.cursor, 3);
}

#[test]
fn push_char_inserts_at_cursor_not_end() {
    let mut input = InputState::new("ac");
    input.cursor_left();
    input.push_char('b');
    assert_eq!(input.value, "abc");
    assert_eq!(input.cursor, 2);
}

#[test]
fn backspace_removes_char_before_cursor() {
    let mut input = InputState::new("abc");
    input.cursor_left();
    input.backspace();
    assert_eq!(input.value, "ac");
    assert_eq!(input.cursor, 1);
}

#[test]
fn backspace_at_start_is_noop() {
    let mut input = InputState::new("abc");
    while input.cursor > 0 {
        input.cursor_left();
    }
    input.backspace();
    assert_eq!(input.value, "abc");
}

#[test]
fn cursor_never_exceeds_length() {
    let mut input = InputState::new("ab");
    input.cursor_right();
    input.cursor_right();
    input.cursor_right();
    assert_eq!(input.cursor, 2);
}

#[test]
fn backspace_with_corrupted_midchar_cursor_does_not_panic() {
    // Regression: cursor used to move by BYTES, so a Left press could
    // land mid-'é' and Backspace panicked on a non-char-boundary
    // remove.  Simulate the corrupted cursor directly.
    let mut input = InputState { value: "éa".into(), cursor: 1 };
    input.backspace();
    // Cursor snaps forward past 'é'; backspace then deletes 'é'.
    assert_eq!(input.value, "a");
}

#[test]
fn multibyte_insert_and_navigation() {
    let mut input = InputState::new("héllo"); // boundaries: 0,1,3,4,5,6
    for _ in 0..4 {
        input.cursor_left(); // land between h and é
    }
    input.push_char('x');
    assert_eq!(input.value, "hxéllo");
    input.backspace();
    assert_eq!(input.value, "héllo");
}

#[test]
fn push_char_respects_max_len_including_multibyte() {
    let mut input = InputState::new("");
    for _ in 0..MAX_INPUT_LEN / 4 {
        input.push_char('🦀'); // 4 bytes each
    }
    let full_len = input.value.len();
    input.push_char('x');
    assert_eq!(input.value.len(), full_len.min(MAX_INPUT_LEN));
}

proptest! {
    /// Any sequence of edits must leave the cursor on a valid char
    /// boundary and preserve the invariant cursor <= value.len().
    #[test]
    fn edits_keep_cursor_on_char_boundary(
        ops in prop::collection::vec(
            (0u8..6, any::<char>()),
            0..200,
        )
    ) {
        let mut input = InputState::new("");
        for (op, ch) in ops {
            match op {
                0 => input.push_char(ch),
                1 => input.backspace(),
                2 => input.cursor_left(),
                3 => input.cursor_right(),
                4 => {
                    // Seed some multibyte content occasionally.
                    input.push_char('é');
                    input.push_char('日');
                    input.cursor_left();
                    input.cursor_left();
                }
                _ => {}
            }
            prop_assert!(input.cursor <= input.value.len());
            prop_assert!(input.value.is_char_boundary(input.cursor));
        }
    }

    /// Deleting every char one at a time must always yield the empty
    /// string without panicking.
    #[test]
    fn full_drain_via_backspace(s in r"[^\pc{C}]{0,64}") {
        let mut input = InputState::new(s.clone());
        while !input.value.is_empty() {
            input.backspace();
        }
        prop_assert_eq!(input.value, "");
        prop_assert_eq!(input.cursor, 0);
    }
}
