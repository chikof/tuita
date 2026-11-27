//! `/settings`: who you are, the list of sections and the log out button.

use topcoat::{
    Result,
    context::Cx,
    runtime::Event,
    view::{View, attributes, component, view},
};
use tuita_server::dto::MeDto;

use crate::{
    auth::flows::sign_out,
    icons::Icon,
    l10n::t,
    paths,
    post::rich_text::user_name,
    settings::Section,
    shell::{page_bar::page_bar, parts::account_avatar},
    ui::{
        button::{ButtonVariant, button},
        icon::icon,
    },
};

/// The profile card linking to the account's page and one row per section.
#[component]
pub(crate) async fn index(cx: &Cx, me: &MeDto) -> Result<impl View> {
    let title = t!(nav_settings).resolve(cx).into_owned();
    let sub = format!("@{}", me.user.handle);
    let rows: Vec<_> = Section::ALL
        .into_iter()
        .map(|section| (section.slug(), section.title(cx), section.description(cx)))
        .collect();
    let own = me.user.id.to_string();
    Ok(view! {
        page_bar(title: &title, sub: sub.as_str())
        <div class="stack" data-c="settings-index">
            <a class="me wash" href=(paths::profile(&me.user.handle))>
                <span aria-hidden="true">
                    account_avatar(
                        handle: &me.user.handle,
                        src: me.user.avatar_url.as_deref(),
                        shape: me.user.avatar_shape,
                        ink: me.user.ink,
                        size: 48
                    )
                </span>
                <span class="who">
                    user_name(
                        name: &me.user.display_name,
                        verified: me.user.verified,
                        bot: me.user.bot,
                        emojis: &me.user.emojis
                    )
                    <span class="handle">
                        "@"
                        (&me.federated_handle)
                    </span>
                </span>
                icon(name: Icon::ChevronRight, size: 20)
            </a>
            <nav aria-label=(&title)>
                <ul>
                    for (slug, name, description) in rows {
                        <li>
                            <a class="entry wash" href=(format!("/settings/{slug}"))>
                                <span class="text">
                                    <span class="title">(name)</span>
                                    <span class="description">(description)</span>
                                </span>
                                icon(name: Icon::ChevronRight, size: 20)
                            </a>
                        </li>
                    }
                </ul>
            </nav>
            button(
                variant: ButtonVariant::Danger,
                attrs: attributes! {
                    data-forget-keys=(&own)
                    @click=$(async |_e: Event| {
                        let done = sign_out(false).await;
                        if done.is_ok() {
                            raw!("location.assign('/')")
                        } else {
                            let msg = done.unwrap_err();
                            raw!(
                                "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:${msg}}))",
                                msg.is_empty(),
                            )
                        }
                    })
                },
                (t!(settings_logout))
            )
        </div>
    })
}
