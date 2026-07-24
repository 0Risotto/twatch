//! Media player interface.

use shaku::Interface;

/// Launches and controls an external media player (mpv or vlc).
pub trait PlayerService: Interface {
    /// Launch the player pointed at `url`, killing any previous
    /// instance first.  `title` is passed to the player window.
    fn play(&self, url: &str, title: &str);

    /// True while a player process is still alive.
    fn is_running(&self) -> bool;

    /// Kill the current player process, if any.
    fn kill(&self);
}
