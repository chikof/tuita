//! The admin console: `/admin` and `/admin/{tab}`.
//!
//! Every tab renders with its page. Lists that change after an action (reports, people, domain
//! blocks, announcements, invites, emotes) are shards that take a `revision` signal: an action's
//! procedure answers with the text for a toast, and the handler bumps the signal so the shard
//! renders again from the API. Forms keep their fields in signals owned by the tab.

pub(crate) mod actor_key;
pub(crate) mod announcements;
pub(crate) mod audit;
pub(crate) mod common;
pub(crate) mod domain_block_row;
pub(crate) mod emotes;
pub(crate) mod federation;
pub(crate) mod federation_tab;
pub(crate) mod filter;
pub(crate) mod instance;
pub(crate) mod invites;
pub(crate) mod labels;
pub(crate) mod overview;
pub(crate) mod page;
pub(crate) mod post_preview;
pub(crate) mod queue_summary;
pub(crate) mod report_card;
pub(crate) mod reports;
pub(crate) mod rules;
pub(crate) mod select_field;
pub(crate) mod setting_card;
pub(crate) mod tabs;
pub(crate) mod user_actions;
pub(crate) mod user_detail;
pub(crate) mod user_menu;
pub(crate) mod user_row;
pub(crate) mod users;

#[cfg(test)]
pub(crate) mod testing;
