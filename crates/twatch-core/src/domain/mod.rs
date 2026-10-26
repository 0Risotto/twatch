//! Pure domain layer — entities, value objects, and service ports.
//!
//! No I/O, no tokio, no UI dependencies.  Concrete adapters live in
//! [`crate::service`]; presentation lives in [`crate::ui`].

pub mod common;
pub mod history;
pub mod player;
pub mod search;
pub mod torrent;
