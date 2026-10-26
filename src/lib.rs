//! twatch — terminal torrent streaming client.
//!
//! Browse, stream, and download torrents from your terminal.  The reusable
//! domain/application layers live in `twatch-core`; concrete adapters live in
//! `twatch-engine`.  The TUI in this crate is one client of those layers.

pub mod app;
pub mod application;
pub mod ui;

pub use twatch_core::{config, domain, model};
pub use twatch_engine::{module, service};
