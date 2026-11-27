//! `/settings/devices`: where the account is signed in.

use time::format_description::well_known::Rfc3339;
use topcoat::{
    Result,
    context::Cx,
    runtime::{Event, Signal, procedure, shard, signal},
    view::{View, attributes, component, view},
};
use tuita_server::dto::{OkDto, RevokedDto, SessionDto};

use crate::{
    api, fmt,
    l10n::t,
    settings::{
        actions::{Done, failed},
        card::setting_card,
        device::device_parts,
        page::settings_page,
    },
    ui::{
        button::{ButtonSize, ButtonVariant, button},
        confirm_dialog::confirm_dialog,
        tag::tag,
    },
};

/// Signs one session out; answers the toast.
#[procedure]
pub(crate) async fn sign_out_session(cx: &Cx, id: String) -> Done {
    let done = failed(api::delete::<OkDto>(cx, &format!("/me/sessions/{}", api::enc(&id))).await);
    Ok(done.map(|_| t!(settings_session_signed_out).resolve(cx).into_owned()))
}

/// Signs out every session but this one; answers the toast with the count.
#[procedure]
pub(crate) async fn sign_out_others(cx: &Cx) -> Done {
    let done = failed(api::delete::<RevokedDto>(cx, "/me/sessions").await);
    Ok(done.map(|out| {
        let count = i32::try_from(out.revoked).unwrap_or(i32::MAX);
        t!(settings_sessions_revoked, count: count)
            .resolve(cx)
            .into_owned()
    }))
}

/// `Chrome on Linux`, or what is known of it.
fn label(cx: &Cx, session: &SessionDto) -> String {
    match device_parts(&session.user_agent, session.client.as_deref()) {
        (Some(browser), Some(os)) => t!(settings_device_on, browser: browser, os: os)
            .resolve(cx)
            .into_owned(),
        (Some(name), None) | (None, Some(name)) => name.to_owned(),
        (None, None) => t!(settings_device_unknown).resolve(cx).into_owned(),
    }
}

/// The sessions of the account; reruns when `rev` moves.
#[shard]
async fn session_rows(cx: &Cx, rev: Signal<u32>) -> Result<impl View> {
    let _ = rev.get();
    let sessions = api::get::<Vec<SessionDto>>(cx, "/me/sessions").await?;
    let others = sessions.iter().any(|s| !s.current);
    let rows: Vec<_> = sessions
        .into_iter()
        .map(|session| {
            let name = label(cx, &session);
            let sid = session.id.to_string();
            let at = session.last_used_at.format(&Rfc3339).unwrap_or_default();
            let active = t!(settings_session_last_active, time: fmt::rel_label(cx, &at))
                .resolve(cx)
                .into_owned();
            let sign_out = t!(settings_session_sign_out_device, device: name.as_str())
                .resolve(cx)
                .into_owned();
            (sid, session, name, active, sign_out)
        })
        .collect();
    let title = t!(settings_sessions_revoke_title).resolve(cx).into_owned();
    let body = t!(settings_sessions_revoke_body).resolve(cx).into_owned();
    let confirm = t!(settings_sessions_revoke_confirm)
        .resolve(cx)
        .into_owned();
    Ok(view! {
        <ul>
            #[key(sid.as_str())]
            for (sid, session, name, active, sign_out) in rows {
                <li>
                    <div class="text">
                        <p class="device">
                            <span>(&name)</span>
                            if session.current {
                                tag(pill: true, (t!(settings_session_current)))
                            }
                        </p>
                        if session.ip.is_some() || session.country.is_some() {
                            <p class="meta">
                                if let Some(ip) = &session.ip {
                                    (ip)
                                }
                                if session.ip.is_some() && session.country.is_some() {
                                    " · "
                                }
                                if let Some(country) = &session.country {
                                    <span data-region=(country)>(country)</span>
                                }
                            </p>
                        }
                        <p class="meta">(&active)</p>
                    </div>
                    if !session.current {
                        button(
                            variant: ButtonVariant::Outline,
                            size: ButtonSize::Sm,
                            attrs: attributes! {
                                aria-label=(&sign_out)
                                @click=$(async |_e: Event| {
                                    let done = sign_out_session(sid).await;
                                    if done.is_ok() {
                                        let msg = done.unwrap();
                                        rev.increment();
                                        raw!(
                                            "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:${msg}}))",
                                            msg.is_empty(),
                                        )
                                    } else {
                                        let msg = done.unwrap_err();
                                        raw!(
                                            "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:${msg}}))",
                                            msg.is_empty(),
                                        )
                                    }
                                })
                            },
                            (t!(settings_session_sign_out))
                        )
                    }
                </li>
            }
        </ul>
        if others {
            <div class="end">
                button(
                    variant: ButtonVariant::Danger,
                    attrs: attributes! { command="show-modal" commandfor="revoke-others" },
                    (t!(settings_sessions_revoke_all))
                )
            </div>
            confirm_dialog(
                id: "revoke-others",
                title: &title,
                body: &body,
                confirm_label: &confirm,
                danger: true,
                action: $(async |_e: Event| {
                    let done = sign_out_others().await;
                    if done.is_ok() {
                        let msg = done.unwrap();
                        rev.increment();
                        raw!(
                            "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:${msg}}))",
                            msg.is_empty(),
                        )
                    } else {
                        let msg = done.unwrap_err();
                        raw!(
                            "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:${msg}}))",
                            msg.is_empty(),
                        )
                    }
                })
            )
        }
    })
}

/// The sessions card.
#[component]
pub(crate) async fn devices(cx: &Cx) -> Result<impl View> {
    let rev = signal(cx, || 0u32);
    let title = t!(settings_devices).resolve(cx).into_owned();
    Ok(view! {
        settings_page(
            title: &title,
            setting_card(
                title: t!(settings_sessions_title).resolve(cx).into_owned(),
                description: t!(settings_sessions_desc).resolve(cx).into_owned(),
                <div data-c="sessions">session_rows(rev: rev)</div>
            )
        )
    })
}
