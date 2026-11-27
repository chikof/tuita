//! The sign-up form.

use topcoat::{
    Result,
    context::Cx,
    metadata::Metadata,
    runtime::{Event, signal},
    view::{View, attributes, component, view},
};

use crate::{
    auth::{SignupState, ask, auth_card::auth_card, flows::sign_up, signup_state},
    l10n::t,
    next::auth_link,
    session, shell,
    ui::{button::button, checkbox::checkbox, text_field::text_field},
};

/// The tab title: "Create account".
///
/// # Errors
///
/// Fails when the viewer cannot be loaded.
pub(crate) async fn title(cx: &Cx) -> Result<Metadata> {
    let text = t!(auth_signup_title).resolve(cx).into_owned();
    shell::titled(cx, &text).await
}

/// The sign-up form, or the "closed" notice. With an invite-only instance it asks for the code.
///
/// The checks of the form (invite given, terms accepted) run in the browser first; the API
/// checks everything again. A success navigates with a full page load to `?next=` or home.
#[component]
pub(crate) async fn signup_form(cx: &Cx) -> Result<impl View> {
    let ask = ask(cx)?;
    let instance = session::instance(cx).await?;
    let state = signup_state(instance);
    let closed = state == SignupState::Closed;
    let invite_only = state == SignupState::Invite;
    let add = ask.add;
    let next_path = ask.next.clone().unwrap_or_default();
    let display_name = signal(cx, String::new);
    let handle = signal(cx, String::new);
    let password = signal(cx, String::new);
    let invite = signal(cx, {
        let prefill = ask.invite.clone();
        move || prefill
    });
    let accepted = signal(cx, || false);
    let error = signal(cx, String::new);
    let busy = signal(cx, || false);
    let invite_missing = t!(auth_invite_required).resolve(cx).into_owned();
    let consent_missing = t!(auth_consent_required).resolve(cx).into_owned();
    let invite_label = t!(auth_invite).resolve(cx).into_owned();
    let invite_hint = t!(auth_invite_hint).resolve(cx).into_owned();
    let name_label = t!(auth_display_name).resolve(cx).into_owned();
    let handle_label = t!(auth_handle).resolve(cx).into_owned();
    let handle_hint = t!(auth_handle_hint).resolve(cx).into_owned();
    let password_label = t!(auth_password).resolve(cx).into_owned();
    let password_hint = t!(auth_password_hint).resolve(cx).into_owned();
    let title = t!(auth_signup_title).resolve(cx).into_owned();
    let to_login = auth_link("/login", add, ask.next.as_deref());
    Ok(view! {
        auth_card(
            title: &title,
            footer: Some(
                view! {
                    if add {
                        <a href="/">(t!(cancel))</a>
                    }
                    <a href=(&to_login)>
                        if closed {
                            (t!(auth_signin))
                        } else {
                            (t!(auth_to_login))
                        }
                    </a>
                }.into(

                ),
            ),
            if closed {
                <p class="closed">(t!(auth_closed))</p>
            } else {
                <form
                    class="form"
                    @submit=$(async |e: Event| {
                        e.prevent_default();
                        error.set("".to_owned());
                        let need_invite = if invite_only {
                            invite.get().trim().is_empty()
                        } else {
                            false
                        };
                        if need_invite {
                            error.set(invite_missing);
                            false
                        } else if !accepted.get() {
                            error.set(consent_missing);
                            false
                        } else {
                            busy.set(true);
                            let done = sign_up(
                                handle.get().trim().to_owned(),
                                display_name.get().trim().to_owned(),
                                password.get(),
                                invite.get().trim().to_owned(),
                                add,
                                next_path,
                            ).await;
                            if done.is_ok() {
                                let to = done.unwrap();
                                raw!("location.assign(${to})", to.is_empty())
                            } else {
                                error.set(done.unwrap_err());
                                busy.set(false);
                                false
                            }
                        }
                    })
                >
                    if invite_only {
                        text_field(
                            label: &invite_label,
                            hint: invite_hint.as_str(),
                            attrs: attributes! {
                                name="invite"
                                autocomplete="off"
                                autocapitalize="characters"
                                spellcheck="false"
                                maxlength="40"
                                required=""
                                :value=$(invite.get())
                                @input=$(|e: Event| invite.set(e.target.value))
                            }
                        )
                    }
                    text_field(
                        label: &name_label,
                        attrs: attributes! {
                            name="name"
                            autocomplete="name"
                            maxlength="50"
                            :value=$(display_name.get())
                            @input=$(|e: Event| display_name.set(e.target.value))
                        }
                    )
                    text_field(
                        label: &handle_label,
                        hint: handle_hint.as_str(),
                        attrs: attributes! {
                            name="handle"
                            autocomplete="username"
                            autocapitalize="off"
                            spellcheck="false"
                            maxlength="16"
                            required=""
                            :value=$(handle.get())
                            @input=$(|e: Event| handle.set(e.target.value))
                        }
                    )
                    text_field(
                        label: &password_label,
                        hint: password_hint.as_str(),
                        attrs: attributes! {
                            name="password"
                            type="password"
                            autocomplete="new-password"
                            required=""
                            :value=$(password.get())
                            @input=$(|e: Event| password.set(e.target.value))
                        }
                    )
                    checkbox(
                        attrs: attributes! {
                            name="accept"
                            :checked=$(accepted.get())
                            @change=$(|e: Event| accepted.set(e.target.checked))
                        },
                        (t!(auth_consent_pre, minAge: i32::from(instance.min_age)))
                        <a href="/terms" target="_blank" rel="noopener">
                            (t!(auth_consent_terms))
                        </a>
                        (t!(auth_consent_and))
                        <a href="/privacy" target="_blank" rel="noopener">
                            (t!(auth_consent_privacy))
                        </a>
                        (t!(auth_consent_end))
                    )
                    <p class="error" role="alert" :hidden=$(error.get().is_empty())>
                        $(error.get())
                    </p>
                    button(
                        submit: true,
                        attrs: attributes! {
                            :disabled=$(busy.get())
                            :aria-busy=$(if busy.get() { "true" } else { "false" })
                        },
                        (t!(auth_create))
                    )
                </form>
            }
        )
    })
}
