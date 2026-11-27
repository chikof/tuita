//! The instance's content filter: what the admin put in `instance_settings.content_filter`,
//! applied to handles, names, locations, bios, posts and list names. With nothing set
//! it blocks no word.

use tuita_core::wordfilter::{FilterConfig, Target, find_blocked, is_reserved_handle};

use crate::dto::ContentFilterDto;

/// Words for a rejection message, as in "Your post contains blocked words".
pub const REASON: &str = "blocked words";

/// The filter as the handlers apply it. Load it with [`crate::db::settings::content_filter`].
#[derive(Clone, Debug, Default)]
pub struct ContentFilter {
    config: FilterConfig,
}

impl ContentFilter {
    /// Builds the filter from the stored settings.
    #[must_use]
    pub fn new(settings: &ContentFilterDto) -> Self {
        Self {
            config: FilterConfig::from_settings(
                &settings.blocked,
                &settings.allowed,
                &settings.reserved,
            ),
        }
    }

    /// Whether a handle contains a blocked word.
    #[must_use]
    pub fn blocks_handle(&self, handle: &str) -> bool {
        find_blocked(handle, Target::Handle, &self.config).is_some()
    }

    /// Whether a display name, location, bio, post or list name contains a blocked word.
    #[must_use]
    pub fn blocks_text(&self, text: &str) -> bool {
        find_blocked(text, Target::Text, &self.config).is_some()
    }

    /// Whether nobody may register `handle`: a built-in or admin-reserved word.
    #[must_use]
    pub fn is_reserved(&self, handle: &str) -> bool {
        is_reserved_handle(handle, &self.config)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn filter(blocked: &str, allowed: &str, reserved: &str) -> ContentFilter {
        ContentFilter::new(&ContentFilterDto {
            blocked: blocked.to_owned(),
            allowed: allowed.to_owned(),
            reserved: reserved.to_owned(),
        })
    }

    #[test]
    fn nothing_is_blocked_by_default() {
        let filter = filter("", "", "");
        assert!(!filter.blocks_text("f4gg0t"));
        assert!(!filter.blocks_handle("f4gg0t"));
        assert!(!filter.blocks_text("gizmo"));
    }

    #[test]
    fn admin_words_apply_to_text_and_handles() {
        let filter = filter("gizmo", "gizmos", "");
        assert!(filter.blocks_text("a g1zm0 here"));
        assert!(filter.blocks_handle("gi_zmo"));
        assert!(!filter.blocks_text("gizmos"));
    }

    #[test]
    fn reserves_built_in_and_admin_handles() {
        let filter = filter("", "", "founders, council");
        assert!(filter.is_reserved("founders"));
        assert!(filter.is_reserved("Council_7"));
        assert!(filter.is_reserved("admin"));
        assert!(!filter.is_reserved("ada"));
    }
}
