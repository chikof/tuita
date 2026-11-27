//! Gallery sections: OAuth.
//!
//! The consent card of the page starts at `ask`; the other states are what answering leads to, so
//! the gallery starts a card there with [`Start`]. The card has no "loading" state (the page waits
//! for the server's check before it answers), so that frame is drawn here with the card's own
//! markup. Cards that are not asking are given no scopes: their question is hidden, and the
//! gallery counts the scopes it shows.

use topcoat::{
    Result,
    context::Cx,
    view::{Child, View, component, view},
};
use tuita_server::dto::{OAuthConsentDto, TokenScope};

use super::section::section;
use crate::{
    l10n::t,
    oauth::{
        Failure,
        consent_card::{Start, consent_card, failed_card},
    },
    ui::{
        card::card,
        loader::{LoaderVariant, loader},
    },
};

const ASK: &str = "{}";
const SWITCH: &str = "/login?add=1";
const CODE: &str = "k3Jx9oQe2mWvT7uYbN1sZ4dLhA6rC0pFgE8iVnXtMqU";

fn consent() -> OAuthConsentDto {
    OAuthConsentDto {
        app_name: "Tuba".to_owned(),
        website: Some("https://tuba.example".to_owned()),
        scopes: vec![TokenScope::Read, TokenScope::Write],
        redirect_uri: "https://tuba.example/oauth/callback".to_owned(),
        redirect_host: "tuba.example".to_owned(),
        oob: false,
        pkce: true,
    }
}

fn read_only() -> OAuthConsentDto {
    OAuthConsentDto {
        website: None,
        scopes: vec![TokenScope::Read],
        redirect_uri: "tusky://oauth-callback".to_owned(),
        redirect_host: "tusky://oauth-callback".to_owned(),
        ..consent()
    }
}

fn long_names() -> OAuthConsentDto {
    OAuthConsentDto {
        app_name: "A very long application name that keeps going and going past the card".to_owned(),
        website: Some(
            "javascript:alert(1)//an-address-that-is-not-a-web-address-and-has-no-link".to_owned(),
        ),
        scopes: vec![TokenScope::Read, TokenScope::Write, TokenScope::Messages],
        redirect_uri:
            "https://a-very-long-hostname-for-an-app.example.org/oauth/callback/with/a/long/path?and=query"
                .to_owned(),
        redirect_host: "a-very-long-hostname-for-an-app.example.org".to_owned(),
        ..consent()
    }
}

fn out_of_band() -> OAuthConsentDto {
    OAuthConsentDto {
        oob: true,
        redirect_host: String::new(),
        redirect_uri: "urn:ietf:wg:oauth:2.0:oob".to_owned(),
        ..consent()
    }
}

/// The consent of a card whose question is hidden.
fn answered(consent: OAuthConsentDto) -> OAuthConsentDto {
    OAuthConsentDto {
        scopes: Vec::new(),
        ..consent
    }
}

/// One state: its label over a card.
#[component]
async fn state(cx: &Cx, label: &str, #[default] child: Child<'_>) -> Result<impl View> {
    Ok(view! {
        <div class="state" data-c="design-oauth-state">
            <span class="mono-tag">(label)</span>
            card((child))
        </div>
    })
}

/// The card while the request is checked, in the markup of the card's other states.
#[component]
async fn loading(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        <div class="consent" data-c="consent-card" data-state="loading">
            <div class="center" role="status">
                loader(variant: LoaderVariant::Dots)
                <p class="dim">(t!(oauth_loading))</p>
            </div>
        </div>
    })
}

#[component]
pub(crate) async fn oauth(cx: &Cx) -> Result<impl View> {
    let (ask, read_only, long, oob) = (consent(), read_only(), long_names(), out_of_band());
    let (sent, code, denied) = (
        answered(consent()),
        answered(out_of_band()),
        answered(out_of_band()),
    );
    Ok(view! {
        section(
            id: "consent-card",
            title: "ConsentCard",
            note: "the body of /oauth/authorize: loading, the request to approve, a refusal, the hand-off, the code of an out-of-band app",
            <div class="grid" data-c="design-oauth-grid">
                state(label: "loading", loading())
                state(
                    label: "ask",
                    consent_card(
                        consent: &ask,
                        account: "@ada",
                        ask: ASK,
                        switch_href: SWITCH
                    )
                )
                state(
                    label: "ask, read only, app scheme, no website",
                    consent_card(
                        consent: &read_only,
                        account: "@ada",
                        ask: ASK,
                        switch_href: SWITCH
                    )
                )
                state(
                    label: "ask, long names",
                    consent_card(
                        consent: &long,
                        account: "@a_long_handle_x",
                        ask: ASK,
                        switch_href: SWITCH
                    )
                )
                state(
                    label: "ask, out of band",
                    consent_card(
                        consent: &oob,
                        account: "@ada",
                        ask: ASK,
                        switch_href: SWITCH
                    )
                )
                state(
                    label: "ask, sending",
                    consent_card(
                        consent: &ask,
                        account: "@ada",
                        ask: ASK,
                        switch_href: SWITCH,
                        start: Start::Sending { approve: true }
                    )
                )
                state(
                    label: "ask, answer failed",
                    consent_card(
                        consent: &ask,
                        account: "@ada",
                        ask: ASK,
                        switch_href: SWITCH,
                        start: Start::Retry(Failure::Rate)
                    )
                )
                state(
                    label: "refused, unknown app",
                    failed_card(failure: Failure::InvalidClient)
                )
                state(
                    label: "refused, scope",
                    failed_card(failure: Failure::InvalidScope)
                )
                state(
                    label: "failed, network",
                    failed_card(failure: Failure::Unreachable)
                )
                state(
                    label: "sent, authorized",
                    consent_card(
                        consent: &sent,
                        account: "@ada",
                        ask: ASK,
                        switch_href: SWITCH,
                        start: Start::Sent { approved: true }
                    )
                )
                state(
                    label: "sent, denied",
                    consent_card(
                        consent: &sent,
                        account: "@ada",
                        ask: ASK,
                        switch_href: SWITCH,
                        start: Start::Sent { approved: false }
                    )
                )
                state(
                    label: "code",
                    consent_card(
                        consent: &code,
                        account: "@ada",
                        ask: ASK,
                        switch_href: SWITCH,
                        start: Start::Code,
                        code_text: CODE
                    )
                )
                state(
                    label: "denied, out of band",
                    consent_card(
                        consent: &denied,
                        account: "@ada",
                        ask: ASK,
                        switch_href: SWITCH,
                        start: Start::Denied
                    )
                )
            </div>
        )
    })
}
