//! shaku dependency injection wiring.
//!
//! The [`module!`] macro ties concrete adapter implementations
//! (from [`crate::service`]) into an [`AppModule`].

use crate::service::{
    RealStorage, SearchServiceImpl, StreamUrlProvider, TokioTaskSpawner, TorrentEngineImpl,
};
use shaku::module;

module! {
    pub AppModule {
        components = [
            TorrentEngineImpl,
            StreamUrlProvider,
            TokioTaskSpawner,
            RealStorage,
            SearchServiceImpl,
        ],
        providers = []
    }
}
