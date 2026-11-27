//! The change password card.

use serde_json::json;
use topcoat::{
    Result,
    context::Cx,
    runtime::{Event, procedure, signal},
    view::{View, attributes, component, view},
};
use tuita_server::dto::OkDto;

use crate::{
    api,
    l10n::t,
    settings::{
        actions::{Done, failed},
        card::setting_card,
    },
    ui::{button::button, text_field::text_field},
};

/// `POST /me/password`; the new password must match its confirmation.
#[procedure]
pub(crate) async fn change_password(
    cx: &Cx,
    current: String,
    next: String,
    confirm: String,
) -> Done {
    if next != confirm {
        return Ok(Err(t!(settings_password_mismatch).resolve(cx).into_owned()));
    }
    let done = failed(
        api::post::<_, OkDto>(
            cx,
            "/me/password",
            &json!({ "current": current, "next": next }),
        )
        .await,
    );
    Ok(done.map(|_| t!(settings_password_updated).resolve(cx).into_owned()))
}

/// Current, new and confirmed password. A refusal shows as a toast; success clears the fields.
#[component]
pub(crate) async fn password_form(cx: &Cx) -> Result<impl View> {
    let current = signal(cx, String::new);
    let next = signal(cx, String::new);
    let confirm = signal(cx, String::new);
    let busy = signal(cx, || false);
    let current_label = t!(settings_password_current).resolve(cx).into_owned();
    let next_label = t!(settings_password_new).resolve(cx).into_owned();
    let next_hint = t!(settings_password_hint).resolve(cx).into_owned();
    let confirm_label = t!(settings_password_confirm).resolve(cx).into_owned();
    Ok(view! {
        setting_card(
            title: t!(settings_password_title).resolve(cx).into_owned(),
            <form
                data-c="password-form"
                novalidate=(true)
                @submit=$(async |e: Event| {
                    e.prevent_default();
                    busy.set(true);
                    let done = change_password(current.get(), next.get(), confirm.get()).await;
                    busy.set(false);
                    if done.is_ok() {
                        let msg = done.unwrap();
                        current.set("".to_owned());
                        next.set("".to_owned());
                        confirm.set("".to_owned());
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
                    label: &current_label,
                    attrs: attributes! {
                        type="password"
                        name="current"
                        autocomplete="current-password"
                        required=(true)
                        :value=$(current.get())
                        @input=$(|e: Event| current.set(e.target.value))
                    }
                )
                text_field(
                    label: &next_label,
                    hint: next_hint.as_str(),
                    attrs: attributes! {
                        type="password"
                        name="next"
                        autocomplete="new-password"
                        minlength="8"
                        maxlength="200"
                        required=(true)
                        :value=$(next.get())
                        @input=$(|e: Event| next.set(e.target.value))
                    }
                )
                text_field(
                    label: &confirm_label,
                    attrs: attributes! {
                        type="password"
                        name="confirm"
                        autocomplete="new-password"
                        required=(true)
                        :value=$(confirm.get())
                        @input=$(|e: Event| confirm.set(e.target.value))
                    }
                )
                <div class="end">
                    button(
                        submit: true,
                        attrs: attributes! { :disabled=$(busy.get()) },
                        (t!(settings_password_submit))
                    )
                </div>
            </form>
        )
    })
}
