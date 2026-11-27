//! `/settings/bots`: the automated flag and API tokens.

use serde_json::json;
use time::format_description::well_known::Rfc3339;
use topcoat::{
    Result,
    context::Cx,
    runtime::{Event, Signal, procedure, shard, signal},
    view::{View, attributes, component, view},
};
use tuita_server::dto::{CreatedTokenDto, MeDto, OkDto, TokenDto, TokenScope};

use crate::{
    api, fmt,
    icons::Icon,
    l10n::t,
    settings::{
        actions::{Done, failed},
        card::setting_card,
        page::settings_page,
        token_secret::token_secret,
        tokens::{DEFAULT_SCOPES, SCOPES, order_scopes, scope_hint, scope_label, scope_list},
    },
    ui::{
        button::{ButtonSize, ButtonVariant, button},
        checkbox::checkbox,
        confirm_dialog::confirm_dialog,
        icon::icon,
        switch::switch,
        text_field::text_field,
    },
};

/// Marks the account automated or not, naming who runs it; answers the toast.
#[procedure]
pub(crate) async fn save_bot(cx: &Cx, bot: bool, owner: String) -> Done {
    let owner = owner.trim().trim_start_matches('@');
    let body = json!({
        "bot": bot,
        "owner": if bot && !owner.is_empty() { Some(owner) } else { None },
    });
    let done = failed(api::patch::<_, MeDto>(cx, "/me/bot", &body).await);
    Ok(done.map(|_| t!(settings_saved).resolve(cx).into_owned()))
}

/// Creates a token with the ticked scopes; answers the secret, which is shown once.
#[procedure]
pub(crate) async fn create_token(
    cx: &Cx,
    name: String,
    read: bool,
    write: bool,
    messages: bool,
) -> Done {
    let ticked: Vec<TokenScope> = [
        (read, TokenScope::Read),
        (write, TokenScope::Write),
        (messages, TokenScope::Messages),
    ]
    .into_iter()
    .filter_map(|(on, scope)| on.then_some(scope))
    .collect();
    let body = json!({ "name": name.trim(), "scopes": order_scopes(&ticked) });
    let done = failed(api::post::<_, CreatedTokenDto>(cx, "/tokens", &body).await);
    Ok(done.map(|made| made.token))
}

/// Revokes a token; answers the toast.
#[procedure]
pub(crate) async fn revoke_token(cx: &Cx, id: String) -> Done {
    let done = failed(api::delete::<OkDto>(cx, &format!("/tokens/{}", api::enc(&id))).await);
    Ok(done.map(|_| t!(token_revoked).resolve(cx).into_owned()))
}

/// The tokens of the account; reruns when `rev` moves.
#[shard]
async fn token_rows(cx: &Cx, rev: Signal<u32>) -> Result<impl View> {
    let _ = rev.get();
    let tokens = api::get::<Vec<TokenDto>>(cx, "/tokens").await?;
    let revoke = t!(token_revoke).resolve(cx).into_owned();
    let body = t!(token_revoke_body).resolve(cx).into_owned();
    let rows: Vec<_> = tokens
        .into_iter()
        .map(|token| {
            let used = match token.last_used_at {
                Some(at) => t!(
                    token_used_ago,
                    time: fmt::rel_label(cx, &at.format(&Rfc3339).unwrap_or_default()),
                )
                .resolve(cx)
                .into_owned(),
                None => t!(token_never_used).resolve(cx).into_owned(),
            };
            let label = t!(token_revoke_label, name: token.name.as_str())
                .resolve(cx)
                .into_owned();
            let title = t!(token_revoke_title, name: token.name.as_str())
                .resolve(cx)
                .into_owned();
            let details = format!("{} · {}", scope_list(cx, &token.scopes), used);
            (
                token.id.to_string(),
                format!("revoke-{}", token.id),
                token,
                details,
                label,
                title,
            )
        })
        .collect();
    Ok(view! {
        if rows.is_empty() {
            <p class="dim">(t!(token_none))</p>
        } else {
            <ul class="tokens">
                #[key(tid.as_str())]
                for (tid, revoke_id, token, details, label, title) in rows {
                    <li data-token=(token.id.to_string())>
                        <span class="icon">icon(name: Icon::Lock, size: 20)</span>
                        <div class="text">
                            <p class="name">(&token.name)</p>
                            <p class="dim">
                                <code>
                                    (&token.prefix)
                                    "…"
                                </code>
                                " · "
                                (&details)
                            </p>
                        </div>
                        button(
                            variant: ButtonVariant::Outline,
                            size: ButtonSize::Sm,
                            attrs: attributes! {
                                aria-label=(&label)
                                command="show-modal"
                                commandfor=(&revoke_id)
                            },
                            (&revoke)
                        )
                        confirm_dialog(
                            id: &revoke_id,
                            title: &title,
                            body: &body,
                            confirm_label: &revoke,
                            danger: true,
                            action: $(async |_e: Event| {
                                let done = revoke_token(tid).await;
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
                }
            </ul>
        }
    })
}

/// The automated switch with its owner field.
#[component]
async fn bot_card(cx: &Cx, me: &MeDto) -> Result<impl View> {
    let bot = signal(cx, || me.user.bot);
    let saved = signal(cx, || me.user.bot);
    let owner = signal(cx, || me.user.bot_owner.clone().unwrap_or_default());
    let busy = signal(cx, || false);
    let owner_label = t!(bot_owner).resolve(cx).into_owned();
    let owner_hint = t!(bot_owner_hint).resolve(cx).into_owned();
    let switch_label = t!(bot_switch).resolve(cx).into_owned();
    Ok(view! {
        setting_card(
            title: t!(bot_title).resolve(cx).into_owned(),
            description: t!(bot_desc).resolve(cx).into_owned(),
            <div
                data-c="bot-switch"
                @click=$(async |_e: Event| {
                    let next = bot.get();
                    if next != saved.get() {
                        saved.set(next);
                        busy.set(true);
                        let done = save_bot(next, owner.get()).await;
                        busy.set(false);
                        if done.is_ok() {
                            let msg = done.unwrap();
                            raw!(
                                "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:${msg}}))",
                                msg.is_empty(),
                            )
                        } else {
                            bot.set(!next);
                            saved.set(!next);
                            let msg = done.unwrap_err();
                            raw!(
                                "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:${msg}}))",
                                msg.is_empty(),
                            )
                        };
                    }
                })
            >
                switch(
                    label: &switch_label,
                    checked: &bot,
                    attrs: attributes! { :disabled=$(busy.get()) }
                )
            </div>
            <form
                class="owner"
                novalidate=(true)
                :hidden=$(!bot.get())
                @submit=$(async |e: Event| {
                    e.prevent_default();
                    busy.set(true);
                    let done = save_bot(true, owner.get()).await;
                    busy.set(false);
                    if done.is_ok() {
                        let msg = done.unwrap();
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
            >
                text_field(
                    label: &owner_label,
                    hint: owner_hint.as_str(),
                    attrs: attributes! {
                        name="owner"
                        maxlength="16"
                        autocapitalize="off"
                        spellcheck="false"
                        :value=$(owner.get())
                        @input=$(|e: Event| owner.set(e.target.value))
                    }
                )
                button(
                    variant: ButtonVariant::Outline,
                    submit: true,
                    attrs: attributes! { :disabled=$(busy.get()) },
                    (t!(bot_owner_save))
                )
            </form>
        )
    })
}

/// The tokens card: the list, the create form and the secret dialog.
#[component]
async fn token_card(cx: &Cx) -> Result<impl View> {
    let rev = signal(cx, || 0u32);
    let name = signal(cx, String::new);
    let read = signal(cx, || DEFAULT_SCOPES.contains(&TokenScope::Read));
    let write = signal(cx, || DEFAULT_SCOPES.contains(&TokenScope::Write));
    let messages = signal(cx, || DEFAULT_SCOPES.contains(&TokenScope::Messages));
    let secret = signal(cx, String::new);
    let busy = signal(cx, || false);
    let error = signal(cx, String::new);
    let name_label = t!(token_name).resolve(cx).into_owned();
    let name_hint = t!(token_name_hint).resolve(cx).into_owned();
    let scopes: Vec<_> = SCOPES
        .into_iter()
        .map(|scope| (scope, scope_label(cx, scope), scope_hint(cx, scope)))
        .collect();
    Ok(view! {
        setting_card(
            title: t!(token_title).resolve(cx).into_owned(),
            description: t!(token_desc).resolve(cx).into_owned(),
            <div data-c="token-list">token_rows(rev: rev.clone())</div>
            <form
                class="create"
                data-c="token-form"
                novalidate=(true)
                @submit=$(async |e: Event| {
                    e.prevent_default();
                    error.set("".to_owned());
                    busy.set(true);
                    let done = create_token(
                        name.get(),
                        read.get(),
                        write.get(),
                        messages.get(),
                    ).await;
                    busy.set(false);
                    if done.is_ok() {
                        secret.set(done.unwrap());
                        name.set("".to_owned());
                        read.set(true);
                        write.set(true);
                        messages.set(false);
                        rev.increment();
                        raw!("document.getElementById('token-secret').showModal()")
                    } else {
                        error.set(done.unwrap_err());
                        false
                    }
                })
            >
                text_field(
                    label: &name_label,
                    hint: name_hint.as_str(),
                    attrs: attributes! {
                        name="token-name"
                        maxlength="40"
                        required=(true)
                        :value=$(name.get())
                        @input=$(|e: Event| name.set(e.target.value))
                    }
                )
                <fieldset>
                    <legend>(t!(token_scopes))</legend>
                    #[key(scope.as_str())]
                    for (scope, label, hint) in scopes {
                        <div class="scope">
                            if scope == TokenScope::Read {
                                checkbox(
                                    label: &label,
                                    attrs: attributes! {
                                        name="scope"
                                        value="read"
                                        :checked=$(read.get())
                                        @change=$(|e: Event| read.set(e.target.checked))
                                    }
                                )
                            } else if scope == TokenScope::Write {
                                checkbox(
                                    label: &label,
                                    attrs: attributes! {
                                        name="scope"
                                        value="write"
                                        :checked=$(write.get())
                                        @change=$(|e: Event| write.set(e.target.checked))
                                    }
                                )
                            } else {
                                checkbox(
                                    label: &label,
                                    attrs: attributes! {
                                        name="scope"
                                        value="messages"
                                        :checked=$(messages.get())
                                        @change=$(|e: Event| messages.set(e.target.checked))
                                    }
                                )
                            }
                            <p class="dim">(&hint)</p>
                        </div>
                    }
                </fieldset>
                <p class="error" role="alert" :hidden=$(error.get().is_empty())>
                    $(error.get())
                </p>
                <div class="end">
                    button(
                        submit: true,
                        attrs: attributes! {
                            :disabled=$(if busy.get() {
                                true
                            } else if name.get().trim().is_empty() {
                                true
                            } else if read.get() {
                                false
                            } else if write.get() {
                                false
                            } else {
                                !messages.get()
                            })
                        },
                        (t!(token_create))
                    )
                </div>
            </form>
            token_secret(secret: &secret)
        )
    })
}

/// The automated account and API token cards, and the link to the developer docs.
#[component]
pub(crate) async fn bots(cx: &Cx, me: &MeDto) -> Result<impl View> {
    let title = t!(settings_bots).resolve(cx).into_owned();
    Ok(view! {
        settings_page(
            title: &title,
            bot_card(me: me)
            token_card()
            <a class="docs wash" data-c="bot-docs" href="/developers">
                <span class="text">
                    <span class="name">(t!(bot_docs))</span>
                    <span class="dim">(t!(bot_docs_desc))</span>
                </span>
                icon(name: Icon::ChevronRight, size: 20)
            </a>
        )
    })
}
