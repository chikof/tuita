//! One post in a list or thread.

use time::{Duration, OffsetDateTime};
use topcoat::{
    Result,
    context::Cx,
    view::{Attributes, Child, View, attributes, class, component, element_id, view},
};
use tuita_server::dto::{PostDto, UserRefDto, Visibility};
use uuid::Uuid;

use crate::{
    fmt::{acct_of, full_time, handle_of, host_of, rel_time},
    icons::Icon,
    l10n::{strip_isolates, t},
    post::{
        content_warning::content_warning,
        domain_chip::domain_chip,
        link_card::link_card,
        media_grid::media_grid,
        poll::poll_view,
        post_actions::post_actions,
        post_path, profile_path,
        quote_card::quote_card,
        rich_text::{around, emote_text, rich_text, user_name},
        share_text,
        visibility_badge::visibility_badge,
    },
    session,
    ui::{
        avatar::{AvatarShape, avatar},
        icon::icon,
        menu::{MenuItem, menu},
    },
};

/// Posts can be edited for 30 minutes.
const EDIT_WINDOW: Duration = Duration::minutes(30);

/// Which side of the avatar a thread line is drawn on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ThreadLine {
    /// Below the avatar: the next post answers this one.
    Down,
    /// Above the avatar: this post answers the one above.
    Up,
}

impl ThreadLine {
    const fn name(self) -> &'static str {
        match self {
            Self::Down => "down",
            Self::Up => "up",
        }
    }
}

/// The avatar shape of an account.
pub(crate) fn shape_of(author: &UserRefDto) -> AvatarShape {
    AvatarShape::from_name(author.avatar_shape.as_str()).unwrap_or_default()
}

/// The accessible name of the card.
fn card_label(cx: &Cx, post: &PostDto) -> String {
    strip_isolates(&t!(post_by, name: post.author.display_name.as_str()).resolve(cx))
}

/// What the menu holds, in order, for this viewer.
#[derive(Clone, Copy)]
enum MenuFacts {
    /// The viewer wrote the post.
    Own { pinned: bool, editable: bool },
    /// Someone else wrote it; `hideable` when it answers one of the viewer's posts.
    Other { hideable: bool },
}

/// Who is looking: what the card needs to tell the viewer's own posts from others.
#[derive(Clone, Copy)]
pub(crate) struct Me<'a> {
    pub(crate) id: Uuid,
    pub(crate) handle: &'a str,
    pub(crate) pinned_post_id: Option<Uuid>,
}

/// The viewer of the request, if there is one.
pub(crate) async fn me_of(cx: &Cx) -> Result<Option<Me<'_>>> {
    Ok(session::viewer(cx).await?.as_ref().map(|viewer| Me {
        id: viewer.me.user.id,
        handle: viewer.me.user.handle.as_str(),
        pinned_post_id: viewer.me.user.pinned_post_id,
    }))
}

/// A post: who, when, text with its warning, media, link card, poll, quote and the action bar.
///
/// `post` is the list entry (a repost entry names who reposted it). `main` is the post a thread
/// page is about: larger, full timestamp, no click-through. `thread` draws the thread line.
/// `footer` goes between the text and the action bar.
///
/// `post.js` makes the card work: a click on the card (not on a link or control) opens `data-href`,
/// the menu items carry `data-act` (copy, share, edit, history, pin, delete, hide, mute, report,
/// block, goto) and `data-hovercard="<acct>"` marks the links that open the profile card. The
/// link-card slot `[data-slot="card"]` is where `stream.js` puts a preview that finished loading.
#[component]
pub(crate) async fn post_card(
    cx: &Cx,
    post: &PostDto,
    #[default] main: bool,
    #[default] thread: Option<ThreadLine>,
    #[default] hide_reply_context: bool,
    #[default] footer: Option<Child<'_>>,
) -> Result<impl View> {
    let me = me_of(cx).await?;
    Ok(view! {
        card_view(
            post: post,
            me: me,
            main: main,
            thread: thread,
            hide_reply_context: hide_reply_context,
            footer: footer
        )
    })
}

/// [`post_card`] for a known viewer.
#[component]
pub(crate) async fn card_view(
    cx: &Cx,
    post: &PostDto,
    me: Option<Me<'_>>,
    #[default] main: bool,
    #[default] thread: Option<ThreadLine>,
    #[default] hide_reply_context: bool,
    #[default] footer: Option<Child<'_>>,
) -> Result<impl View> {
    let author = &post.author;
    let acct = acct_of(&author.handle, author.domain.as_deref());
    let handle = handle_of(&author.handle, author.domain.as_deref());
    let path = post_path(author, post.id);
    let profile = profile_path(author);
    let id = post.id.to_string();
    let uid = element_id(cx);
    let menu_id = format!("post-{uid}");
    let mine = me.is_some_and(|me| me.id == author.id);
    let pinned = me.is_some_and(|me| me.pinned_post_id == Some(post.id));
    let editable = mine && OffsetDateTime::now_utc() - post.created_at < EDIT_WINDOW;
    let hideable = !mine
        && me.is_some_and(|me| {
            post.reply_to
                .as_ref()
                .and_then(|reply| reply.handle.as_deref())
                == Some(me.handle)
        });
    let facts = if mine {
        MenuFacts::Own { pinned, editable }
    } else {
        MenuFacts::Other { hideable }
    };
    let created = post
        .created_at
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default();
    let (reposted_before, reposted_after) =
        around(&t!(post_reposted_by, name: "\u{0}").resolve(cx));
    let label = card_label(cx, post);
    let menu_label = t!(post_menu).resolve(cx).into_owned();
    let menu_sr = menu_label.clone();
    let original_host = post
        .url
        .as_deref()
        .map(host_of)
        .filter(|host| !host.is_empty());
    let original_label = original_host
        .as_ref()
        .map(|host| strip_isolates(&t!(remote_view_original, domain: host.as_str()).resolve(cx)));
    let reply_to = if hide_reply_context {
        None
    } else {
        post.reply_to.as_ref()
    };
    let replying = reply_to.map(|reply| match &reply.handle {
        Some(handle) => {
            strip_isolates(&t!(post_replying_to, handle: format!("@{handle}")).resolve(cx))
        }
        None => t!(post_replying_unavailable).resolve(cx).into_owned(),
    });
    let show_origin =
        author.domain.is_some() || post.visibility != Visibility::Public || post.url.is_some();
    let show_body = !post.text.is_empty()
        || !post.media.is_empty()
        || post.poll.is_some()
        || post.card.is_some();
    let stamp_client = post.client.as_deref().map(|client| format!(" · {client}"));
    let full = full_time(cx, &created);
    let entries = menu_entries(cx, post, facts, &path, &profile);
    let reposted_by = post.reposted_by.as_ref();
    let size = if main { 48 } else { 40 };

    Ok(view! {
        <article
            class=(class!("post", "wash", "main" if main))
            data-c="post-card"
            data-post=(&id)
            data-post-id=(&id)
            data-author-id=(author.id.to_string())
            data-href=((!main).then_some(&path))
            data-main=(main.then_some(""))
            data-thread=(thread.map(ThreadLine::name))
            data-hide-reply=(hide_reply_context.then_some(""))
            tabindex="-1"
            aria-label=(&label)
        >
            if let Some(by) = reposted_by {
                <p class="context">
                    icon(name: Icon::Repeat, size: 16)
                    <a
                        href=(profile_path(by))
                        data-hovercard=(acct_of(&by.handle, by.domain.as_deref()))
                    >
                        (&reposted_before)
                        emote_text(text: &by.display_name, emojis: &by.emojis)
                        (&reposted_after)
                    </a>
                </p>
            } else if post.pinned {
                <p class="context">
                    icon(name: Icon::Flag, size: 16)
                    (t!(post_pinned))
                </p>
            }
            <div class="grid">
                <div
                    class=(class!(
                        "side",
                        "up" if thread == Some(ThreadLine::Up),
                        "down" if thread == Some(ThreadLine::Down),
                    ))
                >
                    <a
                        href=(&profile)
                        tabindex="-1"
                        aria-hidden="true"
                        data-hovercard=(&acct)
                    >
                        avatar(
                            seed: &author.handle,
                            src: author.avatar_url.as_deref(),
                            alt: "",
                            shape: shape_of(author),
                            ink: author.ink.as_str(),
                            size: size
                        )
                    </a>
                </div>
                <div class="content">
                    <header>
                        <a class="who" href=(&profile) data-hovercard=(&acct)>
                            user_name(
                                name: &author.display_name,
                                verified: author.verified,
                                bot: author.bot,
                                emojis: &author.emojis
                            )
                            <span class="handle">(&handle)</span>
                        </a>
                        if !main {
                            <a class="time" href=(&path)>rel_time(ts: &created)</a>
                        }
                        if post.edited_at.is_some() {
                            <button
                                type="button"
                                class="edited"
                                data-act="history"
                                data-post-id=(&id)
                            >
                                (t!(edit_edited))
                            </button>
                        }
                        <span class="menu">
                            menu(
                                id: &menu_id,
                                label: &menu_label,
                                items: menu_items(&entries),
                                plain: true,
                                trigger: view! {
                                    <span class="more">
                                        icon(name: Icon::MoreHorizontal, size: 20)
                                        <span class="sr-only">(&menu_sr)</span>
                                    </span>
                                }.into(

                                )
                            )
                        </span>
                    </header>
                    if show_origin {
                        <p class="origin">
                            if let Some(domain) = &author.domain {
                                domain_chip(domain: domain)
                            }
                            visibility_badge(visibility: post.visibility)
                            if let (Some(url), Some(text)) = (&post.url, &original_label) {
                                <a
                                    class="original"
                                    href=(url)
                                    target="_blank"
                                    rel="noopener noreferrer nofollow ugc"
                                >
                                    icon(name: Icon::ExternalLink, size: 12)
                                    (text)
                                </a>
                            }
                        </p>
                    }
                    if post.hidden_reply {
                        <p class="reply-to">(t!(post_reply_hidden))</p>
                    }
                    if let Some(text) = &replying {
                        <p class="reply-to">(text)</p>
                    }
                    if !post.cw.is_empty() {
                        content_warning(
                            cw: &post.cw,
                            emojis: &post.emojis,
                            post_body(post: post, main: main, mine: mine)
                        )
                    } else if show_body {
                        post_body(post: post, main: main, mine: mine)
                    }
                    if let Some(quote) = &post.quote {
                        quote_card(quote: quote)
                    }
                    if main {
                        <p class="stamp">
                            <time datetime=(&created)>(&full)</time>
                            if let Some(client) = &stamp_client {
                                <span>(client)</span>
                            }
                        </p>
                    }
                    if let Some(footer) = footer {
                        (footer)
                    }
                    post_actions(post: post, big: main)
                </div>
            </div>
        </article>
    })
}

/// The text, media, link card and poll of a post: what a content warning hides.
#[component]
async fn post_body(cx: &Cx, post: &PostDto, main: bool, mine: bool) -> Result<impl View> {
    Ok(view! {
        <div class="post-body" data-c="post-body" data-main=(main.then_some(""))>
            if !post.text.is_empty() {
                <div class="text">
                    rich_text(
                        segments: &post.rich_text,
                        domain: post.author.domain.as_deref(),
                        emojis: &post.emojis
                    )
                </div>
            }
            if !post.media.is_empty() {
                media_grid(
                    media: &post.media,
                    sensitive: post.sensitive,
                    post: Some(post)
                )
            }
            if post.media.is_empty() {
                <div data-slot="card">
                    if let Some(card) = &post.card {
                        link_card(card: card)
                    }
                </div>
            }
            if post.poll.is_some() {
                poll_view(post: post, mine: mine)
            }
        </div>
    })
}

/// One entry of the post menu before it borrows its label.
struct Entry {
    id: &'static str,
    label: String,
    danger: bool,
    attrs: Attributes,
}

fn entry(id: &'static str, label: String, danger: bool, attrs: Attributes) -> Entry {
    Entry {
        id,
        label,
        danger,
        attrs,
    }
}

/// The entries of the post menu for this viewer, each carrying the hooks `post.js` and
/// `share.js` act on.
fn menu_entries(
    cx: &Cx,
    post: &PostDto,
    facts: MenuFacts,
    path: &str,
    profile: &str,
) -> Vec<Entry> {
    let author = &post.author;
    let acct = acct_of(&author.handle, author.domain.as_deref());
    let handle = handle_of(&author.handle, author.domain.as_deref());
    let id = post.id.to_string();
    let author_id = author.id.to_string();
    let say = |text: String| strip_isolates(&text);
    let mut entries = vec![
        entry(
            "link",
            t!(post_copy_link).resolve(cx).into_owned(),
            false,
            attributes! { data-act="copy-link" data-url=(path) },
        ),
        entry(
            "text",
            t!(post_copy_text).resolve(cx).into_owned(),
            false,
            attributes! { data-act="copy-text" data-text=(&post.text) },
        ),
        entry(
            "share",
            t!(post_share_menu).resolve(cx).into_owned(),
            false,
            attributes! {
                data-act="share"
                data-url=(path)
                data-text=(share_text(post))
                data-needs-share=""
                hidden=""
            },
        ),
    ];
    if post.edited_at.is_some() {
        entries.push(entry(
            "history",
            t!(post_edit_history).resolve(cx).into_owned(),
            false,
            attributes! { data-act="history" data-post-id=(&id) },
        ));
    }
    if let MenuFacts::Own { pinned, editable } = facts {
        if editable {
            entries.push(entry(
                "edit",
                t!(post_edit).resolve(cx).into_owned(),
                false,
                attributes! { data-act="edit" data-post-id=(&id) },
            ));
        }
        let pin_label = if pinned {
            t!(post_unpin).resolve(cx).into_owned()
        } else {
            t!(post_pin).resolve(cx).into_owned()
        };
        entries.push(entry(
            "pin",
            pin_label,
            false,
            attributes! {
                data-act="pin"
                data-post-id=(&id)
                data-pinned=(pinned.to_string())
            },
        ));
        entries.push(entry(
            "delete",
            t!(post_delete).resolve(cx).into_owned(),
            true,
            attributes! { data-act="delete" data-post-id=(&id) },
        ));
    } else {
        entries.push(entry(
            "view",
            say(t!(post_view_profile, handle: handle.as_str())
                .resolve(cx)
                .into_owned()),
            false,
            attributes! { data-act="goto" data-url=(profile) },
        ));
        if matches!(facts, MenuFacts::Other { hideable: true }) {
            let hide_label = if post.hidden_reply {
                t!(post_unhide_reply).resolve(cx).into_owned()
            } else {
                t!(post_hide_reply).resolve(cx).into_owned()
            };
            entries.push(entry(
                "hide",
                hide_label,
                false,
                attributes! {
                    data-act="hide"
                    data-post-id=(&id)
                    data-hidden=(post.hidden_reply.to_string())
                },
            ));
        }
        entries.push(entry(
            "mute",
            say(t!(post_mute, handle: handle.as_str())
                .resolve(cx)
                .into_owned()),
            false,
            attributes! {
                data-act="mute"
                data-acct=(&acct)
                data-user-id=(&author_id)
                data-handle=(&handle)
            },
        ));
        entries.push(entry(
            "report",
            t!(post_report).resolve(cx).into_owned(),
            true,
            attributes! { data-act="report" data-post-id=(&id) },
        ));
        entries.push(entry(
            "block",
            say(t!(post_block, handle: handle.as_str())
                .resolve(cx)
                .into_owned()),
            true,
            attributes! {
                data-act="block"
                data-acct=(&acct)
                data-user-id=(&author_id)
                data-handle=(&handle)
            },
        ));
    }
    entries
}

/// The entries as menu items; the attributes are cloned out of the entries so the labels stay
/// borrowed from them.
fn menu_items(entries: &[Entry]) -> Vec<MenuItem<'_>> {
    entries
        .iter()
        .map(|entry| MenuItem {
            id: entry.id,
            label: &entry.label,
            danger: entry.danger,
            attrs: entry.attrs.clone(),
            ..MenuItem::default()
        })
        .collect()
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;
    use tuita_server::dto::{ReplyToDto, SegmentDto};

    use super::*;
    use crate::{
        post::testing::{
            sample_card, sample_emoji, sample_image, sample_poll, sample_post, sample_quote,
            sample_user,
        },
        testing::{attr_values, render},
    };

    fn me() -> Me<'static> {
        Me {
            id: sample_user("ada").id,
            handle: "ada",
            pinned_post_id: None,
        }
    }

    #[component]
    async fn demo(
        cx: &Cx,
        post: PostDto,
        me: Option<Me<'static>>,
        main: bool,
    ) -> Result<impl View> {
        Ok(view! { card_view(post: &post, me: me, main: main) })
    }

    fn show(post: PostDto, me: Option<Me<'static>>, main: bool) -> String {
        render("en", |cx| {
            view! { demo(post: post, me: me, main: main) }.boxed()
        })
    }

    #[test]
    fn the_article_is_named_by_its_author_and_carries_the_hooks() {
        let html = show(sample_post(), None, false);
        assert!(html.contains(r#"class="post wash""#), "{html}");
        assert!(html.contains(r#"aria-label="Post by ADA""#), "{html}");
        assert!(html.contains(r#"data-post="#), "{html}");
        assert!(html.contains(r#"data-href="/@ada/"#), "{html}");
        assert!(html.contains(r#"data-hovercard="ada""#), "{html}");
        assert!(html.contains(r#"class="who""#), "{html}");
        assert!(
            html.contains(r#"<span class="handle">@ada</span>"#),
            "{html}"
        );
        assert!(html.contains(r#"class="time""#), "{html}");
        assert!(
            html.contains("<time datetime=\"2026-11-09T10:00:00Z\""),
            "{html}"
        );
        assert!(html.contains(r#"data-slot="card""#), "{html}");
        assert!(html.contains("hello world"), "{html}");
    }

    #[test]
    fn the_main_post_is_larger_has_no_time_link_and_no_click_through() {
        let html = show(sample_post(), None, true);
        assert!(html.contains(r#"class="post wash main""#), "{html}");
        assert!(!html.contains("data-href"), "{html}");
        assert!(!html.contains(r#"class="time""#), "{html}");
        assert!(html.contains(r#"<p class="stamp">"#), "{html}");
        assert!(html.contains(r#"class="actions big""#), "{html}");
    }

    #[test]
    fn media_poll_quote_and_card_are_drawn_in_order() {
        let mut post = sample_post();
        post.media = vec![sample_image(1, 640, 480, "a picture")];
        let html = show(post.clone(), None, false);
        assert!(html.contains(r#"role="group""#), "{html}");
        assert!(html.contains(r#"aria-label="1 attachment""#), "{html}");
        assert!(html.contains(r#"data-act="lightbox""#), "{html}");
        assert!(html.contains(r#"aria-label="a picture""#), "{html}");
        assert!(html.contains(r#"alt="a picture""#), "{html}");
        assert!(
            !html.contains(r#"class="card""#),
            "a link card hides behind media: {html}"
        );

        post.media.clear();
        post.card = Some(sample_card());
        post.poll = Some(sample_poll());
        post.quote = Some(sample_quote());
        let html = show(post, None, false);
        let slot = html.find(r#"data-slot="card""#).expect("card slot");
        let poll = html.find(r#"class="poll""#).expect("poll");
        let quote = html.find(r#"class="quote""#).expect("quote");
        let actions = html.find(r#"class="actions""#).expect("actions");
        assert!(slot < poll && poll < quote && quote < actions, "{html}");
        assert!(html.contains(r#"class="card""#), "{html}");
        assert!(html.contains("quoted words"), "{html}");
        assert!(html.contains(r#"class="choice""#), "{html}");
    }

    #[test]
    fn a_content_warning_hides_the_body_behind_a_button() {
        let mut post = sample_post();
        post.cw = "spoilers".to_owned();
        let html = show(post, None, false);
        assert!(html.contains(r#"class="cw""#), "{html}");
        assert!(html.contains(r#"aria-expanded="false""#), "{html}");
        assert!(html.contains(r#"class="body" hidden"#), "{html}");
        assert!(
            html.contains(r#"<span class="reason" translate="no">spoilers</span>"#),
            "{html}"
        );
    }

    #[test]
    fn custom_emotes_replace_their_tokens_in_text_and_names() {
        let mut post = sample_post();
        post.text = "hi :blob:".to_owned();
        post.rich_text = vec![SegmentDto::Text {
            text: "hi :blob:".to_owned(),
        }];
        post.emojis = vec![sample_emoji("blob")];
        post.author.display_name = ":blob: Ada".to_owned();
        post.author.emojis = vec![sample_emoji("blob")];
        let html = show(post, None, false);
        assert_eq!(html.matches(r#"class="emote""#).count(), 2, "{html}");
        assert!(html.contains(r#"alt=":blob:""#), "{html}");
    }

    #[test]
    fn the_menu_differs_for_the_author_and_for_others() {
        let mut fresh = sample_post();
        fresh.created_at = OffsetDateTime::now_utc();
        let own = show(fresh, Some(me()), false);
        for item in ["delete", "pin"] {
            assert!(own.contains(&format!(r#"data-id="{item}""#)), "{own}");
        }
        assert!(
            own.contains(r#"data-id="edit""#),
            "the post is fresh enough to edit: {own}"
        );
        assert!(!own.contains(r#"data-id="report""#), "{own}");

        let other = show(sample_post(), None, false);
        for item in ["mute", "report", "block", "view", "link", "text", "share"] {
            assert!(other.contains(&format!(r#"data-id="{item}""#)), "{other}");
        }
        assert!(!other.contains(r#"data-id="delete""#), "{other}");
        assert!(other.contains("Report post"), "{other}");
        assert!(other.contains("Block @ada"), "{other}");
    }

    #[test]
    fn replies_to_the_viewer_can_be_hidden_and_the_context_lines_show() {
        let mut post = sample_post();
        post.author = sample_user("bob");
        post.reply_to = Some(ReplyToDto {
            id: Uuid::from_u128(9),
            handle: Some("ada".to_owned()),
        });
        let html = show(post.clone(), Some(me()), false);
        assert!(html.contains(r#"data-id="hide""#), "{html}");
        assert!(html.contains("Replying to @ada"), "{html}");
        post.reposted_by = Some(sample_user("cy"));
        let html = show(post, Some(me()), false);
        assert!(html.contains(r#"<p class="context">"#), "{html}");
        assert!(html.contains("reposted"), "{html}");
        assert_eq!(
            attr_values(&html, "data-hovercard")
                .iter()
                .filter(|a| *a == "cy")
                .count(),
            1
        );
    }

    #[test]
    fn a_remote_post_names_its_domain_and_links_the_original() {
        let mut post = sample_post();
        post.author.domain = Some("x.example".to_owned());
        post.url = Some("https://x.example/@ada/1".to_owned());
        post.visibility = Visibility::Unlisted;
        let html = show(post, None, false);
        assert!(html.contains(r#"data-domain="x.example""#), "{html}");
        assert!(html.contains(r#"data-visibility="unlisted""#), "{html}");
        assert!(
            html.contains(r#"rel="noopener noreferrer nofollow ugc""#),
            "{html}"
        );
        assert!(html.contains("View on x.example"), "{html}");
        assert!(html.contains(r#"data-hovercard="ada@x.example""#), "{html}");
    }
}
