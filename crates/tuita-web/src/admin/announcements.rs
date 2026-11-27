//! The announcements tab: publish a banner and end the running ones.
//!
//! The banner above every page is part of the shell, so publishing or ending reloads the page
//! (`tuita-reload`, see `admin.js`) once the API has answered, and the toast follows the reload.

use time::OffsetDateTime;
use topcoat::{
    Result,
    context::Cx,
    runtime::{Event, procedure, signal},
    view::{View, attributes, component, view},
};
use tuita_server::dto::{AdminAnnouncementDto, AnnouncementTone, CreateAnnouncementRequest, OkDto};

use crate::{
    admin::{
        common::{message, stamp, tr},
        rules::{AnnouncementState, announcement_state},
        select_field::select_field,
        setting_card::setting_card,
    },
    api,
    fmt::{full_time, rel_time},
    icons::Icon,
    l10n::t,
    ui::{
        button::{ButtonSize, ButtonVariant, button},
        icon::icon,
        tag::tag,
        text_area::text_area,
        text_field::text_field,
    },
};

/// `POST /admin/announcements`. Answers the toast text.
#[procedure]
pub(crate) async fn publish_announcement(
    cx: &Cx,
    text: String,
    link: String,
    tone: String,
    days: String,
) -> Result<std::result::Result<String, String>> {
    let body = CreateAnnouncementRequest {
        text: text.trim().to_owned(),
        link: Some(link.trim().to_owned()),
        tone: AnnouncementTone::parse(&tone),
        days: days.trim().parse().ok(),
    };
    Ok(
        api::post::<_, AdminAnnouncementDto>(cx, "/admin/announcements", &body)
            .await
            .map(|_| tr!(cx, admin_ann_published))
            .map_err(message),
    )
}

/// `DELETE /admin/announcements/{id}`. Answers an empty toast: the page reloads without one.
#[procedure]
pub(crate) async fn end_announcement(
    cx: &Cx,
    id: String,
) -> Result<std::result::Result<String, String>> {
    let path = format!("/admin/announcements/{}", api::enc(&id));
    Ok(api::delete::<OkDto>(cx, &path)
        .await
        .map(|_| String::new())
        .map_err(message))
}

const fn tone_icon(tone: AnnouncementTone) -> Icon {
    match tone {
        AnnouncementTone::Info => Icon::InfoBox,
        AnnouncementTone::Success => Icon::Check,
        AnnouncementTone::Warning => Icon::WarningDiamond,
    }
}

/// One announcement of the list.
struct Row {
    id: String,
    text: String,
    icon: Icon,
    at: String,
    status: String,
    live: bool,
}

#[component]
pub(crate) async fn announcements(cx: &Cx) -> Result<impl View> {
    let items = api::get::<Vec<AdminAnnouncementDto>>(cx, "/admin/announcements").await?;
    let now = OffsetDateTime::now_utc();
    let rows: Vec<Row> = items
        .iter()
        .map(|item| {
            let state = announcement_state(item, now);
            let status = match state {
                AnnouncementState::Ended => tr!(cx, admin_ann_ended),
                AnnouncementState::Expired => tr!(cx, admin_ann_expired),
                AnnouncementState::Live => match item.expires_at {
                    Some(at) => tr!(cx, admin_ann_until, date: full_time(cx, &stamp(at))),
                    None => tr!(cx, admin_ann_until_ended),
                },
            };
            Row {
                id: item.id.to_string(),
                text: item.text.clone(),
                icon: tone_icon(item.tone),
                at: stamp(item.created_at),
                status,
                live: state == AnnouncementState::Live,
            }
        })
        .collect();
    let text = signal(cx, String::new);
    let link = signal(cx, String::new);
    let tone = signal(cx, || "info".to_owned());
    let days = signal(cx, || "7".to_owned());
    let error = signal(cx, String::new);
    let busy = signal(cx, || false);
    let tones: Vec<(String, String)> = vec![
        ("info".to_owned(), tr!(cx, admin_ann_tone_info)),
        ("success".to_owned(), tr!(cx, admin_ann_tone_success)),
        ("warning".to_owned(), tr!(cx, admin_ann_tone_warning)),
    ];
    let durations: Vec<(String, String)> = vec![
        ("1".to_owned(), tr!(cx, admin_ann_days, count: 1)),
        ("3".to_owned(), tr!(cx, admin_ann_days, count: 3)),
        ("7".to_owned(), tr!(cx, admin_ann_week)),
        ("30".to_owned(), tr!(cx, admin_ann_days, count: 30)),
        ("0".to_owned(), tr!(cx, admin_ann_until_ended)),
    ];
    let title = tr!(cx, admin_ann_title);
    let desc = tr!(cx, admin_ann_desc);
    let text_label = tr!(cx, admin_ann_text);
    let link_label = tr!(cx, admin_ann_link);
    let link_hint = tr!(cx, admin_ann_link_hint);
    let tone_label = tr!(cx, admin_ann_tone);
    let duration_label = tr!(cx, admin_ann_duration);
    let list_title = tr!(cx, admin_ann_list);
    let inactive = tr!(cx, admin_ann_inactive);
    let end = tr!(cx, admin_ann_end);
    Ok(view! {
        <div class="tab" data-c="admin-announcements">
            setting_card(
                title: &title,
                description: desc.as_str(),
                <form
                    class="form"
                    novalidate=""
                    @submit=$(async |e: Event| {
                        e.prevent_default();
                        error.set("".to_owned());
                        busy.set(true);
                        let done = publish_announcement(
                            text.get(),
                            link.get(),
                            tone.get(),
                            days.get(),
                        ).await;
                        if done.is_ok() {
                            let msg = done.unwrap();
                            raw!(
                                "dispatchEvent(new CustomEvent('tuita-reload',{detail:''+${msg}}))",
                                msg.is_empty(),
                            )
                        } else {
                            busy.set(false);
                            error.set(done.unwrap_err());
                            false
                        }
                    })
                >
                    text_area(
                        label: &text_label,
                        rows: 2,
                        attrs: attributes! {
                            name="text"
                            maxlength="280"
                            required=""
                            :value=$(text.get())
                            @input=$(|e: Event| text.set(e.target.value))
                        }
                    )
                    text_field(
                        label: &link_label,
                        hint: link_hint.as_str(),
                        attrs: attributes! {
                            name="link"
                            maxlength="200"
                            autocapitalize="off"
                            spellcheck="false"
                            :value=$(link.get())
                            @input=$(|e: Event| link.set(e.target.value))
                        }
                    )
                    <div class="pair">
                        select_field(
                            label: &tone_label,
                            options: &tones,
                            value: "info",
                            attrs: attributes! {
                                name="tone"
                                @change=$(|e: Event| tone.set(e.target.value))
                            }
                        )
                        select_field(
                            label: &duration_label,
                            options: &durations,
                            value: "7",
                            attrs: attributes! {
                                name="days"
                                @change=$(|e: Event| days.set(e.target.value))
                            }
                        )
                    </div>
                    <p class="error" role="alert" :hidden=$(error.get().is_empty())>
                        $(error.get())
                    </p>
                    <div class="end">
                        button(
                            submit: true,
                            attrs: attributes! {
                                :disabled=$(if busy.get() {
                                    true
                                } else {
                                    text.get().trim().is_empty()
                                })
                                :aria-busy=$(if busy.get() { "true" } else { "false" })
                            },
                            (t!(admin_ann_publish))
                        )
                    </div>
                </form>
            )
            setting_card(
                title: &list_title,
                if rows.is_empty() {
                    <p class="dim">(t!(admin_ann_none))</p>
                } else {
                    <ul class="items">
                        #[key(row.id.as_bytes())]
                        for row in rows {
                            <li data-announcement=(&row.id)>
                                <span class="icon">icon(name: row.icon, size: 20)</span>
                                <div class="text">
                                    <p>(&row.text)</p>
                                    <p class="dim">
                                        rel_time(ts: &row.at)
                                        " · "
                                        (&row.status)
                                        if !row.live {
                                            " "
                                            tag((&inactive))
                                        }
                                    </p>
                                </div>
                                if row.live {
                                    let id = row.id.clone();
                                    button(
                                        variant: ButtonVariant::Outline,
                                        size: ButtonSize::Sm,
                                        attrs: attributes! {
                                            :disabled=$(busy.get())
                                            @click=$(async |_e: Event| {
                                                busy.set(true);
                                                let done = end_announcement(id).await;
                                                if done.is_ok() {
                                                    raw!("dispatchEvent(new Event('tuita-reload'))")
                                                } else {
                                                    busy.set(false);
                                                    let msg = done.unwrap_err();
                                                    raw!(
                                                        "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:''+${msg}}))",
                                                        msg.is_empty(),
                                                    )
                                                }
                                            })
                                        },
                                        (&end)
                                    )
                                }
                            </li>
                        }
                    </ul>
                }
            )
        </div>
    })
}
