//! `/scheduled`: posts waiting for their time.
//!
//! The page is rendered once; `scheduled.js` (`[data-island="scheduled"]`) does the actions
//! (change the time, post now, cancel) through the JSON API and then asks for the list again: a
//! `x-tuita-fragment` request answers only the `[data-scheduled-body]` element, which the island
//! swaps in. The page knows no time zone, so the island writes the zone and the local time
//! (`time[data-local]`).

use topcoat::{
    Result,
    context::Cx,
    view::{View, attributes, component, view},
};
use tuita_server::dto::{ScheduledPostDto, ScheduledStatus};

use crate::{
    api,
    fmt::{acct_of, full_time},
    fragment,
    l10n::t,
    paths,
    post::media_grid::media_grid,
    session,
    shell::page_bar::page_bar,
    ui::{
        button::{ButtonSize, ButtonVariant, button},
        dialog::dialog,
        empty_state::empty_state,
        tag::tag,
    },
};

const RESCHEDULE_ID: &str = "reschedule";
const CANCEL_ID: &str = "confirm-scheduled-cancel";

/// The RFC 3339 form of an instant.
fn iso(at: time::OffsetDateTime) -> String {
    at.format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default()
}

/// One scheduled post: status, time, text, media, poll summary, error and its actions.
///
/// `acct` is the viewer, for the link to a published post.
#[component]
pub(crate) async fn scheduled_card(
    cx: &Cx,
    item: &ScheduledPostDto,
    acct: &str,
) -> Result<impl View> {
    let status = match item.status {
        ScheduledStatus::Pending => t!(scheduled_status_pending),
        ScheduledStatus::Published => t!(scheduled_status_published),
        ScheduledStatus::Failed => t!(scheduled_status_failed),
    }
    .resolve(cx)
    .into_owned();
    let at = iso(item.publish_at);
    let when = full_time(cx, &at);
    let id = item.id.to_string();
    let poll = item.poll.as_ref().map(|poll| {
        let count = i64::try_from(poll.options.len()).unwrap_or(0);
        t!(scheduled_poll, count: count, hours: i64::from(poll.duration_hours))
            .resolve(cx)
            .into_owned()
    });
    let warning = (!item.cw.is_empty()).then(|| {
        t!(scheduled_cw, text: item.cw.as_str())
            .resolve(cx)
            .into_owned()
    });
    let view_link = item
        .post_id
        .map(|post| paths::post(acct, &post.to_string()));
    let pending = item.status == ScheduledStatus::Pending;
    let failed = item.status == ScheduledStatus::Failed;
    Ok(view! {
        <article
            class="card"
            data-c="scheduled-card"
            data-id=(&id)
            data-status=(item.status.as_str())
            data-publish-at=(&at)
        >
            <header class="head">
                tag((&status))
                <time datetime=(&at) data-local="">(when)</time>
            </header>
            if let Some(warning) = &warning {
                <p class="cw">(warning)</p>
            }
            if !item.text.is_empty() {
                <p class="text">(&item.text)</p>
            }
            if !item.media.is_empty() {
                media_grid(media: &item.media, sensitive: item.sensitive)
            }
            if let Some(poll) = &poll {
                <p class="dim">(poll)</p>
            }
            if let Some(error) = &item.error {
                <p class="error" role="status">(error)</p>
            }
            <div class="actions">
                if pending {
                    button(
                        variant: ButtonVariant::Outline,
                        size: ButtonSize::Sm,
                        attrs: attributes! { data-act="sched-resched" },
                        (t!(scheduled_change_time))
                    )
                    button(
                        size: ButtonSize::Sm,
                        attrs: attributes! { data-act="sched-publish" },
                        (t!(scheduled_post_now))
                    )
                    button(
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Sm,
                        attrs: attributes! { data-act="sched-cancel" },
                        (t!(cancel))
                    )
                } else if failed {
                    button(
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Sm,
                        attrs: attributes! { data-act="sched-cancel" },
                        (t!(scheduled_remove))
                    )
                } else if let Some(href) = &view_link {
                    <a class="link" href=(href)>(t!(view))</a>
                }
            </div>
        </article>
    })
}

/// The list of scheduled posts, or the empty state. This element is what a refresh swaps.
#[component]
async fn scheduled_body(cx: &Cx, items: &[ScheduledPostDto], acct: &str) -> Result<impl View> {
    let label = t!(scheduled_label).resolve(cx).into_owned();
    let title = t!(scheduled_empty_title).resolve(cx).into_owned();
    let body = t!(scheduled_empty_body).resolve(cx).into_owned();
    Ok(view! {
        <div data-scheduled-body="">
            if items.is_empty() {
                empty_state(context: "scheduled", title: &title, body: body.as_str())
            } else {
                <div class="list" role="list" aria-label=(&label) data-c="post-list">
                    #[key(item.id.as_bytes())]
                    for item in items {
                        <div role="listitem">
                            scheduled_card(item: item, acct: acct)
                        </div>
                    }
                </div>
            }
        </div>
    })
}

/// The dialog that changes the time of a pending post. `scheduled.js` fills and saves it.
#[component]
pub(crate) async fn reschedule_dialog(cx: &Cx) -> Result<impl View> {
    let change = t!(scheduled_change_time).resolve(cx).into_owned();
    Ok(view! {
        dialog(
            id: RESCHEDULE_ID,
            title: &change,
            actions: Some(
                view! {
                    button(
                        variant: ButtonVariant::Ghost,
                        attrs: attributes! { command="close" commandfor=(RESCHEDULE_ID) },
                        (t!(cancel))
                    )
                    button(
                        submit: true,
                        attrs: attributes! { form="reschedule-form" },
                        (t!(scheduled_save))
                    )
                }.into(

                ),
            ),
            <form id="reschedule-form" data-c="reschedule-form" novalidate="">
                <label for="reschedule-when">(t!(scheduled_when))</label>
                <input
                    id="reschedule-when"
                    name="when"
                    type="datetime-local"
                    required=""
                >
                <p class="dim" data-zone-note=""></p>
                <p class="error" role="alert" hidden=""></p>
            </form>
        )
    })
}

/// The page.
#[component]
pub(crate) async fn scheduled(cx: &Cx) -> Result<impl View> {
    let viewer = session::member(cx).await?;
    let acct = acct_of(&viewer.me.user.handle, viewer.me.user.domain.as_deref());
    let items = api::get::<Vec<ScheduledPostDto>>(cx, "/scheduled").await?;
    let title = t!(scheduled_title).resolve(cx).into_owned();
    let ask = t!(scheduled_cancel_title).resolve(cx).into_owned();
    let ask_body = t!(scheduled_cancel_body).resolve(cx).into_owned();
    let cancel_label = t!(scheduled_cancel_confirm).resolve(cx).into_owned();
    let remove_label = t!(scheduled_remove).resolve(cx).into_owned();
    Ok(view! {
        if fragment::requested(cx) {
            scheduled_body(items: &items, acct: &acct)
        } else {
            <div data-c="scheduled" data-island="scheduled">
                page_bar(back: true, title: &title, sub: "")
                scheduled_body(items: &items, acct: &acct)
                reschedule_dialog()
                dialog(
                    id: CANCEL_ID,
                    title: &ask,
                    actions: Some(
                        view! {
                            button(
                                variant: ButtonVariant::Ghost,
                                attrs: attributes! { command="close" commandfor=(CANCEL_ID) },
                                (t!(cancel))
                            )
                            button(
                                variant: ButtonVariant::Danger,
                                attrs: attributes! {
                                    command="close"
                                    commandfor=(CANCEL_ID)
                                    data-act="sched-cancel-yes"
                                    data-pending=(&cancel_label)
                                    data-failed=(&remove_label)
                                },
                                (&cancel_label)
                            )
                        }.into(

                        ),
                    ),
                    <p>(&ask_body)</p>
                )
            </div>
        }
    })
}

#[cfg(test)]
mod test {
    use time::OffsetDateTime;
    use topcoat::view::ViewExt;
    use tuita_server::dto::{ReplyPolicy, ScheduledPostDto, ScheduledStatus};
    use uuid::Uuid;

    use super::*;
    use crate::testing::render;

    fn item(status: ScheduledStatus) -> ScheduledPostDto {
        ScheduledPostDto {
            id: Uuid::from_u128(7),
            publish_at: OffsetDateTime::UNIX_EPOCH,
            status,
            post_id: (status == ScheduledStatus::Published).then(|| Uuid::from_u128(9)),
            error: (status == ScheduledStatus::Failed).then(|| "No room.".to_owned()),
            created_at: OffsetDateTime::UNIX_EPOCH,
            text: "Out later".to_owned(),
            cw: String::new(),
            sensitive: false,
            reply_policy: ReplyPolicy::Everyone,
            media: Vec::new(),
            poll: None,
            quote_of: None,
        }
    }

    fn card(status: ScheduledStatus) -> String {
        render("en", |cx| {
            let item = item(status);
            view! { scheduled_card(item: &item, acct: "ada") }.boxed()
        })
    }

    #[test]
    fn a_pending_post_can_be_moved_posted_now_or_cancelled() {
        let html = card(ScheduledStatus::Pending);
        assert!(html.contains("Out later"), "{html}");
        assert!(html.contains(">Scheduled<"), "{html}");
        for act in ["sched-resched", "sched-publish", "sched-cancel"] {
            assert!(html.contains(&format!(r#"data-act="{act}""#)), "{html}");
        }
        assert!(
            html.contains(r#"datetime="1970-01-01T00:00:00Z""#),
            "{html}"
        );
        assert!(html.contains("data-local"), "{html}");
    }

    #[test]
    fn a_failed_post_shows_why_and_can_only_be_removed() {
        let html = card(ScheduledStatus::Failed);
        assert!(html.contains(r#"role="status">No room."#), "{html}");
        assert!(html.contains(r#"data-act="sched-cancel""#), "{html}");
        assert!(!html.contains("sched-publish"), "{html}");
        assert!(html.contains(">Remove<"), "{html}");
    }

    #[test]
    fn a_published_post_links_to_the_post() {
        let html = card(ScheduledStatus::Published);
        assert!(html.contains(">Posted<"), "{html}");
        assert!(html.contains(r#"href="/@ada/"#), "{html}");
        assert!(!html.contains("data-act"), "{html}");
    }
}
