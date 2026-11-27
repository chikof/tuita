//! One notification row.
//!
//! Replies, mentions and quotes draw the post as a card under a one-line context; every other kind
//! is a link row with the kind's icon, the avatars, the sentence and a snippet of the post. The
//! rows come as fragments (`infinite.js`, `notifications.js`), so they are plain markup.

use time::format_description::well_known::Rfc3339;
use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};
use tuita_server::dto::NotificationGroupDto;

use crate::{
    fmt::rel_time,
    l10n::t,
    notifications::text::{Part, group_path, icon_of, is_mention, name_of, parts},
    post::{
        domain_chip::domain_chip,
        post_card::{Me, card_view},
        profile_path,
        rich_text::{around, emote_text},
    },
    ui::{
        avatar::AvatarShape,
        avatar_stack::{StackPerson, avatar_stack},
        icon::icon,
    },
};

/// The sentence of a group: names in `<b>` (with their emotes), the rest as text.
#[component]
async fn sentence(cx: &Cx, group: &NotificationGroupDto) -> Result<impl View> {
    let pieces = parts(cx, group);
    Ok(view! {
        for piece in pieces {
            if let Part::Name(index) = piece {
                <b translate="no">
                    emote_text(
                        text: name_of(group, index),
                        emojis: &group.actors[index].emojis
                    )
                </b>
            } else if let Part::Text(text) = piece {
                (text)
            }
        }
    })
}

/// A row for group `group`; `unread` highlights it. `me` is the viewer, whose handle is the follower
/// list a group of followers leads to.
#[component]
pub(crate) async fn notification_row(
    cx: &Cx,
    group: &NotificationGroupDto,
    unread: bool,
    me: Option<Me<'_>>,
) -> Result<impl View> {
    let handle = me.map_or("", |me| me.handle);
    let kind_icon = icon_of(group.kind);
    let actor = group.actors.first();
    let post = group.post.as_ref();
    let href = group_path(group, handle);
    let id = group.id.to_string();
    let created = group
        .created_at
        .format(&Rfc3339)
        .unwrap_or_else(|_| group.created_at.to_string());
    let people: Vec<StackPerson<'_>> = group
        .actors
        .iter()
        .map(|a| StackPerson {
            seed: a.handle.as_str(),
            alt: a.display_name.as_str(),
            src: a.avatar_url.as_deref(),
        })
        .collect();
    let shape = actor.map_or(AvatarShape::default(), |a| {
        AvatarShape::from_name(a.avatar_shape.as_str()).unwrap_or_default()
    });
    let mut domains: Vec<&str> = Vec::new();
    for a in &group.actors {
        if let Some(domain) = a.domain.as_deref()
            && !domains.contains(&domain)
        {
            domains.push(domain);
        }
    }
    let (cw_before, cw_after) = around(&t!(notif_cw_snippet, cw: "\u{0}").resolve(cx));
    let snippet = post.map(|p| {
        (
            !p.cw.is_empty(),
            if p.cw.is_empty() { &p.text } else { &p.cw },
        )
    });
    let threaded = post.is_some() && actor.is_some() && is_mention(group.kind);
    Ok(view! {
        if threaded {
            <div
                class=(if unread { "thread unread" } else { "thread" })
                data-c="notification-row"
                data-group-id=(&id)
                data-unread=(unread.then_some(""))
            >
                if let Some(actor) = actor {
                    <a class="context" href=(profile_path(actor))>
                        icon(name: kind_icon, size: 16)
                        <span>sentence(group: group)</span>
                    </a>
                }
                if let Some(post) = post {
                    card_view(post: post, me: me, hide_reply_context: true)
                }
            </div>
        } else {
            <a
                class=(if unread { "row wash unread" } else { "row wash" })
                href=(&href)
                data-c="notification-row"
                data-group-id=(&id)
                data-unread=(unread.then_some(""))
            >
                <span class="kind" aria-hidden="true">
                    icon(name: kind_icon, size: 20)
                </span>
                <span class="main">
                    <span class="avatars" aria-hidden="true">
                        avatar_stack(people: people, max: 3, size: 32, shape: shape)
                    </span>
                    <span class="text">
                        sentence(group: group)
                        <span class="time">
                            "· "
                            rel_time(ts: &created)
                        </span>
                    </span>
                    if !domains.is_empty() {
                        <span class="origins">
                            for domain in domains {
                                domain_chip(domain: domain)
                            }
                        </span>
                    }
                    if let Some((has_cw, text)) = snippet {
                        <span class="snippet" translate="no">
                            if has_cw {
                                (&cw_before)
                            }
                            emote_text(
                                text: text,
                                emojis: post.map_or(&[][..], |p| p.emojis.as_slice())
                            )
                            if has_cw {
                                (&cw_after)
                            }
                        </span>
                    }
                </span>
            </a>
        }
    })
}

#[cfg(test)]
mod test {
    use time::macros::datetime;
    use topcoat::view::ViewExt;
    use tuita_server::dto::NotificationKindDto;
    use uuid::Uuid;

    use super::*;
    use crate::{
        post::testing::{sample_post, sample_user},
        testing::render,
    };

    fn group(kind: NotificationKindDto, with_post: bool) -> NotificationGroupDto {
        NotificationGroupDto {
            id: Uuid::from_u128(9),
            kind,
            read: false,
            created_at: datetime!(2026-11-01 00:00 UTC),
            actors: vec![sample_user("ana")],
            actor_count: 1,
            post: with_post.then(sample_post),
        }
    }

    fn show(group: NotificationGroupDto, unread: bool) -> String {
        render("en", |cx| {
            view! {
                notification_row(
                    group: &group,
                    unread: unread,
                    me: Some(
                        Me { id: Uuid::from_u128(1), handle: "me", pinned_post_id: None },
                    )
                )
            }
            .boxed()
        })
    }

    #[test]
    fn a_like_is_a_link_row_with_the_sentence_and_a_snippet_of_the_post() {
        let html = show(group(NotificationKindDto::Like, true), true);
        assert!(html.contains(r#"class="row wash unread""#), "{html}");
        assert!(html.contains(r#"href="/@ada/"#), "{html}");
        assert!(html.contains("<b translate=\"no\">ANA</b>"), "{html}");
        assert!(html.contains(" liked your post"), "{html}");
        assert!(html.contains("hello world"), "{html}");
        assert!(html.contains(r#"data-group-id="#), "{html}");
        assert!(html.contains("data-rel"), "{html}");
    }

    #[test]
    fn a_read_row_has_no_highlight() {
        let html = show(group(NotificationKindDto::Follow, false), false);
        assert!(html.contains(r#"class="row wash""#), "{html}");
        assert!(!html.contains("data-unread"), "{html}");
        assert!(html.contains(r#"href="/@ana""#), "{html}");
        assert!(html.contains(" followed you"), "{html}");
    }

    #[test]
    fn a_reply_is_a_context_line_over_the_post_card() {
        let html = show(group(NotificationKindDto::Reply, true), false);
        assert!(html.contains(r#"class="thread""#), "{html}");
        assert!(html.contains(r#"class="context""#), "{html}");
        assert!(html.contains(" replied to you"), "{html}");
        assert!(html.contains("<article"), "{html}");
    }
}
