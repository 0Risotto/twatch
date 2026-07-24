use std::sync::Mutex;
use twatch::service::player::{PlayerState, RealPlayer};
use twatch::traits::PlayerService;

mod common;

#[test]
fn fresh_player_is_not_running() {
    let p = RealPlayer { state: Mutex::new(PlayerState::default()) };
    assert!(!p.is_running());
}

#[test]
fn kill_on_fresh_player_does_not_panic() {
    let p = RealPlayer { state: Mutex::new(PlayerState::default()) };
    p.kill();
    assert!(!p.is_running());
}

#[test]
fn mock_player_records_play_calls() {
    let p = common::player::MockPlayerService::new();
    p.play("http://localhost/stream", "Test");
    assert!(p.is_running());
    assert_eq!(*p.last_url.lock().unwrap(), Some("http://localhost/stream".into()));
    assert_eq!(*p.last_title.lock().unwrap(), Some("Test".into()));
}

#[test]
fn mock_player_kill_clears_state() {
    let p = common::player::MockPlayerService::new();
    p.play("url", "title");
    p.kill();
    assert!(!p.is_running());
    assert!(p.last_url.lock().unwrap().is_none());
    assert!(*p.killed.lock().unwrap());
}
