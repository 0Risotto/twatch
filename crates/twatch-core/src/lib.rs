//! Pure domain and application services for twatch.
//!
//! This crate has no dependency on HTTP or desktop presentation.  The daemon,
//! the desktop shells, and tests all share it.

pub mod config;
pub mod domain;
pub mod gateway;
pub mod model;
pub mod theme;
