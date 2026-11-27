//! Core logic of `tuita`: text parsing, validation, feed scoring and ActivityPub types.
//!
//! Keep this crate free of I/O so it stays easy to test and reuse.

pub mod ap;
pub mod emoji;
pub mod feed;
pub mod handle;
pub mod locale;
pub mod notify;
pub mod sealed;
pub mod search;
pub mod text;
pub mod wordfilter;
