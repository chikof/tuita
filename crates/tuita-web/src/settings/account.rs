//! `/settings/account`.

use topcoat::{
    Result,
    context::Cx,
    l10n::locale,
    view::{View, component, view},
};
use tuita_server::dto::MeDto;

use crate::{
    fmt,
    l10n::t,
    post::rich_text::user_name,
    settings::{card::setting_card, page::settings_page, password_form::password_form},
    shell::parts::account_avatar,
};

/// The profile summary with a link to edit it, and the password form.
#[component]
pub(crate) async fn account(cx: &Cx, me: &MeDto) -> Result<impl View> {
    let user = &me.user;
    let joined = t!(
        settings_account_joined,
        date: fmt::month_year_in(locale(cx), user.created_at),
    )
    .resolve(cx)
    .into_owned();
    let title = t!(settings_account).resolve(cx).into_owned();
    Ok(view! {
        settings_page(
            title: &title,
            setting_card(
                title: t!(settings_account_profile).resolve(cx).into_owned(),
                <div class="line" data-c="account-line">
                    <span aria-hidden="true">
                        account_avatar(
                            handle: &user.handle,
                            src: user.avatar_url.as_deref(),
                            shape: user.avatar_shape,
                            ink: user.ink,
                            size: 48
                        )
                    </span>
                    <div class="who">
                        user_name(
                            name: &user.display_name,
                            verified: user.verified,
                            bot: user.bot,
                            emojis: &user.emojis
                        )
                        <span class="meta">
                            "@"
                            (&me.federated_handle)
                        </span>
                        <span class="meta">(&joined)</span>
                    </div>
                </div>
                <a class="edit" data-c="account-edit" href="/settings/profile">
                    (t!(settings_account_edit))
                </a>
            )
            password_form()
        )
    })
}
