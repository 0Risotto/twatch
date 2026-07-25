//! shaku dependency injection wiring.
//!
//! The [`module!`] macro ties concrete component implementations
//! (from [`crate::service`]) into an [`AppModule`].

use crate::service::{RealPlayer, RealStorage, SearchServiceImpl, TorrentEngineImpl};
use shaku::module;

module! {
    pub AppModule {
        components = [TorrentEngineImpl, RealPlayer, RealStorage, SearchServiceImpl],
        providers = []
    }
}
