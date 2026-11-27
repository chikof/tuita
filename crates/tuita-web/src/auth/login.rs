//! The sign-in form.

use topcoat::{
    Result,
    context::Cx,
    metadata::Metadata,
    runtime::{Event, signal},
    view::{View, attributes, component, view},
};
use tuita_server::dto::AccountsDto;

use crate::{
    api,
    auth::{SignupState, ask, auth_card::auth_card, flows::sign_in, signup_state},
    l10n::t,
    next::auth_link,
    session, shell,
    ui::{
        button::{ButtonVariant, button},
        text_field::text_field,
    },
};

/// How many accounts one browser holds when the API cannot say.
const DEFAULT_MAX: u8 = 5;

/// The tab title: "Add another account" with `?add`, else "Sign in".
///
/// # Errors
///
/// A `400` for a malformed query string; fails when the viewer cannot be loaded.
pub(crate) async fn title(cx: &Cx) -> Result<Metadata> {
    let text = if ask(cx)?.add {
        t!(auth_add_title).resolve(cx).into_owned()
    } else {
        t!(auth_login_title).resolve(cx).into_owned()
    };
    shell::titled(cx, &text).await
}

/// The handle and password form, with passkey sign-in under it.
///
/// A successful sign-in navigates with a full page load to `?next=` (a safe path) or home: the
/// page belongs to the previous session. The passkey part is `[data-island="passkey-login"]`
/// (`webauthn.js`): the autofill prompt in the handle field and the "Sign in with a passkey"
/// button, both hidden until the browser can do passkeys.
#[component]
pub(crate) async fn login_form(cx: &Cx) -> Result<impl View> {
    let ask = ask(cx)?;
    let instance = session::instance(cx).await?;
    let max = if ask.add {
        api::get::<AccountsDto>(cx, "/accounts")
            .await
            .map_or(DEFAULT_MAX, |accounts| accounts.max)
    } else {
        DEFAULT_MAX
    };
    let can_join = signup_state(instance) != SignupState::Closed;
    let add = ask.add;
    let next_path = ask.next.clone().unwrap_or_default();
    let handle = signal(cx, String::new);
    let password = signal(cx, String::new);
    let error = signal(cx, String::new);
    let busy = signal(cx, || false);
    let title = if add {
        t!(auth_add_title).resolve(cx).into_owned()
    } else {
        t!(auth_login_title).resolve(cx).into_owned()
    };
    let sub = add.then(|| {
        t!(auth_add_sub, max: i32::from(max))
            .resolve(cx)
            .into_owned()
    });
    let submit_label = if add {
        t!(auth_signin_add).resolve(cx).into_owned()
    } else {
        t!(auth_signin).resolve(cx).into_owned()
    };
    let handle_label = t!(auth_handle).resolve(cx).into_owned();
    let password_label = t!(auth_password).resolve(cx).into_owned();
    let to_signup = auth_link("/signup", add, ask.next.as_deref());
    let domain = instance.domain.clone();
    Ok(view! {
        auth_card(
            title: &title,
            sub: sub.as_deref(),
            footer: Some(
                view! {
                    if add {
                        <a href="/">(t!(cancel))</a>
                    }
                    if can_join {
                        <a href=(&to_signup)>(t!(auth_to_signup))</a>
                    }
                }.into(

                ),
            ),
            <div
                class="login"
                data-island="passkey-login"
                data-add=(if add { "true" } else { "false" })
                data-next=(&next_path)
                data-domain=(&domain)
            >
                <form
                    class="form"
                    @submit=$(async |e: Event| {
                        e.prevent_default();
                        error.set("".to_owned());
                        busy.set(true);
                        let done = sign_in(
                            handle.get().trim().to_owned(),
                            password.get(),
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
                    })
                >
                    text_field(
                        label: &handle_label,
                        attrs: attributes! {
                            name="handle"
                            autocomplete="username webauthn"
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
                        attrs: attributes! {
                            name="password"
                            type="password"
                            autocomplete="current-password"
                            required=""
                            :value=$(password.get())
                            @input=$(|e: Event| password.set(e.target.value))
                        }
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
                        (&submit_label)
                    )
                </form>
                <div data-passkey="" hidden="">
                    <p class="or" aria-hidden="true">(t!(auth_or))</p>
                    button(
                        variant: ButtonVariant::Outline,
                        attrs: attributes! { data-action="passkey" },
                        (t!(auth_passkey))
                    )
                </div>
                <p class="error" role="alert" data-passkey-error="" hidden=""></p>
            </div>
        )
    })
}
