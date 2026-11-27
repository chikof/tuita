//! The `/design` component gallery.
//!
//! One file per section group, each a component that renders its [`section::section`]s in
//! the gallery order. [`page::gallery`] stacks the groups under the toolbar and the
//! component index; `sample` holds the DTO fixtures every group draws on. The gallery is
//! English only: it is a developer page, so its labels are literals (`scripts/lint-strings.sh`
//! skips this directory).

pub(crate) mod actions;
pub(crate) mod admin;
pub(crate) mod dither;
pub(crate) mod domain;
pub(crate) mod encryption;
pub(crate) mod federation;
pub(crate) mod feedback;
pub(crate) mod forms;
pub(crate) mod identity;
pub(crate) mod media;
pub(crate) mod oauth;
pub(crate) mod overlays;
pub(crate) mod page;
#[allow(dead_code)]
pub(crate) mod sample;
pub(crate) mod section;
pub(crate) mod toolbar;
