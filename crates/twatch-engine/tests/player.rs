#![allow(clippy::unwrap_used)]

use std::sync::Mutex;
use twatch_core::domain::player::ports::PlayerPort;
use twatch_engine::service::player::{PlayerState, RealPlayer};

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
