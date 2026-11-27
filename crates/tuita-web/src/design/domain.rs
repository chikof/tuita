//! Gallery sections: domain.
//!
//! The gallery fakes a signed-in viewer; here the view-only variants take the sample viewer
//! explicitly (`Me`, a viewer id, a [`Viewer`]), so the page renders for a guest too.

use topcoat::{
    Result,
    context::Cx,
    runtime::{Event, signal},
    view::{View, attributes, component, view},
};
use tuita_server::dto::{AccountsDto, BadgesDto, MeDto, QuoteDto, Relation, SegmentDto, UserDto};

use super::{
    sample,
    section::{row, section},
};
use crate::{
    auth::language_picker::language_picker,
    compose::{compose_dialog::compose_dialog, composer::composer},
    feed::{list::post_list, new_posts::new_posts_pill},
    notifications::row::notification_row,
    post::{
        content_warning::content_warning,
        heart::heart,
        post_actions::post_actions,
        post_card::{Me, ThreadLine, card_view},
        post_thread::post_thread,
        quote_card::quote_card,
        rich_text::{rich_text, user_name},
    },
    profile::{
        follow_button::follow_button_view, profile_header::profile_header_view,
        user_row::user_row_view,
    },
    session::Viewer,
    shell::{
        account_menu::account_menu,
        bottom_bar::bottom_bar,
        drawer::drawer,
        page_bar::{Heading, page_bar},
        rail_nav::rail_nav,
        shortcuts_dialog::{DIALOG as SHORTCUTS, shortcuts_dialog},
        top_bar::top_bar,
    },
    ui::{
        button::{ButtonVariant, button},
        confirm_dialog::confirm_dialog,
    },
};

/// The id of the drawer of the gallery: the section is named `drawer`, so the dialog cannot be.
const DRAWER: &str = "domain-drawer";
/// The id of the account menu of the gallery: the rail already holds the default one.
const ACCOUNT_MENU: &str = "domain-account";
/// The id of the confirm dialog of the gallery.
const CONFIRM: &str = "domain-confirm";

/// The signed-in viewer the gallery plays: the sample account, two accounts, three unread
/// notifications.
fn viewer() -> Viewer {
    Viewer {
        me: sample::sample_me(),
        accounts: AccountsDto {
            accounts: sample::accounts(),
            max: 3,
        },
        badges: BadgesDto {
            notifications: 3,
            messages: 0,
            reports: None,
        },
    }
}

/// What the cards need to know about the viewer.
fn me_of(me: &MeDto) -> Me<'_> {
    Me {
        id: me.user.id,
        handle: &me.user.handle,
        pinned_post_id: me.user.pinned_post_id,
    }
}

/// `user` as someone the viewer follows.
fn followed(user: UserDto) -> UserDto {
    UserDto {
        relation: Relation {
            following: true,
            follows_you: false,
            blocked: false,
            blocks_you: false,
            muted: false,
            follow_pending: false,
        },
        ..user
    }
}

/// One component per section so each `view!` frame stays small (debug builds give a frame the size of its
/// expansion). The post cards render in a loop for the same reason: unrolled, nine card futures share one frame.
#[component]
pub(crate) async fn domain(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        user_name_section()
        rich_text_section()
        heart_section()
        content_warning_section()
        quote_card_section()
        post_actions_section()
        post_card_section()
        post_thread_section()
        user_row_section()
        follow_button_section()
        profile_header_section()
        notification_row_section()
        composer_section()
        infinite_list_section()
        new_posts_pill_section()
        page_bar_section()
        rail_nav_section()
        top_bar_section()
        bottom_bar_section()
        drawer_section()
        account_menu_section()
        shortcuts_dialog_section()
        confirm_dialog_section()
        language_picker_section()
        landing_section()
        app_shell_section()
    })
}

#[component]
async fn heart_demo(cx: &Cx) -> Result<impl View> {
    let on = signal(cx, || false);
    Ok(view! {
        button(
            variant: ButtonVariant::Outline,
            attrs: attributes! { @click=$(|_e: Event| on.set(!on.get())) },
            "Toggle"
        )
        <span :hidden=$(on.get())>heart(filled: false)</span>
        <span :hidden=$(!on.get())>heart(filled: true, pop: true)</span>
    })
}

#[component]
async fn user_name_section(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        section(
            id: "user-name",
            title: "UserName",
            note: "display name, verified mark, bot label",
            row(
                user_name(name: "Ada Lovelace")
                user_name(name: "Ada Lovelace", verified: true)
                user_name(name: "Weather bot", bot: true)
            )
        )
    })
}

#[component]
async fn rich_text_section(cx: &Cx) -> Result<impl View> {
    let segments = vec![
        SegmentDto::Text {
            text: "Line one\nLine two with ".to_owned(),
        },
        SegmentDto::Mention {
            handle: "bob".to_owned(),
            domain: None,
        },
        SegmentDto::Text {
            text: " and ".to_owned(),
        },
        SegmentDto::Hashtag {
            tag: "dither".to_owned(),
            text: "dither".to_owned(),
        },
        SegmentDto::Text {
            text: " and ".to_owned(),
        },
        SegmentDto::Link {
            url: "https://example.com/a/very/long/path/that/gets/cut/short".to_owned(),
        },
    ];
    Ok(view! {
        section(
            id: "rich-text",
            title: "RichText",
            note: "text, mentions, hashtags and links; whitespace kept",
            row(rich_text(segments: &segments))
        )
    })
}

#[component]
async fn heart_section(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        section(
            id: "heart",
            title: "Heart",
            note: "pixel heart; liking fills it through four Bayer levels",
            row(
                heart(filled: false)
                heart(filled: true)
                heart_demo()
            )
        )
    })
}

#[component]
async fn content_warning_section(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        section(
            id: "content-warning",
            title: "ContentWarning",
            note: "a dither veil that opens",
            row(
                <div class="frame" data-c="design-domain">
                    content_warning(
                        cw: "Spoilers for the finale",
                        <p>"The butler did it."</p>
                    )
                </div>
            )
        )
    })
}

#[component]
async fn quote_card_section(cx: &Cx) -> Result<impl View> {
    let quoting = sample::quoting();
    let gone = quoting.quote.clone().map(|quote| QuoteDto {
        available: false,
        author: None,
        text: String::new(),
        rich_text: Vec::new(),
        ..quote
    });
    Ok(view! {
        section(
            id: "quote-card",
            title: "QuoteCard",
            note: "embedded post; unavailable state",
            <div class="frame stack" data-c="design-domain">
                if let Some(quote) = &quoting.quote {
                    quote_card(quote: quote)
                }
                if let Some(quote) = &gone {
                    quote_card(quote: quote)
                }
            </div>
        )
    })
}

#[component]
async fn post_actions_section(cx: &Cx) -> Result<impl View> {
    let plain = sample::plain();
    let liked = sample::liked();
    let limited = sample::limited();
    Ok(view! {
        section(
            id: "post-actions",
            title: "PostActions",
            note: "reply, repost menu, like, bookmark, share",
            <div class="frame stack" data-c="design-domain">
                post_actions(post: &plain)
                post_actions(post: &liked)
                post_actions(post: &limited)
                post_actions(post: &plain, big: true)
            </div>
        )
    })
}

#[component]
async fn post_card_section(cx: &Cx) -> Result<impl View> {
    let sam = sample::sample_me();
    let cards = [
        (sample::plain(), None, false),
        (sample::liked(), None, false),
        (sample::warned(), None, false),
        (sample::reposted(), None, false),
        (sample::pinned(), None, false),
        (sample::reply(), None, false),
        (sample::quoting(), None, false),
        (sample::plain(), Some(ThreadLine::Down), false),
        (sample::plain(), None, true),
    ];
    Ok(view! {
        section(
            id: "post-card",
            title: "PostCard",
            note: "plain, liked, content warning, repost, pinned, reply, quote, thread line",
            <div class="frame" data-c="design-domain">
                #[key(at)]
                for (at, (post, thread, main)) in cards.iter().enumerate() {
                    card_view(
                        post: post,
                        me: Some(me_of(&sam)),
                        thread: *thread,
                        main: *main
                    )
                }
            </div>
        )
    })
}

#[component]
async fn post_thread_section(cx: &Cx) -> Result<impl View> {
    let thread = sample::thread();
    Ok(view! {
        section(
            id: "post-thread",
            title: "PostThread",
            note: "the main post, stats, inline reply and replies",
            <div class="frame" data-c="design-domain">
                post_thread(thread: &thread, bar: false)
            </div>
        )
    })
}

#[component]
async fn user_row_section(cx: &Cx) -> Result<impl View> {
    let sam = sample::sample_me();
    let alice = sample::user_of(&sample::ada());
    let quiet_bob = UserDto {
        bio: String::new(),
        ..followed(sample::user_of(&sample::bob()))
    };
    Ok(view! {
        section(
            id: "user-row",
            title: "UserRow",
            note: "avatar, name, bio and follow button",
            <div class="frame" data-c="design-domain">
                user_row_view(user: &alice, viewer: Some(sam.user.id))
                user_row_view(user: &quiet_bob, viewer: Some(sam.user.id))
            </div>
        )
    })
}

#[component]
async fn follow_button_section(cx: &Cx) -> Result<impl View> {
    let sam = sample::sample_me();
    let alice = sample::user_of(&sample::ada());
    let bob = followed(sample::user_of(&sample::bob()));
    Ok(view! {
        section(
            id: "follow-button",
            title: "FollowButton",
            note: "optimistic; hidden for yourself",
            row(
                follow_button_view(user: &alice, viewer: Some(sam.user.id))
                follow_button_view(user: &bob, viewer: Some(sam.user.id))
            )
        )
    })
}

#[component]
async fn profile_header_section(cx: &Cx) -> Result<impl View> {
    let sam = sample::sample_me();
    Ok(view! {
        section(
            id: "profile-header",
            title: "ProfileHeader",
            note: "banner is an animated DitherField seeded by the handle, in the profile's ink",
            <div class="frame" data-c="design-domain">
                profile_header_view(
                    user: &sam.user,
                    viewer: Some(sam.user.id),
                    known: None
                )
            </div>
        )
    })
}

#[component]
async fn notification_row_section(cx: &Cx) -> Result<impl View> {
    let sam = sample::sample_me();
    let groups = sample::notifications();
    Ok(view! {
        section(
            id: "notification-row",
            title: "NotificationRow",
            note: "grouped actors; unread has an ink bar",
            <div class="frame" data-c="design-domain">
                #[key(group.id.as_bytes())]
                for (at, group) in groups.iter().enumerate() {
                    notification_row(
                        group: group,
                        unread: at == 0,
                        me: Some(me_of(&sam))
                    )
                }
            </div>
        )
    })
}

#[component]
async fn composer_section(cx: &Cx) -> Result<impl View> {
    let plain = sample::plain();
    let quote_id = plain.id.to_string();
    Ok(view! {
        section(
            id: "composer",
            title: "Composer",
            note: "text, content warning, reply policy, counter, autocomplete",
            <div class="frame" data-c="design-domain">
                composer()
                composer(reply_to: Some(&plain))
            </div>
            row(
                button(
                    variant: ButtonVariant::Outline,
                    attrs: attributes! { data-compose="new" },
                    "Open the compose dialog"
                )
                button(
                    variant: ButtonVariant::Outline,
                    attrs: attributes! { data-compose="quote" data-post-id=(&quote_id) },
                    "Quote dialog"
                )
            )
            compose_dialog()
        )
    })
}

#[component]
async fn infinite_list_section(cx: &Cx) -> Result<impl View> {
    let posts = vec![sample::plain(), sample::liked()];
    Ok(view! {
        section(
            id: "infinite-list",
            title: "InfiniteList",
            note: "sentinel loads pages; skeleton rows; empty state",
            <div class="frame" data-c="design-domain">
                post_list(
                    label: "Posts",
                    posts: &posts,
                    empty: Some(view! { <p class="empty">"Nothing"</p> }.into())
                )
            </div>
            <div class="frame" data-c="design-domain">
                post_list(
                    label: "Empty list",
                    posts: &[],
                    empty: Some(
                        view! {
                            <p class="empty">
                                "An empty list shows its empty snippet."
                            </p>
                        }.into(

                        ),
                    )
                )
            </div>
        )
    })
}

#[component]
async fn new_posts_pill_section(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        section(
            id: "new-posts-pill",
            title: "NewPostsPill",
            note: "appears when the timeline has newer posts",
            <div class="frame pill" data-c="design-domain">
                new_posts_pill(count: 4)
            </div>
        )
    })
}

#[component]
async fn page_bar_section(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        section(
            id: "page-bar",
            title: "PageBar",
            note: "sticky title bar with back, actions and a row below",
            <div class="frame" data-c="design-domain">
                page_bar(
                    title: "Explore",
                    sub: "sample",
                    back: true,
                    heading: Heading::H2
                )
            </div>
        )
    })
}

#[component]
async fn rail_nav_section(cx: &Cx) -> Result<impl View> {
    let viewer = viewer();
    Ok(view! {
        section(
            id: "rail-nav",
            title: "RailNav",
            note: "desktop left rail: nav, badge, compose, account chip",
            <div class="frame tall" data-c="design-domain">
                rail_nav(viewer: &viewer, path: "/design")
            </div>
        )
    })
}

#[component]
async fn top_bar_section(cx: &Cx) -> Result<impl View> {
    let viewer = viewer();
    Ok(view! {
        section(
            id: "top-bar",
            title: "TopBar",
            note: "mobile top bar: avatar opens the drawer",
            <div class="frame" data-c="design-domain">
                top_bar(viewer: &viewer, demo: true, drawer: DRAWER)
            </div>
        )
    })
}

#[component]
async fn bottom_bar_section(cx: &Cx) -> Result<impl View> {
    let viewer = viewer();
    Ok(view! {
        section(
            id: "bottom-bar",
            title: "BottomBar",
            note: "mobile bottom bar; hides when scrolling down",
            <div class="frame bottom" data-c="design-domain">
                bottom_bar(viewer: &viewer, path: "/design", demo: true)
            </div>
        )
    })
}

#[component]
async fn drawer_section(cx: &Cx) -> Result<impl View> {
    let viewer = viewer();
    Ok(view! {
        section(
            id: "drawer",
            title: "Drawer",
            note: "mobile drawer from the avatar: profile, links, accounts",
            row(
                button(
                    variant: ButtonVariant::Outline,
                    attrs: attributes! { command="show-modal" commandfor=(DRAWER) },
                    "Open the drawer"
                )
            )
            drawer(viewer: &viewer, id: DRAWER)
        )
    })
}

#[component]
async fn account_menu_section(cx: &Cx) -> Result<impl View> {
    let viewer = viewer();
    Ok(view! {
        section(
            id: "account-menu",
            title: "AccountMenu",
            note: "switch, add, log out",
            row(account_menu(viewer: &viewer, id: ACCOUNT_MENU))
        )
    })
}

#[component]
async fn shortcuts_dialog_section(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        section(
            id: "shortcuts-dialog",
            title: "ShortcutsDialog",
            note: "the table behind the ? key",
            row(
                button(
                    variant: ButtonVariant::Outline,
                    attrs: attributes! { command="show-modal" commandfor=(SHORTCUTS) },
                    "Show shortcuts"
                )
            )
            shortcuts_dialog()
        )
    })
}

#[component]
async fn confirm_dialog_section(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        section(
            id: "confirm-dialog",
            title: "ConfirmDialog",
            note: "promise-based confirmation",
            row(
                button(
                    variant: ButtonVariant::Danger,
                    attrs: attributes! { command="show-modal" commandfor=(CONFIRM) },
                    "Ask to delete"
                )
            )
            confirm_dialog(
                id: CONFIRM,
                title: "Delete post?",
                body: "This cannot be undone.",
                confirm_label: "Delete",
                danger: true,
                action: $(|_e: Event| raw!(
                        "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:'Deleted (demo)'}))",
                        (),
                    ))
            )
        )
    })
}

#[component]
async fn language_picker_section(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        section(
            id: "language-picker",
            title: "LanguagePicker",
            note: "twelve interface languages, each named in itself; stores the choice and reloads; on Settings and the landing page",
            <div class="picker" data-c="design-domain">language_picker()</div>
        )
    })
}

#[component]
async fn landing_section(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        section(
            id: "landing",
            title: "Landing",
            note: "signed-out bento at /: hero field, live numbers, clock, trends, rules",
            row(<a href="/">"Open the landing page"</a>)
        )
    })
}

#[component]
async fn app_shell_section(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        section(
            id: "app-shell",
            title: "AppShell",
            note: "rail on desktop; top bar, bottom bar and drawer on mobile",
            row(<a href="/explore">"Open the app (sign in first)"</a>)
        )
    })
}
