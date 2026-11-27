//! Gallery sections: federation.
//!
//! The gallery plays a signed-in viewer (`sample::me()`) so the follow buttons and the menus
//! show. `RemoteLookup` has no "looking up" frame and `UserGate` no "pending" one: the server
//! waits for the lookup and answers once, so the browser's own pending navigation is that state.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};
use tuita_server::dto::Visibility;

use super::{
    admin::frame,
    sample,
    section::{row, section},
};
use crate::{
    notifications::row::notification_row,
    post::{
        domain_chip::domain_chip,
        post_card::{Me, card_view},
        visibility_badge::visibility_badge,
    },
    profile::{
        follow_button::follow_button_view, load_user::LookupFailure,
        profile_header::profile_header_view, remote_lookup::remote_lookup, user_row::user_row_view,
    },
    shell::page_bar::{Heading, page_bar},
};

const HANDLE: &str = "@mara@fedi.example";
const DOMAIN: &str = "fedi.example";

/// `domain-chip` to `remote-user`.
#[component]
async fn posts_and_people(cx: &Cx) -> Result<impl View> {
    let viewer = sample::me().id;
    let me = Me {
        id: viewer,
        handle: "sam",
        pinned_post_id: None,
    };
    let (remote, unlisted, followers, direct) = (
        sample::remote_post(),
        sample::unlisted_post(),
        sample::followers_post(),
        sample::direct_post(),
    );
    let (user, pending, followed) = (
        sample::remote_user(),
        sample::remote_pending(),
        sample::remote_followed(),
    );
    Ok(view! {
        section(
            id: "domain-chip",
            title: "DomainChip",
            note: "the home domain of a remote account, in mono; a long domain ellipsizes inside its row",
            row(
                domain_chip(domain: "fedi.example")
                domain_chip(domain: "127.0.0.1:8081")
                domain_chip(domain: "a-very-long-instance-name.social.example.org")
            )
        )
        section(
            id: "visibility-badge",
            title: "VisibilityBadge",
            note: "unlisted, followers only and direct posts carry a badge; public posts have none",
            row(
                visibility_badge(visibility: Visibility::Unlisted)
                visibility_badge(visibility: Visibility::Followers)
                visibility_badge(visibility: Visibility::Direct)
                visibility_badge(visibility: Visibility::Public)
            )
        )
        section(
            id: "remote-post",
            title: "Remote posts",
            note: "PostCard of a remote author: domain chip, link to the original, visibility badge",
            frame(
                card_view(post: &remote, me: Some(me))
                card_view(post: &unlisted, me: Some(me))
                card_view(post: &followers, me: Some(me))
                card_view(post: &direct, me: Some(me))
                card_view(post: &remote, me: Some(me), main: true)
            )
        )
        section(
            id: "remote-user",
            title: "Remote accounts",
            note: "UserRow with the domain chip, and every follow state: Follow, Requested (click cancels), Following",
            frame(
                user_row_view(user: &user, viewer: Some(viewer))
                user_row_view(user: &pending, viewer: Some(viewer))
                user_row_view(user: &followed, viewer: Some(viewer))
            )
            row(
                follow_button_view(user: &user, viewer: Some(viewer))
                follow_button_view(user: &pending, viewer: Some(viewer))
                follow_button_view(user: &followed, viewer: Some(viewer))
            )
        )
    })
}

/// `remote-profile` to `user-gate`.
#[component]
async fn remote_pages(cx: &Cx) -> Result<impl View> {
    let viewer = sample::me().id;
    let me = Me {
        id: viewer,
        handle: "sam",
        pinned_post_id: None,
    };
    let (user, locked, pending) = (
        sample::remote_user(),
        sample::remote_locked(),
        sample::remote_pending(),
    );
    let notifications = sample::remote_notifications();
    let not_found = LookupFailure {
        status: 404,
        message: "User not found.".to_owned(),
    };
    let unreachable = LookupFailure {
        status: 502,
        message: "Couldn’t reach fedi.example right now.".to_owned(),
    };
    let unreadable = LookupFailure {
        status: 502,
        message: "fedi.example sent something tuita can’t read.".to_owned(),
    };
    Ok(view! {
        section(
            id: "remote-profile",
            title: "Remote profile",
            note: "home domain, link to the original profile, locked tag and note; a locked account shows Requested while it decides",
            frame(
                profile_header_view(
                    user: &user,
                    viewer: Some(viewer),
                    known: None,
                    scope: "remote-user"
                )
            )
            frame(
                profile_header_view(
                    user: &locked,
                    viewer: Some(viewer),
                    known: None,
                    scope: "remote-locked"
                )
            )
            frame(
                profile_header_view(
                    user: &pending,
                    viewer: Some(viewer),
                    known: None,
                    scope: "remote-pending"
                )
            )
        )
        section(
            id: "remote-notifications",
            title: "Remote notifications",
            note: "remote actors show their home domain under the sentence",
            frame(
                for group in &notifications {
                    notification_row(group: group, unread: !group.read, me: Some(me))
                }
            )
        )
        section(
            id: "remote-lookup",
            title: "RemoteLookup",
            note: "while an account on another instance loads, and when the lookup fails: not found, unreachable, unreadable",
            frame(remote_lookup(handle: HANDLE, domain: DOMAIN, failure: &not_found))
            frame(remote_lookup(handle: HANDLE, domain: DOMAIN, failure: &unreachable))
            frame(remote_lookup(handle: HANDLE, domain: DOMAIN, failure: &unreadable))
        )
        section(
            id: "user-gate",
            title: "UserGate",
            note: "the page of a handle: a local account is there at once, a remote one shows the lookup while it runs and the reason when it fails",
            frame(user_row_view(user: &user, viewer: Some(viewer)))
            frame(
                page_bar(back: true, title: HANDLE, heading: Heading::H2)
                remote_lookup(handle: HANDLE, domain: DOMAIN, failure: &unreachable)
            )
        )
    })
}

#[component]
pub(crate) async fn federation(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        posts_and_people()
        remote_pages()
    })
}
