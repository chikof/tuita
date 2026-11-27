//! Profiles and follow lists: the profile page, the follow button, people rows and the lookup of
//! accounts on other instances.

pub(crate) mod follow_button;
pub(crate) mod follow_list;
pub(crate) mod list;
pub(crate) mod load_user;
pub(crate) mod profile_header;
pub(crate) mod profile_view;
pub(crate) mod remote_lookup;
#[cfg(test)]
pub(crate) mod testing;
pub(crate) mod user_row;
