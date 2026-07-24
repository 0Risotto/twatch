//! Media player interface.

use shaku::Interface;

/// Spawns and controls an external media player (mpv or vlc).
///
/// All methods take `&self` (interior mutability) so implementations
/// are compatible with shaku's `Arc<dyn PlayerService>` storage.
pub trait PlayerService: Interface {
    /// Launch a media player pointed at `url`, killing any previously
    /// running instance first.  `title` is passed to the player for
    /// its window title.
    fn play(&self, url: &str, title: &str);

    /// True while a player process is still alive.
    fn is_running(&self) -> bool;

    /// Kill the current player process, if any.
    fn kill(&self);
}
