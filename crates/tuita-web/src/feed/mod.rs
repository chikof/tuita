//! Home feed, new-posts pill, pull to refresh, bookmarks, tag pages and scheduled posts.
//!
//! Every list is server-rendered. A page that pages ends its items with a sentinel
//! (`list::post_items`) that `infinite.js` replaces with the next page, fetched with the header
//! `x-tuita-fragment: 1`; the page then renders only the items. Nothing inside a list holds
//! Topcoat state, because fetched markup is never hydrated: `feed.js`-style work is done by the
//! `newposts.js` and `pull.js` islands.

pub(crate) mod bookmarks;
pub(crate) mod home;
pub(crate) mod list;
pub(crate) mod new_posts;
pub(crate) mod scheduled;
pub(crate) mod tags;

/// The two home timelines.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Feed {
    ForYou,
    Following,
}

impl Feed {
    /// The name in `?feed=`, the cookie and the API path.
    #[must_use]
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::ForYou => "foryou",
            Self::Following => "following",
        }
    }

    /// The feed called `name`, if there is one.
    #[must_use]
    pub(crate) fn parse(name: &str) -> Option<Self> {
        match name {
            "foryou" => Some(Self::ForYou),
            "following" => Some(Self::Following),
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::Feed;

    #[test]
    fn a_feed_round_trips_through_its_name() {
        for feed in [Feed::ForYou, Feed::Following] {
            assert_eq!(Feed::parse(feed.name()), Some(feed));
        }
        assert_eq!(Feed::parse("popular"), None);
        assert_eq!(Feed::parse(""), None);
    }
}
