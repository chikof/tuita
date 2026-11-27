//! A profile page: bar, header and the tabs with their post lists.
//!
//! The tab and the media filter are URL state (`?tab=replies|media|likes`, `&media=photo|video`),
//! so every tab is a link and a full page load. More posts come through `?before=` fragments.

use topcoat::{
    Result,
    context::Cx,
    view::{View, attributes, component, view},
};
use tuita_server::dto::{Page, PostDto, ProfileTab, UserDto};

use crate::{
    api::{self, enc},
    feed::list::post_items,
    fmt::{acct_of, compact, handle_of},
    icons::Icon,
    l10n::{strip_isolates, t},
    paths,
    profile::{
        list::{infinite_list, next_href},
        profile_header::profile_header,
    },
    session,
    shell::page_bar::page_bar,
    ui::{
        button::{ButtonVariant, button},
        empty_state::empty_state,
        icon::icon,
        link_tabs::{LinkTab, link_tabs},
        popover::popover,
    },
};

/// The photo and video filter of the media tab.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum MediaFilter {
    #[default]
    All,
    Photo,
    Video,
}

impl MediaFilter {
    /// The filter a `?media=` value names; anything else is all media.
    pub(crate) fn parse(text: Option<&str>) -> Self {
        match text {
            Some("photo") => Self::Photo,
            Some("video") => Self::Video,
            _ => Self::All,
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Photo => "photo",
            Self::Video => "video",
        }
    }
}

/// The tab a `?tab=` value names; anything else is the posts tab.
pub(crate) fn tab_of(text: Option<&str>) -> ProfileTab {
    text.and_then(ProfileTab::parse)
        .unwrap_or(ProfileTab::Posts)
}

/// `/@ada`, `/@ada?tab=media` or `/@ada?tab=media&media=photo`.
fn tab_href(base: &str, tab: ProfileTab, kind: MediaFilter) -> String {
    match (tab, kind) {
        (ProfileTab::Posts, _) => base.to_owned(),
        (ProfileTab::Media, MediaFilter::Photo | MediaFilter::Video) => {
            format!("{base}?tab=media&media={}", kind.name())
        }
        _ => format!("{base}?tab={}", tab.as_str()),
    }
}

/// The labels of the four tabs.
struct TabLabels {
    posts: String,
    replies: String,
    media: String,
    likes: String,
}

/// The four tabs, each a link.
fn tab_items<'a>(base: &str, labels: &'a TabLabels) -> Vec<LinkTab<'a>> {
    [
        (ProfileTab::Posts, &labels.posts),
        (ProfileTab::Replies, &labels.replies),
        (ProfileTab::Media, &labels.media),
        (ProfileTab::Likes, &labels.likes),
    ]
    .into_iter()
    .map(|(tab, label)| LinkTab {
        id: tab.as_str(),
        label,
        href: tab_href(base, tab, MediaFilter::All),
    })
    .collect()
}

/// `GET /users/<acct>/posts` for a tab, the page after `before` when given.
pub(crate) async fn fetch_posts(
    cx: &Cx,
    acct: &str,
    tab: ProfileTab,
    kind: MediaFilter,
    before: Option<&str>,
) -> Result<Page<PostDto>> {
    let kind = (tab == ProfileTab::Media && kind != MediaFilter::All).then(|| kind.name());
    let query = api::query(&[
        ("tab", Some(tab.as_str())),
        ("kind", kind),
        ("before", before),
    ]);
    Ok(api::get::<Page<PostDto>>(cx, &format!("/users/{}/posts{query}", enc(acct))).await?)
}

/// A page of posts for `infinite.js`: the entries and the link to the next page, nothing else.
#[component]
pub(crate) async fn profile_fragment(
    cx: &Cx,
    acct: &str,
    tab: ProfileTab,
    kind: MediaFilter,
    before: Option<&str>,
) -> Result<impl View> {
    let page = fetch_posts(cx, acct, tab, kind, before).await?;
    let more = page.next.as_deref().map(|cursor| next_href(cx, cursor));
    Ok(view! { post_items(posts: &page.items, more: more.as_deref()) })
}

/// The title and body of a tab with nothing in it.
fn blank(
    cx: &Cx,
    user: &UserDto,
    tab: ProfileTab,
    kind: MediaFilter,
    mine: bool,
) -> (String, String) {
    let who = handle_of(&user.handle, user.domain.as_deref());
    let text = |title: String, body: String| (title, body);
    match tab {
        ProfileTab::Posts if mine => text(
            t!(profile_empty_posts_self_title).resolve(cx).into_owned(),
            t!(profile_empty_posts_self_body).resolve(cx).into_owned(),
        ),
        ProfileTab::Posts => text(
            strip_isolates(&t!(profile_empty_posts_title, handle: who.as_str()).resolve(cx)),
            t!(profile_empty_posts_body).resolve(cx).into_owned(),
        ),
        ProfileTab::Replies => text(
            t!(profile_empty_replies_title).resolve(cx).into_owned(),
            t!(profile_empty_replies_body).resolve(cx).into_owned(),
        ),
        ProfileTab::Media if kind == MediaFilter::Video => text(
            t!(profile_empty_videos_title).resolve(cx).into_owned(),
            t!(profile_empty_videos_body).resolve(cx).into_owned(),
        ),
        ProfileTab::Media => text(
            t!(profile_empty_media_title).resolve(cx).into_owned(),
            t!(profile_empty_media_body).resolve(cx).into_owned(),
        ),
        ProfileTab::Likes => text(
            t!(profile_empty_likes_title).resolve(cx).into_owned(),
            t!(profile_empty_likes_body).resolve(cx).into_owned(),
        ),
    }
}

/// The page of an account that was found: the bar, the header, and the tabs with their posts, or
/// the explanation when the viewer blocked the account or was blocked by it.
#[component]
pub(crate) async fn profile_view(
    cx: &Cx,
    user: &UserDto,
    tab: ProfileTab,
    kind: MediaFilter,
    before: Option<&str>,
) -> Result<impl View> {
    let viewer = session::viewer(cx).await?.as_ref().map(|v| v.me.user.id);
    let mine = viewer == Some(user.id);
    let acct = acct_of(&user.handle, user.domain.as_deref());
    let handle = handle_of(&user.handle, user.domain.as_deref());
    let base = paths::profile(&acct);
    let sub = strip_isolates(
        &t!(
            profile_posts,
            count: user.counts.posts,
            compact: compact(cx, i64::from(user.counts.posts)),
        )
        .resolve(cx),
    );
    let hidden = user.relation.blocked || user.relation.blocks_you;
    let page = if hidden {
        None
    } else {
        Some(fetch_posts(cx, &acct, tab, kind, before).await?)
    };
    let more = page
        .as_ref()
        .and_then(|page| page.next.as_deref())
        .map(|cursor| next_href(cx, cursor));
    let tab_labels = TabLabels {
        posts: t!(profile_tab_posts).resolve(cx).into_owned(),
        replies: t!(profile_tab_replies).resolve(cx).into_owned(),
        media: t!(profile_tab_media).resolve(cx).into_owned(),
        likes: t!(profile_tab_likes).resolve(cx).into_owned(),
    };
    let tabs_label = t!(profile_tabs).resolve(cx).into_owned();
    let filter_label = t!(profile_media_filter).resolve(cx).into_owned();
    let kinds = [
        (
            MediaFilter::All,
            t!(profile_media_all).resolve(cx).into_owned(),
        ),
        (
            MediaFilter::Photo,
            t!(profile_media_photos).resolve(cx).into_owned(),
        ),
        (
            MediaFilter::Video,
            t!(profile_media_videos).resolve(cx).into_owned(),
        ),
    ];
    let current_kind = kinds
        .iter()
        .find(|(option, _)| *option == kind)
        .map(|(_, label)| label.clone())
        .unwrap_or_default();
    let kind_links: Vec<(bool, String, String)> = kinds
        .into_iter()
        .map(|(option, label)| {
            (
                option == kind,
                tab_href(&base, ProfileTab::Media, option),
                label,
            )
        })
        .collect();
    let empty = page.as_ref().is_some_and(|page| page.items.is_empty());
    let (empty_title, empty_body) = blank(cx, user, tab, kind, mine);
    let blocked_title =
        strip_isolates(&t!(profile_blocked_title, handle: handle.as_str()).resolve(cx));
    let hidden_body = strip_isolates(&t!(profile_hidden_body, handle: handle.as_str()).resolve(cx));
    let blocked_context = format!("blocked:{}", user.handle);
    let hidden_context = format!("hidden:{}", user.handle);
    let empty_context = format!("{}:{}", tab.as_str(), user.handle);
    let posts_label = t!(profile_posts_label).resolve(cx).into_owned();
    let unblock = t!(profile_unblock_button).resolve(cx).into_owned();
    let blocked_body = t!(profile_blocked_body).resolve(cx).into_owned();
    let hidden_title = t!(profile_hidden_title).resolve(cx).into_owned();

    Ok(view! {
        page_bar(back: true, title: &user.display_name, sub: sub.as_str())
        profile_header(user: user)
        if user.relation.blocked {
            empty_state(
                context: &blocked_context,
                title: &blocked_title,
                body: blocked_body.as_str(),
                action: Some(
                    view! {
                        button(
                            variant: ButtonVariant::Danger,
                            attrs: attributes! {
                                data-profile-act="unblock"
                                data-acct=(&acct)
                                data-handle=(&handle)
                            },
                            (&unblock)
                        )
                    }.into(

                    ),
                )
            )
        } else if user.relation.blocks_you {
            empty_state(
                context: &hidden_context,
                title: &hidden_title,
                body: hidden_body.as_str()
            )
        } else {
            link_tabs(
                items: tab_items(&base, &tab_labels),
                selected: tab.as_str(),
                label: &tabs_label
            )
            if tab == ProfileTab::Media {
                <div class="filter" data-c="media-filter">
                    popover(
                        id: "media-filter",
                        label: &filter_label,
                        trigger: view! {
                            (&current_kind)
                            icon(name: Icon::ChevronDown, size: 16)
                        }.into(

                        ),
                        <div class="kinds" role="radiogroup" aria-label=(&filter_label)>
                            for (selected, href, label) in kind_links {
                                <a
                                    role="radio"
                                    aria-checked=(selected.to_string())
                                    class="kind"
                                    href=(href)
                                >
                                    <span class="mark">
                                        if selected {
                                            icon(name: Icon::Check, size: 16)
                                        }
                                    </span>
                                    (label)
                                </a>
                            }
                        </div>
                    )
                </div>
            }
            if let Some(page) = &page {
                infinite_list(
                    label: &posts_label,
                    post_items(posts: &page.items, more: more.as_deref())
                )
            }
            if empty {
                empty_state(
                    context: &empty_context,
                    title: &empty_title,
                    body: empty_body.as_str()
                )
            }
        }
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn the_tab_and_the_media_filter_fall_back_to_posts_and_all_media() {
        assert_eq!(tab_of(None), ProfileTab::Posts);
        assert_eq!(tab_of(Some("media")), ProfileTab::Media);
        assert_eq!(tab_of(Some("nope")), ProfileTab::Posts);
        assert_eq!(MediaFilter::parse(Some("video")), MediaFilter::Video);
        assert_eq!(MediaFilter::parse(Some("nope")), MediaFilter::All);
        assert_eq!(MediaFilter::parse(None), MediaFilter::All);
    }

    #[test]
    fn tab_links_carry_the_tab_and_only_the_media_tab_carries_a_filter() {
        assert_eq!(
            tab_href("/@ada", ProfileTab::Posts, MediaFilter::Photo),
            "/@ada"
        );
        assert_eq!(
            tab_href("/@ada", ProfileTab::Replies, MediaFilter::All),
            "/@ada?tab=replies"
        );
        assert_eq!(
            tab_href("/@ada", ProfileTab::Media, MediaFilter::All),
            "/@ada?tab=media"
        );
        assert_eq!(
            tab_href("/@ada", ProfileTab::Media, MediaFilter::Photo),
            "/@ada?tab=media&media=photo"
        );
    }
}
