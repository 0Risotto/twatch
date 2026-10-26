//! Background task spawning port.
//!
//! Decouples use cases from any concrete executor: tokio in
//! production, inline blocking in tests.

use shaku::Interface;
use std::future::Future;
use std::pin::Pin;

/// Spawns `Send` background futures on the application runtime.
pub trait TaskSpawner: Interface {
    /// Spawn a boxed `Send` future to run to completion in the background.
    fn spawn(&self, future: Pin<Box<dyn Future<Output = ()> + Send>>);
}
