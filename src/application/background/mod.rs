//! Background flows.
//!
//! Each flow owns its own channel and background-task lifecycle, so a
//! new background feature is a new module here — no central event enum
//! or routing match needs to change.

pub mod download;
pub mod preview;
pub mod search;
