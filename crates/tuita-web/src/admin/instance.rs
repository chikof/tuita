//! The instance tab: name, rules and who may sign up.

use topcoat::{
    Result,
    context::Cx,
    runtime::{Event, procedure, signal},
    view::{View, attributes, component, view},
};
use tuita_server::dto::{AdminSettingsDto, PatchSettingsRequest, SignupsMode};

use crate::{
    admin::{
        common::{message, tr},
        rules::lines_of,
        setting_card::setting_card,
    },
    api,
    l10n::t,
    ui::{button::button, radio::radio, text_area::text_area, text_field::text_field},
};

/// `PATCH /admin/settings` with the fields of the form. Answers the toast text.
#[procedure]
pub(crate) async fn save_instance(
    cx: &Cx,
    name: String,
    description: String,
    contact: String,
    min_age: String,
    rules: String,
    signups: String,
) -> Result<std::result::Result<String, String>> {
    let body = PatchSettingsRequest {
        name: Some(name.trim().to_owned()),
        description: Some(description.trim().to_owned()),
        contact: Some(contact.trim().to_owned()),
        rules: Some(lines_of(&rules)),
        min_age: min_age.trim().parse().ok(),
        signups: SignupsMode::parse(&signups),
        filter: None,
    };
    Ok(
        api::patch::<_, AdminSettingsDto>(cx, "/admin/settings", &body)
            .await
            .map(|_| tr!(cx, admin_instance_saved))
            .map_err(message),
    )
}

/// `GET /admin/settings`.
#[component]
pub(crate) async fn instance(cx: &Cx) -> Result<impl View> {
    let loaded = api::get::<AdminSettingsDto>(cx, "/admin/settings").await?;
    let name = signal(cx, || loaded.name.clone());
    let description = signal(cx, || loaded.description.clone());
    let contact = signal(cx, || loaded.contact.clone());
    let min_age = signal(cx, || loaded.min_age.to_string());
    let rules = signal(cx, || loaded.rules.join("\n"));
    let signups = signal(cx, || loaded.signups.as_str().to_owned());
    let error = signal(cx, String::new);
    let busy = signal(cx, || false);
    let modes = [
        (
            SignupsMode::Open,
            tr!(cx, admin_signups_open),
            tr!(cx, admin_signups_open_hint),
        ),
        (
            SignupsMode::Invite,
            tr!(cx, admin_signups_invite),
            tr!(cx, admin_signups_invite_hint),
        ),
        (
            SignupsMode::Closed,
            tr!(cx, admin_signups_closed),
            tr!(cx, admin_signups_closed_hint),
        ),
    ];
    let min_age_text = loaded.min_age.to_string();
    let rules_text = loaded.rules.join("\n");
    let title = tr!(cx, admin_instance_title);
    let signups_title = tr!(cx, admin_signups_title);
    let signups_desc = tr!(cx, admin_signups_desc);
    let name_label = tr!(cx, admin_instance_name);
    let description_label = tr!(cx, admin_instance_description);
    let contact_label = tr!(cx, admin_instance_contact);
    let contact_hint = tr!(cx, admin_instance_contact_hint);
    let min_age_label = tr!(cx, admin_instance_min_age);
    let rules_label = tr!(cx, admin_instance_rules);
    let rules_hint = tr!(cx, admin_instance_rules_hint);
    Ok(view! {
        <div class="tab" data-c="admin-instance">
            <form
                class="form"
                novalidate=""
                @submit=$(async |e: Event| {
                    e.prevent_default();
                    error.set("".to_owned());
                    busy.set(true);
                    let done = save_instance(
                        name.get(),
                        description.get(),
                        contact.get(),
                        min_age.get(),
                        rules.get(),
                        signups.get(),
                    ).await;
                    busy.set(false);
                    if done.is_ok() {
                        let msg = done.unwrap();
                        raw!(
                            "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:''+${msg}}))",
                            msg.is_empty(),
                        )
                    } else {
                        error.set(done.unwrap_err());
                        false
                    }
                })
            >
                setting_card(
                    title: &title,
                    description: loaded.domain.as_str(),
                    text_field(
                        label: &name_label,
                        value: &loaded.name,
                        attrs: attributes! {
                            name="name"
                            maxlength="40"
                            required=""
                            :value=$(name.get())
                            @input=$(|e: Event| name.set(e.target.value))
                        }
                    )
                    text_area(
                        label: &description_label,
                        value: &loaded.description,
                        rows: 3,
                        attrs: attributes! {
                            name="description"
                            maxlength="300"
                            :value=$(description.get())
                            @input=$(|e: Event| description.set(e.target.value))
                        }
                    )
                    text_field(
                        label: &contact_label,
                        hint: contact_hint.as_str(),
                        value: &loaded.contact,
                        attrs: attributes! {
                            name="contact"
                            type="email"
                            maxlength="120"
                            :value=$(contact.get())
                            @input=$(|e: Event| contact.set(e.target.value))
                        }
                    )
                    text_field(
                        label: &min_age_label,
                        value: &min_age_text,
                        attrs: attributes! {
                            name="minAge"
                            type="number"
                            min="13"
                            max="21"
                            :value=$(min_age.get())
                            @input=$(|e: Event| min_age.set(e.target.value))
                        }
                    )
                    text_area(
                        label: &rules_label,
                        hint: rules_hint.as_str(),
                        value: &rules_text,
                        rows: 6,
                        attrs: attributes! {
                            name="rules"
                            maxlength="3000"
                            :value=$(rules.get())
                            @input=$(|e: Event| rules.set(e.target.value))
                        }
                    )
                )
                setting_card(
                    title: &signups_title,
                    description: signups_desc.as_str(),
                    <div class="modes" role="radiogroup" aria-label=(&signups_title)>
                        for (mode, label, hint) in &modes {
                            <div class="mode">
                                radio(
                                    group: "signups",
                                    value: mode.as_str(),
                                    label: label,
                                    checked: loaded.signups == *mode,
                                    attrs: attributes! { @change=$(|e: Event| signups.set(e.target.value)) }
                                )
                                <p class="hint">(hint)</p>
                            </div>
                        }
                    </div>
                )
                <p class="error" role="alert" :hidden=$(error.get().is_empty())>
                    $(error.get())
                </p>
                <div class="end">
                    <a class="preview" href="/terms">(t!(admin_instance_preview))</a>
                    button(
                        submit: true,
                        attrs: attributes! {
                            :disabled=$(busy.get())
                            :aria-busy=$(if busy.get() { "true" } else { "false" })
                        },
                        (t!(admin_instance_save))
                    )
                </div>
                <p class="hint">(t!(admin_instance_template_note))</p>
            </form>
        </div>
    })
}
