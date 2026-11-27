//! Lists: the index, a list's posts and members, the header with its owner actions and the
//! create/edit dialog.
//!
//! Pages are rendered once. What changes later is done by the `lists.js` island through the JSON
//! API: `[data-act="list-new"]` and `[data-act="list-edit"]` open the editor dialog fetched from
//! `/_tuita/dialog/list-editor[/{id}]`, `list-delete-yes` deletes, `list-remove-member` removes a
//! member. A list picker (`post::dialogs::list_picker_dialog`) offers `list-new` too.

pub(crate) mod editor_dialog;
pub(crate) mod list_header;
pub(crate) mod list_row;
pub(crate) mod member_row;
pub(crate) mod pages;

use topcoat::context::Cx;
use tuita_server::dto::ListDto;

use crate::{
    fmt::compact,
    l10n::{strip_isolates, t},
};

/// "12 members" for a list.
#[must_use]
pub(crate) fn member_count(cx: &Cx, list: &ListDto) -> String {
    let count = list.members;
    strip_isolates(&t!(lists_members, count: count, compact: compact(cx, count)).resolve(cx))
}

#[cfg(test)]
pub(crate) mod testing {
    use time::OffsetDateTime;
    use tuita_server::dto::{ListDto, UserRefDto};
    use uuid::Uuid;

    /// A list of two people owned by the viewer.
    pub(crate) fn sample_list(name: &str, private: bool) -> ListDto {
        ListDto {
            id: Uuid::from_u128(0x11_57),
            name: name.to_owned(),
            description: String::new(),
            private,
            created_at: OffsetDateTime::UNIX_EPOCH,
            owner: sample_owner(),
            is_owner: true,
            members: 2,
            has_member: None,
        }
    }

    fn sample_owner() -> UserRefDto {
        crate::post::testing::sample_user("ada")
    }
}
