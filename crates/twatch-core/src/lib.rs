//! Pure domain and application services for twatch.
//!
//! This crate has no dependency on terminal or HTTP presentation.  The TUI,
//! the web gateway, and tests all share it.

pub mod config;
pub mod domain;
pub mod gateway;
pub mod model;
pub mod theme;
