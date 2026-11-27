//! The composer: the post box with its poll, schedule, media tray and emoji picker, and the
//! compose dialog the shell opens.
//!
//! The server renders the markup of every state the box can be in (a hidden poll editor, a hidden
//! tray, `<template>`s for what the browser adds); the `composer.js` and `upload.js` islands hold
//! the state. `composer.js` documents the contract at its top.

pub(crate) mod compose_dialog;
pub(crate) mod composer;
pub(crate) mod emoji;
pub(crate) mod emoji_picker;
pub(crate) mod media_tray;
pub(crate) mod poll_editor;
pub(crate) mod schedule_picker;

/// Most characters of a post, as the server counts them (a URL counts 23).
pub(crate) const MAX_TEXT: i64 = 500;
/// Longest content warning.
pub(crate) const MAX_CW: usize = 100;
/// The poll lengths on offer, in hours.
pub(crate) const POLL_HOURS: [u32; 5] = [1, 6, 24, 72, 168];
/// Most answers of a poll.
pub(crate) const MAX_POLL_OPTIONS: usize = 4;
/// Longest answer of a poll.
pub(crate) const MAX_OPTION_LENGTH: usize = 25;
