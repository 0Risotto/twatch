//! twatch — terminal torrent streaming client.
//!
//! Browse, stream, and download torrents from your terminal.

pub mod app;
pub mod application;
pub mod module;
pub mod service;
pub mod ui;

pub use twatch_core::{config, domain, model};
