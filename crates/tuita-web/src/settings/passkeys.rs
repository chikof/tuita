//! `/settings/passkeys`.
//!
//! The browser ceremony is the `passkey-register` island of `webauthn.js`: it registers a passkey
//! and fires `tuita-passkey`, which makes the list fetch again.

use serde_json::json;
use time::format_description::well_known::Rfc3339;
use topcoat::{
    Result,
    context::Cx,
    router::request::headers,
    runtime::{Event, Signal, procedure, shard, signal},
    view::{View, attributes, component, view},
};
use tuita_server::dto::{OkDto, PasskeyDto, PasskeysDto};

use crate::{
    api, fmt,
    icons::Icon,
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
        dialog::dialog,
        icon::icon,
        text_field::text_field,
    },
};

/// The most passkeys one account may have.
const MAX_PASSKEYS: usize = 20;

/// Renames a passkey.
#[procedure]
pub(crate) async fn rename_passkey(cx: &Cx, id: String, name: String) -> Done {
    let path = format!("/passkeys/{}", api::enc(&id));
    let done =
        failed(api::patch::<_, PasskeyDto>(cx, &path, &json!({ "name": name.trim() })).await);
    Ok(done.map(|_| String::new()))
}

/// Removes a passkey; answers the toast.
#[procedure]
pub(crate) async fn remove_passkey(cx: &Cx, id: String) -> Done {
    let done = failed(api::delete::<OkDto>(cx, &format!("/passkeys/{}", api::enc(&id))).await);
    Ok(done.map(|_| t!(passkey_removed).resolve(cx).into_owned()))
}

/// `Chrome on Linux`: a name that tells this device's passkeys apart; empty lets the API pick.
fn device_name(cx: &Cx) -> String {
    let agent = headers(cx)
        .get("user-agent")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    match device_parts(agent, None) {
        (Some(browser), Some(os)) => t!(settings_device_on, browser: browser, os: os)
            .resolve(cx)
            .into_owned(),
        (Some(name), None) | (None, Some(name)) => name.to_owned(),
        (None, None) => t!(passkey_default_name).resolve(cx).into_owned(),
    }
}

/// One passkey with its rename dialog and remove confirmation.
#[component]
async fn passkey_row(cx: &Cx, passkey: &PasskeyDto, rev: &Signal<u32>) -> Result<impl View> {
    let id = passkey.id.to_string();
    let id_rename = id.clone();
    let id_remove = id.clone();
    let draft = signal(cx, || passkey.name.clone());
    let busy = signal(cx, || false);
    let busy_save = busy.clone();
    let draft_save = draft.clone();
    let failure = signal(cx, String::new);
    let created = passkey.created_at.format(&Rfc3339).unwrap_or_default();
    let used = match passkey.last_used_at {
        Some(at) => t!(
            passkey_used_ago,
            time: fmt::rel_label(cx, &at.format(&Rfc3339).unwrap_or_default()),
        )
        .resolve(cx)
        .into_owned(),
        None => t!(passkey_never_used).resolve(cx).into_owned(),
    };
    let added = t!(passkey_added_ago, time: fmt::rel_label(cx, &created))
        .resolve(cx)
        .into_owned();
    let rename_label = t!(passkey_rename_label, name: passkey.name.as_str())
        .resolve(cx)
        .into_owned();
    let remove_label = t!(passkey_remove_label, name: passkey.name.as_str())
        .resolve(cx)
        .into_owned();
    let remove_title = t!(passkey_remove_title, name: passkey.name.as_str())
        .resolve(cx)
        .into_owned();
    let remove_body = t!(passkey_remove_body).resolve(cx).into_owned();
    let remove = t!(passkey_remove).resolve(cx).into_owned();
    let rename_title = t!(passkey_rename_title).resolve(cx).into_owned();
    let name_label = t!(passkey_name).resolve(cx).into_owned();
    let rename_id = format!("rename-{id}");
    let remove_id = format!("remove-{id}");
    let form_id = format!("form-{id}");
    let rename_id_inner = rename_id.clone();
    let form_id_inner = form_id.clone();
    let cancel = t!(cancel).resolve(cx).into_owned();
    let save = t!(passkey_rename_save).resolve(cx).into_owned();
    Ok(view! {
        <li data-passkey=(&id)>
            <span class="icon">icon(name: Icon::Lock, size: 20)</span>
            <div class="text">
                <p class="name">(&passkey.name)</p>
                <p class="dim">
                    (&added)
                    " · "
                    (&used)
                </p>
            </div>
            button(
                variant: ButtonVariant::Ghost,
                size: ButtonSize::Sm,
                attrs: attributes! {
                    aria-label=(&rename_label)
                    command="show-modal"
                    commandfor=(&rename_id)
                },
                (t!(passkey_rename))
            )
            button(
                variant: ButtonVariant::Outline,
                size: ButtonSize::Sm,
                attrs: attributes! {
                    aria-label=(&remove_label)
                    command="show-modal"
                    commandfor=(&remove_id)
                },
                (&remove)
            )
            dialog(
                id: &rename_id,
                title: &rename_title,
                actions: Some(
                    view! {
                        button(
                            variant: ButtonVariant::Ghost,
                            attrs: attributes! { command="close" commandfor=(&rename_id_inner) },
                            (&cancel)
                        )
                        button(
                            submit: true,
                            attrs: attributes! {
                                form=(&form_id_inner)
                                :disabled=$(if busy_save.get() {
                                    true
                                } else {
                                    draft_save.get().trim().is_empty()
                                })
                            },
                            (&save)
                        )
                    }.into(

                    ),
                ),
                <form
                    id=(&form_id)
                    data-c="passkey-rename"
                    novalidate=(true)
                    @submit=$(async |e: Event| {
                        e.prevent_default();
                        busy.set(true);
                        failure.set("".to_owned());
                        let done = rename_passkey(id_rename, draft.get()).await;
                        busy.set(false);
                        if done.is_ok() {
                            rev.increment();
                            false
                        } else {
                            failure.set(done.unwrap_err());
                            false
                        }
                    })
                >
                    text_field(
                        label: &name_label,
                        attrs: attributes! {
                            name="name"
                            maxlength="60"
                            required=(true)
                            :value=$(draft.get())
                            @input=$(|e: Event| draft.set(e.target.value))
                        }
                    )
                    <p class="error" role="alert" :hidden=$(failure.get().is_empty())>
                        $(failure.get())
                    </p>
                </form>
            )
            confirm_dialog(
                id: &remove_id,
                title: &remove_title,
                body: &remove_body,
                confirm_label: &remove,
                danger: true,
                action: $(async |_e: Event| {
                    let done = remove_passkey(id_remove).await;
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
        </li>
    })
}

/// The passkeys, the notices and the button that adds one; reruns when `rev` moves.
#[shard]
async fn passkey_rows(cx: &Cx, rev: Signal<u32>) -> Result<impl View> {
    let _ = rev.get();
    let data = api::get::<PasskeysDto>(cx, "/passkeys").await?;
    let name = crate::l10n::strip_isolates(&device_name(cx));
    let allowed = data.rp_id.is_some() && data.passkeys.len() < MAX_PASSKEYS;
    let needs_domain = data.rp_id.is_none();
    Ok(view! {
        if data.passkeys.is_empty() {
            <p class="dim">(t!(passkey_none))</p>
        } else {
            <ul>
                #[key(passkey.id.as_bytes())]
                for passkey in &data.passkeys {
                    passkey_row(passkey: passkey, rev: &rev)
                }
            </ul>
        }
        <p class="dim" role="status" data-passkey-unsupported="" hidden="">
            (t!(passkey_unsupported))
        </p>
        if needs_domain {
            <p class="dim" role="status">(t!(passkey_needs_domain))</p>
        }
        <div class="end">
            <div
                data-island="passkey-register"
                @tuita-passkey=$(|_e: Event| rev.increment())
            >
                <input type="hidden" data-passkey-name="" value=(&name)>
                button(
                    attrs: attributes! { data-action="register" },
                    disabled: !allowed,
                    (t!(passkey_add))
                )
            </div>
        </div>
    })
}

/// The passkeys card.
#[component]
pub(crate) async fn passkeys(cx: &Cx) -> Result<impl View> {
    let rev = signal(cx, || 0u32);
    let title = t!(settings_passkeys).resolve(cx).into_owned();
    Ok(view! {
        settings_page(
            title: &title,
            setting_card(
                title: t!(passkey_title).resolve(cx).into_owned(),
                description: t!(passkey_desc).resolve(cx).into_owned(),
                <div data-c="passkeys">passkey_rows(rev: rev)</div>
            )
        )
    })
}
