//! A member of a list: the person row and, for the owner, a Remove button.

use topcoat::{
    Result,
    context::Cx,
    view::{View, attributes, component, view},
};
use tuita_server::dto::UserDto;

use crate::{
    fmt::acct_of,
    l10n::t,
    profile::user_row::user_row,
    ui::button::{ButtonSize, ButtonVariant, button},
};

/// The person (`UserRow`: picture, name, handle, bio and follow button) and, when `owner`, a
/// button that `lists.js` turns into `DELETE /lists/{list}/members/{acct}`.
#[component]
pub(crate) async fn member_row(
    cx: &Cx,
    list_id: &str,
    user: &UserDto,
    owner: bool,
) -> Result<impl View> {
    let acct = acct_of(&user.handle, user.domain.as_deref());
    Ok(view! {
        <div class="member" data-c="member-row" data-user-id=(user.id.to_string())>
            user_row(user: user)
            if owner {
                <div class="remove">
                    button(
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Sm,
                        attrs: attributes! {
                            data-act="list-remove-member"
                            data-list-id=(list_id)
                            data-acct=(&acct)
                        },
                        (t!(lists_remove))
                    )
                </div>
            }
        </div>
    })
}
