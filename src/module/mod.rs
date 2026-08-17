//! shaku dependency injection wiring.
//!
//! The [`module!`] macro ties concrete adapter implementations
//! (from [`crate::service`]) into an [`AppModule`].

use crate::service::{
    RealPlayer, RealStorage, SearchServiceImpl, StreamUrlProvider, TokioTaskSpawner,
    TorrentEngineImpl,
};
use shaku::module;

module! {
    pub AppModule {
        components = [
            TorrentEngineImpl,
            StreamUrlProvider,
            TokioTaskSpawner,
            RealPlayer,
            RealStorage,
            SearchServiceImpl,
        ],
        providers = []
    }
}
