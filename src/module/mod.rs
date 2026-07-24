//! shaku dependency injection wiring.
//!
//! The [`module!`] macro ties together concrete component
//! implementations (from [`crate::service`]) into a single
//! [`AppModule`] that can be built, configured, and resolved.
//!
//! To swap an implementation for testing, use
//! [`ModuleBuilder::with_component_override`]:
//!
//! ```ignore
//! let module = AppModule::builder()
//!     .with_component_override::<dyn TorrentService>(Box::new(MockTorrent))
//!     .build();
//! ```

use crate::service::{RealPlayer, RealStorage, TorrentEngineImpl};
use shaku::module;

module! {
    pub AppModule {
        components = [TorrentEngineImpl, RealPlayer, RealStorage],
        providers = []
    }
}
