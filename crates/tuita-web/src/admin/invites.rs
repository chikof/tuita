//! The invites tab.

use time::OffsetDateTime;
use topcoat::{
    Result,
    context::Cx,
    runtime::{Event, Signal, procedure, shard, signal},
    view::{View, attributes, component, view},
};
use tuita_server::dto::{CreateInviteRequest, InviteDto, OkDto};

use crate::{
    admin::{
        common::{message, stamp, tr},
        rules::{InviteState, invite_state},
        select_field::select_field,
        setting_card::setting_card,
    },
    api,
    fmt::full_time,
    icons::Icon,
    l10n::t,
    ui::{
        button::{ButtonSize, ButtonVariant, button},
        confirm_dialog::confirm_dialog,
        icon_button::icon_button,
        tag::tag,
        text_field::text_field,
    },
};

/// `POST /admin/invites`. Answers the new code.
#[procedure]
pub(crate) async fn create_invite(
    cx: &Cx,
    note: String,
    max_uses: String,
    days: String,
) -> Result<std::result::Result<String, String>> {
    let body = CreateInviteRequest {
        note: Some(note.trim().to_owned()),
        max_uses: max_uses.trim().parse().ok(),
        days: days.trim().parse().ok(),
    };
    Ok(api::post::<_, InviteDto>(cx, "/admin/invites", &body)
        .await
        .map(|invite| invite.code)
        .map_err(message))
}

/// `DELETE /admin/invites/{code}`.
#[procedure]
pub(crate) async fn revoke_invite(
    cx: &Cx,
    code: String,
) -> Result<std::result::Result<bool, String>> {
    let path = format!("/admin/invites/{}", api::enc(&code));
    Ok(api::delete::<OkDto>(cx, &path)
        .await
        .map(|_| true)
        .map_err(message))
}

/// The line under a code: uses, expiry and who used it.
fn details(cx: &Cx, invite: &InviteDto) -> String {
    let mut parts = vec![tr!(cx, admin_inv_used, uses: invite.uses, max: invite.max_uses)];
    if let Some(at) = invite.expires_at {
        let date = full_time(cx, &stamp(at));
        parts.push(tr!(cx, admin_inv_expires, date: date));
    }
    if !invite.used_by.is_empty() {
        let who: Vec<String> = invite
            .used_by
            .iter()
            .map(|handle| format!("@{handle}"))
            .collect();
        parts.push(who.join(", "));
    }
    parts.join(" · ")
}

fn state_label(cx: &Cx, state: InviteState) -> String {
    match state {
        InviteState::Active => tr!(cx, admin_inv_state_active),
        InviteState::Revoked => tr!(cx, admin_inv_state_revoked),
        InviteState::Used => tr!(cx, admin_inv_state_used),
        InviteState::Expired => tr!(cx, admin_inv_state_expired),
    }
}

/// One code of the list with the words its controls need.
struct Row {
    code: String,
    note: String,
    state: InviteState,
    state_text: String,
    line: String,
    copy_label: String,
}

/// The codes, from the API again whenever `revision` changes.
#[shard]
async fn invite_list(cx: &Cx, revision: Signal<u32>) -> Result<impl View> {
    let _ = revision.get();
    let items = api::get::<Vec<InviteDto>>(cx, "/admin/invites").await?;
    let now = OffsetDateTime::now_utc();
    let rows: Vec<Row> = items
        .iter()
        .map(|invite| {
            let state = invite_state(invite, now);
            Row {
                code: invite.code.clone(),
                note: invite.note.clone(),
                state,
                state_text: state_label(cx, state),
                line: details(cx, invite),
                copy_label: tr!(cx, admin_inv_copy, code: invite.code.as_str()),
            }
        })
        .collect();
    let revoke_title = tr!(cx, admin_inv_revoke_title);
    let revoke_body = tr!(cx, admin_inv_revoke_body);
    let revoke = tr!(cx, admin_inv_revoke);
    Ok(view! {
        if rows.is_empty() {
            <p class="dim">(t!(admin_inv_none))</p>
        } else {
            <ul class="items">
                #[key(row.code.as_bytes())]
                for row in rows {
                    let code = row.code;
                    let to_copy = code.clone();
                    let to_revoke = code.clone();
                    let dialog = format!("revoke-{code}");
                    <li
                        data-invite=(&code)
                        class=((row.state != InviteState::Active).then_some("dead"))
                    >
                        <div class="text">
                            <p class="head">
                                <code>(&code)</code>
                                tag(pill: true, compact: true, (&row.state_text))
                            </p>
                            if !row.note.is_empty() {
                                <p>(&row.note)</p>
                            }
                            <p class="dim">(&row.line)</p>
                        </div>
                        if row.state == InviteState::Active {
                            icon_button(
                                glyph: Icon::Copy,
                                label: &row.copy_label,
                                attrs: attributes! {
                                    @click=$(|_e: Event| raw!(
                                            "document.dispatchEvent(new CustomEvent('tuita-copy',{detail:{text:''+${to_copy},ok:'admin_inv_copied',fail:'admin_inv_copy_failed',v:'code'}}))",
                                            to_copy.is_empty(),
                                        ))
                                }
                            )
                            button(
                                variant: ButtonVariant::Outline,
                                size: ButtonSize::Sm,
                                attrs: attributes! { command="show-modal" commandfor=(&dialog) },
                                (&revoke)
                            )
                            confirm_dialog(
                                id: &dialog,
                                title: &revoke_title,
                                body: &revoke_body,
                                confirm_label: &revoke,
                                danger: true,
                                action: $(async |_e: Event| {
                                    let done = revoke_invite(to_revoke).await;
                                    if done.is_ok() {
                                        revision.increment();
                                        false
                                    } else {
                                        let msg = done.unwrap_err();
                                        raw!(
                                            "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:''+${msg}}))",
                                            msg.is_empty(),
                                        )
                                    }
                                })
                            )
                        }
                    </li>
                }
            </ul>
        }
    })
}

#[component]
pub(crate) async fn invites(cx: &Cx) -> Result<impl View> {
    let note = signal(cx, String::new);
    let max_uses = signal(cx, || "1".to_owned());
    let days = signal(cx, || "30".to_owned());
    let error = signal(cx, String::new);
    let busy = signal(cx, || false);
    let revision = signal(cx, || 0u32);
    let uses: Vec<(String, String)> = [1, 5, 25, 100]
        .into_iter()
        .map(|n: i32| (n.to_string(), tr!(cx, admin_inv_use_count, count: n)))
        .collect();
    let expiry = vec![
        ("7".to_owned(), tr!(cx, admin_inv_week)),
        ("30".to_owned(), tr!(cx, admin_inv_days, count: 30)),
        ("0".to_owned(), tr!(cx, admin_inv_never)),
    ];
    let title = tr!(cx, admin_inv_title);
    let desc = tr!(cx, admin_inv_desc);
    let list_title = tr!(cx, admin_inv_list);
    let note_label = tr!(cx, admin_inv_note);
    let uses_label = tr!(cx, admin_inv_uses);
    let expiry_label = tr!(cx, admin_inv_expiry);
    Ok(view! {
        <div class="tab" data-c="admin-invites">
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
                        let done = create_invite(note.get(), max_uses.get(), days.get()).await;
                        busy.set(false);
                        if done.is_ok() {
                            let code = done.unwrap();
                            note.set("".to_owned());
                            raw!(
                                "document.dispatchEvent(new CustomEvent('tuita-copy',{detail:{text:''+${code},ok:'admin_inv_created_copied',fail:'admin_inv_created',v:'code'}}));",
                                drop(code),
                            );
                            revision.increment();
                            false
                        } else {
                            error.set(done.unwrap_err());
                            false
                        }
                    })
                >
                    text_field(
                        label: &note_label,
                        attrs: attributes! {
                            name="note"
                            maxlength="80"
                            :value=$(note.get())
                            @input=$(|e: Event| note.set(e.target.value))
                        }
                    )
                    <div class="pair">
                        select_field(
                            label: &uses_label,
                            options: &uses,
                            value: "1",
                            attrs: attributes! {
                                name="maxUses"
                                @change=$(|e: Event| max_uses.set(e.target.value))
                            }
                        )
                        select_field(
                            label: &expiry_label,
                            options: &expiry,
                            value: "30",
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
                                :disabled=$(busy.get())
                                :aria-busy=$(if busy.get() { "true" } else { "false" })
                            },
                            (t!(admin_inv_create))
                        )
                    </div>
                </form>
            )
            setting_card(title: &list_title, invite_list(revision: revision.clone()))
        </div>
    })
}
