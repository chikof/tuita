//! `/oauth/authorize`: the consent card. Guests go to sign-in and come back.

use topcoat::{
    Result,
    context::Cx,
    metadata::Metadata,
    router::{page, request::uri},
    view::{View, view},
};

use crate::{
    auth::auth_card::auth_card,
    l10n::t,
    next::{auth_link, safe_next},
    oauth::{
        self,
        consent_card::{consent_card, failed_card},
    },
    session, shell,
};

#[page(
    with(Metadata::from_fn(
        async |cx| {
            let title = t!(oauth_title).resolve(cx).into_owned();
            shell::titled(cx, &title).await
        },
    ))
)]
async fn authorize_page(cx: &Cx) -> Result<impl View> {
    let viewer = session::member(cx).await?;
    let ask = oauth::ask(cx)?;
    let (consent, failure) = match oauth::check(cx, &ask).await? {
        Ok(consent) => (Some(consent), None),
        Err(failure) => (None, Some(failure)),
    };
    let uri = uri(cx);
    let here = uri.query().map_or_else(
        || uri.path().to_owned(),
        |query| format!("{}?{query}", uri.path()),
    );
    let switch_href = auth_link("/login", true, safe_next(Some(&here)).as_deref());
    let account = format!("@{}", viewer.me.user.handle);
    let ask_json = serde_json::to_string(&ask)?;
    let title = t!(oauth_title).resolve(cx).into_owned();
    Ok(view! {
        auth_card(
            title: &title,
            if let Some(consent) = &consent {
                consent_card(
                    consent: consent,
                    account: &account,
                    ask: &ask_json,
                    switch_href: &switch_href
                )
            } else if let Some(failure) = failure {
                failed_card(failure: failure)
            }
        )
    })
}
