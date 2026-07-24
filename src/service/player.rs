//! Default [`PlayerService`](crate::traits::PlayerService) implementation.
//!
//! Spawns either **mpv** or **vlc** as an external process to play an
//! HTTP stream.  State is protected by a [`Mutex`] so the component
//! can be stored in an `Arc` without requiring `&mut self`.

use crate::traits::PlayerService;
use shaku::Component;
use std::process::{Child, Command};
use std::sync::Mutex;

/// Mutable state of the external media player process.
#[derive(Default)]
pub struct PlayerState {
    child: Option<Child>,
}

/// Production [`PlayerService`] that launches mpv (preferred) or vlc (fallback).
#[derive(Component)]
#[shaku(interface = PlayerService)]
pub struct RealPlayer {
    #[shaku(default)]
    pub state: Mutex<PlayerState>,
}

impl PlayerService for RealPlayer {
    fn play(&self, url: &str, title: &str) {
        // Kill old process outside the lock so wait() doesn't block all callers.
        let old_child = {
            let mut state = state_lock(&self.state);
            state.child.take()
        };
        if let Some(mut child) = old_child {
            let _ = child.kill();
            let _ = child.wait();
        }

        let new_child = spawn_player(url, title);
        let sanitized = sanitize(title);

        match new_child {
            Ok(child) => {
                state_lock(&self.state).child = Some(child);
                tracing::info!("Launched player for: {sanitized}");
            }
            Err(e) => {
                tracing::error!("Neither mpv nor vlc found. Install mpv or vlc. Error: {e}");
            }
        }
    }

    fn is_running(&self) -> bool {
        state_lock(&self.state).child.is_some()
    }

    fn kill(&self) {
        let old_child = {
            let mut state = state_lock(&self.state);
            state.child.take()
        };
        if let Some(mut child) = old_child {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// Try mpv first; fall back to vlc.
fn spawn_player(url: &str, title: &str) -> std::io::Result<Child> {
    Command::new("mpv")
        .args([url, "--force-window=yes", "--title"])
        .arg(title)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .or_else(|_| {
            tracing::info!("mpv not found, trying vlc...");
            Command::new("vlc")
                .args([url, "--meta-title"])
                .arg(title)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
        })
}

/// Strip control characters so malicious torrent titles cannot inject
/// escape sequences into log output.
fn sanitize(raw: &str) -> String {
    raw.chars().filter(|c| !c.is_control()).collect()
}

/// Lock the player state; recover from a poisoned mutex instead of panicking.
fn state_lock(state: &Mutex<PlayerState>) -> std::sync::MutexGuard<'_, PlayerState> {
    state.lock().unwrap_or_else(|e| e.into_inner())
}
