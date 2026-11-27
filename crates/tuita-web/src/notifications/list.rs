//! The notification list as entries and the link to the next page.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};
use tuita_server::dto::NotificationGroupDto;

use crate::{
    notifications::row::notification_row, post::post_card::me_of, profile::list::more_item,
};

/// The groups as list entries, then the `more` item when there is a next page. `me` is the
/// viewer's own
/// posts are told apart by the cards.
#[component]
pub(crate) async fn notification_items(
    cx: &Cx,
    groups: &[NotificationGroupDto],
    more: Option<&str>,
) -> Result<impl View> {
    let me = me_of(cx).await?;
    Ok(view! {
        #[key(group.id.as_bytes())]
        for group in groups {
            <div role="listitem">
                notification_row(group: group, unread: !group.read, me: me)
            </div>
        }
        if let Some(href) = more {
            more_item(href: href)
        }
    })
}
