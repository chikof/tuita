//! The top of a profile: banner, picture, name, bio, facts, counts and the actions.
//!
//! The follow button and the menu items are plain markup that `follow.js` and `post.js` drive:
//! mute, unmute, block and unblock carry `data-profile-act` (`follow.js` runs them and reloads the
//! page), "Add to lists" and "Report" carry `data-act` (`post.js` opens their dialogs) and "Copy
//! profile link" is `share.js`'s `data-act="copy-link"`.

use time::format_description::well_known::Rfc3339;
use topcoat::{
    Result,
    context::Cx,
    runtime::Event,
    view::{View, attributes, component, view},
};
use tuita_server::dto::{KnownFollowersDto, Role, UserDto, UserRefDto};
use uuid::Uuid;

use crate::{
    api::{self, enc},
    fmt::{acct_of, compact, handle_of, host_of, month_year},
    icons::Icon,
    l10n::{strip_isolates, t},
    paths,
    post::{
        domain_chip::domain_chip,
        rich_text::{emote_text, rich_text, segments_of, short_url, user_name},
    },
    profile::follow_button::{FollowView, follow_button_view, may_follow},
    session,
    shell::parts::nav_button,
    ui::{
        avatar::{AvatarShape, avatar},
        avatar_stack::{StackPerson, avatar_stack},
        button::{ButtonSize, ButtonVariant},
        confirm_dialog::confirm_dialog,
        dither_field::{FieldKind, dither_field},
        icon::icon,
        menu::{MenuItem, menu},
        tag::tag,
    },
};

/// The id of the question the profile's block entry asks.
pub(crate) const BLOCK_DIALOG: &str = "confirm-profile-block";

/// The labels of the profile menu.
struct MenuLabels {
    copy: String,
    lists: String,
    mute: String,
    report: String,
    block: String,
}

/// The entries of the profile menu. Mute, unmute, block and unblock are `follow.js`'s
/// `data-profile-act`; the rest are `data-act` entries of `share.js` and `post.js`.
fn menu_items<'a>(
    cx: &Cx,
    user: &UserDto,
    labels: &'a MenuLabels,
    acct: &str,
    handle: &str,
    path: &str,
) -> Vec<MenuItem<'a>> {
    vec![
        MenuItem {
            id: "link",
            label: &labels.copy,
            attrs: attributes! { data-act="copy-link" data-url=(path) },
            ..MenuItem::default()
        },
        MenuItem {
            id: "lists",
            label: &labels.lists,
            attrs: attributes! { data-act="list-picker" data-acct=(acct) },
            ..MenuItem::default()
        },
        MenuItem {
            id: "mute",
            label: &labels.mute,
            attrs: attributes! {
                data-profile-act=(if user.relation.muted { "unmute" } else { "mute" })
                data-acct=(acct)
                data-handle=(handle)
            },
            ..MenuItem::default()
        },
        MenuItem {
            id: "report",
            label: &labels.report,
            danger: true,
            attrs: attributes! { data-act="report-user" data-acct=(acct) },
            ..MenuItem::default()
        },
        MenuItem {
            id: "block",
            label: &labels.block,
            danger: !user.relation.blocked,
            attrs: attributes! {
                data-profile-act=(if user.relation.blocked { "unblock" } else { "block" })
                data-acct=(acct)
                data-handle=(handle)
            },
            ..MenuItem::default()
        },
    ]
}

/// The profile header for the signed-in viewer; it asks the API for the followers the viewer
/// knows (none on the viewer's own profile or when the call fails).
#[component]
pub(crate) async fn profile_header(cx: &Cx, user: &UserDto) -> Result<impl View> {
    let viewer = session::viewer(cx).await?.as_ref().map(|v| v.me.user.id);
    let known = if viewer.is_none_or(|id| id == user.id) {
        None
    } else {
        let acct = acct_of(&user.handle, user.domain.as_deref());
        api::get::<KnownFollowersDto>(
            cx,
            &format!(
                "/users/{}/followed_by_known{}",
                enc(&acct),
                api::query(&[("limit", Some("3"))])
            ),
        )
        .await
        .ok()
    };
    Ok(view! { profile_header_view(user: user, viewer: viewer, known: known.as_ref()) })
}

/// What "Followed by …" says: the first two names, then how many more.
fn known_text(cx: &Cx, known: &KnownFollowersDto) -> String {
    let count = known.count;
    let names: Vec<&str> = known
        .users
        .iter()
        .map(|u: &UserRefDto| u.display_name.as_str())
        .collect();
    let first = names.first().copied().unwrap_or("");
    let second = names.get(1).copied().unwrap_or("");
    strip_isolates(
        &t!(
            profile_followed_by,
            count: if second.is_empty() { 1 } else { count },
            others: count - 2,
            a: first,
            b: second,
        )
        .resolve(cx),
    )
}

/// [`profile_header`] for a known viewer id and known followers. `scope` keeps the ids of the
/// menu and of the block question unique when one page shows several headers (the design
/// gallery); without it they are `profile-more` and [`BLOCK_DIALOG`], which `follow.js` opens.
#[component]
pub(crate) async fn profile_header_view(
    cx: &Cx,
    user: &UserDto,
    viewer: Option<Uuid>,
    known: Option<&KnownFollowersDto>,
    #[default]
    #[into]
    scope: Option<&str>,
) -> Result<impl View> {
    let menu_id = scope.map_or_else(
        || "profile-more".to_owned(),
        |s| format!("profile-more-{s}"),
    );
    let block_id = scope.map_or_else(
        || BLOCK_DIALOG.to_owned(),
        |s| format!("{BLOCK_DIALOG}-{s}"),
    );
    let mine = viewer == Some(user.id);
    let signed_in = viewer.is_some();
    let acct = acct_of(&user.handle, user.domain.as_deref());
    let handle = handle_of(&user.handle, user.domain.as_deref());
    let base = paths::profile(&acct);
    let shape = AvatarShape::from_name(user.avatar_shape.as_str()).unwrap_or_default();
    let can_message = !user.relation.blocked && !user.relation.blocks_you;
    let segments = segments_of(&user.bio);
    let created = user
        .created_at
        .format(&Rfc3339)
        .unwrap_or_else(|_| user.created_at.to_string());
    let joined = strip_isolates(&t!(profile_joined, date: month_year(cx, &created)).resolve(cx));
    let following = compact(cx, i64::from(user.counts.following));
    let followers = compact(cx, i64::from(user.counts.followers));
    let followers_label =
        strip_isolates(&t!(profile_followers, count: user.counts.followers).resolve(cx));
    let known_line = known.filter(|k| k.count > 0).map(|k| {
        let people: Vec<StackPerson<'_>> = k
            .users
            .iter()
            .map(|u| StackPerson {
                seed: u.handle.as_str(),
                alt: "",
                src: u.avatar_url.as_deref(),
            })
            .collect();
        (people, known_text(cx, k))
    });
    let original = user
        .url
        .as_deref()
        .filter(|url| !host_of(url).is_empty())
        .map(|url| {
            (
                url,
                strip_isolates(&t!(profile_original, domain: host_of(url)).resolve(cx)),
            )
        });
    let website = (!user.website.is_empty()).then(|| short_url(&user.website));

    let more_label = t!(profile_more).resolve(cx).into_owned();
    let more_sr = more_label.clone();
    let labels = MenuLabels {
        copy: t!(profile_copy_link).resolve(cx).into_owned(),
        lists: t!(profile_add_to_lists).resolve(cx).into_owned(),
        mute: if user.relation.muted {
            strip_isolates(&t!(profile_unmute, handle: handle.as_str()).resolve(cx))
        } else {
            strip_isolates(&t!(profile_mute, handle: handle.as_str()).resolve(cx))
        },
        report: strip_isolates(&t!(profile_report, handle: handle.as_str()).resolve(cx)),
        block: if user.relation.blocked {
            strip_isolates(&t!(profile_unblock, handle: handle.as_str()).resolve(cx))
        } else {
            strip_isolates(&t!(profile_block, handle: handle.as_str()).resolve(cx))
        },
    };
    let block_title = strip_isolates(&t!(block_title, handle: handle.as_str()).resolve(cx));
    let block_body = t!(block_body).resolve(cx).into_owned();
    let block_confirm = t!(block_confirm).resolve(cx).into_owned();
    let messages = paths::messages(&acct);
    let follow_state = FollowView::of(user.relation);
    let note_locked = user.locked && !mine && follow_state != FollowView::Following;

    Ok(view! {
        <section
            class="profile"
            data-c="profile-header"
            data-user-id=(user.id.to_string())
        >
            <div class="banner" data-ink=(user.ink.as_str())>
                if let Some(banner) = &user.banner_url {
                    <img src=(banner) alt="">
                } else {
                    dither_field(
                        field: FieldKind::Seeded,
                        seed: user.handle.as_str(),
                        cell: 6,
                        opacity: 0.55
                    )
                }
            </div>
            <div class="body">
                <div class="top">
                    <div class="ring">
                        avatar(
                            seed: user.handle.as_str(),
                            src: user.avatar_url.as_deref(),
                            alt: "",
                            shape: shape,
                            ink: user.ink.as_str(),
                            size: 96
                        )
                    </div>
                    <div class="actions">
                        if mine {
                            nav_button(
                                href: "/settings/profile",
                                variant: ButtonVariant::Outline,
                                (t!(profile_edit))
                            )
                        } else if signed_in {
                            menu(
                                id: &menu_id,
                                label: &more_label,
                                items: menu_items(cx, user, &labels, &acct, &handle, &base),
                                trigger: view! {
                                    icon(name: Icon::MoreHorizontal, size: 20)
                                    <span class="sr-only">(&more_sr)</span>
                                }.into(

                                )
                            )
                            if can_message {
                                nav_button(
                                    href: &messages,
                                    variant: ButtonVariant::Outline,
                                    (t!(profile_message))
                                )
                            }
                            follow_button_view(
                                user: user,
                                viewer: viewer,
                                size: ButtonSize::Md
                            )
                        }
                    </div>
                </div>

                if user.suspended {
                    <p class="alert" role="status">
                        icon(name: Icon::WarningDiamond, size: 16)
                        (t!(profile_suspended))
                    </p>
                }

                <div class="name">
                    user_name(
                        name: &user.display_name,
                        verified: user.verified,
                        bot: user.bot,
                        emojis: &user.emojis
                    )
                </div>
                <div class="line">
                    <span class="handle">(&handle)</span>
                    if let Some(domain) = &user.domain {
                        domain_chip(domain: domain)
                    }
                    if user.locked {
                        tag(
                            pill: true,
                            compact: true,
                            <span class="lock">
                                icon(name: Icon::Lock, size: 12)
                                (t!(profile_locked))
                            </span>
                        )
                    }
                    if user.relation.follows_you {
                        tag(pill: true, (t!(follows_you)))
                    }
                    if user.relation.muted {
                        tag(pill: true, (t!(profile_muted)))
                    }
                    if user.org {
                        tag(pill: true, (t!(profile_org)))
                    }
                    if user.staff == Some(Role::Admin) {
                        tag(pill: true, (t!(profile_role_admin)))
                    }
                    if user.staff == Some(Role::Moderator) {
                        tag(pill: true, (t!(profile_role_moderator)))
                    }
                </div>

                if !user.bio.is_empty() {
                    <p class="bio">
                        rich_text(
                            segments: &segments,
                            domain: user.domain.as_deref(),
                            emojis: &user.emojis
                        )
                    </p>
                }

                <div class="meta">
                    if !user.location.is_empty() {
                        <span class="item">
                            icon(name: Icon::MapPin, size: 16)
                            <span translate="no">
                                emote_text(text: &user.location, emojis: &user.emojis)
                            </span>
                        </span>
                    }
                    if let Some(affiliation) = &user.affiliation {
                        <span class="item">
                            icon(name: Icon::Briefcase, size: 16)
                            <a href=(format!("/@{}", affiliation.handle))>
                                (&affiliation.display_name)
                            </a>
                            if !affiliation.label.is_empty() {
                                <span>
                                    (t!(
                                        profile_affiliation_role,
                                        label: affiliation.label.as_str(),
                                    ))
                                </span>
                            }
                        </span>
                    }
                    if let Some(short) = &website {
                        <span class="item">
                            icon(name: Icon::Link, size: 16)
                            <a
                                href=(&user.website)
                                target="_blank"
                                rel="noopener noreferrer nofollow"
                            >
                                (short)
                            </a>
                        </span>
                    }
                    if let Some((url, text)) = original {
                        <span class="item">
                            icon(name: Icon::ExternalLink, size: 16)
                            <a
                                href=(url)
                                target="_blank"
                                rel="noopener noreferrer nofollow ugc"
                            >
                                (text)
                            </a>
                        </span>
                    }
                    <span class="item">
                        icon(name: Icon::Calendar, size: 16)
                        (joined)
                    </span>
                </div>

                <nav class="counts" aria-label=(t!(profile_counts))>
                    <a href=(paths::following(&acct))>
                        <b data-count="following">(following)</b>
                        " "
                        (t!(profile_following_label))
                    </a>
                    <a href=(paths::followers(&acct))>
                        <b data-count="followers">(followers)</b>
                        " "
                        <span data-count-label="followers">(followers_label)</span>
                    </a>
                </nav>

                if let Some((people, text)) = known_line {
                    <a class="known" href=(paths::followers(&acct))>
                        avatar_stack(people: people, size: 24, max: 3)
                        <span>(text)</span>
                    </a>
                }

                if let Some(domain) = &user.domain {
                    <p class="note">
                        (t!(profile_remote_note, domain: domain.as_str()))
                    </p>
                }
                if note_locked {
                    <p class="note">(t!(profile_locked_hint))</p>
                }
            </div>
        </section>
        if !mine && signed_in && may_follow(viewer, user) {
            confirm_dialog(
                id: &block_id,
                title: &block_title,
                body: &block_body,
                confirm_label: &block_confirm,
                danger: true,
                action: $(|_e: Event| raw!("dispatchEvent(new Event('profile-block'))"))
            )
        }
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::{profile::testing, testing::render};

    fn show(user: UserDto, viewer: Option<Uuid>, known: Option<KnownFollowersDto>) -> String {
        render("en", |cx| {
            view! {
                profile_header_view(user: &user, viewer: viewer, known: known.as_ref())
            }
            .boxed()
        })
    }

    #[test]
    fn someone_elses_profile_offers_menu_message_and_follow() {
        let mut ada = testing::user("ada");
        ada.bio = "pixels".to_owned();
        let html = show(ada, Some(Uuid::from_u128(1)), None);
        assert!(html.contains(r#"aria-label="More""#), "{html}");
        assert!(html.contains("Message"), "{html}");
        assert!(html.contains("data-follow"), "{html}");
        assert!(html.contains(r#"data-act="list-picker""#), "{html}");
        assert!(html.contains(r#"data-act="report-user""#), "{html}");
        assert!(html.contains(r#"data-profile-act="mute""#), "{html}");
        assert!(html.contains(r#"data-profile-act="block""#), "{html}");
        assert!(html.contains("Mute @ada"), "{html}");
        assert!(html.contains(r#"id="confirm-profile-block""#), "{html}");
        assert!(
            html.contains("<b data-count=\"followers\">1.2K</b>"),
            "{html}"
        );
        assert!(html.contains("Joined November 2026"), "{html}");
    }

    #[test]
    fn your_own_profile_offers_edit_only() {
        let ada = testing::user("ada");
        let me = ada.id;
        let html = show(ada, Some(me), None);
        assert!(html.contains("Edit profile"), "{html}");
        assert!(!html.contains("data-follow"), "{html}");
        assert!(!html.contains(r#"aria-label="More""#), "{html}");
        assert!(!html.contains("confirm-profile-block"), "{html}");
    }

    #[test]
    fn a_remote_account_shows_its_domain_a_link_to_the_original_and_the_remote_note() {
        let mut mara = testing::remote("mara", "fedi.example");
        mara.locked = true;
        let html = show(mara, Some(Uuid::from_u128(1)), None);
        assert!(html.contains("@mara@fedi.example"), "{html}");
        assert!(html.contains("View profile on fedi.example"), "{html}");
        assert!(
            html.contains(r#"href="https://fedi.example/@mara""#),
            "{html}"
        );
        assert!(html.contains("A copy from"), "{html}");
        assert!(html.contains("Follow requests need this account"), "{html}");
        assert!(html.contains("Approves followers"), "{html}");
    }

    #[test]
    fn a_blocked_account_offers_to_unblock_from_the_menu() {
        let mut bob = testing::user("bob");
        bob.relation.blocked = true;
        let html = show(bob, Some(Uuid::from_u128(1)), None);
        assert!(html.contains(r#"data-profile-act="unblock""#), "{html}");
        assert!(html.contains("Unblock @bob"), "{html}");
        assert!(!html.contains("data-follow"), "{html}");
        assert!(!html.contains(">Message<"), "{html}");
    }

    #[test]
    fn followers_you_know_are_named_with_a_count_of_the_rest() {
        let ada = testing::user("ada");
        let known = KnownFollowersDto {
            users: vec![
                crate::post::testing::sample_user("cy"),
                crate::post::testing::sample_user("di"),
            ],
            count: 5,
        };
        let html = show(ada, Some(Uuid::from_u128(1)), Some(known));
        assert!(
            html.contains("Followed by CY, DI and 3 others you follow"),
            "{html}"
        );
    }
}
