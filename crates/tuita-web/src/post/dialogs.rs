//! The dialogs of a post: edit, edit history, report, add to list and who liked, reposted or
//! quoted it.
//!
//! Each is one `ui::dialog` with the id [`DIALOG_ID`]. `post.js` fetches it from a `/_tuita/dialog/..`
//! route, puts it into `#post-dialogs`, opens it with `showModal()` and removes it when it closes;
//! the forms are handled there (`form[data-post-form]`), not by Topcoat, because inserted markup is
//! not hydrated.

use topcoat::{
    Result,
    context::Cx,
    view::{View, attributes, component, view},
};
use tuita_core::text::parse_post;
use tuita_server::dto::{ListDto, PostDto, PostEditsDto, UserDto};

use crate::{
    fmt::{full_time, handle_of},
    icons::Icon,
    l10n::{strip_isolates, t},
    post::{
        post_card::post_card,
        post_thread::ListKind,
        rich_text::{emote_text, rich_text, segments_of},
    },
    profile::user_row::user_row,
    ui::{
        button::{ButtonVariant, button},
        char_counter::char_counter,
        checkbox::checkbox,
        dialog::dialog,
        empty_state::empty_state,
        icon::icon,
        radio::radio,
        tag::tag,
        text_area::text_area,
        text_field::text_field,
    },
};

/// The id of every post dialog; `post.js` keeps at most one in the DOM.
pub(crate) const DIALOG_ID: &str = "post-dialog";

/// Most characters in a post.
const MAX_POST_LEN: i64 = 500;

/// The reasons a report can give, with their message keys.
const REASONS: [&str; 7] = [
    "spam",
    "harassment",
    "hate",
    "violence",
    "misinformation",
    "impersonation",
    "other",
];

fn reason_texts(cx: &Cx, reason: &str) -> (String, String) {
    macro_rules! pair {
        ($label:ident, $body:ident) => {
            (
                t!($label).resolve(cx).into_owned(),
                t!($body).resolve(cx).into_owned(),
            )
        };
    }
    match reason {
        "spam" => pair!(report_spam, report_spam_body),
        "harassment" => pair!(report_harassment, report_harassment_body),
        "hate" => pair!(report_hate, report_hate_body),
        "violence" => pair!(report_violence, report_violence_body),
        "misinformation" => pair!(report_misinformation, report_misinformation_body),
        "impersonation" => pair!(report_impersonation, report_impersonation_body),
        _ => pair!(report_other, report_other_body),
    }
}

/// Edit the text and the content warning of a post. The form is `data-post-form="edit"`;
/// `post.js` enables Save while the text is changed, not empty (unless the post has media or a
/// poll) and within 500 characters, sends `PATCH /api/v1/posts/<id>` and refreshes the card.
#[component]
pub(crate) async fn edit_dialog(cx: &Cx, post: &PostDto) -> Result<impl View> {
    let length = i64::try_from(parse_post(post.text.trim()).len()).unwrap_or(i64::MAX);
    let keeps = !post.media.is_empty() || post.poll.is_some();
    let title = t!(edit_title).resolve(cx).into_owned();
    let cancel = t!(cancel).resolve(cx).into_owned();
    let cw_label = t!(composer_cw_label).resolve(cx).into_owned();
    let text_label = t!(composer_text_label).resolve(cx).into_owned();
    let id = post.id.to_string();
    Ok(view! {
        dialog(
            id: DIALOG_ID,
            title: &title,
            actions: Some(
                view! {
                    button(
                        variant: ButtonVariant::Ghost,
                        attrs: attributes! { command="close" commandfor=(DIALOG_ID) },
                        (&cancel)
                    )
                    button(
                        submit: true,
                        disabled: true,
                        attrs: attributes! { form="edit-post" data-act="edit-save" },
                        (t!(edit_save))
                    )
                }.into(

                ),
            ),
            <form
                id="edit-post"
                data-c="edit-post-form"
                novalidate=""
                data-post-form="edit"
                data-post-id=(&id)
                data-keeps=(keeps.then_some(""))
                data-text=(&post.text)
                data-cw=(&post.cw)
            >
                <p class="note">(t!(edit_note))</p>
                text_field(
                    label: &cw_label,
                    value: &post.cw,
                    attrs: attributes! { name="cw" maxlength="100" }
                )
                <div class="text">
                    text_area(
                        label: &text_label,
                        value: &post.text,
                        rows: 5,
                        attrs: attributes! { name="text" }
                    )
                    <div class="counter">
                        char_counter(value: length, max: MAX_POST_LEN)
                    </div>
                </div>
                <p class="failure" role="alert" hidden=""></p>
            </form>
        )
    })
}

/// The versions of a post, current first.
#[component]
pub(crate) async fn history_dialog(
    cx: &Cx,
    post: &PostDto,
    edits: &PostEditsDto,
) -> Result<impl View> {
    let title = t!(edit_history_title).resolve(cx).into_owned();
    let last = edits.edits.len().saturating_sub(1);
    let mut rows = vec![(
        &edits.current,
        t!(edit_history_current).resolve(cx).into_owned(),
        true,
    )];
    for (index, version) in edits.edits.iter().enumerate() {
        let label = if index == last {
            t!(edit_history_original).resolve(cx).into_owned()
        } else {
            t!(edit_history_earlier).resolve(cx).into_owned()
        };
        rows.push((version, label, false));
    }
    let rows: Vec<_> = rows
        .into_iter()
        .map(|(version, label, current)| {
            let at = version
                .at
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default();
            (
                version,
                label,
                current,
                full_time(cx, &at),
                at,
                segments_of(&version.text),
            )
        })
        .collect();
    Ok(view! {
        dialog(
            id: DIALOG_ID,
            title: &title,
            <ol data-c="edit-history">
                for (version, label, current, shown, at, segments) in rows {
                    <li>
                        <div class="head">
                            tag(pill: current, (label))
                            <time datetime=(at)>(shown)</time>
                        </div>
                        if !version.cw.is_empty() {
                            <p class="cw">
                                icon(name: Icon::WarningDiamond, size: 16)
                                <span translate="no">
                                    emote_text(text: &version.cw, emojis: &post.emojis)
                                </span>
                            </p>
                        }
                        <div class="text">
                            rich_text(
                                segments: &segments,
                                domain: post.author.domain.as_deref(),
                                emojis: &post.emojis
                            )
                        </div>
                    </li>
                }
            </ol>
        )
    })
}

/// Report a post (`post_id`) or an account (`handle`). `post.js` enables Send once a reason is
/// picked and sends `POST /api/v1/reports`.
#[component]
pub(crate) async fn report_dialog(
    cx: &Cx,
    #[default] post: Option<&PostDto>,
    #[default] handle: Option<&str>,
) -> Result<impl View> {
    let who = match post {
        Some(post) => handle_of(&post.author.handle, post.author.domain.as_deref()),
        None => format!("@{}", handle.unwrap_or_default()),
    };
    let title = if post.is_some() {
        t!(report_post_title).resolve(cx).into_owned()
    } else {
        strip_isolates(&t!(report_user_title, handle: who.as_str()).resolve(cx))
    };
    let intro = strip_isolates(&t!(report_intro, handle: who.as_str()).resolve(cx));
    let cancel = t!(cancel).resolve(cx).into_owned();
    let note_label = t!(report_note).resolve(cx).into_owned();
    let reasons: Vec<_> = REASONS
        .iter()
        .map(|reason| {
            let (label, body) = reason_texts(cx, reason);
            (*reason, label, body)
        })
        .collect();
    let post_id = post.map(|post| post.id.to_string());
    Ok(view! {
        dialog(
            id: DIALOG_ID,
            title: &title,
            actions: Some(
                view! {
                    button(
                        variant: ButtonVariant::Ghost,
                        attrs: attributes! { command="close" commandfor=(DIALOG_ID) },
                        (&cancel)
                    )
                    button(
                        submit: true,
                        disabled: true,
                        attrs: attributes! { form="report" data-act="report-send" },
                        (t!(report_send))
                    )
                }.into(

                ),
            ),
            <form
                id="report"
                novalidate=""
                data-post-form="report"
                data-post-id=(post_id)
                data-handle=(handle)
                data-c="report-form"
            >
                <p class="note">(&intro)</p>
                <fieldset>
                    <legend class="sr-only">(t!(report_reason))</legend>
                    for (reason, label, body) in reasons {
                        <div class="reason">
                            radio(group: "reason", value: reason, label: &label)
                            <p>(body)</p>
                        </div>
                    }
                </fieldset>
                text_area(
                    label: &note_label,
                    rows: 3,
                    attrs: attributes! { name="note" maxlength="1000" }
                )
                <p class="failure" role="alert" hidden=""></p>
            </form>
        )
    })
}

/// The lists of the viewer with a box for each: ticking one adds `acct` to it, clearing it removes
/// them (`post.js`, `data-act="list-member"`: `POST` or `DELETE
/// /api/v1/lists/<id>/members/<acct>`).
#[component]
pub(crate) async fn list_picker_dialog(
    cx: &Cx,
    acct: &str,
    lists: &[ListDto],
) -> Result<impl View> {
    let handle = format!("@{acct}");
    let title = strip_isolates(&t!(lists_picker_title, handle: handle.as_str()).resolve(cx));
    Ok(view! {
        dialog(
            id: DIALOG_ID,
            title: &title,
            actions: Some(
                view! {
                    button(
                        variant: ButtonVariant::Outline,
                        attrs: attributes! { data-act="list-new" data-acct=(acct) },
                        (t!(lists_new))
                    )
                    button(
                        attrs: attributes! { command="close" commandfor=(DIALOG_ID) },
                        (t!(close))
                    )
                }.into(

                ),
            ),
            <div data-c="list-picker">
                if lists.is_empty() {
                    <p class="dim">(t!(lists_picker_none))</p>
                } else {
                    <ul>
                        for list in lists {
                            <li>
                                checkbox(
                                    label: &list.name,
                                    checked: list.has_member == Some(true),
                                    attrs: attributes! {
                                        data-act="list-member"
                                        data-list-id=(list.id.to_string())
                                        data-acct=(acct)
                                    }
                                )
                            </li>
                        }
                    </ul>
                }
            </div>
        )
    })
}

/// What a post list holds.
pub(crate) enum ListItems<'a> {
    People(&'a [UserDto]),
    Posts(&'a [PostDto]),
}

/// The entries of one page of a post list and the link to the next page (`more`, a path).
/// `infinite.js` replaces that link with the next entries.
#[component]
pub(crate) async fn post_list_items<'a>(
    cx: &Cx,
    items: ListItems<'a>,
    more: Option<&str>,
) -> Result<impl View> {
    let (people, posts): (&[UserDto], &[PostDto]) = match items {
        ListItems::People(people) => (people, &[]),
        ListItems::Posts(posts) => (&[], posts),
    };
    Ok(view! {
        for user in people {
            <li class="entry">user_row(user: user)</li>
        }
        #[key(post.entry_id.as_bytes())]
        for post in posts {
            <li class="entry">post_card(post: post)</li>
        }
        if let Some(path) = more {
            <li class="more"><a data-more="" href=(path)>(t!(loading))</a></li>
        }
    })
}

/// The first page of "liked by", "reposted by" or "quotes" of a post.
#[component]
pub(crate) async fn post_list_dialog<'a>(
    cx: &Cx,
    post_id: &str,
    kind: ListKind,
    items: ListItems<'a>,
    more: Option<&str>,
) -> Result<impl View> {
    let title = match kind {
        ListKind::Likes => t!(thread_list_likes).resolve(cx).into_owned(),
        ListKind::Reposts => t!(thread_list_reposts).resolve(cx).into_owned(),
        ListKind::Quotes => t!(thread_list_quotes).resolve(cx).into_owned(),
    };
    let empty = match &items {
        ListItems::People(people) => people.is_empty(),
        ListItems::Posts(posts) => posts.is_empty(),
    };
    let context = format!("{}:{post_id}", kind.name());
    let empty_title = t!(thread_list_empty).resolve(cx).into_owned();
    Ok(view! {
        dialog(
            id: DIALOG_ID,
            title: &title,
            <div data-c="post-list">
                if empty {
                    empty_state(context: &context, title: &empty_title)
                } else {
                    <ul aria-label=(&title)>
                        post_list_items(items: items, more: more)
                    </ul>
                }
            </div>
        )
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::{
        post::testing::{sample_post, sample_user},
        testing::render,
    };

    #[test]
    fn the_edit_form_carries_the_text_and_starts_unsendable() {
        let mut post = sample_post();
        post.cw = "spoilers".to_owned();
        let html = render("en", |cx| view! { edit_dialog(post: &post) }.boxed());
        assert!(html.contains(r#"<dialog id="post-dialog""#), "{html}");
        assert!(
            html.contains(r#"<h2 id="post-dialog-title">Edit post</h2>"#),
            "{html}"
        );
        assert!(html.contains(r#"data-post-form="edit""#), "{html}");
        assert!(html.contains(">hello world</textarea>"), "{html}");
        assert!(html.contains(r#"maxlength="100""#), "{html}");
        assert!(html.contains("data-act=\"edit-save\""), "{html}");
        assert!(html.contains("disabled"), "{html}");
        assert!(html.contains(r#"role="alert""#), "{html}");
    }

    #[test]
    fn the_report_dialog_offers_the_seven_reasons() {
        let post = sample_post();
        let html = render("en", |cx| {
            view! { report_dialog(post: Some(&post)) }.boxed()
        });
        assert_eq!(html.matches(r#"type="radio""#).count(), 7, "{html}");
        assert!(html.contains("Report post"), "{html}");
        assert!(html.contains(r#"data-post-form="report""#), "{html}");
        let user = render("en", |cx| {
            view! { report_dialog(handle: Some("bob")) }.boxed()
        });
        assert!(user.contains("Report @bob"), "{user}");
    }

    #[test]
    fn the_list_picker_ticks_the_lists_that_hold_the_account() {
        let lists = vec![ListDto {
            id: uuid::Uuid::from_u128(1),
            name: "Friends".to_owned(),
            description: String::new(),
            private: false,
            created_at: time::macros::datetime!(2026-11-09 10:00 UTC),
            owner: sample_user("ada"),
            is_owner: true,
            members: 1,
            has_member: Some(true),
        }];
        let html = render("en", |cx| {
            view! { list_picker_dialog(acct: "bob", lists: &lists) }.boxed()
        });
        assert!(html.contains("Friends"), "{html}");
        assert!(html.contains("checked"), "{html}");
        assert!(html.contains(r#"data-act="list-member""#), "{html}");
        assert!(html.contains(r#"data-acct="bob""#), "{html}");
    }

    #[test]
    fn an_empty_post_list_says_nobody_yet() {
        let html = render("en", |cx| {
            view! {
                post_list_dialog(
                    post_id: "p1",
                    kind: ListKind::Likes,
                    items: ListItems::People(&[]),
                    more: None
                )
            }
            .boxed()
        });
        assert!(html.contains("Liked by"), "{html}");
        assert!(html.contains("Nobody yet"), "{html}");
    }
}
